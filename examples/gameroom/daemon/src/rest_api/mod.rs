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

mod error;
mod routes;

use actix_web::{client::Client, web, App, HttpServer, Result};
use gameroom_database::ConnectionPool;
use std::sync::mpsc;
use std::thread;

pub use error::{RestApiResponseError, RestApiServerError};

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
    splinterd_url: &str,
    database_connection: ConnectionPool,
) -> Result<
    (
        RestApiShutdownHandle,
        thread::JoinHandle<Result<(), RestApiServerError>>,
    ),
    RestApiServerError,
> {
    let bind_url = bind_url.to_owned();
    let splinterd_url = splinterd_url.to_owned();
    let (tx, rx) = mpsc::channel();
    let join_handle = thread::Builder::new()
        .name("GameroomDRestApi".into())
        .spawn(move || {
            let sys = actix::System::new("GameroomD-Rest-API");

            let addr = HttpServer::new(move || {
                App::new()
                    .data(database_connection.clone())
                    .data(Client::new())
                    .data(splinterd_url.to_owned())
                    .service(
                        web::resource("/nodes/{identity}")
                            .route(web::get().to_async(routes::fetch_node)),
                    )
                    .service(web::resource("/nodes").route(web::get().to_async(routes::list_nodes)))
                    .service(
                        web::resource("/gamerooms/propose")
                            .route(web::post().to_async(routes::propose_gameroom)),
                    )
                    .service(
                        web::scope("/users")
                            .service(
                                web::resource("").route(web::post().to_async(routes::register)),
                            )
                            .service(
                                web::resource("/authenticate")
                                    .route(web::post().to_async(routes::login)),
                            ),
                    )
                    .service(
                        web::scope("/proposals")
                            .service(
                                web::resource("/{proposal_id}")
                                    .route(web::get().to_async(routes::fetch_proposal)),
                            )
                            .service(
                                web::resource("")
                                    .route(web::get().to_async(routes::list_proposals)),
                            ),
                    )
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
