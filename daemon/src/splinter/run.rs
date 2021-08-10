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

use std::convert::TryFrom;
use std::ops::Deref;
use std::path::PathBuf;
use std::process;
#[cfg(feature = "cylinder-jwt-support")]
use std::sync::Mutex;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use cylinder::load_key;
#[cfg(feature = "cylinder-jwt-support")]
use cylinder::{secp256k1::Secp256k1Context, Context};
use grid_sdk::backend::SplinterBackendClient;
use grid_sdk::commits::store::Commit;
use grid_sdk::commits::{CommitStore, DieselCommitStore};
use grid_sdk::error::InvalidStateError;
#[cfg(feature = "rest-api")]
use grid_sdk::rest_api::actix_web_3::Endpoint;
#[cfg(feature = "integration")]
use grid_sdk::rest_api::actix_web_3::KeyState;
use grid_sdk::rest_api::actix_web_3::{BackendState, StoreState};
use grid_sdk::store::{create_store_factory, ConnectionUri};
use splinter::events::Reactor;

use crate::config::GridConfig;
use crate::database::ConnectionPool;
use crate::error::DaemonError;
use crate::event::{db_handler::DatabaseEventHandler, CommitEvent, EventError, EventHandler};
use crate::rest_api;

use super::{
    app_auth_handler, event::processors::EventProcessors, event::ScabbardEventConnectionFactory,
};

enum EventCmd {
    Event(CommitEvent),
    Exit,
}

struct ChannelEventHandler {
    sender: std::sync::mpsc::Sender<EventCmd>,
}

impl EventHandler for ChannelEventHandler {
    fn handle_event(&self, event: &CommitEvent) -> Result<(), EventError> {
        self.sender
            .send(EventCmd::Event(event.clone()))
            .map_err(|_| EventError("Unable to send event due to closed channel".into()))
    }

    fn cloned_box(&self) -> Box<dyn EventHandler> {
        Box::new(ChannelEventHandler {
            sender: self.sender.clone(),
        })
    }
}

