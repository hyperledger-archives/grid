// Copyright 2018-2022 Cargill Incorporated
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

#[macro_use]
extern crate log;

mod error;
#[cfg(feature = "griddle-builder")]
pub mod internals;
#[cfg(feature = "rest-api")]
pub mod rest_api;
#[cfg(feature = "key-load")]
pub mod signing;

use std::env;
#[cfg(all(
    any(feature = "database-postgres", feature = "database-sqlite"),
    not(feature = "griddle-builder-run"),
))]
use std::str::FromStr;
#[cfg(feature = "griddle-builder-run")]
use std::sync::mpsc::channel;
#[cfg(not(feature = "griddle-builder-run"))]
use std::sync::Arc;

#[cfg(feature = "griddle-builder-run")]
use clap::ArgMatches;
use clap::{App, Arg};
#[cfg(all(feature = "diesel", not(feature = "griddle-builder-run")))]
use diesel::r2d2::{ConnectionManager, Pool};
use flexi_logger::{DeferredNow, LogSpecBuilder, Logger};
#[cfg(any(feature = "proxy", feature = "griddle-builder-run"))]
use grid_sdk::proxy::{ProxyClient, ReqwestProxyClient};
use grid_sdk::rest_api::actix_web_4::Endpoint;
#[cfg(all(
    any(feature = "database-postgres", feature = "database-sqlite"),
    not(feature = "griddle-builder-run"),
))]
use grid_sdk::store::ConnectionUri;
#[cfg(not(feature = "griddle-builder-run"))]
use grid_sdk::{
    batch_processor::submitter::{
        BatchSubmitter, SawtoothBatchSubmitter, SawtoothConnection, SplinterBatchSubmitter,
    },
    rest_api::actix_web_4::{self, KeyState, StoreState},
};
#[cfg(feature = "griddle-builder-run")]
use grid_sdk::{rest_api::actix_web_4::Backend, threading::lifecycle::ShutdownHandle};
use log::Record;
use users::get_current_username;

#[cfg(feature = "griddle-builder-run")]
use crate::{
    error::Error,
    internals::{DLTBackend, GriddleBuilder, GriddleRestApiVariant},
};
#[cfg(not(feature = "griddle-builder-run"))]
use error::Error;

fn log_format(
    w: &mut dyn std::io::Write,
    _: &mut DeferredNow,
    record: &Record,
) -> Result<(), std::io::Error> {
    write!(w, "{}", record.args(),)
}

#[cfg(not(any(
    feature = "database-postgres",
    feature = "database-sqlite",
    feature = "griddle-builder-run"
)))]
fn griddle_store_state(_db_url: &str) -> Result<StoreState, Error> {
    Err(Error::from_message(
        "no database feature was enabled during compilation",
    ))
}
#[cfg(all(
    any(feature = "database-postgres", feature = "database-sqlite"),
    not(feature = "griddle-builder-run"),
))]
fn griddle_store_state(db_url: &str) -> Result<StoreState, Error> {
    let connection_url =
        ConnectionUri::from_str(db_url).map_err(|err| Error::from_message(&format!("{}", err)))?;

    Ok(match connection_url {
        #[cfg(feature = "database-postgres")]
        ConnectionUri::Postgres(_) => {
            let connection_manager = ConnectionManager::<diesel::pg::PgConnection>::new(db_url);
            let pool_builder = Pool::builder();
            let pool = pool_builder
                .build(connection_manager)
                .map_err(|err| Error::from_message(&format!("{}", err)))?;
            StoreState::with_pg_pool(pool)
        }
        #[cfg(feature = "database-sqlite")]
        ConnectionUri::Sqlite(_) => {
            let connection_manager =
                ConnectionManager::<diesel::sqlite::SqliteConnection>::new(db_url);
            let pool_builder = Pool::builder();
            let pool = pool_builder
                .build(connection_manager)
                .map_err(|err| Error::from_message(&format!("{}", err)))?;

            StoreState::with_sqlite_pool(pool)
        }
    })
}

#[cfg(not(feature = "griddle-builder-run"))]
fn batch_submitter(endpoint: Endpoint) -> Arc<dyn BatchSubmitter> {
    if endpoint.is_sawtooth() {
        let connection = SawtoothConnection::new(&endpoint.url());
        Arc::new(SawtoothBatchSubmitter::new(connection.get_sender()))
    } else {
        Arc::new(SplinterBatchSubmitter::new(&endpoint.url()))
    }
}

