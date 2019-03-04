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

mod actions;
mod error;

use crate::actions::{do_add, do_show};
use crate::error::CliError;

use ::log::LogLevel;
use ::log::{error, log};
use clap::clap_app;

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn run() -> Result<(), CliError> {
    let matches = clap_app!(myapp =>
        (name: APP_NAME)
        (version: VERSION)
        (author: "Cargill")
        (about: "Command line to interact with the Private Counter")
        (@arg url: --url  +takes_value "Private Counter url, ex 127.0.0.1:800")
        (@arg verbose: -v +multiple "Log verbosely")
        (@setting SubcommandRequiredElseHelp)
        (@subcommand show =>
            (about: "Show the current value")
        )
        (@subcommand add =>
            (about: "Add the provided integer to the current value")
            (@arg value: +takes_value "The integer to add, must be an u32")
        )
    )
    .get_matches();

    let logger = match matches.occurrences_of("verbose") {
        0 => simple_logger::init_with_level(LogLevel::Warn),
        1 => simple_logger::init_with_level(LogLevel::Info),
        _ => simple_logger::init_with_level(LogLevel::Debug),
    };

    logger.expect("Failed to create logger");

    let url = matches.value_of("url").unwrap_or("localhost:8000");

    match matches.subcommand() {
        ("show", Some(_)) => do_show(url).map_err(|err| err),
        ("add", Some(m)) => do_add(url, m.value_of("value").unwrap()).map_err(|err| err),
        _ => Err(CliError::InvalidSubcommand),
    }
}

fn main() {
    if let Err(e) = run() {
        error!("{:?}", e);
        std::process::exit(1);
    }
}
