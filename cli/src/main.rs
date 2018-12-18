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

#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
extern crate libsplinter;
extern crate messaging;
extern crate protobuf;
extern crate simple_logger;
extern crate splinter_client;

mod actions;
mod error;

use actions::{do_create_circuit, do_destroy_circuit, do_gossip};
use error::CliError;
use log::LogLevel;

const APP_NAME: &'static str = env!("CARGO_PKG_NAME");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn run() -> Result<(), CliError> {
    let matches = clap_app!(myapp =>
        (name: APP_NAME)
        (version: VERSION)
        (author: "Cargill")
        (about: "Command line for Splinter")
        (@arg url: --url  +takes_value "Splinter node url")
        (@arg verbose: -v +multiple "Log verbosely")
        (@setting SubcommandRequiredElseHelp)
        (@subcommand circuit =>
            (about: "Circuit commands")
            (@setting SubcommandRequiredElseHelp)
            (@subcommand create =>
                (about: "Create a new circuit")
                (@arg name: -n +required +takes_value "Name of circuit")
                (@arg participants: -p +takes_value +multiple
                    "The service and the node the service will connect to. service_id,node_url ")
            )
            (@subcommand destroy =>
                (about: "Destroy a circuit")
                (@arg name: -n +takes_value +required "Name of circuit")
            )
            (@subcommand gossip =>
                (about: "Gossip a message to all nodes participating in a circuit")
                (@arg name: -n +required +takes_value "Name of circuit")
                (@arg payload: -d +required +takes_value "File path containing payload")
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
        ("circuit", Some(m)) => match m.subcommand() {
            ("create", Some(m)) => do_create_circuit(
                url,
                m.value_of("name").unwrap(),
                m.values_of("participants")
                    .unwrap_or(clap::Values::default())
                    .map(String::from)
                    .collect(),
            )
            .map_err(CliError::from),
            ("destroy", Some(m)) => {
                do_destroy_circuit(url, m.value_of("name").unwrap()).map_err(CliError::from)
            }
            ("gossip", Some(m)) => do_gossip(
                url,
                m.value_of("name").unwrap(),
                m.value_of("payload").unwrap(),
            )
            .map_err(CliError::from),
            _ => Err(CliError::InvalidSubcommand),
        },
        _ => Err(CliError::InvalidSubcommand),
    }
}

fn main() {
    if let Err(e) = run() {
        error!("{:?}", e);
        std::process::exit(1);
    }
}
