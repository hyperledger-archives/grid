/*
 * Copyright 2019 Bitwise IO, Inc.
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

#[macro_use]
extern crate clap;
#[macro_use]
extern crate diesel;
extern crate diesel_migrations;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_json;
#[cfg(feature = "splinter-support")]
#[macro_use]
extern crate serde;

mod config;
mod database;
mod error;
mod event;
mod rest_api;
#[cfg(feature = "sawtooth-support")]
mod sawtooth;
#[cfg(feature = "splinter-support")]
mod splinter;
mod submitter;

use std::process;
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(feature = "splinter-support")]
use ::splinter::events::Reactor;
use flexi_logger::{LogSpecBuilder, Logger};

use crate::config::{GridConfig, GridConfigBuilder};
use crate::database::{ConnectionPool, DatabaseError};
use crate::error::DaemonError;
use crate::event::{db_handler::DatabaseEventHandler, EventProcessor};
#[cfg(feature = "sawtooth-support")]
use crate::sawtooth::{batch_submitter::SawtoothBatchSubmitter, connection::SawtoothConnection};
#[cfg(feature = "splinter-support")]
use crate::splinter::{
    app_auth_handler, batch_submitter::SplinterBatchSubmitter,
    event::ScabbardEventConnectionFactory, key::load_scabbard_admin_key,
};
use grid_sdk::grid_db::commits::store::CommitStore;
use grid_sdk::store::create_store_factory;
use grid_sdk::store::ConnectionUri;

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn run() -> Result<(), DaemonError> {
    let matches = clap_app!(myapp =>
        (name: APP_NAME)
        (version: VERSION)
        (author: "Contributors to Hyperledger Grid")
        (about: "Daemon Package for Hyperledger Grid")
        (@arg connect: -C --connect +takes_value "connection endpoint for sawtooth or splinter")
        (@arg verbose: -v +multiple "Log verbosely")
        (@arg database_url: --("database-url") +takes_value
         "specifies the database URL to connect to.")
        (@arg bind: -b --bind +takes_value "connection endpoint for rest API")
        (@arg admin_key_dir: --("admin-key-dir") +takes_value "directory containing the Scabbard admin key files"))
    .get_matches();

    let log_level = match matches.occurrences_of("verbose") {
        0 => log::LevelFilter::Warn,
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };
    let mut log_spec_builder = LogSpecBuilder::new();
    log_spec_builder.default(log_level);
    log_spec_builder.module("hyper", log::LevelFilter::Warn);
    log_spec_builder.module("tokio", log::LevelFilter::Warn);

    Logger::with(log_spec_builder.build()).start()?;

    let config = GridConfigBuilder::default()
        .with_cli_args(&matches)
        .build()?;

    #[cfg(feature = "postgres")]
    let connection_pool: ConnectionPool<diesel::pg::PgConnection> =
        ConnectionPool::new(config.database_url())?;
    let connection_uri = config
        .database_url()
        .parse()
        .map_err(|err| DaemonError::StartUpError(Box::new(err)))?;

    let store_factory = create_store_factory(&connection_uri)
        .map_err(|err| DaemonError::StartUpError(Box::new(err)))?;
    let commit_store = store_factory.get_grid_commit_store();

    match connection_uri {
        ConnectionUri::Memory => {
            if config.endpoint().is_sawtooth() {
                run_sawtooth(config, connection_pool, &commit_store)?;
            } else if config.endpoint().is_splinter() {
                run_splinter(config, connection_pool, &commit_store)?;
            } else {
                return Err(DaemonError::UnsupportedEndpoint(format!(
                    "Unsupported endpoint type: {}",
                    config.endpoint().url()
                )));
            };
        }
        ConnectionUri::Postgres(_url) => {
            if config.endpoint().is_sawtooth() {
                run_sawtooth(config, connection_pool, &commit_store)?;
            } else if config.endpoint().is_splinter() {
                run_splinter(config, connection_pool, &commit_store)?;
            } else {
                return Err(DaemonError::UnsupportedEndpoint(format!(
                    "Unsupported endpoint type: {}",
                    config.endpoint().url()
                )));
            }
        }
        ConnectionUri::Sqlite(_conn_str) => {
            if config.endpoint().is_sawtooth() {
                run_sawtooth(config, connection_pool, &commit_store)?;
            } else if config.endpoint().is_splinter() {
                run_splinter(config, connection_pool, &commit_store)?;
            } else {
                return Err(DaemonError::UnsupportedEndpoint(format!(
                    "Unsupported endpoint type: {}",
                    config.endpoint().url()
                )));
            }
        }
    }
    Ok(())
}

#[cfg(all(feature = "sawtooth-support", feature = "postgres"))]
fn run_sawtooth(
    config: GridConfig,
    connection_pool: ConnectionPool<diesel::pg::PgConnection>,
    commit_store: &dyn CommitStore,
) -> Result<(), DaemonError> {
    let sawtooth_connection = SawtoothConnection::new(&config.endpoint().url());
    let current_commit =
        commit_store
            .get_current_commit_id()
            .map_err(|err| DatabaseError::ConnectionError {
                context: "Could not get current commit ID".to_string(),
                source: Box::new(err),
            })?;

    let batch_submitter = Box::new(SawtoothBatchSubmitter::new(
        sawtooth_connection.get_sender(),
    ));

    let (rest_api_shutdown_handle, rest_api_join_handle) = rest_api::run(
        config.rest_api_endpoint(),
        connection_pool.clone(),
        batch_submitter,
        config.endpoint().clone(),
    )?;

    let evt_processor = EventProcessor::start(
        sawtooth_connection,
        current_commit.as_deref(),
        event_handlers![DatabaseEventHandler::new(connection_pool)],
    )
    .map_err(|err| DaemonError::EventProcessorError(Box::new(err)))?;

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

#[cfg(all(not(feature = "sawtooth-support"), feature = "postgres"))]
fn run_sawtooth(
    config: GridConfig,
    connection_pool: ConnectionPool<diesel::pg::PgConnection>,
    commit_store: &dyn CommitStore,
) -> Result<(), DaemonError> {
    Err(DaemonError::UnsupportedEndpoint(format!(
        "A Sawtooth connection endpoint ({}) was provided but Sawtooth support is not enabled for this binary.",
        config.endpoint().url()
    )))
}

#[cfg(all(feature = "splinter-support", feature = "postgres"))]
fn run_splinter(
    config: GridConfig,
    connection_pool: ConnectionPool<diesel::pg::PgConnection>,
    _commit_store: &dyn CommitStore,
) -> Result<(), DaemonError> {
    let reactor = Reactor::new();

    let scabbard_admin_key = load_scabbard_admin_key(&config.admin_key_dir())
        .map_err(|err| DaemonError::StartUpError(Box::new(err)))?;

    let scabbard_event_connection_factory =
        ScabbardEventConnectionFactory::new(&config.endpoint().url(), reactor.igniter());

    app_auth_handler::run(
        config.endpoint().url(),
        scabbard_event_connection_factory,
        connection_pool.clone(),
        reactor.igniter(),
        scabbard_admin_key,
    )?;

    let batch_submitter = Box::new(SplinterBatchSubmitter::new(config.endpoint().url()));

    let (rest_api_shutdown_handle, rest_api_join_handle) = rest_api::run(
        config.rest_api_endpoint(),
        connection_pool,
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

#[cfg(all(not(feature = "splinter-support"), feature = "postgres"))]
fn run_splinter(
    config: GridConfig,
    _connection_pool: ConnectionPool<diesel::pg::PgConnection>,
    _commit_store: &dyn CommitStore,
) -> Result<(), DaemonError> {
    Err(DaemonError::UnsupportedEndpoint(format!(
        "A Splinter connection endpoint ({}) was provided but Splinter support is not enabled for this binary.",
        config.endpoint().url()
    )))
}

fn main() {
    if let Err(e) = run() {
        error!("{}", e);
        std::process::exit(1);
    }
}
