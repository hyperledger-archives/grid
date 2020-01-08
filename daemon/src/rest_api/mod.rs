// Copyright 2019 Bitwise IO, Inc.
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
mod routes;

use std::sync::mpsc;
use std::thread;

use crate::database::ConnectionPool;
pub use crate::rest_api::error::RestApiServerError;
use crate::rest_api::routes::DbExecutor;
use crate::rest_api::routes::{
    fetch_agent, fetch_grid_schema, fetch_organization, fetch_product, fetch_record,
    fetch_record_property, get_batch_statuses, list_agents, list_grid_schemas, list_organizations,
    list_products, list_records, submit_batches,
};
use crate::submitter::BatchSubmitter;
use actix::{Addr, SyncArbiter};
use actix_web::{web, App, HttpServer, Result};
use futures::Future;

const SYNC_ARBITER_THREAD_COUNT: usize = 2;

#[derive(Clone)]
pub struct AppState {
    batch_submitter: Box<dyn BatchSubmitter + 'static>,
    database_connection: Addr<DbExecutor>,
}

impl AppState {
    pub fn new(
        batch_submitter: Box<dyn BatchSubmitter + 'static>,
        connection_pool: ConnectionPool,
    ) -> Self {
        let database_connection = SyncArbiter::start(SYNC_ARBITER_THREAD_COUNT, move || {
            DbExecutor::new(connection_pool.clone())
        });

        AppState {
            batch_submitter,
            database_connection,
        }
    }
}

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
    database_connection: ConnectionPool,
    batch_submitter: Box<dyn BatchSubmitter + 'static>,
) -> Result<
    (
        RestApiShutdownHandle,
        thread::JoinHandle<Result<(), RestApiServerError>>,
    ),
    RestApiServerError,
> {
    let bind_url = bind_url.to_owned();
    let (tx, rx) = mpsc::channel();

    let join_handle = thread::Builder::new()
        .name("GridRestApi".into())
        .spawn(move || {
            let sys = actix::System::new("Grid-Rest-API");
            let state = AppState::new(batch_submitter, database_connection);

            let addr = HttpServer::new(move || {
                App::new()
                    .data(state.clone())
                    .service(web::resource("/batches").route(web::post().to_async(submit_batches)))
                    .service(
                        web::resource("/batch_statuses")
                            .name("batch_statuses")
                            .route(web::get().to_async(get_batch_statuses)),
                    )
                    .service(
                        web::scope("/agent")
                            .service(web::resource("").route(web::get().to_async(list_agents)))
                            .service(
                                web::resource("/{public_key}")
                                    .route(web::get().to_async(fetch_agent)),
                            ),
                    )
                    .service(
                        web::scope("/organization")
                            .service(
                                web::resource("").route(web::get().to_async(list_organizations)),
                            )
                            .service(
                                web::resource("/{id}")
                                    .route(web::get().to_async(fetch_organization)),
                            ),
                    )
                    .service(
                        web::scope("/product")
                            .service(web::resource("").route(web::get().to_async(list_products)))
                            .service(
                                web::resource("/{id}").route(web::get().to_async(fetch_product)),
                            ),
                    )
                    .service(
                        web::scope("/schema")
                            .service(
                                web::resource("").route(web::get().to_async(list_grid_schemas)),
                            )
                            .service(
                                web::resource("/{name}")
                                    .route(web::get().to_async(fetch_grid_schema)),
                            ),
                    )
                    .service(
                        web::scope("/record")
                            .service(web::resource("").route(web::get().to_async(list_records)))
                            .service(
                                web::scope("/{record_id}")
                                    .service(
                                        web::resource("").route(web::get().to_async(fetch_record)),
                                    )
                                    .service(
                                        web::resource("/property/{property_name}")
                                            .route(web::get().to_async(fetch_record_property)),
                                    ),
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
        if let Err(err) = addr.stop(true).wait() {
            error!("Failed to shutdown rest api cleanly: {:?}", err);
        }
        debug!("Graceful signal sent to Rest API");

        Ok(())
    });

    Ok((RestApiShutdownHandle { do_shutdown }, join_handle))
}
