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
extern crate diesel;
extern crate diesel_migrations;

mod actions;
mod error;
mod http;
mod key;
#[cfg(feature = "sawtooth")]
mod sawtooth;
#[cfg(feature = "splinter")]
mod splinter;
mod transaction;
mod yaml_parser;

use std::{collections::HashMap, env, fs::File, io::prelude::*, path::PathBuf};

use clap::ArgMatches;
use flexi_logger::{DeferredNow, LogSpecBuilder, Logger};
use grid_sdk::protocol::{
    location::payload::{
        LocationCreateActionBuilder, LocationDeleteActionBuilder, LocationNamespace,
        LocationUpdateActionBuilder,
    },
    pike::{
        payload::{
            CreateAgentActionBuilder, CreateOrganizationActionBuilder, UpdateAgentActionBuilder,
            UpdateOrganizationActionBuilder,
        },
        state::{KeyValueEntry, KeyValueEntryBuilder},
    },
    product::{
        payload::{
            ProductCreateActionBuilder, ProductDeleteActionBuilder, ProductUpdateActionBuilder,
        },
        state::ProductNamespace,
    },
    schema::state::{LatLongBuilder, PropertyValue, PropertyValueBuilder},
};
use log::Record;

use crate::error::CliError;

use actions::{agents, database, keygen, locations, organizations as orgs, products, schemas};

#[cfg(feature = "admin-keygen")]
use actions::admin;

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

const GRID_DAEMON_KEY: &str = "GRID_DAEMON_KEY";
const GRID_DAEMON_ENDPOINT: &str = "GRID_DAEMON_ENDPOINT";
const GRID_SERVICE_ID: &str = "GRID_SERVICE_ID";

const SYSTEM_KEY_PATH: &str = "/etc/grid/keys";

// log format for cli that will only show the log message
pub fn log_format(
    w: &mut dyn std::io::Write,
    _now: &mut DeferredNow,
    record: &Record,
) -> Result<(), std::io::Error> {
    write!(w, "{}", record.args(),)
}

