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
extern crate log;

mod action;
mod cert;
mod error;

use crate::error::CliError;
use action::{admin, network, service, Action, SubcommandActions};

use clap::clap_app;
use log::LogLevel;

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn run() -> Result<(), CliError> {
    let matches = clap_app!(myapp =>
        (name: APP_NAME)
        (version: VERSION)
        (author: "Cargill")
        (about: "Command line to test Splinter")
        (@arg verbose: -v +multiple "Log verbosely")
        (@setting SubcommandRequiredElseHelp)
        (@subcommand admin =>
            (about: "Administrative commands")
            (@subcommand keygen =>
                (about: "generates secp256k1 keys to use when signing circuit proposals")
                (@arg key_name: +takes_value "name of the key to create; defaults to \"splinter\"")
                (@arg key_dir: -d --("key-dir") +takes_value
                 "name of the directory in which to create the keys; defaults to current working directory")
                (@arg force: --force "overwrite files if they exist")
                (@arg quiet: -q --quiet "do not display output")
            )
            (@subcommand keyregistry =>
                (about: "generates a key registry yaml file and keys, based on a registry \
                 specification")
                (@arg target_dir: -d --("target-dir") +takes_value
                 "name of the directory in which to create the registry file and keys; \
                 defaults to /var/lib/splinter or the value of SPLINTER_STATE_DIR environment \
                 variable")
                (@arg registry_file: -o --("registry-file") +takes_value
                 "name of the target registry file (in the target directory); \
                 defaults to \"keys.yaml\"")
                (@arg registry_spec_path: -i --("input-registry-spec") +takes_value
                 "name of the input key registry specification; \
                 defaults to \"./key_registry_spec.yaml\"")
                (@arg force: --force "overwrite files if they exist")
                (@arg quiet: -q --quiet "do not display output")
            )
        )
        (@subcommand echo =>
            (about: "Echo message")
            (@arg recipient: +takes_value "Splinter node id to send the message to")
            (@arg ttl: +takes_value "Number of times to echo the message")
        )
        (@subcommand service =>
            (about: "Service messages")
            (@subcommand connect =>
                (about: "Connect a service to circuit")
                (@arg url: --url  +takes_value "Splinter node url")
                (@arg circuit: +takes_value "The circuit name to connect to")
                (@arg service: +takes_value "The id of the service connecting to the node")
            )
            (@subcommand disconnect =>
                (about: "Disconnect a service from circuit")
                (@arg url: --url  +takes_value "Splinter node url")
                (@arg circuit: +takes_value "The circuit name to disconnect from")
                (@arg service: +takes_value "The id of the service disconnecting from the node")
            )
            (@subcommand send =>
                (about: "Connect a service to circuit")
                (@arg url: --url  +takes_value "Splinter node url")
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

    SubcommandActions::new()
        .with_command(
            "admin",
            SubcommandActions::new()
                .with_command("keygen", admin::KeyGenAction)
                .with_command("keyregistry", admin::KeyRegistryGenerationAction),
        )
        .with_command("echo", network::EchoAction)
        .with_command(
            "service",
            SubcommandActions::new()
                .with_command("connect", service::ConnectAction)
                .with_command("disconnect", service::DisconnectAction)
                .with_command("send", service::SendAction),
        )
        .run(Some(&matches))
}

fn main() {
    if let Err(e) = run() {
        error!("{:?}", e);
        std::process::exit(1);
    }
}
