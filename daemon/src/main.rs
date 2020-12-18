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

#[macro_use]
extern crate clap;
extern crate diesel;
extern crate diesel_migrations;
#[macro_use]
extern crate log;
#[cfg(any(
    feature = "pike",
    feature = "schema",
    feature = "product",
    feature = "location"
))]
#[macro_use]
extern crate serde_json;
#[cfg(feature = "splinter-support")]
#[macro_use]
extern crate serde;

mod config;
#[cfg(feature = "database")]
mod database;
mod error;
#[cfg(feature = "event")]
#[macro_use]
mod event;
#[cfg(feature = "rest-api")]
mod rest_api;
#[cfg(feature = "sawtooth-support")]
mod sawtooth;
#[cfg(feature = "splinter-support")]
mod splinter;
#[cfg(feature = "submitter")]
mod submitter;

use flexi_logger::{LogSpecBuilder, Logger};

use crate::config::{Backend, GridConfigBuilder};
use crate::error::DaemonError;
#[cfg(feature = "sawtooth-support")]
use crate::sawtooth::run_sawtooth;
#[cfg(feature = "splinter-support")]
use crate::splinter::run_splinter;

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn run() -> Result<(), DaemonError> {
    #[allow(unused_mut)]
    let mut app = clap_app!(myapp =>
        (name: APP_NAME)
        (version: VERSION)
        (author: "Contributors to Hyperledger Grid")
        (about: "Daemon Package for Hyperledger Grid")
        (@arg connect: -C --connect +takes_value "connection endpoint for sawtooth or splinter")
        (@arg verbose: -v +multiple "Log verbosely")
        (@arg database_url: --("database-url") +takes_value
         "specifies the database URL to connect to.")
        (@arg bind: -b --bind +takes_value "connection endpoint for rest API")
        (@arg admin_key_dir: --("admin-key-dir") +takes_value "directory containing the Scabbard admin key files"));

    #[cfg(feature = "integration")]
    {
        use clap::Arg;
        app = app.arg(
            Arg::with_name("key")
                .short("k")
                .long("key")
                .takes_value(true)
                .help("Base name for private signing key file"),
        );
    }

    let matches = app.get_matches();

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

    match config.endpoint().backend() {
        Backend::Sawtooth => {
            #[cfg(feature = "sawtooth-support")]
            {
                run_sawtooth(config)?;
                Ok(())
            }
            #[cfg(not(feature = "sawtooth-support"))]
            Err(DaemonError::UnsupportedEndpoint(format!(
                "A Sawtooth connection endpoint ({}) was provided but Sawtooth support is not enabled for this binary.",
                config.endpoint().url()
            )))
        }
        Backend::Splinter => {
            #[cfg(feature = "splinter-support")]
            {
                run_splinter(config)?;
                Ok(())
            }
            #[cfg(not(feature = "splinter-support"))]
            Err(DaemonError::UnsupportedEndpoint(format!(
                "A Splinter connection endpoint ({}) was provided but Splinter support is not enabled for this binary.",
                config.endpoint().url()
            )))
        }
    }
}

fn main() {
    if let Err(e) = run() {
        error!("{}", e);
        std::process::exit(1);
    }
}
