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
extern crate log;

mod config;
mod error;

use simple_logger;

use crate::config::GridConfigBuilder;
use crate::error::DaemonError;

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn run() -> Result<(), DaemonError> {
    let matches = clap_app!(myapp =>
        (name: APP_NAME)
        (version: VERSION)
        (author: "Contributors to Hyperledger Grid")
        (about: "Daemon Package for Hyperledger Grid")
        (@arg connect: -C --connect +takes_value "connection endpoint for validator")
        (@arg verbose: -v +multiple "Log verbosely")
    )
    .get_matches();

    let config = GridConfigBuilder::default()
        .with_cli_args(&matches)
        .build()?;

    simple_logger::init_with_level(config.log_level())?;

    info!("Connecting to validator at {}", config.validator_endpoint());

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        error!("{:?}", e);
        std::process::exit(1);
    }
}
