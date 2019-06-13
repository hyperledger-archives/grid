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

mod error;
mod routes;

use std::sync::mpsc;
use std::thread;

use crate::database::ConnectionPool;
pub use crate::rest_api::error::RestApiServerError;
use crate::rest_api::routes::{
    fetch_agent, fetch_grid_schema, fetch_organization, fetch_record, fetch_record_property,
    get_batch_statuses, list_agents, list_grid_schemas, list_organizations, list_records,
    submit_batches,
};
use crate::rest_api::routes::{DbExecutor, SawtoothMessageSender};
use actix::{Actor, Addr, Context, SyncArbiter};
use actix_web::{body, http::Method, App, HttpServer};
use sawtooth_sdk::messaging::stream::MessageSender;

pub struct AppState {
    sawtooth_connection: Addr<SawtoothMessageSender>,
    database_connection: Addr<DbExecutor>,
}

pub struct RestApiShutdownHandle {
    do_shutdown: Box<dyn Fn() -> Result<(), RestApiServerError> + Send>,
}

impl RestApiShutdownHandle {
    pub fn shutdown(&self) -> Result<(), RestApiServerError> {
        (*self.do_shutdown)()
    }
}

fn create_app(
    sawtooth_connection: Addr<SawtoothMessageSender>,
    database_connection: Addr<DbExecutor>,
) -> App<AppState, body::Body> {
    App::new()
        .data(AppState {
            sawtooth_connection,
            database_connection,
        })
        .resource("/batches", |r| {
            r.method(Method::POST).with_async(submit_batches)
        })
        .resource("/batch_statuses", |r| {
            r.name("batch_statuses");
            r.method(Method::GET).with_async(get_batch_statuses)
        })
        .resource("/agent", |r| r.method(Method::GET).with_async(list_agents))
        .resource("/agent/{public_key}", |r| {
            r.method(Method::GET).with_async(fetch_agent)
        })
        .resource("/organization", |r| {
            r.method(Method::GET).with_async(list_organizations)
        })
        .resource("/organization/{id}", |r| {
            r.method(Method::GET).with_async(fetch_organization)
        })
        .resource("/schema", |r| {
            r.method(Method::GET).with_async(list_grid_schemas)
        })
        .resource("/schema/{name}", |r| {
            r.method(Method::GET).with_async(fetch_grid_schema)
        })
        .resource("/record", |r| {
            r.method(Method::GET).with_async(list_records)
        })
        .resource("/record/{record_id}", |r| {
            r.method(Method::GET).with_async(fetch_record)
        })
        .resource("/record/{record_id}/property/{property_name}", |r| {
            r.method(Method::GET).with_async(fetch_record_property)
        })
}

pub fn run(
    bind_url: &str,
    zmq_sender: Box<dyn MessageSender + Send>,
    connection_pool: ConnectionPool,
) -> Result<
    (
        RestApiShutdownHandle,
        thread::JoinHandle<Result<(), RestApiServerError>>,
    ),
    RestApiServerError,
> {
    let (tx, rx) = mpsc::channel();
    let bind_url = bind_url.to_owned();
    let join_handle = thread::Builder::new()
        .name("GridRestApi".into())
        .spawn(move || {
            let sys = actix::System::new("Grid-Rest-API");
            let zmq_connection_addr =
                SawtoothMessageSender::create(move |_ctx: &mut Context<SawtoothMessageSender>| {
                    SawtoothMessageSender::new(zmq_sender)
                });
            let db_executor_addr =
                SyncArbiter::start(2, move || DbExecutor::new(connection_pool.clone()));
            info!("Starting Rest API at {}", &bind_url);
            let addr = HttpServer::new(move || {
                create_app(zmq_connection_addr.clone(), db_executor_addr.clone())
            })
            .bind(bind_url)?
            .disable_signals()
            .system_exit()
            .start();

            tx.send(addr).map_err(|err| {
                RestApiServerError::StartUpError(format!("Unable to send Server Addr: {}", err))
            })?;

            sys.run();

            info!("Rest API terminating");

            Ok(())
        })?;

    let addr = rx.recv().map_err(|err| {
        RestApiServerError::StartUpError(format!("Unable to receive Server Addr: {}", err))
    })?;

    let do_shutdown = Box::new(move || {
        debug!("Shutting down Rest API");
        addr.do_send(HttpServer::StopServer { graceful: true });
        debug!("Graceful signal sent to Rest API");

        Ok(())
    });

    Ok((RestApiShutdownHandle { do_shutdown }, join_handle))
}