#[cfg(not(feature = "griddle-builder-run"))]
async fn run() -> Result<(), Error> {
    #[allow(unused_mut)]
    let mut app = App::new("griddle")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Contributors to Hyperledger Grid")
        .about("Grid Integration Component")
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .multiple(true)
                .global(true)
                .help("Log verbosely"),
        )
        .arg(
            Arg::with_name("quiet")
                .short("q")
                .long("quiet")
                .global(true)
                .conflicts_with("verbose")
                .help("Do not display output"),
        )
        .arg(
            Arg::with_name("bind")
                .short("b")
                .long("bind")
                .takes_value(true)
                .help("Connection endpoint for REST API"),
        )
        .arg(
            Arg::with_name("database_url")
                .long("database-url")
                .takes_value(true)
                .help("URL for datatbase to be used by griddle"),
        )
        .arg(
            Arg::with_name("connect")
                .long("connect")
                .short("C")
                .takes_value(true)
                .help("URL for splinter or sawtooth node to be used by griddle"),
        )
        .arg(
            Arg::with_name("key")
                .short("k")
                .long("key")
                .takes_value(true)
                .help("Base name for private signing key file"),
        );

    #[cfg(feature = "proxy")]
    {
        app = app.arg(
            Arg::with_name("forward_url")
                .long("forward-url")
                .takes_value(true)
                .help("URL for Grid node to be used by griddle"),
        );
    }

    let matches = app.get_matches();

    let log_level = if matches.is_present("quiet") {
        log::LevelFilter::Error
    } else {
        match matches.occurrences_of("verbose") {
            0 => log::LevelFilter::Info,
            1 => log::LevelFilter::Debug,
            _ => log::LevelFilter::Trace,
        }
    };
    let mut log_spec_builder = LogSpecBuilder::new();
    log_spec_builder.default(log_level);

    Logger::with(log_spec_builder.build())
        .format(log_format)
        .start()
        .map_err(|_| Error::from_message("Failed to start logger"))?;

    let key = matches
        .value_of("key")
        .map(String::from)
        .or_else(|| env::var("GRIDDLE_KEY_DIR").ok())
        .or_else(|| get_current_username().and_then(|os_str| os_str.into_string().ok()))
        .ok_or_else(|| {
            Error::from_message("Could not find signing key: unable to determine username")
        })?;

    let bind = matches
        .value_of("bind")
        .map(String::from)
        .or_else(|| env::var("GRIDDLE_BIND").ok())
        .unwrap_or_else(|| "localhost:8000".into());

    let connect = matches
        .value_of("connect")
        .map(String::from)
        .or_else(|| env::var("CONNECT_URL").ok())
        .unwrap_or_else(|| "http://localhost:8085".into());

    let database_url = matches
        .value_of("database_url")
        .map(String::from)
        .or_else(|| env::var("GRIDDLE_DATABASE_URL").ok())
        .unwrap_or_else(|| "sqlite_db_file".into());

    #[cfg(feature = "proxy")]
    let forward_url = matches
        .value_of("forward_url")
        .map(String::from)
        .or_else(|| env::var("GRIDDLE_FORWARD_URL").ok())
        .unwrap_or_else(|| "http://localhost:8080".into());

    #[cfg(feature = "proxy")]
    let client = ReqwestProxyClient::new(&forward_url)
        .map_err(|err| Error::from_message(&format!("Unable to create proxy client: {err}")))?;

    let store_state = griddle_store_state(&database_url)?;
    let key_state = KeyState::new(&key);

    let _batch_submitter = batch_submitter(Endpoint::from(connect.as_ref()));

    actix_web_4::run(
        &bind,
        store_state,
        key_state,
        #[cfg(feature = "proxy")]
        Box::new(client),
    )
    .await
    .map_err(|err| Error::from_message(&format!("{}", err)))?;

    Ok(())
}

#[cfg(not(feature = "griddle-builder-run"))]
#[actix_web::main]
async fn main() {
    if let Err(e) = run().await {
        error!("{}", e);
        std::process::exit(1);
    }
}

