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

#[cfg(feature = "integration")]
use grid_sdk::rest_api::actix_web_3::State as IntegrationState;
use grid_sdk::store::{create_store_factory, ConnectionUri};

use crate::config::GridConfig;
use crate::database::{ConnectionPool, DatabaseError};
use crate::error::DaemonError;
use crate::event::{db_handler::DatabaseEventHandler, EventProcessor};
use crate::rest_api;

use super::{batch_submitter::SawtoothBatchSubmitter, connection::SawtoothConnection};

pub fn run_sawtooth(config: GridConfig) -> Result<(), DaemonError> {
    let connection_uri = config
        .database_url()
        .parse()
        .map_err(|err| DaemonError::StartUpError(Box::new(err)))?;

    let store_factory = create_store_factory(&connection_uri)
        .map_err(|err| DaemonError::StartUpError(Box::new(err)))?;

    let sawtooth_connection = SawtoothConnection::new(&config.endpoint().url());
    let batch_submitter = Box::new(SawtoothBatchSubmitter::new(
        sawtooth_connection.get_sender(),
    ));
    let (db_executor, evt_processor) = {
        let commit_store = store_factory.get_grid_commit_store();
        let current_commit =
            commit_store
                .get_current_commit_id()
                .map_err(|err| DatabaseError::ConnectionError {
                    context: "Could not get current commit ID".to_string(),
                    source: Box::new(err),
                })?;

        match connection_uri {
            ConnectionUri::Postgres(_) => {
                let connection_pool: ConnectionPool<diesel::pg::PgConnection> =
                    ConnectionPool::new(config.database_url())?;
                let evt_processor = EventProcessor::start(
                    sawtooth_connection,
                    current_commit.as_deref(),
                    event_handlers![DatabaseEventHandler::from_pg_pool(connection_pool.clone())],
                )
                .map_err(|err| DaemonError::EventProcessorError(Box::new(err)))?;

                (
                    rest_api::DbExecutor::from_pg_pool(connection_pool),
                    evt_processor,
                )
            }
            ConnectionUri::Sqlite(_) => {
                let connection_pool: ConnectionPool<diesel::sqlite::SqliteConnection> =
                    ConnectionPool::new(config.database_url())?;
                let evt_processor = EventProcessor::start(
                    sawtooth_connection,
                    current_commit.as_deref(),
                    event_handlers![DatabaseEventHandler::from_sqlite_pool(
                        connection_pool.clone()
                    )],
                )
                .map_err(|err| DaemonError::EventProcessorError(Box::new(err)))?;

                (
                    rest_api::DbExecutor::from_sqlite_pool(connection_pool),
                    evt_processor,
                )
            }
        }
    };

    #[cfg(feature = "integration")]
    let integration_state = match connection_uri {
        ConnectionUri::Postgres(_) => {
            let connection_pool: ConnectionPool<diesel::pg::PgConnection> =
                ConnectionPool::new(config.database_url())?;
            IntegrationState::with_pg_pool(
                &config.key_file_name().to_string(),
                connection_pool.pool,
            )
        }
        ConnectionUri::Sqlite(_) => {
            let connection_pool: ConnectionPool<diesel::sqlite::SqliteConnection> =
                ConnectionPool::new(config.database_url())?;
            IntegrationState::with_sqlite_pool(
                &config.key_file_name().to_string(),
                connection_pool.pool,
            )
        }
    };

    let (rest_api_shutdown_handle, rest_api_join_handle) = rest_api::run(
        config.rest_api_endpoint(),
        db_executor,
        batch_submitter,
        config.endpoint().clone(),
        #[cfg(feature = "integration")]
        integration_state,
    )?;

    let (event_processor_shutdown_handle, event_processor_join_handle) =
        evt_processor.take_shutdown_controls();

    let ctrlc_triggered = AtomicBool::new(false);
    ctrlc::set_handler(move || {
        if ctrlc_triggered.load(Ordering::SeqCst) {
            eprintln!("Aborting due to multiple Ctrl-C events");
            process::exit(1);
        }

        ctrlc_triggered.store(true, Ordering::SeqCst);

        rest_api_shutdown_handle.shutdown();

        if let Err(err) = event_processor_shutdown_handle.shutdown() {
            error!("Unable to gracefully shutdown Event Processor: {}", err);
        }
    })
    .map_err(|err| DaemonError::StartUpError(Box::new(err)))?;

    rest_api_join_handle
        .join()
        .map_err(|_| {
            DaemonError::ShutdownError("Unable to cleanly join the REST API thread".into())
        })
        .and_then(|res| res.map_err(DaemonError::from))?;

    event_processor_join_handle
        .join()
        .map_err(|_| {
            DaemonError::ShutdownError("Unable to cleanly join the event processor".into())
        })
        .and_then(|res| res.map_err(DaemonError::from))?;

    Ok(())
}
