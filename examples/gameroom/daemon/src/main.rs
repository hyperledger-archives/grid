// Copyright 2019 Cargill Incorporated
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
extern crate clap;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

mod application_metadata;
mod authorization_handler;
mod config;
mod error;
mod rest_api;

use flexi_logger::{LogSpecBuilder, Logger};
use gameroom_database::ConnectionPool;
use sawtooth_sdk::signing::create_context;
use splinter::events::Reactor;

use crate::config::{get_node, GameroomConfigBuilder};
use crate::error::GameroomDaemonError;

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn run() -> Result<(), GameroomDaemonError> {
    let matches = clap_app!(myapp =>
        (name: APP_NAME)
        (version: VERSION)
        (author: "Cargill Incorporated")
        (about: "Daemon Package for Gameroom")
        (@arg verbose: -v +multiple "Log verbosely")
        (@arg database_url: --("database-url") +takes_value "Database connection for Gameroom rest API")
        (@arg bind: -b --bind +takes_value "connection endpoint for Gameroom rest API")
        (@arg splinterd_url: --("splinterd-url") +takes_value "connection endpoint to SplinterD rest API")
    )
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
    log_spec_builder.module("trust_dns", log::LevelFilter::Warn);

    Logger::with(log_spec_builder.build()).start()?;

    let config = GameroomConfigBuilder::default()
        .with_cli_args(&matches)
        .build()?;

    let connection_pool: ConnectionPool =
        gameroom_database::create_connection_pool(config.database_url())?;

    // Generate a public/private key pair
    let context = create_context("secp256k1")?;
    let private_key = context.new_random_private_key()?;
    let public_key = context.get_public_key(&*private_key)?;

    // Get splinterd node information
    let node = get_node(config.splinterd_url())?;

    let reactor = Reactor::new();

    authorization_handler::run(
        config.splinterd_url().into(),
        node.identity.clone(),
        connection_pool.clone(),
        private_key.as_hex(),
        reactor.igniter(),
    )?;

    let (rest_api_shutdown_handle, rest_api_join_handle) = rest_api::run(
        config.rest_api_endpoint(),
        config.splinterd_url(),
        node,
        connection_pool.clone(),
        public_key.as_hex(),
    )?;

    ctrlc::set_handler(move || {
        info!("Received Shutdown");

        if let Err(err) = rest_api_shutdown_handle.shutdown() {
            error!("Unable to cleanly shutdown REST API server: {}", err);
        }
    })
    .expect("Error setting Ctrl-C handler");

    let _ = rest_api_join_handle.join();

    if let Err(err) = reactor.shutdown() {
        error!(
            "Unable to cleanly shutdown application authorization handler reactor: {}",
            err
        );
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        error!("{}", e);
        std::process::exit(1);
    }
}
