// Copyright 2019 Cargill Incorporated
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

pub mod error;
pub mod routes;

use crate::node_registry::yaml::YamlNodeRegistry;
use crate::registry_config::RegistryConfig;
use actix_web::{middleware, web, App, HttpServer};
use error::RestApiServerError;
use libsplinter::node_registry::NodeRegistry;
use std::sync::mpsc;
use std::thread;

pub struct RestApiShutdownHandle {
    do_shutdown: Box<dyn Fn() -> Result<(), RestApiServerError> + Send>,
}

impl RestApiShutdownHandle {
    pub fn shutdown(&self) -> Result<(), RestApiServerError> {
        (*self.do_shutdown)()
    }
}

pub fn run(
    bind_url: &str,
    registry_config: &RegistryConfig,
) -> Result<
    (
        RestApiShutdownHandle,
        thread::JoinHandle<Result<(), RestApiServerError>>,
    ),
    RestApiServerError,
> {
    let bind_url = bind_url.to_owned();
    let (tx, rx) = mpsc::channel();
    let node_registry = create_node_registry(&registry_config)?;

    let join_handle = thread::Builder::new()
        .name("SplinterDRestApi".into())
        .spawn(move || {
            let sys = actix::System::new("SplinterD-Rest-API");
            let addr = HttpServer::new(move || {
                App::new()
                    .data(node_registry.clone())
                    .wrap(middleware::Logger::default())
                    .service(
                        web::resource("/status").route(web::get().to_async(routes::get_status)),
                    )
                    .service(
                        web::resource("/openapi.yml")
                            .route(web::get().to_async(routes::get_openapi)),
                    )
                    .service(
                        web::resource("/nodes/{identity}")
                            .route(web::get().to_async(routes::fetch_node)),
                    )
                    .service(web::resource("/nodes").route(web::get().to_async(routes::list_nodes)))
            })
            .bind(bind_url)?
            .disable_signals()
            .system_exit()
            .start();

            tx.send(addr).map_err(|err| {
                RestApiServerError::StartUpError(format!("Unable to send Server Addr: {}", err))
            })?;
            sys.run()?;

            info!("Rest API terminating");

            Ok(())
        })?;

    let addr = rx.recv().map_err(|err| {
        RestApiServerError::StartUpError(format!("Unable to receive Server Addr: {}", err))
    })?;

    let do_shutdown = Box::new(move || {
        debug!("Shutting down Rest API");
        addr.stop(true);
        debug!("Graceful signal sent to Rest API");

        Ok(())
    });

    Ok((RestApiShutdownHandle { do_shutdown }, join_handle))
}

fn create_node_registry(
    registry_config: &RegistryConfig,
) -> Result<Box<dyn NodeRegistry>, RestApiServerError> {
    match &registry_config.registry_backend() as &str {
        "FILE" => Ok(Box::new(
            YamlNodeRegistry::new(&registry_config.registry_file()).map_err(|err| {
                RestApiServerError::StartUpError(format!(
                    "Failed to initialize YamlNodeRegistry: {}",
                    err
                ))
            })?,
        )),
        _ => Err(RestApiServerError::StartUpError(
            "NodeRegistry type is not supported".to_string(),
        )),
    }
}
