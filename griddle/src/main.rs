// Copyright 2018-2021 Cargill Incorporated
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

use std::env;
#[cfg(any(feature = "database-postgres", feature = "database-sqlite"))]
use std::str::FromStr;
use std::sync::Arc;

use clap::{App, Arg};
#[cfg(feature = "diesel")]
use diesel::r2d2::{ConnectionManager, Pool};
use flexi_logger::{DeferredNow, LogSpecBuilder, Logger};
#[cfg(any(feature = "database-postgres", feature = "database-sqlite"))]
use grid_sdk::store::ConnectionUri;
use grid_sdk::{
    batch_processor::{
        submitter::{
            BatchSubmitter, SawtoothBatchSubmitter, SawtoothConnection, SplinterBatchSubmitter,
        },
        BatchProcessorBuilder,
    },
    rest_api::actix_web_3::{self, Endpoint, KeyState, StoreState},
};
use log::Record;
use users::get_current_username;

use error::Error;

fn log_format(
    w: &mut dyn std::io::Write,
    _: &mut DeferredNow,
    record: &Record,
) -> Result<(), std::io::Error> {
    write!(w, "{}", record.args(),)
}

#[cfg(not(any(feature = "database-postgres", feature = "database-sqlite")))]
fn griddle_store_state(_db_url: &str) -> Result<StoreState, Error> {
    Err(Error::from_message(
        "no database feature was enabled during compilation",
    ))
}

#[cfg(any(feature = "database-postgres", feature = "database-sqlite"))]
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

fn batch_submitter(endpoint: Endpoint) -> Arc<dyn BatchSubmitter> {
    if endpoint.is_sawtooth() {
        let connection = SawtoothConnection::new(&endpoint.url());
        Arc::new(SawtoothBatchSubmitter::new(connection.get_sender()))
    } else {
        Arc::new(SplinterBatchSubmitter::new(&endpoint.url()))
    }
}

async fn run() -> Result<(), Error> {
    let matches = App::new("griddle")
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
        )
        .get_matches();

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

    let store_state = griddle_store_state(&database_url)?;
    let key_state = KeyState::new(&key);

    let batch_submitter = batch_submitter(Endpoint::from(connect.as_ref()));
    let batch_processor =
        BatchProcessorBuilder::new(store_state.store_factory.clone_box(), batch_submitter);

    batch_processor
        .start()
        .map_err(|err| Error::from_message(&format!("Failed to start batch processor: {}", err)))?;

    actix_web_3::run(&bind, store_state, key_state)
        .await
        .map_err(|err| Error::from_message(&format!("{}", err)))?;

    Ok(())
}

#[actix_web::main]
async fn main() {
    if let Err(e) = run().await {
        error!("{}", e);
        std::process::exit(1);
    }
}
