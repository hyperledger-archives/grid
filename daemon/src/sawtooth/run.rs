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
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use grid_sdk::backend::SawtoothBackendClient;
#[cfg(feature = "rest-api")]
use grid_sdk::rest_api::actix_web_3::Endpoint;
#[cfg(feature = "integration")]
use grid_sdk::rest_api::actix_web_3::KeyState;
use grid_sdk::rest_api::actix_web_3::{BackendState, StoreState};
use grid_sdk::store::{create_store_factory, ConnectionUri};

use crate::config::GridConfig;
use crate::database::ConnectionPool;
use crate::error::DaemonError;
use crate::event::{db_handler::DatabaseEventHandler, EventProcessor};
use crate::rest_api;

use super::connection::SawtoothConnection;

pub fn run_sawtooth(config: GridConfig) -> Result<(), DaemonError> {
    let sawtooth_endpoint = Endpoint::from(config.endpoint());
    let connection_uri = config
        .database_url()
        .parse()
        .map_err(|err| DaemonError::from_source(Box::new(err)))?;

    let store_factory = create_store_factory(&connection_uri)
        .map_err(|err| DaemonError::from_source(Box::new(err)))?;

    let sawtooth_connection = SawtoothConnection::new(&sawtooth_endpoint.url());
    let backend_client = SawtoothBackendClient::new(sawtooth_connection.get_sender());
    let backend_state = BackendState::new(Arc::new(backend_client));

    #[cfg(not(any(feature = "database-postgres", feature = "database-sqlite")))]
    return Err(DaemonError::with_message(
        "A database backend is required to be active. Supported backends are postgreSQL and SQLite",
    ));

    #[cfg(any(feature = "database-postgres", feature = "database-sqlite"))]
    let (store_state, evt_processor) = {
        let commit_store = store_factory.get_grid_commit_store();
        let current_commit = commit_store
            .get_current_commit_id()
            .map_err(|err| DaemonError::from_source(Box::new(err)))?;

        match connection_uri {
            #[cfg(feature = "database-postgres")]
            ConnectionUri::Postgres(_) => {
                let connection_pool: ConnectionPool<diesel::pg::PgConnection> =
                    ConnectionPool::new(config.database_url())
                        .map_err(|err| DaemonError::from_source(Box::new(err)))?;
                let evt_processor = EventProcessor::start(
                    sawtooth_connection,
                    current_commit.as_deref(),
                    event_handlers![DatabaseEventHandler::from_pg_pool(connection_pool.clone())],
                )
                .map_err(|err| DaemonError::from_source(Box::new(err)))?;

                (
                    StoreState::with_pg_pool(connection_pool.pool),
                    evt_processor,
                )
            }
            #[cfg(feature = "database-sqlite")]
            ConnectionUri::Sqlite(_) => {
                let connection_pool: ConnectionPool<diesel::sqlite::SqliteConnection> =
                    ConnectionPool::new(config.database_url())
                        .map_err(|err| DaemonError::from_source(Box::new(err)))?;
                let evt_processor = EventProcessor::start(
                    sawtooth_connection,
                    current_commit.as_deref(),
                    event_handlers![DatabaseEventHandler::from_sqlite_pool(
                        connection_pool.clone()
                    )],
                )
                .map_err(|err| DaemonError::from_source(Box::new(err)))?;

                (
                    StoreState::with_sqlite_pool(connection_pool.pool),
                    evt_processor,
                )
            }
        }
    };

    #[cfg(feature = "integration")]
    let key_state = KeyState::new(config.key_file_name());

    #[cfg(feature = "rest-api")]
    let (rest_api_shutdown_handle, rest_api_join_handle) = rest_api::run(
        config.rest_api_endpoint(),
        store_state,
        backend_state,
        #[cfg(feature = "integration")]
        key_state,
        sawtooth_endpoint,
    )
    .map_err(|err| DaemonError::from_source(Box::new(err)))?;

    let (event_processor_shutdown_handle, event_processor_join_handle) =
        evt_processor.take_shutdown_controls();

    let ctrlc_triggered = AtomicBool::new(false);
    ctrlc::set_handler(move || {
        if ctrlc_triggered.load(Ordering::SeqCst) {
            eprintln!("Aborting due to multiple Ctrl-C events");
            process::exit(1);
        }

        ctrlc_triggered.store(true, Ordering::SeqCst);

        #[cfg(feature = "rest-api")]
        rest_api_shutdown_handle.shutdown();

        if let Err(err) = event_processor_shutdown_handle.shutdown() {
            error!("Unable to gracefully shutdown Event Processor: {}", err);
        }
    })
    .map_err(|err| DaemonError::from_source(Box::new(err)))?;

    #[cfg(feature = "rest-api")]
    rest_api_join_handle
        .join()
        .map_err(|_| DaemonError::with_message("Unable to cleanly join the REST API thread"))
        .and_then(|res| res.map_err(|err| DaemonError::from_source(Box::new(err))))?;

    event_processor_join_handle
        .join()
        .map_err(|_| DaemonError::with_message("Unable to cleanly join the event processor"))
        .and_then(|res| res.map_err(|err| DaemonError::from_source(Box::new(err))))?;

    Ok(())
}
