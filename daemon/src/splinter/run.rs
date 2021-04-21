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

use std::path::PathBuf;
use std::process;
use std::sync::atomic::{AtomicBool, Ordering};

use cylinder::load_key;
#[cfg(feature = "integration")]
use grid_sdk::rest_api::actix_web_3::KeyState;
use grid_sdk::rest_api::actix_web_3::{BatchSubmitterState, StoreState};
use grid_sdk::store::ConnectionUri;
use grid_sdk::submitter::SplinterBatchSubmitter;
use splinter::events::Reactor;

use crate::config::GridConfig;
use crate::database::ConnectionPool;
use crate::error::DaemonError;
use crate::event::{db_handler::DatabaseEventHandler, EventHandler};
use crate::rest_api;

use super::{app_auth_handler, event::ScabbardEventConnectionFactory};

pub fn run_splinter(config: GridConfig) -> Result<(), DaemonError> {
    let reactor = Reactor::new();

    let scabbard_admin_key = load_key("gridd", &[PathBuf::from(config.admin_key_dir())])
        .map_err(|err| DaemonError::from_source(Box::new(err)))?
        .ok_or_else(|| DaemonError::with_message("no private key found"))?
        .as_hex();

    let scabbard_event_connection_factory =
        ScabbardEventConnectionFactory::new(&config.endpoint().url(), reactor.igniter());

    let (store_state, db_handler): (StoreState, Box<dyn EventHandler + Sync + 'static>) = {
        let connection_uri = config
            .database_url()
            .parse()
            .map_err(|err| DaemonError::from_source(Box::new(err)))?;
        match connection_uri {
            ConnectionUri::Postgres(_) => {
                let connection_pool: ConnectionPool<diesel::pg::PgConnection> =
                    ConnectionPool::new(config.database_url())
                        .map_err(|err| DaemonError::from_source(Box::new(err)))?;
                (
                    StoreState::with_pg_pool(connection_pool.pool.clone()),
                    Box::new(DatabaseEventHandler::from_pg_pool(connection_pool)),
                )
            }
            ConnectionUri::Sqlite(_) => {
                let connection_pool: ConnectionPool<diesel::sqlite::SqliteConnection> =
                    ConnectionPool::new(config.database_url())
                        .map_err(|err| DaemonError::from_source(Box::new(err)))?;
                (
                    StoreState::with_sqlite_pool(connection_pool.pool.clone()),
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
    )
    .map_err(|err| DaemonError::from_source(Box::new(err)))?;

    let batch_submitter = SplinterBatchSubmitter::new(config.endpoint().url());
    let batch_submitter_state = BatchSubmitterState::with_splinter(batch_submitter);

    #[cfg(feature = "integration")]
    let key_state = KeyState::new(&config.key_file_name());

    let (rest_api_shutdown_handle, rest_api_join_handle) = rest_api::run(
        config.rest_api_endpoint(),
        store_state,
        batch_submitter_state,
        #[cfg(feature = "integration")]
        key_state,
        config.endpoint().clone(),
    )
    .map_err(|err| DaemonError::from_source(Box::new(err)))?;

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
    .map_err(|err| DaemonError::from_source(Box::new(err)))?;

    rest_api_join_handle
        .join()
        .map_err(|_| DaemonError::with_message("Unable to cleanly join the REST API thread"))
        .and_then(|res| res.map_err(|err| DaemonError::from_source(Box::new(err))))?;

    if let Err(err) = reactor.wait_for_shutdown() {
        error!("Unable to shutdown splinter event reactor: {}", err);
    }

    Ok(())
}