pub fn run_splinter(config: GridConfig) -> Result<(), DaemonError> {
    let splinter_endpoint = Endpoint::from(config.endpoint());
    let reactor = Reactor::new();

    let gridd_key = load_key("gridd", &[PathBuf::from(config.admin_key_dir())])
        .map_err(|err| DaemonError::from_source(Box::new(err)))?
        .ok_or_else(|| DaemonError::with_message("no private key found"))?;

    let scabbard_admin_key = &gridd_key.as_hex();

    let scabbard_event_connection_factory =
        ScabbardEventConnectionFactory::new(&splinter_endpoint.url(), reactor.igniter());

    let event_processors = EventProcessors::new(scabbard_event_connection_factory);

    #[cfg(not(any(feature = "database-postgres", feature = "database-sqlite")))]
    return Err(DaemonError::with_message(
        "A database backend is required to be active. Supported backends are postgreSQL and SQLite",
    ));

    #[cfg(any(feature = "database-postgres", feature = "database-sqlite"))]
    let (store_state, db_handler, previous_commits): (_, Box<dyn EventHandler>, Vec<Commit>) = {
        let connection_uri = config
            .database_url()
            .parse()
            .map_err(|err| DaemonError::from_source(Box::new(err)))?;

        match connection_uri {
            #[cfg(feature = "database-postgres")]
            ConnectionUri::Postgres(_) => {
                let connection_pool: ConnectionPool<diesel::pg::PgConnection> =
                    ConnectionPool::new(config.database_url())
                        .map_err(|err| DaemonError::from_source(Box::new(err)))?;
                let store_factory = create_store_factory(&connection_uri)
                    .map_err(|err| DaemonError::from_source(Box::new(err)))?;
                let event_handler = DatabaseEventHandler::new(store_factory);

                let commit_store = DieselCommitStore::new(connection_pool.pool.clone());
                let commits = commit_store
                    .get_current_service_commits()
                    .map_err(|err| DaemonError::from_source(Box::new(err)))?;

                (
                    StoreState::with_pg_pool(connection_pool.pool),
                    Box::new(event_handler),
                    commits,
                )
            }
            #[cfg(feature = "database-sqlite")]
            ConnectionUri::Sqlite(_) => {
                let connection_pool: ConnectionPool<diesel::sqlite::SqliteConnection> =
                    ConnectionPool::new(config.database_url())
                        .map_err(|err| DaemonError::from_source(Box::new(err)))?;
                let store_factory = create_store_factory(&connection_uri)
                    .map_err(|err| DaemonError::from_source(Box::new(err)))?;
                let event_handler = DatabaseEventHandler::new(store_factory);

                let commit_store = DieselCommitStore::new(connection_pool.pool.clone());
                let commits = commit_store
                    .get_current_service_commits()
                    .map_err(|err| DaemonError::from_source(Box::new(err)))?;

                (
                    StoreState::with_sqlite_pool(connection_pool.pool),
                    Box::new(event_handler),
                    commits,
                )
            }
        }
    };

    let (event_tx, event_rx) = std::sync::mpsc::channel();
    let chan_event_handler: Box<dyn EventHandler> = Box::new(ChannelEventHandler {
        sender: event_tx.clone(),
    });

    let db_event_handler_join_handler = std::thread::Builder::new()
        .name("db-event-handler-splinter".into())
        .spawn(move || loop {
            match event_rx.recv() {
                Ok(EventCmd::Event(evt)) => {
                    if let Err(err) = db_handler.handle_event(&evt) {
                        error!("{}", err.to_string());
                    }
                }
                Ok(EventCmd::Exit) => break,
                Err(_) => break,
            }
        })
        .map_err(|_| DaemonError::with_message("Unable to spawn db handler thread"))?;

    for commit in previous_commits {
        if let Some(service_id) = commit.service_id {
            let service_id = match ServiceId::try_from(service_id.deref()) {
                Ok(service_id) => service_id,
                Err(_) => {
                    warn!(
                        "\"{}\" does not conform to the Splinter service id format; skipping",
                        service_id
                    );
                    continue;
                }
            };

            debug!(
                "Reconnecting event processing on service {} (from {})",
                service_id, commit.commit_id
            );

            event_processors
                .add_once(
                    service_id.circuit_id,
                    service_id.service_id,
                    Some(&commit.commit_id),
                    || vec![chan_event_handler.cloned_box()],
                )
                .map_err(|err| DaemonError::from_source(Box::new(err)))?;
        }
    }

    app_auth_handler::run(
        splinter_endpoint.url(),
        event_processors,
        chan_event_handler,
        reactor.igniter(),
        scabbard_admin_key.to_string(),
    )
    .map_err(|err| DaemonError::from_source(Box::new(err)))?;

    #[cfg(feature = "cylinder-jwt-support")]
    let signer = Secp256k1Context::new().new_signer(gridd_key);

    #[cfg(feature = "cylinder-jwt-support")]
    let backend_client =
        SplinterBackendClient::new(splinter_endpoint.url(), Arc::new(Mutex::new(signer)));
    #[cfg(not(feature = "cylinder-jwt-support"))]
    let backend_client = SplinterBackendClient::new(splinter_endpoint.url());
    let backend_state = BackendState::new(Arc::new(backend_client));

    #[cfg(feature = "integration")]
    let key_state = KeyState::new(config.key_file_name());

    #[cfg(feature = "rest-api")]
    let (rest_api_shutdown_handle, rest_api_join_handle) = rest_api::run(
        config.rest_api_endpoint(),
        store_state,
        backend_state,
        #[cfg(feature = "integration")]
        key_state,
        splinter_endpoint,
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

        #[cfg(feature = "rest-api")]
        rest_api_shutdown_handle.shutdown();
        if let Err(err) = event_tx.send(EventCmd::Exit) {
            error!(
                "Unable to signal shutdown to the DB event handler thread: {}",
                err
            );
        }
    })
    .map_err(|err| DaemonError::from_source(Box::new(err)))?;

    #[cfg(feature = "rest-api")]
    rest_api_join_handle
        .join()
        .map_err(|_| DaemonError::with_message("Unable to cleanly join the REST API thread"))
        .and_then(|res| res.map_err(|err| DaemonError::from_source(Box::new(err))))?;

    if db_event_handler_join_handler.join().is_err() {
        error!("Unable to cleanly join the DB event handler thread");
    }

    if let Err(err) = reactor.wait_for_shutdown() {
        error!("Unable to shutdown splinter event reactor: {}", err);
    }

    Ok(())
}

struct ServiceId<'a> {
    circuit_id: &'a str,
    service_id: &'a str,
}

impl<'a> TryFrom<&'a str> for ServiceId<'a> {
    type Error = InvalidStateError;

    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        let mut splits = s.split("::");

        match (splits.next(), splits.next()) {
            (Some(circuit_id), Some(service_id)) => Ok(ServiceId {
                circuit_id,
                service_id,
            }),
            (Some(_), None) => Err(InvalidStateError::with_message(
                "Service ID must include {circuit_id}::{service_id}".into(),
            )),
            // The first value can never be None, when using split
            (None, _) => unreachable!(),
        }
    }
}

impl<'a> std::fmt::Display for ServiceId<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}::{}", self.circuit_id, self.service_id)
    }
}
