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
#[cfg(feature = "database")]
extern crate diesel;

mod action;
mod error;

use crate::action::{admin, certs, Action, SubcommandActions};
use crate::error::CliError;

use clap::clap_app;
use flexi_logger::{DeferredNow, LogSpecBuilder, Logger};
use log::Record;

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

// log format for cli that will only show the log message
pub fn log_format(
    w: &mut dyn std::io::Write,
    _now: &mut DeferredNow,
    record: &Record,
) -> Result<(), std::io::Error> {
    write!(w, "{}", record.args(),)
}

fn run() -> Result<(), CliError> {
    // ignore unused_mut while there are experimental features
    #[allow(unused_mut)]
    let mut app = clap_app!(myapp =>
        (name: APP_NAME)
        (version: VERSION)
        (author: "Cargill")
        (about: "Command line for Splinter")
        (@arg verbose: -v +multiple "Log verbosely")
        (@setting SubcommandRequiredElseHelp)
        (@subcommand admin =>
            (about: "Administrative commands")
            (@subcommand keygen =>
                (about: "Generates secp256k1 keys to use when signing circuit proposals")
                (@arg key_name: +takes_value "Name of the key to create; defaults to \"splinter\"")
                (@arg key_dir: -d --("key-dir") +takes_value
                 "Name of the directory in which to create the keys; defaults to current working directory")
                (@arg force: --force "Overwrite files if they exist")
                (@arg quiet: -q --quiet "Do not display output")
            )
            (@subcommand keyregistry =>
                (about: "Generates a key registry yaml file and keys, based on a registry \
                 specification")
                (@arg target_dir: -d --("target-dir") +takes_value
                 "Name of the directory in which to create the registry file and keys; \
                 defaults to /var/lib/splinter or the value of SPLINTER_STATE_DIR environment \
                 variable")
                (@arg registry_file: -o --("registry-file") +takes_value
                 "Name of the target registry file (in the target directory); \
                 defaults to \"keys.yaml\"")
                (@arg registry_spec_path: -i --("input-registry-spec") +takes_value
                 "Name of the input key registry specification; \
                 defaults to \"./key_registry_spec.yaml\"")
                (@arg force: --force "Overwrite files if they exist")
                (@arg quiet: -q --quiet "Do not display output")
            )
        )
        (@subcommand cert =>
            (about: "Generate certificates that can be used for development")
            (@subcommand generate =>
                (about: "Generate certificates and keys for the ca, server and client")
                (@arg common_name: --("common-name") +takes_value
                  "The common name that should be used in the generated cert, default localhost")
                (@arg cert_dir: -d --("cert-dir") +takes_value
                  "Name of the directory in which to create the certificates")
                (@arg force: --force  conflicts_with[skip] "Overwrite files if they exist")
                (@arg skip: --skip conflicts_with[force] "Check if files exists, generate if missing")
                (@arg quiet: -q --quiet "Do not display output")
            )
        )
    );

    #[cfg(feature = "health")]
    {
        use clap::{Arg, SubCommand};

        app = app.subcommand(
            SubCommand::with_name("health")
                .about("Displays information about network health")
                .subcommand(
                    SubCommand::with_name("status")
                        .about(
                            "Displays a node's version, endpoint, node id, and a list\n\
                             of endpoints of its connected peers",
                        )
                        .arg(
                            Arg::with_name("url")
                                .short("U")
                                .takes_value(true)
                                .help("URL of node"),
                        ),
                ),
        );
    }

    #[cfg(feature = "database")]
    {
        use clap::{Arg, SubCommand};

        app = app.subcommand(
            SubCommand::with_name("database")
                .about("Database commands")
                .subcommand(
                    SubCommand::with_name("migrate")
                        .about("Runs database migrations for the enabled Splinter features")
                        .arg(
                            Arg::with_name("connect")
                                .short("C")
                                .takes_value(true)
                                .help("Database connection URI"),
                        ),
                ),
        )
    }

    #[cfg(feature = "circuit")]
    {
        use clap::{Arg, SubCommand};

        app = app.subcommand(
            SubCommand::with_name("circuit")
                .about("Provides circuit management functionality")
                .subcommand(
                    SubCommand::with_name("create")
                        .about("Propose that a new circuit is created")
                        .arg(
                            Arg::with_name("url")
                                .short("U")
                                .takes_value(true)
                                .help("URL of Splinter Daemon"),
                        )
                        .arg(
                            Arg::with_name("private_key_file")
                                .value_name("private-key-file")
                                .short("k")
                                .takes_value(true)
                                .help("Path to private key file"),
                        )
                        .arg(
                            Arg::with_name("path")
                                .takes_value(true)
                                .required(true)
                                .help("Path to a yaml file that defines the circuit proposal"),
                        ),
                )
                .subcommand(
                    SubCommand::with_name("vote")
                        .about("Vote on a new circuit proposal")
                        .arg(
                            Arg::with_name("url")
                                .short("U")
                                .takes_value(true)
                                .help("URL of Splinter Daemon"),
                        )
                        .arg(
                            Arg::with_name("private_key_file")
                                .value_name("private-key-file")
                                .short("k")
                                .takes_value(true)
                                .help("Path to private key file"),
                        )
                        .arg(
                            Arg::with_name("circuit_id")
                                .value_name("circuit-id")
                                .takes_value(true)
                                .required(true)
                                .help("The circuit id of the proposed circuit"),
                        )
                        .arg(
                            Arg::with_name("accept")
                                .required(true)
                                .long("accept")
                                .conflicts_with("reject")
                                .possible_values(&["accept", "reject"])
                                .help("Accept the proposal"),
                        )
                        .arg(
                            Arg::with_name("reject")
                                .required(true)
                                .long("reject")
                                .conflicts_with("accept")
                                .possible_values(&["accept", "reject"])
                                .help("Reject the proposal"),
                        ),
                ),
        );
    }

    let matches = app.get_matches();

    // set default to info
    let log_level = match matches.occurrences_of("verbose") {
        0 => log::LevelFilter::Info,
        1 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };

    let mut log_spec_builder = LogSpecBuilder::new();
    log_spec_builder.default(log_level);

    let mut logger_handle = Logger::with(log_spec_builder.build())
        .format(log_format)
        .start()
        .expect("Failed to create logger");

    let mut subcommands = SubcommandActions::new()
        .with_command(
            "admin",
            SubcommandActions::new()
                .with_command("keygen", admin::KeyGenAction)
                .with_command("keyregistry", admin::KeyRegistryGenerationAction),
        )
        .with_command(
            "cert",
            SubcommandActions::new().with_command("generate", certs::CertGenAction),
        );

    #[cfg(feature = "health")]
    {
        use action::health;
        subcommands = subcommands.with_command(
            "health",
            SubcommandActions::new().with_command("status", health::StatusAction),
        );
    }

    #[cfg(feature = "database")]
    {
        use action::database;
        subcommands = subcommands.with_command(
            "database",
            SubcommandActions::new().with_command("migrate", database::MigrateAction),
        )
    }

    #[cfg(feature = "circuit")]
    {
        use action::circuit;
        subcommands = subcommands.with_command(
            "circuit",
            SubcommandActions::new()
                .with_command("create", circuit::CircuitCreateAction)
                .with_command("vote", circuit::CircuitVoteAction),
        );
    }

    subcommands.reconfigure_logging(Some(&matches), &mut logger_handle)?;
    subcommands.run(Some(&matches))
}

fn main() {
    if let Err(e) = run() {
        error!("ERROR: {}", e);
        std::process::exit(1);
    }
}