fn run() -> Result<(), CliError> {
    #[allow(unused_mut)]
    let mut app = clap_app!(myapp =>
        (name: APP_NAME)
        (version: VERSION)
        (author: "Contributors to Hyperledger Grid")
        (about: "Command line for Hyperledger Grid")
        (@arg verbose: -v +multiple +global "Log verbosely")
        (@arg quiet: -q --quiet +global conflicts_with[verbose] "Do not display output")
        (@subcommand database =>
            (about: "Manage Grid Daemon database")
            (@setting SubcommandRequiredElseHelp)
            (@subcommand migrate =>
                (about: "Run database migrations")
                (@arg connect: -C --("connect") +takes_value
                    "URL for database")
            )
        )
        (@subcommand keygen =>
            (about: "Generates keys with which the user can sign transactions and batches.")
            (@arg key_name: +takes_value "Name of the key to create")
            (@arg force: --force "Overwrite files if they exist")
            (@arg key_dir: -d --key_dir +takes_value conflicts_with[system] "Specify the directory for the key files")
            (@arg system: --system "Generate system keys in /etc/grid/keys")
        )

    );

    #[cfg(feature = "admin-keygen")]
    {
        use clap::{Arg, SubCommand};

        app = app.subcommand(
            SubCommand::with_name("admin")
                .about("Administrative commands for gridd")
                .setting(clap::AppSettings::SubcommandRequiredElseHelp)
                .subcommand(
                    SubCommand::with_name("keygen")
                        .about("Generates keys for gridd to use to sign transactions and batches.")
                        .arg(
                            Arg::with_name("directory")
                                .long("dir")
                                .short("d")
                                .takes_value(true)
                                .help(
                                    "Specify the directory for the key files; \
                                     defaults to /etc/grid/keys",
                                ),
                        )
                        .arg(
                            Arg::with_name("force")
                                .long("force")
                                .conflicts_with("skip")
                                .help("Overwrite files if they exist"),
                        )
                        .arg(
                            Arg::with_name("skip")
                                .long("skip")
                                .conflicts_with("force")
                                .help("Check if files exist; generate if missing"),
                        ),
                ),
        );
    }

    #[cfg(feature = "pike")]
    {
        use clap::{Arg, SubCommand};

        app = app.subcommand(
            SubCommand::with_name("agent")
                .about("Update or create agent")
                .setting(clap::AppSettings::SubcommandRequiredElseHelp)
                .arg(
                    Arg::with_name("service_id")
                        .long("service-id")
                        .takes_value(true)
                        .help(
                            "The ID of the service the payload should be \
                         sent to; required if running on Splinter. Format \
                         <circuit-id>::<service-id>",
                        ),
                )
                .arg(Arg::with_name("url")
                    .long("url")
                    .takes_value(true)
                    .help("URL for the REST API")
                )
                .subcommand(
                    SubCommand::with_name("create")
                        .about("Create an Agent")
                        .arg(
                            Arg::with_name("org_id")
                                .takes_value(true)
                                .required(true)
                                .help("organization ID"),
                        )
                        .arg(
                            Arg::with_name("public_key")
                                .takes_value(true)
                                .required(true)
                                .help("public key"),
                        )
                        .arg(
                            Arg::with_name("active")
                                .long("active")
                                .conflicts_with("inactive")
                                .required_unless("inactive")
                                .help("Set agent as active"),
                        )
                        .arg(
                            Arg::with_name("inactive")
                                .long("inactive")
                                .conflicts_with("active")
                                .required_unless("active")
                                .help("Set agent as inactive"),
                        )
                        .arg(
                            Arg::with_name("role")
                                .long("role")
                                .takes_value(true)
                                .use_delimiter(true)
                                .multiple(true)
                                .help("Roles assigned to agent"),
                        )
                        .arg(
                            Arg::with_name("metadata")
                                .long("metadata")
                                .takes_value(true)
                                .use_delimiter(true)
                                .multiple(true)
                                .help("Key-value pairs (format: <key>=<value>) in a comma-separated list"),
                        )
                        .arg(
                            Arg::with_name("key")
                                .long("key")
                                .short("k")
                                .takes_value(true)
                                .help("Base name for private signing key file")
                        )
                        .arg(
                            Arg::with_name("wait")
                                .long("wait")
                                .takes_value(true)
                                .help("How long to wait for transaction to be committed")
                        ),
                )
                .subcommand(
                    SubCommand::with_name("update")
                        .about("Update an Agent")
                        .arg(
                            Arg::with_name("org_id")
                                .takes_value(true)
                                .required(true)
                                .help("organization ID"),
                        )
                        .arg(
                            Arg::with_name("public_key")
                                .takes_value(true)
                                .required(true)
                                .help("public key"),
                        )
                        .arg(
                            Arg::with_name("active")
                                .long("active")
                                .conflicts_with("inactive")
                                .required_unless("inactive")
                                .help("Set agent as active"),
                        )
                        .arg(
                            Arg::with_name("inactive")
                                .long("inactive")
                                .conflicts_with("active")
                                .required_unless("active")
                                .help("Set agent as inactive"),
                        )
                        .arg(
                            Arg::with_name("role")
                                .long("role")
                                .takes_value(true)
                                .use_delimiter(true)
                                .multiple(true)
                                .help("Roles assigned to agent"),
                        )
                        .arg(
                            Arg::with_name("metadata")
                                .long("metadata")
                                .takes_value(true)
                                .use_delimiter(true)
                                .multiple(true)
                                .help("Key-value pairs (format: <key>=<value>) in a comma-separated list"),
                        )
                        .arg(
                            Arg::with_name("key")
                                .long("key")
                                .short("k")
                                .takes_value(true)
                                .help("Base name for private signing key file")
                        )
                        .arg(
                            Arg::with_name("wait")
                                .long("wait")
                                .takes_value(true)
                                .help("How long to wait for transaction to be committed")
                        ),
                    )

        )
        .subcommand(
            SubCommand::with_name("organization")
                .about("Update or create organization")
                .setting(clap::AppSettings::SubcommandRequiredElseHelp)
                .arg(
                    Arg::with_name("service_id")
                        .long("service-id")
                        .takes_value(true)
                        .help(
                            "The ID of the service the payload should be \
                         sent to; required if running on Splinter. Format \
                         <circuit-id>::<service-id>",
                        ),
                )
                .arg(Arg::with_name("url")
                    .long("url")
                    .takes_value(true)
                    .help("URL for the REST API")
                )
                .subcommand(
                    SubCommand::with_name("create")
                        .about("Create an Organization")
                        .arg(
                            Arg::with_name("org_id")
                                .takes_value(true)
                                .required(true)
                                .help("Unique ID for organization"),
                        )
                        .arg(
                            Arg::with_name("name")
                                .takes_value(true)
                                .required(true)
                                .help("Name of organization"),
                        )
                        .arg(
                            Arg::with_name("address")
                                .takes_value(true)
                                .help("Physical address for organization"),
                        )
                        .arg(
                            Arg::with_name("metadata")
                                .long("metadata")
                                .takes_value(true)
                                .multiple(true)
                                .use_delimiter(true)
                                .help("Key-value pairs (format: <key>=<value>) in a comma-separated list"),
                        )
                        .arg(
                            Arg::with_name("key")
                                .long("key")
                                .short("k")
                                .takes_value(true)
                                .help("Base name for private signing key file")
                        )
                        .arg(
                            Arg::with_name("wait")
                                .long("wait")
                                .takes_value(true)
                                .help("How long to wait for transaction to be committed")
                        ),
                )
                .subcommand(
                    SubCommand::with_name("update")
                        .about("Update an organization")
                        .arg(
                            Arg::with_name("org_id")
                                .takes_value(true)
                                .required(true)
                                .help("Unique ID for organization"),
                        )
                        .arg(
                            Arg::with_name("name")
                                .takes_value(true)
                                .required(true)
                                .help("Name of organization"),
                        )
                        .arg(
                            Arg::with_name("address")
                                .takes_value(true)
                                .help("Physical address for organization"),
                        )
                        .arg(
                            Arg::with_name("metadata")
                                .long("metadata")
                                .takes_value(true)
                                .multiple(true)
                                .use_delimiter(true)
                                .help("Key-value pairs (format: <key>=<value>) in a comma-separated list"),
                        )
                        .arg(
                            Arg::with_name("key")
                                .long("key")
                                .short("k")
                                .takes_value(true)
                                .help("Base name for private signing key file")
                        )
                        .arg(
                            Arg::with_name("wait")
                                .long("wait")
                                .takes_value(true)
                                .help("How long to wait for transaction to be committed")
                        ),
                )
        );
    }

    #[cfg(feature = "schema")]
    {
        use clap::{Arg, SubCommand};

        app = app.subcommand(
            SubCommand::with_name("schema")
                .about("Create, update, list, or show schemas")
                .setting(clap::AppSettings::SubcommandRequiredElseHelp)
                .arg(
                    Arg::with_name("service_id")
                        .long("service-id")
                        .takes_value(true)
                        .help(
                            "The ID of the service the payload should be \
                         sent to; required if running on Splinter. Format \
                         <circuit-id>::<service-id>",
                        ),
                )
                .arg(
                    Arg::with_name("url")
                        .long("url")
                        .takes_value(true)
                        .help("URL for the REST API"),
                )
                .subcommand(
                    SubCommand::with_name("create")
                        .about("Create schemas from a yaml file")
                        .arg(
                            Arg::with_name("path")
                                .takes_value(true)
                                .required(true)
                                .help("Path to yaml file containing a list of schema definitions"),
                        )
                        .arg(
                            Arg::with_name("key")
                                .long("key")
                                .short("k")
                                .takes_value(true)
                                .help("Base name for private signing key file"),
                        )
                        .arg(
                            Arg::with_name("wait")
                                .long("wait")
                                .takes_value(true)
                                .help("How long to wait for transaction to be committed"),
                        ),
                )
                .subcommand(
                    SubCommand::with_name("update")
                        .about("Update schemas from a yaml file")
                        .arg(
                            Arg::with_name("path")
                                .takes_value(true)
                                .required(true)
                                .help("Path to yaml file containing a list of schema definitions"),
                        )
                        .arg(
                            Arg::with_name("key")
                                .long("key")
                                .short("k")
                                .takes_value(true)
                                .help("Base name for private signing key file"),
                        )
                        .arg(
                            Arg::with_name("wait")
                                .long("wait")
                                .takes_value(true)
                                .help("How long to wait for transaction to be committed"),
                        ),
                )
                .subcommand(SubCommand::with_name("list").about("List currently defined schemas"))
                .subcommand(
                    SubCommand::with_name("show")
                        .about("Show schema specified by name argument")
                        .arg(
                            Arg::with_name("name")
                                .takes_value(true)
                                .required(true)
                                .help("Name of schema"),
                        ),
                ),
        );
    }

    #[cfg(feature = "product")]
    {
        use clap::{Arg, SubCommand};

        app = app.subcommand(
            SubCommand::with_name("product")
                .about("Create, update, delete, list, or show products")
                .setting(clap::AppSettings::SubcommandRequiredElseHelp)
                .arg(
                    Arg::with_name("service_id")
                        .long("service-id")
                        .takes_value(true)
                        .help(
                            "The ID of the service the payload should be \
                     sent to; required if running on Splinter. Format \
                     <circuit-id>::<service-id>",
                        ),
                )
                .arg(
                    Arg::with_name("url")
                        .long("url")
                        .takes_value(true)
                        .help("URL for the REST API"),
                )
                .subcommand(
                    SubCommand::with_name("create")
                        .about("Create a product")
                        .arg(
                            Arg::with_name("product_id")
                                .conflicts_with("file")
                                .takes_value(true)
                                .required_unless("file")
                                .help("Unique ID for product"),
                        )
                        .arg(
                            Arg::with_name("product_namespace")
                                .long("namespace")
                                .takes_value(true)
                                .conflicts_with("file")
                                .help("Product namespace (example: GS1)"),
                        )
                        .arg(
                            Arg::with_name("owner")
                                .long("owner")
                                .takes_value(true)
                                .conflicts_with("file")
                                .help("Pike organization ID"),
                        )
                        .arg(
                            Arg::with_name("property")
                                .long("property")
                                .use_delimiter(true)
                                .takes_value(true)
                                .multiple(true)
                                .conflicts_with("file")
                                .help("Key value pair specifying a product property formatted as key=value"),
                        )
                        .arg(
                            Arg::with_name("file")
                                .long("file")
                                .short("f")
                                .takes_value(true)
                                .help("Path to yaml file containing a list of products"),
                        )
                        .arg(
                            Arg::with_name("key")
                                .long("key")
                                .short("k")
                                .takes_value(true)
                                .help("Base name for private signing key file"),
                        )
                        .arg(
                            Arg::with_name("wait")
                                .long("wait")
                                .takes_value(true)
                                .help("How long to wait for transaction to be committed"),
                        ),
                )
                .subcommand(
                    SubCommand::with_name("update")
                        .about("Update products from a yaml file")
                        .arg(
                            Arg::with_name("product_id")
                                .conflicts_with("file")
                                .takes_value(true)
                                .required_unless("file")
                                .help("Unique ID for product"),
                        )
                        .arg(
                            Arg::with_name("product_namespace")
                                .long("namespace")
                                .takes_value(true)
                                .conflicts_with("file")
                                .help("Product namespace (example: GS1)"),
                        )
                        .arg(
                            Arg::with_name("property")
                                .long("property")
                                .use_delimiter(true)
                                .takes_value(true)
                                .multiple(true)
                                .conflicts_with("file")
                                .help("Key value pair specifying a product property formatted as key=value"),
                        )
                        .arg(
                            Arg::with_name("file")
                                .long("file")
                                .short("f")
                                .takes_value(true)
                                .help("Path to yaml file containing a list of products"),
                        )
                        .arg(
                            Arg::with_name("key")
                                .long("key")
                                .short("k")
                                .takes_value(true)
                                .help("Base name for private signing key file"),
                        )
                        .arg(
                            Arg::with_name("wait")
                                .long("wait")
                                .takes_value(true)
                                .help("How long to wait for transaction to be committed"),
                        ),
                )
                .subcommand(
                    SubCommand::with_name("delete")
                        .about("Delete a product")
                        .arg(
                            Arg::with_name("product_id")
                                .takes_value(true)
                                .required(true)
                                .help("Unique ID for product"),
                        )
                        .arg(
                            Arg::with_name("product_namespace")
                                .long("namespace")
                                .takes_value(true)
                                .help("Product namespace (example: GS1)"),
                        )
                        .arg(
                            Arg::with_name("key")
                                .long("key")
                                .short("k")
                                .takes_value(true)
                                .help("Base name for private signing key file"),
                        )
                        .arg(
                            Arg::with_name("wait")
                                .long("wait")
                                .takes_value(true)
                                .help("How long to wait for transaction to be committed"),
                        ),
                )
                .subcommand(SubCommand::with_name("list").about("List currently defined products"))
                .subcommand(
                    SubCommand::with_name("show")
                        .about("Show product specified by ID argument")
                        .arg(
                            Arg::with_name("product_id")
                                .takes_value(true)
                                .required(true)
                                .help("ID of product"),
                        ),
                ),
        );
    }

    #[cfg(feature = "location")]
    {
        use clap::{Arg, SubCommand};

        app = app.subcommand(
            SubCommand::with_name("location")
                .about("Create, update, delete, list, or show locations")
                .setting(clap::AppSettings::SubcommandRequiredElseHelp)
                .arg(
                    Arg::with_name("service_id")
                        .long("service-id")
                        .takes_value(true)
                        .help(
                            "The ID of the service the payload should be \
                     sent to; required if running on Splinter. Format \
                     <circuit-id>::<service-id>",
                        ),
                )
                .arg(
                    Arg::with_name("url")
                        .long("url")
                        .takes_value(true)
                        .help("URL for the REST API"),
                )
                .subcommand(
                    SubCommand::with_name("create")
                        .about("Create a new location")
                        .arg(
                            Arg::with_name("location_id")
                                .conflicts_with("file")
                                .takes_value(true)
                                .required_unless("file")
                                .help("Unique identifier for location"),
                        )
                        .arg(
                            Arg::with_name("location_namespace")
                                .long("namespace")
                                .takes_value(true)
                                .conflicts_with("file")
                                .help("Location name space (example: GS1)"),
                        )
                        .arg(
                            Arg::with_name("owner")
                                .long("owner")
                                .takes_value(true)
                                .conflicts_with("file")
                                .help("Pike organization ID"),
                        )
                        .arg(
                            Arg::with_name("property")
                                .long("property")
                                .use_delimiter(true)
                                .takes_value(true)
                                .multiple(true)
                                .conflicts_with("file")
                                .help("Key value pair specifying a location property formatted as key=value"),
                        )
                        .arg(
                            Arg::with_name("file")
                                .long("file")
                                .short("f")
                                .takes_value(true)
                                .help("Path to yaml file containing a list of locations"),
                        )
                        .arg(
                            Arg::with_name("key")
                                .long("key")
                                .short("k")
                                .takes_value(true)
                                .help("Base name for private signing key file"),
                        )
                        .arg(
                            Arg::with_name("wait")
                                .long("wait")
                                .takes_value(true)
                                .help("How long to wait for transaction to be committed"),
                        ),
                )
                .subcommand(
                    SubCommand::with_name("update")
                        .about("Update an existing location")
                        .arg(
                            Arg::with_name("location_id")
                                .conflicts_with("file")
                                .takes_value(true)
                                .required_unless("file")
                                .help("Unique identifier for location"),
                        )
                        .arg(
                            Arg::with_name("location_namespace")
                                .long("namespace")
                                .takes_value(true)
                                .conflicts_with("file")
                                .help("Location namespace (example: GS1)"),
                        )
                        .arg(
                            Arg::with_name("property")
                                .long("property")
                                .use_delimiter(true)
                                .takes_value(true)
                                .multiple(true)
                                .conflicts_with("file")
                                .help("Key value pair specifying a location property formatted as key=value"),
                        )
                        .arg(
                            Arg::with_name("file")
                                .long("file")
                                .short("f")
                                .takes_value(true)
                                .help("Path to yaml file containing a list of locations"),
                        )
                        .arg(
                            Arg::with_name("key")
                                .long("key")
                                .short("k")
                                .takes_value(true)
                                .help("Base name for private signing key file"),
                        )
                        .arg(
                            Arg::with_name("wait")
                                .long("wait")
                                .takes_value(true)
                                .help("How long to wait for transaction to be committed"),
                        ),
                )
                .subcommand(
                    SubCommand::with_name("delete")
                        .about("Delete a location")
                        .arg(
                            Arg::with_name("location_id")
                                .takes_value(true)
                                .required(true)
                                .help("Unique identifier for location"),
                        )
                        .arg(
                            Arg::with_name("location_namespace")
                                .long("namespace")
                                .takes_value(true)
                                .help("Location namespace (example: GS1)"),
                        )
                        .arg(
                            Arg::with_name("key")
                                .long("key")
                                .short("k")
                                .takes_value(true)
                                .help("Base name for private signing key file"),
                        )
                        .arg(
                            Arg::with_name("wait")
                                .long("wait")
                                .takes_value(true)
                                .help("How long to wait for transaction to be committed"),
                        ),
                )
                .subcommand(SubCommand::with_name("list").about("List currently defined locations"))
                .subcommand(
                    SubCommand::with_name("show")
                        .about("Show locations specified by ID argument")
                        .arg(
                            Arg::with_name("location_id")
                                .takes_value(true)
                                .required(true)
                                .help("Unique identifier for location"),
                        ),
                ),
        );
    }

    let matches = app.get_matches();

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

    Logger::with(log_spec_builder.build())
        .format(log_format)
        .start()?;

    match matches.subcommand() {
        #[cfg(feature = "admin-keygen")]
        ("admin", Some(m)) => match m.subcommand() {
            ("keygen", Some(m)) => {
                let conflict_strategy = if m.is_present("force") {
                    admin::ConflictStrategy::Force
                } else if m.is_present("skip") {
                    admin::ConflictStrategy::Skip
                } else {
                    admin::ConflictStrategy::Error
                };

                admin::do_keygen(m.value_of("directory"), conflict_strategy)?;
            }
            _ => unreachable!(),
        },
        ("agent", Some(m)) => {
            let url = m
                .value_of("url")
                .map(String::from)
                .or_else(|| env::var(GRID_DAEMON_ENDPOINT).ok())
                .unwrap_or_else(|| String::from("http://localhost:8000"));

            let service_id = m
                .value_of("service_id")
                .map(String::from)
                .or_else(|| env::var(GRID_SERVICE_ID).ok());

            match m.subcommand() {
                ("create", Some(m)) => {
                    let key = m
                        .value_of("key")
                        .map(String::from)
                        .or_else(|| env::var(GRID_DAEMON_KEY).ok());

                    let active = if m.is_present("inactive") {
                        false
                    } else if m.is_present("active") {
                        true
                    } else {
                        return Err(CliError::UserError(
                            "--active or --inactive flag must be provided".to_string(),
                        ));
                    };

                    let wait = value_t!(m, "wait", u64).unwrap_or(0);

                    let create_agent = CreateAgentActionBuilder::new()
                        .with_org_id(m.value_of("org_id").unwrap().into())
                        .with_public_key(m.value_of("public_key").unwrap().into())
                        .with_active(active)
                        .with_roles(
                            m.values_of("role")
                                .unwrap_or_default()
                                .map(String::from)
                                .collect::<Vec<String>>(),
                        )
                        .with_metadata(parse_metadata(&m)?)
                        .build()
                        .map_err(|err| CliError::UserError(format!("{}", err)))?;

                    info!("Submitting request to create agent...");
                    agents::do_create_agent(&url, key, wait, create_agent, service_id)?;
                }
                ("update", Some(m)) => {
                    let key = m
                        .value_of("key")
                        .map(String::from)
                        .or_else(|| env::var(GRID_DAEMON_KEY).ok());

                    let active = if m.is_present("inactive") {
                        false
                    } else if m.is_present("active") {
                        true
                    } else {
                        return Err(CliError::UserError(
                            "--active or --inactive flag must be provided".to_string(),
                        ));
                    };

                    let wait = value_t!(m, "wait", u64).unwrap_or(0);

                    let update_agent = UpdateAgentActionBuilder::new()
                        .with_org_id(m.value_of("org_id").unwrap().into())
                        .with_public_key(m.value_of("public_key").unwrap().into())
                        .with_active(active)
                        .with_roles(
                            m.values_of("role")
                                .unwrap_or_default()
                                .map(String::from)
                                .collect::<Vec<String>>(),
                        )
                        .with_metadata(parse_metadata(&m)?)
                        .build()
                        .map_err(|err| CliError::UserError(format!("{}", err)))?;

                    info!("Submitting request to update agent...");
                    agents::do_update_agent(&url, key, wait, update_agent, service_id)?;
                }
                _ => return Err(CliError::UserError("Subcommand not recognized".into())),
            }
        }
        ("organization", Some(m)) => {
            let url = m
                .value_of("url")
                .map(String::from)
                .or_else(|| env::var(GRID_DAEMON_ENDPOINT).ok())
                .unwrap_or_else(|| String::from("http://localhost:8000"));

            let service_id = m
                .value_of("service_id")
                .map(String::from)
                .or_else(|| env::var(GRID_SERVICE_ID).ok());

            match m.subcommand() {
                ("create", Some(m)) => {
                    let key = m
                        .value_of("key")
                        .map(String::from)
                        .or_else(|| env::var(GRID_DAEMON_KEY).ok());

                    let wait = value_t!(m, "wait", u64).unwrap_or(0);

                    let create_org = CreateOrganizationActionBuilder::new()
                        .with_org_id(m.value_of("org_id").unwrap().into())
                        .with_name(m.value_of("name").unwrap().into())
                        .with_address(m.value_of("address").unwrap().into())
                        .with_metadata(parse_metadata(&m)?)
                        .build()
                        .map_err(|err| CliError::UserError(format!("{}", err)))?;

                    info!("Submitting request to create organization...");
                    orgs::do_create_organization(&url, key, wait, create_org, service_id)?;
                }
                ("update", Some(m)) => {
                    let key = m
                        .value_of("key")
                        .map(String::from)
                        .or_else(|| env::var(GRID_DAEMON_KEY).ok());

                    let wait = value_t!(m, "wait", u64).unwrap_or(0);

                    let update_org = UpdateOrganizationActionBuilder::new()
                        .with_org_id(m.value_of("org_id").unwrap().into())
                        .with_name(m.value_of("name").unwrap().into())
                        .with_address(m.value_of("address").unwrap().into())
                        .with_metadata(parse_metadata(&m)?)
                        .build()
                        .map_err(|err| CliError::UserError(format!("{}", err)))?;

                    info!("Submitting request to update organization...");
                    orgs::do_update_organization(&url, key, wait, update_org, service_id)?;
                }
                _ => return Err(CliError::UserError("Subcommand not recognized".into())),
            }
        }
        ("schema", Some(m)) => {
            let url = m
                .value_of("url")
                .map(String::from)
                .or_else(|| env::var(GRID_DAEMON_ENDPOINT).ok())
                .unwrap_or_else(|| String::from("http://localhost:8000"));

            let service_id = m
                .value_of("service_id")
                .map(String::from)
                .or_else(|| env::var(GRID_SERVICE_ID).ok());

            match m.subcommand() {
                ("create", Some(m)) => {
                    let key = m
                        .value_of("key")
                        .map(String::from)
                        .or_else(|| env::var(GRID_DAEMON_KEY).ok());

                    let wait = value_t!(m, "wait", u64).unwrap_or(0);

                    info!("Submitting request to create schema...");
                    schemas::do_create_schemas(
                        &url,
                        key,
                        wait,
                        m.value_of("path").unwrap(),
                        service_id,
                    )?;
                }
                ("update", Some(m)) => {
                    let key = m
                        .value_of("key")
                        .map(String::from)
                        .or_else(|| env::var(GRID_DAEMON_KEY).ok());

                    let wait = value_t!(m, "wait", u64).unwrap_or(0);

                    info!("Submitting request to update schema...");
                    schemas::do_update_schemas(
                        &url,
                        key,
                        wait,
                        m.value_of("path").unwrap(),
                        service_id,
                    )?;
                }
                ("list", Some(_)) => schemas::do_list_schemas(&url, service_id)?,
                ("show", Some(m)) => {
                    schemas::do_show_schema(&url, m.value_of("name").unwrap(), service_id)?
                }
                _ => return Err(CliError::UserError("Subcommand not recognized".into())),
            }
        }
        ("database", Some(m)) => match m.subcommand() {
            ("migrate", Some(m)) => database::run_migrations(
                m.value_of("connect")
                    .unwrap_or("postgres://grid:grid_example@localhost/grid"),
            )?,
            _ => return Err(CliError::UserError("Subcommand not recognized".into())),
        },
        ("keygen", Some(m)) => {
            let key_name = m
                .value_of("key_name")
                .map(String::from)
                .unwrap_or_else(whoami::username);

            let key_dir = if let Some(dir) = m.value_of("key_dir") {
                PathBuf::from(dir)
            } else if m.is_present("system") {
                PathBuf::from(SYSTEM_KEY_PATH)
            } else {
                dirs::home_dir()
                    .map(|mut p| {
                        p.push(".grid/keys");
                        p
                    })
                    .ok_or_else(|| CliError::UserError("Home directory not found".into()))?
            };

            keygen::generate_keys(key_name, m.is_present("force"), key_dir)?
        }
        ("product", Some(m)) => {
            let url = m
                .value_of("url")
                .map(String::from)
                .or_else(|| env::var(GRID_DAEMON_ENDPOINT).ok())
                .unwrap_or_else(|| String::from("http://localhost:8000"));

            let service_id = m
                .value_of("service_id")
                .map(String::from)
                .or_else(|| env::var(GRID_SERVICE_ID).ok());

            match m.subcommand() {
                ("create", Some(m)) if m.is_present("file") => {
                    let key = m
                        .value_of("key")
                        .map(String::from)
                        .or_else(|| env::var(GRID_DAEMON_KEY).ok());

                    let wait = value_t!(m, "wait", u64).unwrap_or(0);

                    let actions = products::create_product_payloads_from_file(
                        m.value_of("file").unwrap(),
                        &url,
                        service_id.as_deref(),
                    )?;

                    info!("Submitting request to create product...");
                    products::do_create_products(&url, key, wait, actions, service_id)?;
                }
                ("create", Some(m)) => {
                    let key = m
                        .value_of("key")
                        .map(String::from)
                        .or_else(|| env::var(GRID_DAEMON_KEY).ok());

                    let wait = value_t!(m, "wait", u64).unwrap_or(0);

                    let namespace = match m.value_of("product_namespace").unwrap_or("GS1") {
                        "GS1" => ProductNamespace::GS1,
                        unknown => {
                            return Err(CliError::UserError(format!(
                                "Unrecognized namespace {}",
                                unknown
                            )))
                        }
                    };

                    let properties = parse_properties(
                        &url,
                        m.value_of("product_namespace").unwrap_or("gs1_product"),
                        service_id.as_deref(),
                        &m,
                    )?;

                    let action = ProductCreateActionBuilder::new()
                        .with_product_id(m.value_of("product_id").unwrap().into())
                        .with_owner(m.value_of("owner").unwrap().into())
                        .with_product_namespace(namespace)
                        .with_properties(properties)
                        .build()
                        .map_err(|err| CliError::UserError(format!("{}", err)))?;

                    info!("Submitting request to create product...");
                    products::do_create_products(&url, key, wait, vec![action], service_id)?;
                }
                ("update", Some(m)) if m.is_present("file") => {
                    let key = m
                        .value_of("key")
                        .map(String::from)
                        .or_else(|| env::var(GRID_DAEMON_KEY).ok());

                    let wait = value_t!(m, "wait", u64).unwrap_or(0);

                    let actions = products::update_product_payloads_from_file(
                        m.value_of("file").unwrap(),
                        &url,
                        service_id.as_deref(),
                    )?;

                    info!("Submitting request to update product...");
                    products::do_update_products(&url, key, wait, actions, service_id)?;
                }
                ("update", Some(m)) => {
                    let key = m
                        .value_of("key")
                        .map(String::from)
                        .or_else(|| env::var(GRID_DAEMON_KEY).ok());

                    let wait = value_t!(m, "wait", u64).unwrap_or(0);

                    let namespace = match m.value_of("product_namespace").unwrap_or("GS1") {
                        "GS1" => ProductNamespace::GS1,
                        unknown => {
                            return Err(CliError::UserError(format!(
                                "Unrecognized namespace {}",
                                unknown
                            )))
                        }
                    };

                    let properties = parse_properties(
                        &url,
                        m.value_of("product_namespace").unwrap_or("gs1_product"),
                        service_id.as_deref(),
                        &m,
                    )?;

                    let action = ProductUpdateActionBuilder::new()
                        .with_product_id(m.value_of("product_id").unwrap().into())
                        .with_product_namespace(namespace)
                        .with_properties(properties)
                        .build()
                        .map_err(|err| CliError::UserError(format!("{}", err)))?;

                    info!("Submitting request to update product...");
                    products::do_update_products(&url, key, wait, vec![action], service_id)?;
                }
                ("delete", Some(m)) => {
                    let key = m
                        .value_of("key")
                        .map(String::from)
                        .or_else(|| env::var(GRID_DAEMON_KEY).ok());

                    let wait = value_t!(m, "wait", u64).unwrap_or(0);

                    let namespace = match m.value_of("product_namespace").unwrap_or("GS1") {
                        "GS1" => ProductNamespace::GS1,
                        unknown => {
                            return Err(CliError::UserError(format!(
                                "Unrecognized namespace {}",
                                unknown
                            )))
                        }
                    };

                    let action = ProductDeleteActionBuilder::new()
                        .with_product_id(m.value_of("product_id").unwrap().into())
                        .with_product_namespace(namespace)
                        .build()
                        .map_err(|err| CliError::UserError(format!("{}", err)))?;

                    info!("Submitting request to delete product...");
                    products::do_delete_products(&url, key, wait, action, service_id)?;
                }
                ("list", Some(_)) => products::do_list_products(&url, service_id)?,
                ("show", Some(m)) => {
                    products::do_show_products(&url, m.value_of("product_id").unwrap(), service_id)?
                }
                _ => return Err(CliError::UserError("Subcommand not recognized".into())),
            }
        }
        ("location", Some(m)) => {
            let url = m
                .value_of("url")
                .map(String::from)
                .or_else(|| env::var(GRID_DAEMON_ENDPOINT).ok())
                .unwrap_or_else(|| String::from("http://localhost:8000"));

            let service_id = m
                .value_of("service_id")
                .map(String::from)
                .or_else(|| env::var(GRID_SERVICE_ID).ok());

            match m.subcommand() {
                ("create", Some(m)) if m.is_present("file") => {
                    let key = m
                        .value_of("key")
                        .map(String::from)
                        .or_else(|| env::var(GRID_DAEMON_KEY).ok());

                    let wait = value_t!(m, "wait", u64).unwrap_or(0);

                    let actions = locations::create_location_payloads_from_file(
                        m.value_of("file").unwrap(),
                        &url,
                        service_id.as_deref(),
                    )?;

                    info!("Submitting request to create location...");
                    locations::do_create_location(&url, key, wait, actions, service_id.as_deref())?;
                }
                ("create", Some(m)) => {
                    let key = m
                        .value_of("key")
                        .map(String::from)
                        .or_else(|| env::var(GRID_DAEMON_KEY).ok());

                    let wait = value_t!(m, "wait", u64).unwrap_or(0);

                    let namespace = match m.value_of("location_namespace").unwrap_or("GS1") {
                        "GS1" => LocationNamespace::GS1,
                        unknown => {
                            return Err(CliError::UserError(format!(
                                "Unrecognized namespace {}",
                                unknown
                            )))
                        }
                    };

                    let properties = parse_properties(
                        &url,
                        m.value_of("location_namespace").unwrap_or("gs1_location"),
                        service_id.as_deref(),
                        &m,
                    )?;

                    let action = LocationCreateActionBuilder::new()
                        .with_location_id(m.value_of("location_id").unwrap().into())
                        .with_owner(m.value_of("owner").unwrap().into())
                        .with_namespace(namespace)
                        .with_properties(properties)
                        .build()
                        .map_err(|err| CliError::UserError(format!("{}", err)))?;

                    info!("Submitting request to create location...");
                    locations::do_create_location(
                        &url,
                        key,
                        wait,
                        vec![action],
                        service_id.as_deref(),
                    )?;
                }
                ("update", Some(m)) if m.is_present("file") => {
                    let key = m
                        .value_of("key")
                        .map(String::from)
                        .or_else(|| env::var(GRID_DAEMON_KEY).ok());

                    let wait = value_t!(m, "wait", u64).unwrap_or(0);

                    let actions = locations::update_location_payloads_from_file(
                        m.value_of("file").unwrap(),
                        &url,
                        service_id.as_deref(),
                    )?;

                    info!("Submitting request to update location...");
                    locations::do_update_location(&url, key, wait, actions, service_id.as_deref())?;
                }
                ("update", Some(m)) => {
                    let key = m
                        .value_of("key")
                        .map(String::from)
                        .or_else(|| env::var(GRID_DAEMON_KEY).ok());

                    let wait = value_t!(m, "wait", u64).unwrap_or(0);

                    let namespace = match m.value_of("location_namespace").unwrap_or("GS1") {
                        "GS1" => LocationNamespace::GS1,
                        unknown => {
                            return Err(CliError::UserError(format!(
                                "Unrecognized namespace {}",
                                unknown
                            )))
                        }
                    };

                    let properties = parse_properties(
                        &url,
                        m.value_of("location_namespace").unwrap_or("gs1_location"),
                        service_id.as_deref(),
                        &m,
                    )?;

                    let action = LocationUpdateActionBuilder::new()
                        .with_location_id(m.value_of("location_id").unwrap().into())
                        .with_namespace(namespace)
                        .with_properties(properties)
                        .build()
                        .map_err(|err| CliError::UserError(format!("{}", err)))?;

                    info!("Submitting request to update location...");
                    locations::do_update_location(
                        &url,
                        key,
                        wait,
                        vec![action],
                        service_id.as_deref(),
                    )?;
                }
                ("delete", Some(m)) => {
                    let key = m
                        .value_of("key")
                        .map(String::from)
                        .or_else(|| env::var(GRID_DAEMON_KEY).ok());

                    let wait = value_t!(m, "wait", u64).unwrap_or(0);

                    let namespace = match m.value_of("location_namespace").unwrap_or("GS1") {
                        "GS1" => LocationNamespace::GS1,
                        unknown => {
                            return Err(CliError::UserError(format!(
                                "Unrecognized namespace {}",
                                unknown
                            )))
                        }
                    };

                    let action = LocationDeleteActionBuilder::new()
                        .with_location_id(m.value_of("location_id").unwrap().into())
                        .with_namespace(namespace)
                        .build()
                        .map_err(|err| CliError::UserError(format!("{}", err)))?;

                    info!("Submitting request to delete location...");
                    locations::do_delete_location(&url, key, wait, action, service_id.as_deref())?;
                }
                ("list", Some(_)) => locations::do_list_locations(&url, service_id.as_deref())?,
                ("show", Some(_)) => locations::do_show_location(
                    &url,
                    m.value_of("location_id").unwrap(),
                    service_id.as_deref(),
                )?,
                _ => return Err(CliError::UserError("Subcommand not recognized".into())),
            }
        }
        _ => return Err(CliError::UserError("Subcommand not recognized".into())),
    }

    Ok(())
}

