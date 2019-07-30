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

mod config;
mod error;
mod rest_api;

use gameroom_database::ConnectionPool;
use simple_logger;

use crate::config::GameroomConfigBuilder;
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

    match matches.occurrences_of("verbose") {
        0 => simple_logger::init_with_level(log::Level::Warn),
        1 => simple_logger::init_with_level(log::Level::Info),
        _ => simple_logger::init_with_level(log::Level::Debug),
    }?;

    let config = GameroomConfigBuilder::default()
        .with_cli_args(&matches)
        .build()?;

    let connection_pool: ConnectionPool =
        gameroom_database::create_connection_pool(config.database_url())?;

    rest_api::run(
        config.rest_api_endpoint(),
        config.splinterd_url(),
        connection_pool.clone(),
    )?;

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        error!("{}", e);
        std::process::exit(1);
    }
}
