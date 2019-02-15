// Copyright 2018 Cargill Incorporated
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

use crate::actions::{do_connect, do_disconnect, do_echo, do_send};
use crate::error::CliError;

use std::str::FromStr;

use ::log::LogLevel;
use ::log::{error, log};
use clap::clap_app;

const APP_NAME: &'static str = env!("CARGO_PKG_NAME");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn run() -> Result<(), CliError> {
    let matches = clap_app!(myapp =>
        (name: APP_NAME)
        (version: VERSION)
        (author: "Cargill")
        (about: "Command line to test Splinter")
        (@arg url: --url  +takes_value "Splinter node url")
        (@arg verbose: -v +multiple "Log verbosely")
        (@setting SubcommandRequiredElseHelp)
        (@subcommand echo =>
            (about: "Echo message")
            (@arg recipient: +takes_value "Splinter node id to send the message to")
            (@arg ttl: +takes_value "Number of times to echo the message")
        )
        (@subcommand service =>
            (about: "Service messages")
            (@subcommand connect =>
                (about: "Connect a service to circuit")
                (@arg circuit: +takes_value "The circuit name to connect to")
                (@arg service: +takes_value "The id of the service connecting to the node")
            )
            (@subcommand disconnect =>
                (about: "Disconnect a service from circuit")
                (@arg circuit: +takes_value "The circuit name to disconnect from")
                (@arg service: +takes_value "The id of the service disconnecting from the node")
            )
            (@subcommand send =>
                (about: "Connect a service to circuit")
                (@arg circuit: +takes_value "The circuit name to connect to")
                (@arg sender: +takes_value "The id of the service sending the message")
                (@arg recipient: +takes_value "The id of the service sending the message")
                (@arg payload: +takes_value "Path to a payload file")
            )
        )
    )
    .get_matches();

    let logger = match matches.occurrences_of("verbose") {
        0 => simple_logger::init_with_level(LogLevel::Warn),
        1 => simple_logger::init_with_level(LogLevel::Info),
        _ => simple_logger::init_with_level(LogLevel::Debug),
    };

    logger.expect("Failed to create logger");

    let url = matches.value_of("url").unwrap_or("tcp://localhost:8045");

    match matches.subcommand() {
        ("echo", Some(m)) => do_echo(
            url,
            m.value_of("recipient").unwrap().to_string(),
            FromStr::from_str(m.value_of("ttl").unwrap()).unwrap(),
        )
        .map_err(CliError::from),
        ("service", Some(m)) => {
            match m.subcommand() {
                ("connect", Some(m)) => do_connect(
                    url,
                    m.value_of("circuit").unwrap().to_string(),
                    m.value_of("service").unwrap().to_string(),
                )
                .map_err(CliError::from),
                ("disconnect", Some(m)) => do_disconnect(
                    url,
                    m.value_of("circuit").unwrap().to_string(),
                    m.value_of("service").unwrap().to_string(),
                )
                .map_err(CliError::from),
                ("send", Some(m)) => do_send(
                    url,
                    m.value_of("circuit").unwrap().to_string(),
                    m.value_of("sender").unwrap().to_string(),
                    m.value_of("recipient").unwrap().to_string(),
                    m.value_of("payload").unwrap().to_string(),
                )
                .map_err(CliError::from),
                _ => Err(CliError::InvalidSubcommand),
            }
            .map_err(CliError::from)
        }
        .map_err(CliError::from),
        _ => Err(CliError::InvalidSubcommand),
    }
}

fn main() {
    if let Err(e) = run() {
        error!("{:?}", e);
        std::process::exit(1);
    }
}