#[cfg(feature = "griddle-builder-run")]
fn main() {
    #[allow(unused_mut)]
    let mut app = App::new("griddle")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Contributors to Hyperledger Grid")
        .about("Grid Integration Component")
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .multiple(true)
                .global(true)
                .help("Log verbosely"),
        )
        .arg(
            Arg::with_name("quiet")
                .short("q")
                .long("quiet")
                .global(true)
                .conflicts_with("verbose")
                .help("Do not display output"),
        )
        .arg(
            Arg::with_name("bind")
                .short("b")
                .long("bind")
                .takes_value(true)
                .help("Connection endpoint for REST API"),
        )
        .arg(
            Arg::with_name("connect")
                .long("connect")
                .short("C")
                .takes_value(true)
                .help("URL for splinter or sawtooth node to be used by griddle"),
        )
        .arg(
            Arg::with_name("key")
                .short("k")
                .long("key")
                .takes_value(true)
                .help("Base name for private signing key file"),
        );

    #[cfg(feature = "proxy")]
    {
        app = app.arg(
            Arg::with_name("forward_url")
                .long("forward-url")
                .takes_value(true)
                .help("URL for Grid node to be used by griddle"),
        );
    }

    if let Err(err) = start_griddle(app.get_matches()) {
        error!("Failed to start Griddle, {}", err);
        std::process::exit(1);
    }
}

#[cfg(feature = "griddle-builder-run")]
fn start_griddle(matches: ArgMatches) -> Result<(), Error> {
    let log_level = if matches.is_present("quiet") {
        log::LevelFilter::Error
    } else {
        match matches.occurrences_of("verbose") {
            0 => log::LevelFilter::Info,
            1 => log::LevelFilter::Debug,
            _ => log::LevelFilter::Trace,
        }
    };
    let mut log_spec_builder = LogSpecBuilder::new();
    log_spec_builder.default(log_level);

    Logger::with(log_spec_builder.build())
        .format(crate::log_format)
        .start()
        .map_err(|_| Error::from_message("Failed to start logger"))?;

    let key = matches
        .value_of("key")
        .map(String::from)
        .or_else(|| env::var("GRIDDLE_KEY_DIR").ok())
        .or_else(|| get_current_username().and_then(|os_str| os_str.into_string().ok()));
    let signer = signing::load_signer(key)?;

    let griddle_bind = matches
        .value_of("bind")
        .map(String::from)
        .or_else(|| env::var("GRIDDLE_BIND").ok())
        .unwrap_or_else(|| "localhost:8000".into());

    let backend_connection = matches
        .value_of("connect")
        .map(String::from)
        .or_else(|| env::var("CONNECT_URL").ok())
        .unwrap_or_else(|| "http://localhost:8085".into());
    let backend_endpoint = Endpoint::from(backend_connection.as_ref());

    #[cfg(feature = "proxy")]
    let forward_url = matches
        .value_of("forward_url")
        .map(String::from)
        .or_else(|| env::var("GRIDDLE_FORWARD_URL").ok())
        .unwrap_or_else(|| "http://localhost:8080".into());

    #[cfg(feature = "proxy")]
    let client = ReqwestProxyClient::new(&forward_url)
        .map_err(|err| Error::from_message(&format!("Unable to create proxy client: {err}")))?;

    let mut griddle_builder = GriddleBuilder::default()
        .with_rest_api_variant(GriddleRestApiVariant::ActixWeb4)
        .with_rest_api_endpoint(griddle_bind)
        .with_dlt_backend(convert_endpoint_to_backend(backend_endpoint))
        .with_signer(signer);

    #[cfg(feature = "proxy")]
    {
        griddle_builder = griddle_builder.with_proxy_client(client.cloned_box());
    }

    let mut running_griddle = griddle_builder.build()?.run()?;

    // Set the Ctrl-C handler to shut down Griddle
    let (shutdown_tx, shutdown_rx) = channel();
    ctrlc::set_handler(move || {
        if shutdown_tx.send(()).is_err() {
            // This was the second ctrl-c (as the receiver is dropped after the first one).
            std::process::exit(0);
        }
    })
    .expect("Error setting Ctrl-C handler");

    // recv that value, ignoring the result.
    let _ = shutdown_rx.recv();
    drop(shutdown_rx);
    info!("Initiating graceful shutdown (press Ctrl+C again to force)");

    running_griddle.signal_shutdown();

    if let Err(err) = running_griddle.wait_for_shutdown() {
        error!("Unable to cleanly shut down Griddle: {err}");
    }

    Ok(())
}

#[cfg(feature = "griddle-builder-run")]
fn convert_endpoint_to_backend(backend_endpoint: Endpoint) -> DLTBackend {
    match backend_endpoint.backend {
        Backend::Sawtooth => DLTBackend::Sawtooth,
        Backend::Splinter => DLTBackend::Splinter,
    }
}
