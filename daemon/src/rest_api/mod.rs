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
mod route_handler;

use std::sync::mpsc;
use std::thread;

use crate::database::ConnectionPool;
pub use crate::rest_api::error::RestApiServerError;
use crate::rest_api::route_handler::{
    get_batch_statuses, submit_batches, DbExecutor, SawtoothMessageSender,
};
use actix::{Actor, Addr, Context, SyncArbiter};
use actix_web::{http::Method, server, App};
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
) -> App<AppState> {
    App::with_state(AppState {
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
            let addr = server::new(move || {
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
        addr.do_send(server::StopServer { graceful: true });
        debug!("Graceful signal sent to Rest API");

        Ok(())
    });

    Ok((RestApiShutdownHandle { do_shutdown }, join_handle))
}
