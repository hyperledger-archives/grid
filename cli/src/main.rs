// Copyright 2018-2020 Cargill Incorporated
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
mod store;

use clap::{clap_app, AppSettings, Arg, SubCommand};
use flexi_logger::{DeferredNow, LogSpecBuilder, Logger};
use log::Record;

use action::{admin, certs, Action, SubcommandActions};
use error::CliError;

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
    let mut app = clap_app!(myapp =>
        (name: APP_NAME)
        (version: VERSION)
        (author: "Cargill")
        (about: "Command line for Splinter")
        (@arg verbose: -v +multiple +global "Log verbosely")
        (@arg quiet: -q --quiet +global "Do not display output")
        (@setting SubcommandRequiredElseHelp)
        (@subcommand admin =>
            (about: "Administrative commands")
            (@subcommand keygen =>
                (about: "Generates secp256k1 keys to use when signing circuit proposals")
                (@arg key_name: +takes_value "Name of the key to create; defaults to \"splinter\"")
                (@arg key_dir: -d --("key-dir") +takes_value
                 "Name of the directory in which to create the keys; defaults to current working directory")
                (@arg force: --force "Overwrite files if they exist")
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
            )
        )
    );

    #[cfg(feature = "keygen")]
    {
        app = app.subcommand(
            SubCommand::with_name("keygen")
                .about(
                    "Generates secp256k1 keys. By default, keys are stored in\n\
                     the user's home directory, $HOME/splinter/keys. The --system\n\
                     option generates keys for the Splinter daemon (splinterd) that\n\
                     are stored in /etc/splinter/keys.",
                )
                .arg(
                    Arg::with_name("key-name")
                        .takes_value(true)
                        .help("Name of keys generated; defaults to user name"),
                )
                .arg(
                    Arg::with_name("force")
                        .short("f")
                        .long("force")
                        .help("Overwrite files if they exist"),
                )
                .arg(
                    Arg::with_name("system")
                        .long("system")
                        .help("Generate system keys in /etc/splinter/keys"),
                ),
        )
    }

    #[cfg(feature = "health")]
    {
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
        let create_circuit = SubCommand::with_name("create")
            .about("Propose that a new circuit is created")
            .arg(
                Arg::with_name("url")
                    .short("U")
                    .long("url")
                    .takes_value(true)
                    .help("URL of Splinter Daemon"),
            )
            .arg(
                Arg::with_name("key")
                    .value_name("private-key-file")
                    .short("k")
                    .long("key")
                    .takes_value(true)
                    .help("Path to private key file"),
            )
            .arg(
                Arg::with_name("node")
                    .long("node")
                    .takes_value(true)
                    .required(true)
                    .multiple(true)
                    .long_help(
                        "Node that are part of the circuit. \
                         Format: <node_id>::<endpoint>. \
                         Endpoint is optional if node alias has been set.",
                    ),
            )
            .arg(
                Arg::with_name("service")
                    .long("service")
                    .takes_value(true)
                    .multiple(true)
                    .min_values(2)
                    .required(true)
                    .long_help(
                        "Service ID and allowed node. \
                         Format <service-id>::<allowed_nodes>",
                    ),
            )
            .arg(
                Arg::with_name("service_argument")
                    .long("service-arg")
                    .takes_value(true)
                    .multiple(true)
                    .long_help(
                        "Special arguments to be passed to the service. \
                         Format <service_id>::<key>=<value>",
                    ),
            )
            .arg(
                Arg::with_name("service_peer_group")
                    .long("service-peer-group")
                    .takes_value(true)
                    .multiple(true)
                    .help("List of peer services"),
            )
            .arg(
                Arg::with_name("management_type")
                    .long("management")
                    .takes_value(true)
                    .help("Management type for the circuit"),
            )
            .arg(
                Arg::with_name("service_type")
                    .long("service-type")
                    .takes_value(true)
                    .multiple(true)
                    .long_help(
                        "Service type for a service. \
                         Format <service-id>::<service_type>",
                    ),
            )
            .arg(
                Arg::with_name("metadata")
                    .long("metadata")
                    .value_name("application_metadata")
                    .takes_value(true)
                    .multiple(true)
                    .help("Application metadata for the circuit proposal"),
            )
            .arg(
                Arg::with_name("metadata_encoding")
                    .long("metadata-encoding")
                    .takes_value(true)
                    .possible_values(&["json", "string"])
                    .default_value("string")
                    .requires("metadata")
                    .help("Set the encoding for the application metadata"),
            );

        #[cfg(feature = "circuit-auth-type")]
        let create_circuit = create_circuit.arg(
            Arg::with_name("authorization_type")
                .long("auth-type")
                .possible_values(&["trust"])
                .default_value("trust")
                .takes_value(true)
                .help("Authorization type for the circuit"),
        );

        app = app.subcommand(
            SubCommand::with_name("circuit")
                .about("Provides circuit management functionality")
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .subcommand(create_circuit)
                .subcommand(
                    SubCommand::with_name("vote")
                        .about("Vote on a new circuit proposal")
                        .arg(
                            Arg::with_name("url")
                                .short("U")
                                .long("url")
                                .takes_value(true)
                                .help("URL of Splinter Daemon"),
                        )
                        .arg(
                            Arg::with_name("private_key_file")
                                .value_name("private-key-file")
                                .short("k")
                                .long("key")
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
                                .help("Accept the proposal"),
                        )
                        .arg(
                            Arg::with_name("reject")
                                .required(true)
                                .long("reject")
                                .conflicts_with("accept")
                                .help("Reject the proposal"),
                        ),
                )
                .subcommand(
                    SubCommand::with_name("list")
                        .about("List the circuits")
                        .arg(
                            Arg::with_name("url")
                                .short("U")
                                .long("url")
                                .help("The URL of the Splinter daemon REST API")
                                .takes_value(true),
                        )
                        .arg(
                            Arg::with_name("member")
                                .short("m")
                                .long("member")
                                .help("Filter the circuits by a node ID in the member list")
                                .takes_value(true),
                        )
                        .arg(
                            Arg::with_name("format")
                                .short("f")
                                .long("format")
                                .help("Output format")
                                .possible_values(&["human", "csv"])
                                .default_value("human")
                                .takes_value(true),
                        ),
                )
                .subcommand(
                    SubCommand::with_name("show")
                        .about("Show a specific circuit or proposal")
                        .arg(
                            Arg::with_name("url")
                                .short("U")
                                .long("url")
                                .help("The URL of the Splinter daemon REST API")
                                .takes_value(true),
                        )
                        .arg(
                            Arg::with_name("circuit")
                                .help("The circuit ID of the circuit to be shown")
                                .takes_value(true),
                        )
                        .arg(
                            Arg::with_name("format")
                                .short("f")
                                .long("format")
                                .help("Output format")
                                .possible_values(&["human", "yaml", "json"])
                                .default_value("human")
                                .takes_value(true),
                        ),
                )
                .subcommand(
                    SubCommand::with_name("proposals")
                        .about("List the circuit proposals")
                        .arg(
                            Arg::with_name("url")
                                .short("U")
                                .long("url")
                                .help("The URL of the Splinter daemon REST API")
                                .takes_value(true),
                        )
                        .arg(
                            Arg::with_name("management_type")
                                .short("m")
                                .long("management-type")
                                .long_help(
                                    "Filter the circuit proposals by the circuit \
                                     management type of the circuits",
                                )
                                .takes_value(true),
                        )
                        .arg(
                            Arg::with_name("format")
                                .short("f")
                                .long("format")
                                .help("Output format")
                                .possible_values(&["human", "csv"])
                                .default_value("human")
                                .takes_value(true),
                        ),
                )
                .subcommand(
                    SubCommand::with_name("default")
                        .about("Manage default values for circuit creation")
                        .setting(AppSettings::SubcommandRequiredElseHelp)
                        .subcommand(
                            SubCommand::with_name("set")
                                .about("Set a default value")
                                .arg(
                                    Arg::with_name("name")
                                        .takes_value(true)
                                        .value_name("name")
                                        .possible_values(&["service-type", "management-type"])
                                        .help("The name of the default setting"),
                                )
                                .arg(
                                    Arg::with_name("value")
                                        .takes_value(true)
                                        .value_name("value")
                                        .help("The value for the default setting"),
                                )
                                .arg(
                                    Arg::with_name("force")
                                        .short("f")
                                        .long("force")
                                        .help("Overwrite default if it is already set"),
                                ),
                        )
                        .subcommand(
                            SubCommand::with_name("unset")
                                .about("Unset a default value")
                                .arg(
                                    Arg::with_name("name")
                                        .takes_value(true)
                                        .value_name("name")
                                        .possible_values(&["service-type", "management-type"])
                                        .help("The name of the default setting"),
                                ),
                        )
                        .subcommand(SubCommand::with_name("list").about("List set default values"))
                        .subcommand(
                            SubCommand::with_name("show")
                                .about("Show a default value")
                                .arg(
                                    Arg::with_name("name")
                                        .takes_value(true)
                                        .value_name("name")
                                        .possible_values(&["service-type", "management-type"])
                                        .help("The name of the default setting"),
                                ),
                        ),
                ),
        );
    }

    #[cfg(feature = "node-alias")]
    {
        app = app.subcommand(
            SubCommand::with_name("node")
                .about("Provides node management functionality")
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .subcommand(
                    SubCommand::with_name("alias")
                        .about("Manage node alias")
                        .setting(AppSettings::SubcommandRequiredElseHelp)
                        .subcommand(
                            SubCommand::with_name("add")
                                .about("Add a new node alias")
                                .arg(
                                    Arg::with_name("alias")
                                        .takes_value(true)
                                        .help("Alias for the node"),
                                )
                                .arg(
                                    Arg::with_name("node_id")
                                        .takes_value(true)
                                        .value_name("node-id")
                                        .help("ID of the node"),
                                )
                                .arg(
                                    Arg::with_name("endpoint")
                                        .takes_value(true)
                                        .help("Endpoint for the node"),
                                )
                                .arg(
                                    Arg::with_name("force")
                                        .short("f")
                                        .long("force")
                                        .help("Overwrite alias data if it already exists"),
                                ),
                        )
                        .subcommand(SubCommand::with_name("list").about("List all node alias"))
                        .subcommand(
                            SubCommand::with_name("show")
                                .about("Show endpoint for a node")
                                .arg(
                                    Arg::with_name("alias")
                                        .takes_value(true)
                                        .help("Alias for the node"),
                                ),
                        )
                        .subcommand(
                            SubCommand::with_name("delete")
                                .about("Delete alias for a node")
                                .arg(
                                    Arg::with_name("alias")
                                        .takes_value(true)
                                        .help("Alias for the node"),
                                ),
                        ),
                ),
        );
    }

    let matches = app.get_matches();

    // set default to info
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
    log_spec_builder.module("reqwest", log::LevelFilter::Warn);
    log_spec_builder.module("hyper", log::LevelFilter::Warn);
    log_spec_builder.module("mio", log::LevelFilter::Warn);
    log_spec_builder.module("want", log::LevelFilter::Warn);

    Logger::with(log_spec_builder.build())
        .format(log_format)
        .start()
        .expect("Failed to create logger");

    let mut subcommands = SubcommandActions::new()
        .with_command(
            "admin",
            SubcommandActions::new()
                .with_command("keygen", admin::AdminKeyGenAction)
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
                .with_command("vote", circuit::CircuitVoteAction)
                .with_command("list", circuit::CircuitListAction)
                .with_command("show", circuit::CircuitShowAction)
                .with_command("proposals", circuit::CircuitProposalsAction)
                .with_command(
                    "default",
                    SubcommandActions::new()
                        .with_command("set", circuit::defaults::SetDefaultValueAction)
                        .with_command("unset", circuit::defaults::UnsetDefaultValueAction)
                        .with_command("list", circuit::defaults::ListDefaultsAction)
                        .with_command("show", circuit::defaults::ShowDefaultValueAction),
                ),
        );
    }

    #[cfg(feature = "node-alias")]
    {
        use action::node;
        subcommands = subcommands.with_command(
            "node",
            SubcommandActions::new().with_command(
                "alias",
                SubcommandActions::new()
                    .with_command("add", node::AddNodeAliasAction)
                    .with_command("show", node::ShowNodeAliasAction)
                    .with_command("list", node::ListNodeAliasAction)
                    .with_command("delete", node::DeleteNodeAliasAction),
            ),
        )
    }

    #[cfg(feature = "keygen")]
    {
        use action::keygen;
        subcommands = subcommands.with_command("keygen", keygen::KeyGenAction);
    }

    subcommands.run(Some(&matches))
}

fn main() {
    if let Err(e) = run() {
        error!("ERROR: {}", e);
        std::process::exit(1);
    }
}