fn parse_metadata(matches: &ArgMatches) -> Result<Vec<KeyValueEntry>, CliError> {
    let metadata = matches
        .values_of("metadata")
        .unwrap_or_default()
        .map(String::from)
        .collect::<Vec<String>>();

    let mut key_value_entries = Vec::new();

    for data in metadata {
        let entries = data.split('=').map(String::from).collect::<Vec<String>>();

        let (key, value) = if entries.len() != 2 {
            return Err(CliError::UserError(format!("Metadata malformed: {}", data)));
        } else {
            (entries[0].clone(), entries[1].clone())
        };

        key_value_entries.push(
            KeyValueEntryBuilder::new()
                .with_key(key)
                .with_value(value)
                .build()
                .map_err(|err| CliError::UserError(format!("Metadata malformed: {}", err)))?,
        );
    }

    Ok(key_value_entries)
}

fn parse_properties(
    url: &str,
    namespace: &str,
    service_id: Option<&str>,
    matches: &ArgMatches,
) -> Result<Vec<PropertyValue>, CliError> {
    let properties = matches
        .values_of("property")
        .unwrap_or_default()
        .map(String::from)
        .try_fold(HashMap::new(), |mut acc, data| {
            let entries = data.split('=').map(String::from).collect::<Vec<String>>();

            let (key, value) = if entries.len() != 2 {
                return Err(CliError::UserError(format!("Metadata malformed: {}", data)));
            } else {
                (entries[0].clone(), entries[1].clone())
            };

            acc.insert(key, value);

            Ok(acc)
        })?;

    let schemas = schemas::get_schema(url, namespace, service_id)?;

    let mut property_values = Vec::new();

    for property in schemas.properties {
        let value = if let Some(value) = properties.get(&property.name) {
            value
        } else if !property.required {
            continue;
        } else {
            return Err(CliError::UserError(format!(
                "Field {} not found",
                property.name
            )));
        };

        match property.data_type {
            schemas::DataType::Number => {
                let number = if let Ok(i) = value.parse::<i64>() {
                    i
                } else {
                    return Err(CliError::UserError(format!("{} in not a number", value)));
                };

                let property_value = PropertyValueBuilder::new()
                    .with_name(property.name)
                    .with_data_type(property.data_type.into())
                    .with_number_value(number)
                    .build()
                    .map_err(|err| CliError::UserError(format!("{}", err)))?;

                property_values.push(property_value);
            }
            schemas::DataType::Enum => {
                let enum_idx = if let Ok(i) = value.parse::<u32>() {
                    i
                } else {
                    return Err(CliError::UserError(format!("{} in not an enum", value)));
                };

                let property_value = PropertyValueBuilder::new()
                    .with_name(property.name)
                    .with_data_type(property.data_type.into())
                    .with_enum_value(enum_idx)
                    .build()
                    .map_err(|err| CliError::UserError(format!("{}", err)))?;

                property_values.push(property_value);
            }
            schemas::DataType::String => {
                let property_value = PropertyValueBuilder::new()
                    .with_name(property.name)
                    .with_data_type(property.data_type.into())
                    .with_string_value(value.into())
                    .build()
                    .map_err(|err| CliError::UserError(format!("{}", err)))?;

                property_values.push(property_value);
            }
            schemas::DataType::LatLong => {
                let lat_long = value
                    .split(',')
                    .map(|x| {
                        x.parse::<i64>()
                            .map_err(|err| CliError::UserError(format!("{}", err)))
                    })
                    .collect::<Result<Vec<i64>, CliError>>()?;

                if lat_long.len() != 2 {
                    return Err(CliError::UserError(format!(
                        "{:?} is not a valid latitude longitude",
                        lat_long
                    )));
                }

                let lat_long = LatLongBuilder::new()
                    .with_lat_long(lat_long[0], lat_long[1])
                    .build()
                    .map_err(|err| CliError::UserError(format!("{}", err)))?;

                let property_value = PropertyValueBuilder::new()
                    .with_name(property.name)
                    .with_data_type(property.data_type.into())
                    .with_lat_long_value(lat_long)
                    .build()
                    .map_err(|err| CliError::UserError(format!("{}", err)))?;

                property_values.push(property_value);
            }
            schemas::DataType::Boolean => {
                let boolean = if let Ok(i) = value.parse::<bool>() {
                    i
                } else {
                    return Err(CliError::UserError(format!("{} in not a boolean", value)));
                };

                let property_value = PropertyValueBuilder::new()
                    .with_name(property.name)
                    .with_data_type(property.data_type.into())
                    .with_boolean_value(boolean)
                    .build()
                    .map_err(|err| CliError::UserError(format!("{}", err)))?;

                property_values.push(property_value);
            }
            schemas::DataType::Bytes => {
                let mut f = File::open(&value)?;
                let mut buffer = Vec::new();
                f.read_to_end(&mut buffer)?;

                let property_value = PropertyValueBuilder::new()
                    .with_name(property.name)
                    .with_data_type(property.data_type.into())
                    .with_bytes_value(buffer)
                    .build()
                    .map_err(|err| CliError::UserError(format!("{}", err)))?;

                property_values.push(property_value);
            }
            schemas::DataType::Struct => {
                return Err(CliError::UserError(
                    "Structs cannot be added via command line, use --file option".into(),
                ))
            }
        }
    }

    Ok(property_values)
}

fn main() {
    if let Err(e) = run() {
        error!("{}", e);
        std::process::exit(1);
    }
}
