/*
 * Copyright 2019 Bitwise IO, Inc.
 * Copyright 2020-2021 Cargill Incorporated
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * -----------------------------------------------------------------------------
 */

use std::process;
use std::sync::atomic::{AtomicBool, Ordering};

use grid_sdk::store::ConnectionUri;
use splinter::events::Reactor;

use crate::config::GridConfig;
use crate::database::ConnectionPool;
use crate::error::DaemonError;
use crate::event::{db_handler::DatabaseEventHandler, EventHandler};
use crate::rest_api;

use super::{
    app_auth_handler, batch_submitter::SplinterBatchSubmitter,
    event::ScabbardEventConnectionFactory, key::load_scabbard_admin_key,
};

pub fn run_splinter(config: GridConfig) -> Result<(), DaemonError> {
    let reactor = Reactor::new();

    let scabbard_admin_key = load_scabbard_admin_key(&config.admin_key_dir())
        .map_err(|err| DaemonError::StartUpError(Box::new(err)))?;

    let scabbard_event_connection_factory =
        ScabbardEventConnectionFactory::new(&config.endpoint().url(), reactor.igniter());

    let (db_executor, db_handler): (rest_api::DbExecutor, Box<dyn EventHandler + Sync + 'static>) = {
        let connection_uri = config
            .database_url()
            .parse()
            .map_err(|err| DaemonError::StartUpError(Box::new(err)))?;
        match connection_uri {
            ConnectionUri::Postgres(_) => {
                let connection_pool: ConnectionPool<diesel::pg::PgConnection> =
                    ConnectionPool::new(config.database_url())?;
                (
                    rest_api::DbExecutor::from_pg_pool(connection_pool.clone()),
                    Box::new(DatabaseEventHandler::from_pg_pool(connection_pool)),
                )
            }
            ConnectionUri::Sqlite(_) | ConnectionUri::Memory => {
                let connection_pool: ConnectionPool<diesel::sqlite::SqliteConnection> =
                    ConnectionPool::new(config.database_url())?;
                (
                    rest_api::DbExecutor::from_sqlite_pool(connection_pool.clone()),
                    Box::new(DatabaseEventHandler::from_sqlite_pool(connection_pool)),
                )
            }
        }
    };

    app_auth_handler::run(
        config.endpoint().url(),
        scabbard_event_connection_factory,
        db_handler,
        reactor.igniter(),
        scabbard_admin_key,
    )?;

    let batch_submitter = Box::new(SplinterBatchSubmitter::new(config.endpoint().url()));

    let (rest_api_shutdown_handle, rest_api_join_handle) = rest_api::run(
        config.rest_api_endpoint(),
        db_executor,
        batch_submitter,
        config.endpoint().clone(),
    )?;

    let reactor_shutdown_signaler = reactor.shutdown_signaler();

    let ctrlc_triggered = AtomicBool::new(false);
    ctrlc::set_handler(move || {
        if ctrlc_triggered.load(Ordering::SeqCst) {
            eprintln!("Aborting due to multiple Ctrl-C events");
            process::exit(1);
        }

        ctrlc_triggered.store(true, Ordering::SeqCst);

        if let Err(err) = reactor_shutdown_signaler.signal_shutdown() {
            error!(
                "Unable to signal shutdown to splinter event reactor: {}",
                err
            );
        }

        rest_api_shutdown_handle.shutdown();
    })
    .map_err(|err| DaemonError::StartUpError(Box::new(err)))?;

    rest_api_join_handle
        .join()
        .map_err(|_| {
            DaemonError::ShutdownError("Unable to cleanly join the REST API thread".into())
        })
        .and_then(|res| res.map_err(DaemonError::from))?;

    if let Err(err) = reactor.wait_for_shutdown() {
        error!("Unable to shutdown splinter event reactor: {}", err);
    }

    Ok(())
}
