// Copyright 2021 Cargill Incorporated
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

mod actions;
mod error;
#[cfg(feature = "sawtooth")]
mod sawtooth;
#[cfg(any(
    feature = "location",
    feature = "pike",
    feature = "product",
    feature = "schema",
))]
mod signing;
#[cfg(any(
    feature = "location",
    feature = "pike",
    feature = "product",
    feature = "schema",
))]
mod transaction;
#[cfg(feature = "schema")]
mod yaml_parser;

use std::path::PathBuf;

#[cfg(any(feature = "purchase-order"))]
use std::convert::{TryFrom, TryInto};
#[cfg(any(feature = "purchase-order"))]
use std::time::{SystemTime, UNIX_EPOCH};

use std::env;
#[cfg(any(feature = "location", feature = "product",))]
use std::{collections::HashMap, fs::File, io::prelude::*};

#[cfg(any(feature = "pike", feature = "schema",))]
use clap::ArgMatches;
use flexi_logger::{DeferredNow, LogSpecBuilder, Logger};

#[cfg(any(
    feature = "location",
    feature = "pike",
    feature = "product",
    feature = "schema",
))]
use grid_sdk::client::{reqwest::ReqwestClientFactory, ClientFactory};

#[cfg(feature = "schema")]
use grid_sdk::client::{schema as grid_schema_client, schema::SchemaClient};

#[cfg(feature = "location")]
use grid_sdk::protocol::location::payload::{
    LocationCreateActionBuilder, LocationDeleteActionBuilder, LocationNamespace,
    LocationUpdateActionBuilder,
};
#[cfg(feature = "pike")]
use grid_sdk::protocol::pike::{
    payload::{
        CreateAgentActionBuilder, CreateOrganizationActionBuilder, CreateRoleActionBuilder,
        DeleteRoleActionBuilder, UpdateAgentActionBuilder, UpdateOrganizationActionBuilder,
        UpdateRoleActionBuilder,
    },
    state::{AlternateId, AlternateIdBuilder, KeyValueEntry, KeyValueEntryBuilder},
};
#[cfg(feature = "product")]
use grid_sdk::protocol::product::{
    payload::{ProductCreateActionBuilder, ProductDeleteActionBuilder, ProductUpdateActionBuilder},
    state::ProductNamespace,
};
#[cfg(any(feature = "location", feature = "product",))]
use grid_sdk::protocol::schema::state::{LatLongBuilder, PropertyValue, PropertyValueBuilder};
#[cfg(any(feature = "purchase-order"))]
use grid_sdk::{
    client::purchase_order::AlternateId as POClientAlternateId,
    data_validation::{purchase_order::validate_alt_id_format, validate_order_xml_3_4},
    protocol::purchase_order::payload::{
        CreatePurchaseOrderPayloadBuilder, CreateVersionPayloadBuilder, PayloadRevisionBuilder,
        UpdatePurchaseOrderPayloadBuilder, UpdateVersionPayloadBuilder,
    },
    protocol::purchase_order::state::PurchaseOrderAlternateId as POProtocolAlternateId,
    purchase_order::store::ListPOFilters,
};

use log::Record;

use crate::error::CliError;

#[cfg(feature = "database")]
use actions::database;
use actions::keygen;
#[cfg(feature = "location")]
use actions::location;
#[cfg(feature = "product")]
use actions::product;
#[cfg(feature = "purchase-order")]
use actions::purchase_order;
#[cfg(feature = "schema")]
use actions::schema;
#[cfg(feature = "pike")]
use actions::{agent, organization as orgs, role};

#[cfg(feature = "purchase-order")]
use actions::{purchase_order::GRID_ORDER_SCHEMA_DIR, DEFAULT_SCHEMA_DIR};

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(any(
    feature = "location",
    feature = "pike",
    feature = "product",
    feature = "schema",
))]
const GRID_DAEMON_KEY: &str = "GRID_DAEMON_KEY";
#[cfg(any(
    feature = "location",
    feature = "pike",
    feature = "product",
    feature = "schema",
    feature = "purchase-order",
))]
const GRID_DAEMON_ENDPOINT: &str = "GRID_DAEMON_ENDPOINT";
#[cfg(any(
    feature = "location",
    feature = "pike",
    feature = "product",
    feature = "schema",
    feature = "purchase-order",
))]
const GRID_SERVICE_ID: &str = "GRID_SERVICE_ID";

const SYSTEM_KEY_PATH: &str = "/etc/grid/keys";
const DEFAULT_SYSTEM_KEY_NAME: &str = "gridd";

#[cfg(any(
    feature = "location",
    feature = "pike",
    feature = "product",
    feature = "schema",
    feature = "purchase-order"
))]
const AFTER_HELP_WITHOUT_KEY: &str = r"ENV:
    GRID_DAEMON_ENDPOINT   Specifies a default value for --url
    GRID_SERVICE_ID        Specifies a default value for --service-id";

#[cfg(any(
    feature = "location",
    feature = "pike",
    feature = "product",
    feature = "schema",
    feature = "purchase-order"
))]
const AFTER_HELP_WITH_KEY: &str = r"ENV:
    CYLINDER_PATH          Path to search for private signing keys
    GRID_DAEMON_ENDPOINT   Specifies a default value for --url
    GRID_DAEMON_KEY        Specifies a default value for -k, --key
    GRID_SERVICE_ID        Specifies a default value for --service-id";

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
            (about: "Manage the Grid Daemon database")
            (@setting SubcommandRequiredElseHelp)
            (@subcommand migrate =>
                (about: "Run database migrations")
                (@arg connect: -C --("connect") +takes_value
                    "URL for database")
            )
        )
        (@subcommand keygen =>
            (about: "Generates keys with which the user can sign transactions and batches")
            (@arg key_name: +takes_value "Name of the key to create")
            (@arg force: --force conflicts_with[skip] "Overwrite files if they exist")
            (@arg skip: --skip conflicts_with[force] "Check if files exist; generate if missing" )
            (@arg key_dir: -d --("key-dir") +takes_value conflicts_with[system]
                "Specify the directory for the key files")
            (@arg system: --system "Generate system keys in /etc/grid/keys")
        )

    );

    #[cfg(feature = "pike")]
    {
        use clap::{Arg, SubCommand};

        app = app.subcommand(
            SubCommand::with_name("agent")
                .about("Create, update, list, or show an agent")
                .setting(clap::AppSettings::SubcommandRequiredElseHelp)
                .arg(
                    Arg::with_name("service_id")
                        .long("service-id")
                        .takes_value(true)
                        .global(true)
                        .help(
                            "The ID of the service the payload should be \
                         sent to; required if running on Splinter. Format \
                         <circuit-id>::<service-id>",
                        ),
                )
                .arg(Arg::with_name("url")
                    .long("url")
                    .takes_value(true)
                    .global(true)
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
                                .help("Public key and unique identifier for agents"),
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
                                .help("Key-value pairs (format: <key>=<value>) in a \
                                    comma-separated list"),
                        )
                        .arg(
                            Arg::with_name("key")
                                .long("key")
                                .short("k")
                                .takes_value(true)
                                .help("Base name or path for private signing key file")
                        )
                        .arg(
                            Arg::with_name("wait")
                                .long("wait")
                                .takes_value(true)
                                .help("How long to wait for transaction to be committed")
                        )
                        .after_help(AFTER_HELP_WITH_KEY)
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
                                .help("Public key and unique identifier for agents"),
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
                                .help("Key-value pairs (format: <key>=<value>) in a \
                                    comma-separated list"),
                        )
                        .arg(
                            Arg::with_name("key")
                                .long("key")
                                .short("k")
                                .takes_value(true)
                                .help("Base name or path for private signing key file")
                        )
                        .arg(
                            Arg::with_name("wait")
                                .long("wait")
                                .takes_value(true)
                                .help("How long to wait for transaction to be committed")
                        )
                        .after_help(AFTER_HELP_WITH_KEY),
                    )
                    .subcommand(
                        SubCommand::with_name("show")
                            .about("Show agent specified by public key")
                            .arg(
                                Arg::with_name("public_key")
                                    .takes_value(true)
                                    .required(true)
                                    .help("Public key and unique identifier for agents"),
                            )
                            .after_help(AFTER_HELP_WITHOUT_KEY),
                    )
                    .subcommand(
                        SubCommand::with_name("list")
                            .about("List all agents for a given service")
                            .arg(
                                Arg::with_name("format")
                                    .short("F")
                                    .long("format")
                                    .help("Output format")
                                    .possible_values(&["human", "csv"])
                                    .default_value("human")
                                    .takes_value(true),
                            )
                            .arg(
                                Arg::with_name("line-per-role")
                                    .long("line-per-role")
                                    .help("Displays agent information for each role on it's own \
                                        line. Useful when filtering by role.")
                            )
                            .after_help(AFTER_HELP_WITHOUT_KEY),
                    ),
        )
        .subcommand(
            SubCommand::with_name("organization")
                .about("Create, update, list, or show organizations")
                .setting(clap::AppSettings::SubcommandRequiredElseHelp)
                .arg(
                    Arg::with_name("service_id")
                        .long("service-id")
                        .takes_value(true)
                        .global(true)
                        .help(
                            "The ID of the service the payload should be \
                         sent to; required if running on Splinter. Format \
                         <circuit-id>::<service-id>",
                        ),
                )
                .arg(Arg::with_name("url")
                    .long("url")
                    .takes_value(true)
                    .global(true)
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
                            Arg::with_name("alternate_ids")
                                .long("alternate-ids")
                                .multiple(true)
                                .use_delimiter(true)
                                .help("Alternate IDs for organization (format: <id_type>:<id>) in \
                                    a comma-separated list"),
                        )
                        .arg(
                            Arg::with_name("metadata")
                                .long("metadata")
                                .takes_value(true)
                                .multiple(true)
                                .use_delimiter(true)
                                .help("Key-value pairs (format: <key>=<value>) in a \
                                    comma-separated list"),
                        )
                        .arg(
                            Arg::with_name("key")
                                .long("key")
                                .short("k")
                                .takes_value(true)
                                .help("Base name or path for private signing key file")
                        )
                        .arg(
                            Arg::with_name("wait")
                                .long("wait")
                                .takes_value(true)
                                .help("How long to wait for transaction to be committed")
                        )
                        .after_help(AFTER_HELP_WITH_KEY),
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
                            Arg::with_name("locations")
                                .long("locations")
                                .takes_value(true)
                                .multiple(true)
                                .use_delimiter(true)
                                .help("Locations for an organization"),
                        )
                        .arg(
                            Arg::with_name("alternate_ids")
                                .long("alternate-ids")
                                .multiple(true)
                                .use_delimiter(true)
                                .help("Alternate IDs for organization (format: <id_type>:<id>) in \
                                    a comma-separated list"),
                        )
                        .arg(
                            Arg::with_name("metadata")
                                .long("metadata")
                                .takes_value(true)
                                .multiple(true)
                                .use_delimiter(true)
                                .help("Key-value pairs (format: <key>=<value>) in a \
                                    comma-separated list"),
                        )
                        .arg(
                            Arg::with_name("key")
                                .long("key")
                                .short("k")
                                .takes_value(true)
                                .help("Base name or path for private signing key file")
                        )
                        .arg(
                            Arg::with_name("wait")
                                .long("wait")
                                .takes_value(true)
                                .help("How long to wait for transaction to be committed")
                        )
                        .after_help(AFTER_HELP_WITH_KEY),
                )
                .subcommand(
                    SubCommand::with_name("list")
                    .about("List organizations")
                    .arg(
                        Arg::with_name("alternate_ids")
                            .long("alternate-ids")
                            .help("List organizations with the associated Alternate IDs")
                    )
                    .arg(
                        Arg::with_name("format")
                            .short("F")
                            .long("format")
                            .help("Output format")
                            .possible_values(&["human", "csv"])
                            .default_value("human")
                            .takes_value(true),
                    )
                    .after_help(AFTER_HELP_WITHOUT_KEY)
                )
                .subcommand(
                    SubCommand::with_name("show")
                    .about("Show an organization specified by ID")
                    .arg(
                        Arg::with_name("org_id")
                            .takes_value(true)
                            .required(true)
                            .help("Unique ID for organization")
                    )
                    .after_help(AFTER_HELP_WITHOUT_KEY)
                )
        )
        .subcommand(
            SubCommand::with_name("role")
                .about("Create, update, delete, list, or show a role")
                .setting(clap::AppSettings::SubcommandRequiredElseHelp)
                .arg(
                    Arg::with_name("service_id")
                        .long("service-id")
                        .takes_value(true)
                        .global(true)
                        .help(
                            "The ID of the service the payload should be \
                         sent to; required if running on Splinter. Format \
                         <circuit-id>::<service-id>",
                        ),
                )
                .arg(Arg::with_name("url")
                    .long("url")
                    .takes_value(true)
                    .global(true)
                    .help("URL for the REST API")
                )
                .subcommand(
                    SubCommand::with_name("create")
                        .about("Create a role")
                        .arg(
                            Arg::with_name("org_id")
                                .takes_value(true)
                                .required(true)
                                .help("Unique ID for owning organization"),
                        )
                        .arg(
                            Arg::with_name("name")
                                .takes_value(true)
                                .required(true)
                                .help("Name for the role"),
                        )
                        .arg(
                            Arg::with_name("description")
                                .long("description")
                                .short("d")
                                .takes_value(true)
                                .required(false)
                                .help("Description of the role"),
                        )
                        .arg(
                            Arg::with_name("permissions")
                                .long("permissions")
                                .takes_value(true)
                                .multiple(true)
                                .use_delimiter(true)
                                .help("List of permissions belonging to the role"),
                        )
                        .arg(
                            Arg::with_name("allowed_orgs")
                                .long("allowed-orgs")
                                .takes_value(true)
                                .multiple(true)
                                .use_delimiter(true)
                                .help("List of IDs of organizations allowed use of the role"),
                        )
                        .arg(
                            Arg::with_name("inherit_from")
                                .long("inherit-from")
                                .takes_value(true)
                                .multiple(true)
                                .use_delimiter(true)
                                .help("List of roles to inherit permissions from"),
                        )
                        .arg(
                            Arg::with_name("active")
                                .long("active")
                                .conflicts_with("inactive")
                                .help("Set role as active"),
                        )
                        .arg(
                            Arg::with_name("inactive")
                                .long("inactive")
                                .conflicts_with("active")
                                .help("Set role as inactive"),
                        )
                        .arg(
                            Arg::with_name("key")
                                .long("key")
                                .short("k")
                                .takes_value(true)
                                .help("Base name or path for private signing key file")
                        )
                        .arg(
                            Arg::with_name("wait")
                                .long("wait")
                                .takes_value(true)
                                .help("How long to wait for transaction to be committed")
                        )
                        .after_help(AFTER_HELP_WITH_KEY),
                )
                .subcommand(
                    SubCommand::with_name("update")
                        .about("Update a role")
                        .arg(
                            Arg::with_name("org_id")
                                .takes_value(true)
                                .required(true)
                                .help("Unique ID for owning organization"),
                        )
                        .arg(
                            Arg::with_name("name")
                                .takes_value(true)
                                .required(true)
                                .help("Name for the role"),
                        )
                        .arg(
                            Arg::with_name("description")
                                .long("description")
                                .short("d")
                                .takes_value(true)
                                .required(false)
                                .help("Description of the role"),
                        )
                        .arg(
                            Arg::with_name("permissions")
                                .long("permissions")
                                .short("p")
                                .takes_value(true)
                                .multiple(true)
                                .use_delimiter(true)
                                .help("List of permissions belonging to the role"),
                        )
                        .arg(
                            Arg::with_name("allowed_orgs")
                                .long("allowed-orgs")
                                .takes_value(true)
                                .multiple(true)
                                .use_delimiter(true)
                                .help("List of organizations allowed use of the role"),
                        )
                        .arg(
                            Arg::with_name("inherit_from")
                                .long("inherit-from")
                                .takes_value(true)
                                .multiple(true)
                                .use_delimiter(true)
                                .help("List of roles to inherit permissions from"),
                        )
                        .arg(
                            Arg::with_name("active")
                                .long("active")
                                .conflicts_with("inactive")
                                .help("Set role as active"),
                        )
                        .arg(
                            Arg::with_name("inactive")
                                .long("inactive")
                                .conflicts_with("active")
                                .help("Set role as inactive"),
                        )
                        .arg(
                            Arg::with_name("key")
                                .long("key")
                                .short("k")
                                .takes_value(true)
                                .help("Base name or path for private signing key file")
                        )
                        .arg(
                            Arg::with_name("wait")
                                .long("wait")
                                .takes_value(true)
                                .help("How long to wait for transaction to be committed")
                        )
                        .after_help(AFTER_HELP_WITH_KEY),
                )
                .subcommand(
                    SubCommand::with_name("delete")
                        .about("Delete a role")
                        .arg(
                            Arg::with_name("org_id")
                                .takes_value(true)
                                .required(true)
                                .help("Unique ID for owning organization"),
                        )
                        .arg(
                            Arg::with_name("name")
                                .takes_value(true)
                                .required(true)
                                .help("Name for the role"),
                        )
                        .arg(
                            Arg::with_name("key")
                                .long("key")
                                .short("k")
                                .takes_value(true)
                                .help("Base name or path for private signing key file")
                        )
                        .arg(
                            Arg::with_name("wait")
                                .long("wait")
                                .takes_value(true)
                                .help("How long to wait for transaction to be committed")
                        )
                        .after_help(AFTER_HELP_WITH_KEY),
                )
                .subcommand(
                    SubCommand::with_name("show")
                        .about("Show role specified by org ID and name")
                        .arg(
                            Arg::with_name("org_id")
                                .takes_value(true)
                                .required(true)
                                .help("Org ID of role"),
                        )
                        .arg(
                            Arg::with_name("name")
                                .takes_value(true)
                                .required(true)
                                .help("Name of role"),
                        )
                        .after_help(AFTER_HELP_WITHOUT_KEY),
                )
                .subcommand(
                    SubCommand::with_name("list")
                        .about("List all roles for a given org ID")
                        .arg(
                            Arg::with_name("org_id")
                                .takes_value(true)
                                .required(true)
                                .help("Org ID of role"),
                        )
                        .after_help(AFTER_HELP_WITHOUT_KEY),
                ),
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
                        .global(true)
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
                        .global(true)
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
                                .help("Base name or path for private signing key file"),
                        )
                        .arg(
                            Arg::with_name("wait")
                                .long("wait")
                                .takes_value(true)
                                .help("How long to wait for transaction to be committed"),
                        )
                        .after_help(AFTER_HELP_WITH_KEY),
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
                                .help("Base name or path for private signing key file"),
                        )
                        .arg(
                            Arg::with_name("wait")
                                .long("wait")
                                .takes_value(true)
                                .help("How long to wait for transaction to be committed"),
                        )
                        .after_help(AFTER_HELP_WITH_KEY),
                )
                .subcommand(
                    SubCommand::with_name("list")
                        .about("List currently defined schemas")
                        .after_help(AFTER_HELP_WITHOUT_KEY),
                )
                .subcommand(
                    SubCommand::with_name("show")
                        .about("Show schema specified by name argument")
                        .arg(
                            Arg::with_name("name")
                                .takes_value(true)
                                .required(true)
                                .help("Name of schema"),
                        )
                        .after_help(AFTER_HELP_WITHOUT_KEY),
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
                        .global(true)
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
                        .global(true)
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
                            Arg::with_name("file")
                                .long("file")
                                .short("f")
                                .takes_value(true)
                                .multiple(true)
                                .number_of_values(1)
                                .display_order(1)
                                .help("Path to file containing a list of products"),
                        )
                        .arg(
                            Arg::with_name("key")
                                .long("key")
                                .short("k")
                                .takes_value(true)
                                .display_order(2)
                                .help("Base name or path for private signing key file"),
                        )
                        .arg(
                            Arg::with_name("product_namespace")
                                .long("namespace")
                                .takes_value(true)
                                .conflicts_with("file")
                                .display_order(3)
                                .help("Product namespace (example: GS1)"),
                        )
                        .arg(
                            Arg::with_name("owner")
                                .long("owner")
                                .takes_value(true)
                                .display_order(4)
                                .help("Pike organization ID"),
                        )
                        .arg(
                            Arg::with_name("property")
                                .long("property")
                                .use_delimiter(true)
                                .takes_value(true)
                                .multiple(true)
                                .conflicts_with("file")
                                .display_order(5)
                                .help(
                                    "Key value pair specifying a product property formatted as \
                                    key=value",
                                ),
                        )
                        .arg(
                            Arg::with_name("wait")
                                .long("wait")
                                .takes_value(true)
                                .help("How long to wait for transaction to be committed"),
                        )
                        .after_help(AFTER_HELP_WITH_KEY),
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
                                .help(
                                    "Key value pair specifying a product property formatted as \
                                    key=value",
                                ),
                        )
                        .arg(
                            Arg::with_name("file")
                                .long("file")
                                .short("f")
                                .takes_value(true)
                                .multiple(true)
                                .number_of_values(1)
                                .help("Path to file containing a list of products"),
                        )
                        .arg(
                            Arg::with_name("key")
                                .long("key")
                                .short("k")
                                .takes_value(true)
                                .help("Base name or path for private signing key file"),
                        )
                        .arg(
                            Arg::with_name("wait")
                                .long("wait")
                                .takes_value(true)
                                .help("How long to wait for transaction to be committed"),
                        )
                        .after_help(AFTER_HELP_WITH_KEY),
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
                                .help("Base name or path for private signing key file"),
                        )
                        .arg(
                            Arg::with_name("wait")
                                .long("wait")
                                .takes_value(true)
                                .help("How long to wait for transaction to be committed"),
                        )
                        .after_help(AFTER_HELP_WITH_KEY),
                )
                .subcommand(
                    SubCommand::with_name("list")
                        .about("List currently defined products")
                        .after_help(AFTER_HELP_WITHOUT_KEY),
                )
                .subcommand(
                    SubCommand::with_name("show")
                        .about("Show product specified by ID argument")
                        .arg(
                            Arg::with_name("product_id")
                                .takes_value(true)
                                .required(true)
                                .help("ID of product"),
                        )
                        .after_help(AFTER_HELP_WITHOUT_KEY),
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
                        .global(true)
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
                        .global(true)
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
                                .required_unless("file")
                                .help("Pike organization ID"),
                        )
                        .arg(
                            Arg::with_name("property")
                                .long("property")
                                .use_delimiter(true)
                                .takes_value(true)
                                .multiple(true)
                                .conflicts_with("file")
                                .help(
                                    "Key value pair specifying a location property formatted as \
                                    key=value",
                                ),
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
                                .help("Base name or path for private signing key file"),
                        )
                        .arg(
                            Arg::with_name("wait")
                                .long("wait")
                                .takes_value(true)
                                .help("How long to wait for transaction to be committed"),
                        )
                        .after_help(AFTER_HELP_WITH_KEY),
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
                                .help(
                                    "Key value pair specifying a location property formatted as \
                                    key=value",
                                ),
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
                                .help("Base name or path for private signing key file"),
                        )
                        .arg(
                            Arg::with_name("wait")
                                .long("wait")
                                .takes_value(true)
                                .help("How long to wait for transaction to be committed"),
                        )
                        .after_help(AFTER_HELP_WITH_KEY),
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
                                .help("Base name or path for private signing key file"),
                        )
                        .arg(
                            Arg::with_name("wait")
                                .long("wait")
                                .takes_value(true)
                                .help("How long to wait for transaction to be committed"),
                        )
                        .after_help(AFTER_HELP_WITH_KEY),
                )
                .subcommand(
                    SubCommand::with_name("list")
                        .about("List currently defined locations")
                        .after_help(AFTER_HELP_WITHOUT_KEY),
                )
                .subcommand(
                    SubCommand::with_name("show")
                        .about("Show locations specified by ID argument")
                        .arg(
                            Arg::with_name("location_id")
                                .takes_value(true)
                                .required(true)
                                .help("Unique identifier for location"),
                        )
                        .after_help(AFTER_HELP_WITHOUT_KEY),
                ),
        );
    }

    #[cfg(feature = "purchase-order")]
    {
        use clap::{Arg, SubCommand};

        let po_version = SubCommand::with_name("version")
            .about("Create, update, or list Purchase Order versions")
            .subcommand(
                SubCommand::with_name("create")
                    .about("Create a Purchase Order version")
                    .arg(
                        Arg::with_name("po")
                            .value_name("order_id")
                            .takes_value(true)
                            .required(true)
                            .help(
                                "ID of the Purchase Order this version belongs to. \
                        May be the Purchase Order's unique ID or an Alternate ID \
                        (Alternate ID format: <alternate_id_type>:<alternate_id>)",
                            ),
                    )
                    .arg(
                        Arg::with_name("version_id")
                            .value_name("version_id")
                            .takes_value(true)
                            .required(true)
                            .help("Identifier for this Purchase Order version"),
                    )
                    .arg(
                        Arg::with_name("workflow_state")
                            .value_name("status")
                            .long("workflow-state")
                            .takes_value(true)
                            .required(true)
                            .help("Workflow state of this Purchase Order version"),
                    )
                    .arg(
                        Arg::with_name("draft")
                            .long("draft")
                            .conflicts_with("not-draft")
                            .help(
                                "Specify this Purchase Order version is a draft. \
                                By default, a newly created version is a draft.",
                            ),
                    )
                    .arg(
                        Arg::with_name("not_draft")
                            .long("not-draft")
                            .conflicts_with("draft")
                            .help(
                                "Specify this Purchase Order version is not a draft. \
                                By default, a newly created version is a draft.",
                            ),
                    )
                    .arg(
                        Arg::with_name("order_xml")
                            .value_name("file")
                            .long("order-xml")
                            .takes_value(true)
                            .required(true)
                            .help(
                                "Specify the path to a Purchase Order XML file. \
                                    (Formatting must abide by GS1 XML standards 3.4)",
                            ),
                    )
                    .arg(
                        Arg::with_name("key")
                            .long("key")
                            .short("k")
                            .takes_value(true)
                            .help("Base name or path for private signing key file"),
                    )
                    .arg(
                        Arg::with_name("wait")
                            .long("wait")
                            .takes_value(true)
                            .help("How long to wait for transaction to be committed"),
                    )
                    .after_help(AFTER_HELP_WITH_KEY),
            )
            .subcommand(
                SubCommand::with_name("update")
                    .about("Update a Purchase Order version")
                    .arg(
                        Arg::with_name("po")
                            .value_name("order_id")
                            .takes_value(true)
                            .required(true)
                            .help(
                                "ID of the Purchase Order this version belongs to. \
                        May be the Purchase Order's UID or an Alternate ID \
                        (Alternate ID format: <alternate_id_type>:<alternate_id>)",
                            ),
                    )
                    .arg(
                        Arg::with_name("version_id")
                            .value_name("version_id")
                            .takes_value(true)
                            .required(true)
                            .help("ID of the Purchase Order version to be updated"),
                    )
                    .arg(
                        Arg::with_name("workflow_state")
                            .value_name("status")
                            .long("workflow-state")
                            .takes_value(true)
                            .help("The updated workflow state of this Purchase Order version"),
                    )
                    .arg(
                        Arg::with_name("draft")
                            .long("draft")
                            .conflicts_with("not_draft")
                            .help("Specify this Purchase Order version is a draft"),
                    )
                    .arg(
                        Arg::with_name("not_draft")
                            .long("not-draft")
                            .conflicts_with("draft")
                            .help("Specify this Purchase Order version is not a draft"),
                    )
                    .arg(
                        Arg::with_name("order_xml")
                            .value_name("file")
                            .long("order-xml")
                            .takes_value(true)
                            .help(
                                "Specify the path to a Purchase Order XML file. \
                                    (Formatting must abide by GS1 XML standards 3.4)",
                            ),
                    )
                    .arg(
                        Arg::with_name("key")
                            .long("key")
                            .short("k")
                            .takes_value(true)
                            .help("Base name or path for private signing key file"),
                    )
                    .arg(
                        Arg::with_name("wait")
                            .long("wait")
                            .takes_value(true)
                            .help("How long to wait for transaction to be committed"),
                    )
                    .after_help(AFTER_HELP_WITH_KEY),
            )
            .subcommand(
                SubCommand::with_name("list")
                    .about("List Purchase Order versions")
                    .arg(
                        Arg::with_name("po_uid")
                            .takes_value(true)
                            .required(true)
                            .help("UID of the Purchase Order this version belongs to."),
                    )
                    .arg(
                        Arg::with_name("accepted")
                            .long("accepted")
                            .conflicts_with("not_accepted")
                            .help("List only Purchase Order versions that have been accepted"),
                    )
                    .arg(
                        Arg::with_name("not_accepted")
                            .long("not-accepted")
                            .conflicts_with("accepted")
                            .help("List only Purchase Order versions that have not been accepted"),
                    )
                    .arg(
                        Arg::with_name("draft")
                            .long("draft")
                            .conflicts_with("not_draft")
                            .help("List only Purchase Order version drafts"),
                    )
                    .arg(
                        Arg::with_name("not_draft")
                            .long("not-draft")
                            .conflicts_with("draft")
                            .help("List only Purchase Order versions that are not drafts"),
                    )
                    .arg(
                        Arg::with_name("format")
                            .short("F")
                            .long("format")
                            .help("Output format")
                            .possible_values(&["human", "csv", "yaml", "json"])
                            .default_value("human")
                            .takes_value(true),
                    )
                    .after_help(AFTER_HELP_WITHOUT_KEY),
            )
            .subcommand(
                SubCommand::with_name("show")
                    .about("Show a Purchase Order version")
                    .arg(
                        Arg::with_name("po_uid")
                            .takes_value(true)
                            .required(true)
                            .help("Identifier for the Purchase Order the version belongs to"),
                    )
                    .arg(
                        Arg::with_name("version_id")
                            .takes_value(true)
                            .required(true)
                            .help("Identifier for the Purchase Order version"),
                    )
                    .arg(
                        Arg::with_name("format")
                            .short("F")
                            .long("format")
                            .help("Output format")
                            .possible_values(&["human", "csv", "yaml", "json"])
                            .default_value("human")
                            .takes_value(true),
                    )
                    .after_help(AFTER_HELP_WITHOUT_KEY),
            );

        let po_revision = SubCommand::with_name("revision")
            .about("Show and list Purchase Order version revisions")
            .subcommand(
                SubCommand::with_name("list")
                    .about("List revisions for a Purchase Order version")
                    .arg(
                        Arg::with_name("po_uid")
                            .takes_value(true)
                            .required(true)
                            .help("Identifier for the Purchase Order the revision belongs to"),
                    )
                    .arg(
                        Arg::with_name("version_id")
                            .takes_value(true)
                            .required(true)
                            .help("Identifier for the Purchase Order version the revision is for"),
                    ),
            )
            .subcommand(
                SubCommand::with_name("show")
                    .about("Show a revision for a Purchase Order version")
                    .arg(
                        Arg::with_name("po_uid")
                            .takes_value(true)
                            .required(true)
                            .help("Identifier for the Purchase Order the revision belongs to"),
                    )
                    .arg(
                        Arg::with_name("version_id")
                            .takes_value(true)
                            .required(true)
                            .help("Identifier for the Purchase Order version the revision is for"),
                    )
                    .arg(
                        Arg::with_name("revision_number")
                            .takes_value(true)
                            .required(true)
                            .help("The revision number to show"),
                    ),
            );

        app = app.subcommand(
            SubCommand::with_name("po")
                .about("Create, update, list, or show Purchase Orders, Versions and Revisions")
                .setting(clap::AppSettings::SubcommandRequiredElseHelp)
                .arg(
                    Arg::with_name("service_id")
                        .long("service-id")
                        .takes_value(true)
                        .global(true)
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
                        .global(true)
                        .help("URL for the REST API"),
                )
                .subcommand(po_version)
                .subcommand(po_revision)
                .subcommand(
                    SubCommand::with_name("create")
                        .about("Create a Purchase Order")
                        .arg(
                            Arg::with_name("buyer_org_id")
                                .value_name("buyer_org_id")
                                .long("buyer-org")
                                .short("buyer")
                                .takes_value(true)
                                .required(true)
                                .help("ID of the organization which is buying the Purchase Order"),
                        )
                        .arg(
                            Arg::with_name("seller_org_id")
                                .value_name("seller_org_id")
                                .long("seller-org")
                                .short("seller")
                                .takes_value(true)
                                .required(true)
                                .help("ID of the organization which is selling the Purchase Order"),
                        )
                        .arg(Arg::with_name("uid").long("uid").takes_value(true).help(
                            "Unique ID for Purchase Order. \
                                Defaults to randomly-generated unique ID",
                        ))
                        .arg(
                            Arg::with_name("alternate_id")
                                .long("alternate-id")
                                .takes_value(true)
                                .multiple(true)
                                .help(
                                    "Alternate IDs for the Purchase Order \
                                (format: <alternate_id_type>:<alternate_id>) \
                                in a comma-separated list",
                                ),
                        )
                        .arg(
                            Arg::with_name("workflow_state")
                                .value_name("status")
                                .long("workflow-state")
                                .takes_value(true)
                                .required(true)
                                .help("Workflow state of the Purchase Order"),
                        )
                        .arg(
                            Arg::with_name("key")
                                .long("key")
                                .short("k")
                                .takes_value(true)
                                .help("Base name or path for private signing key file"),
                        )
                        .arg(
                            Arg::with_name("wait")
                                .long("wait")
                                .takes_value(true)
                                .help("How long to wait for transaction to be committed"),
                        )
                        .after_help(AFTER_HELP_WITH_KEY),
                )
                .subcommand(
                    SubCommand::with_name("list")
                        .about("List Purchase Orders")
                        .arg(
                            Arg::with_name("buyer_org")
                                .long("buyer-org")
                                .takes_value(true)
                                .help("Only list Purchase Orders from the specified buyer organization"),
                        )
                        .arg(
                            Arg::with_name("seller_org")
                                .long("seller-org")
                                .takes_value(true)
                                .help("Only list Purchase Orders from the specified seller organization"),
                        )
                        .arg(
                            Arg::with_name("accepted")
                                .long("accepted")
                                .conflicts_with("not-accepted")
                                .help("List Purchase Orders that have an accepted version"),
                        )
                        .arg(
                            Arg::with_name("not_accepted")
                                .long("not-accepted")
                                .conflicts_with("accepted")
                                .help("List Purchase Orders that do not have an accepted version"),
                        )
                        .arg(
                            Arg::with_name("open")
                                .long("open")
                                .conflicts_with("closed")
                                .help("List Purchase Orders that are open"),
                        )
                        .arg(
                            Arg::with_name("closed")
                                .long("closed")
                                .conflicts_with("open")
                                .help("List Purchase Orders that have been closed"),
                        )
                        .arg(
                            Arg::with_name("format")
                                .short("F")
                                .long("format")
                                .help("Output format")
                                .possible_values(&["human", "csv", "yaml", "json"])
                                .default_value("human")
                                .takes_value(true),
                        )
                        .after_help(AFTER_HELP_WITHOUT_KEY),
                )
                .subcommand(
                    SubCommand::with_name("update")
                        .about("Update a Purchase Order")
                        .arg(Arg::with_name("id").takes_value(true).required(true).help(
                            "ID of the Purchase Order. \
                                    May be the Purchase Order's UID or an Alternate ID \
                                    (Alternate ID format: <alternate_id_type>:<alternate_id>)",
                        ))
                        .arg(
                            Arg::with_name("add_id")
                                .long("add-id")
                                .takes_value(true)
                                .help(
                                    "Add an Alternate ID to Purchase Order \
                                (format: <alternate_id_type>:<alternate_id>)",
                                ),
                        )
                        .arg(
                            Arg::with_name("rm_id")
                                .long("rm-id")
                                .takes_value(true)
                                .help(
                                    "Remove an Alternate ID from Purchase Order \
                                    (format: <alternate_id_type>:<alternate_id>)",
                                ),
                        )
                        .arg(
                            Arg::with_name("workflow_state")
                                .value_name("status")
                                .long("workflow-state")
                                .takes_value(true)
                                .help("The updated workflow state of the Purchase Order"),
                        )
                        .arg(
                            Arg::with_name("is_closed")
                                .long("is-closed")
                                .help("Specify the Purchase Order has been closed"),
                        )
                        .arg(
                            Arg::with_name("accepted_version")
                                .value_name("version_id")
                                .long("accepted-version")
                                .takes_value(true)
                                .help("Specify the ID of the accepted Purchase Order version"),
                        )
                        .arg(
                            Arg::with_name("key")
                                .long("key")
                                .short("k")
                                .takes_value(true)
                                .help("Base name or path for private signing key file"),
                        )
                        .arg(
                            Arg::with_name("wait")
                                .long("wait")
                                .takes_value(true)
                                .help("How long to wait for transaction to be committed"),
                        )
                        .after_help(AFTER_HELP_WITH_KEY),
                )
                .subcommand(
                    SubCommand::with_name("show")
                        .about("Show a Purchase Order")
                        .arg(
                            Arg::with_name("id")
                                .takes_value(true)
                                .required(true)
                                .help(
                                    "ID of the Purchase Order. \
                                    May be the Purchase Order's UID or an Alternate ID \
                                    (Alternate ID format: <alternate_id_type>:<alternate_id>)",
                                ),
                        )
                        .arg(
                            Arg::with_name("format")
                                .short("F")
                                .long("format")
                                .help("Output format")
                                .possible_values(&["human", "csv", "yaml", "json"])
                                .default_value("human")
                                .takes_value(true),
                        )
                        .after_help(AFTER_HELP_WITHOUT_KEY),
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

    #[cfg(any(
        feature = "location",
        feature = "pike",
        feature = "product",
        feature = "schema",
    ))]
    let client_factory = Box::new(ReqwestClientFactory::new());

    match matches.subcommand() {
        #[cfg(feature = "pike")]
        ("agent", Some(m)) => match m.subcommand() {
            ("create", Some(m)) => {
                let url = value_of_url(m)?;
                let service_id_str = value_of_service_id(m)?;
                let service_id = service_id_str.as_deref();
                let pike_client = client_factory.get_pike_client(url);
                let key = value_of_key(m)?;
                let signer = signing::load_signer(key)?;

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
                    .with_metadata(parse_metadata(m)?)
                    .build()
                    .map_err(|err| CliError::UserError(format!("{}", err)))?;

                info!("Submitting request to create agent...");
                agent::do_create_agent(pike_client, signer, wait, create_agent, service_id)?;
            }
            ("update", Some(m)) => {
                let url = value_of_url(m)?;
                let service_id_str = value_of_service_id(m)?;
                let service_id = service_id_str.as_deref();
                let pike_client = client_factory.get_pike_client(url);
                let key = value_of_key(m)?;
                let signer = signing::load_signer(key)?;

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
                    .with_metadata(parse_metadata(m)?)
                    .build()
                    .map_err(|err| CliError::UserError(format!("{}", err)))?;

                info!("Submitting request to update agent...");
                agent::do_update_agent(pike_client, signer, wait, update_agent, service_id)?;
            }
            ("list", Some(m)) => {
                let url = value_of_url(m)?;
                let service_id_str = value_of_service_id(m)?;
                let service_id = service_id_str.as_deref();
                let pike_client = client_factory.get_pike_client(url);
                agent::do_list_agents(
                    pike_client,
                    service_id,
                    m.value_of("format").unwrap(),
                    m.is_present("line-per-role"),
                )?;
            }
            ("show", Some(m)) => {
                let url = value_of_url(m)?;
                let service_id_str = value_of_service_id(m)?;
                let service_id = service_id_str.as_deref();
                let pike_client = client_factory.get_pike_client(url);
                agent::do_show_agents(pike_client, m.value_of("public_key").unwrap(), service_id)?
            }
            _ => return Err(CliError::UserError("Subcommand not recognized".into())),
        },
        #[cfg(feature = "pike")]
        ("organization", Some(m)) => match m.subcommand() {
            ("create", Some(m)) => {
                let url = value_of_url(m)?;
                let service_id_str = value_of_service_id(m)?;
                let service_id = service_id_str.as_deref();
                let pike_client = client_factory.get_pike_client(url);
                let key = value_of_key(m)?;
                let signer = signing::load_signer(key)?;
                let wait = value_t!(m, "wait", u64).unwrap_or(0);

                let create_org = CreateOrganizationActionBuilder::new()
                    .with_org_id(m.value_of("org_id").unwrap().into())
                    .with_name(m.value_of("name").unwrap().into())
                    .with_alternate_ids(parse_alternate_ids(m)?)
                    .with_metadata(parse_metadata(m)?)
                    .build()
                    .map_err(|err| CliError::UserError(format!("{}", err)))?;

                info!("Submitting request to create organization...");
                orgs::do_create_organization(pike_client, signer, wait, create_org, service_id)?;
            }
            ("update", Some(m)) => {
                let url = value_of_url(m)?;
                let service_id_str = value_of_service_id(m)?;
                let service_id = service_id_str.as_deref();
                let pike_client = client_factory.get_pike_client(url);
                let key = value_of_key(m)?;
                let signer = signing::load_signer(key)?;
                let wait = value_t!(m, "wait", u64).unwrap_or(0);

                let update_org = UpdateOrganizationActionBuilder::new()
                    .with_org_id(m.value_of("org_id").unwrap().into())
                    .with_name(m.value_of("name").unwrap().into())
                    .with_locations(
                        m.values_of("locations")
                            .unwrap_or_default()
                            .map(String::from)
                            .collect::<Vec<String>>(),
                    )
                    .with_alternate_ids(parse_alternate_ids(m)?)
                    .with_metadata(parse_metadata(m)?)
                    .build()
                    .map_err(|err| CliError::UserError(format!("{}", err)))?;

                info!("Submitting request to update organization...");
                orgs::do_update_organization(pike_client, signer, wait, update_org, service_id)?;
            }
            ("list", Some(m)) => {
                let url = value_of_url(m)?;
                let service_id_str = value_of_service_id(m)?;
                let service_id = service_id_str.as_deref();
                let pike_client = client_factory.get_pike_client(url);
                orgs::do_list_organizations(
                    pike_client,
                    service_id,
                    m.value_of("format").unwrap(),
                    m.is_present("alternate_ids"),
                )?
            }
            ("show", Some(m)) => {
                let url = value_of_url(m)?;
                let service_id_str = value_of_service_id(m)?;
                let service_id = service_id_str.as_deref();
                let pike_client = client_factory.get_pike_client(url);
                orgs::do_show_organization(pike_client, service_id, m.value_of("org_id").unwrap())?
            }
            _ => return Err(CliError::UserError("Subcommand not recognized".into())),
        },
        #[cfg(feature = "pike")]
        ("role", Some(m)) => match m.subcommand() {
            ("create", Some(m)) => {
                let url = value_of_url(m)?;
                let service_id_str = value_of_service_id(m)?;
                let service_id = service_id_str.as_deref();
                let pike_client = client_factory.get_pike_client(url);
                let key = value_of_key(m)?;
                let signer = signing::load_signer(key)?;
                let wait = value_t!(m, "wait", u64).unwrap_or(0);

                let active = if m.is_present("inactive") {
                    false
                } else {
                    m.is_present("active")
                };

                let create_role = CreateRoleActionBuilder::new()
                    .with_org_id(m.value_of("org_id").unwrap().into())
                    .with_name(m.value_of("name").unwrap().into())
                    .with_description(m.value_of("description").unwrap_or("").into())
                    .with_permissions(
                        m.values_of("permissions")
                            .unwrap_or_default()
                            .map(String::from)
                            .collect::<Vec<String>>(),
                    )
                    .with_allowed_organizations(
                        m.values_of("allowed_orgs")
                            .unwrap_or_default()
                            .map(String::from)
                            .collect::<Vec<String>>(),
                    )
                    .with_inherit_from(
                        m.values_of("inherit_from")
                            .unwrap_or_default()
                            .map(String::from)
                            .collect::<Vec<String>>(),
                    )
                    .with_active(active)
                    .build()
                    .map_err(|err| CliError::UserError(format!("{}", err)))?;

                info!("Submitting request to create role...");
                role::do_create_role(pike_client, signer, wait, create_role, service_id)?;
            }
            ("update", Some(m)) => {
                let url = value_of_url(m)?;
                let service_id_str = value_of_service_id(m)?;
                let service_id = service_id_str.as_deref();
                let pike_client = client_factory.get_pike_client(url);
                let key = value_of_key(m)?;
                let signer = signing::load_signer(key)?;
                let wait = value_t!(m, "wait", u64).unwrap_or(0);

                let active = if m.is_present("inactive") {
                    false
                } else {
                    m.is_present("active")
                };

                let update_role = UpdateRoleActionBuilder::new()
                    .with_org_id(m.value_of("org_id").unwrap().into())
                    .with_name(m.value_of("name").unwrap().into())
                    .with_description(m.value_of("description").unwrap_or("").into())
                    .with_permissions(
                        m.values_of("permissions")
                            .unwrap_or_default()
                            .map(String::from)
                            .collect::<Vec<String>>(),
                    )
                    .with_allowed_organizations(
                        m.values_of("allowed_orgs")
                            .unwrap_or_default()
                            .map(String::from)
                            .collect::<Vec<String>>(),
                    )
                    .with_inherit_from(
                        m.values_of("inherit_from")
                            .unwrap_or_default()
                            .map(String::from)
                            .collect::<Vec<String>>(),
                    )
                    .with_active(active)
                    .build()
                    .map_err(|err| CliError::UserError(format!("{}", err)))?;

                info!("Submitting request to update role...");
                role::do_update_role(pike_client, signer, wait, update_role, service_id)?;
            }
            ("delete", Some(m)) => {
                let url = value_of_url(m)?;
                let service_id_str = value_of_service_id(m)?;
                let service_id = service_id_str.as_deref();
                let pike_client = client_factory.get_pike_client(url);
                let key = value_of_key(m)?;
                let signer = signing::load_signer(key)?;
                let wait = value_t!(m, "wait", u64).unwrap_or(0);

                let delete_role = DeleteRoleActionBuilder::new()
                    .with_org_id(m.value_of("org_id").unwrap().into())
                    .with_name(m.value_of("name").unwrap().into())
                    .build()
                    .map_err(|err| CliError::UserError(format!("{}", err)))?;

                info!("Submitting request to delete role...");
                role::do_delete_role(pike_client, signer, wait, delete_role, service_id)?;
            }
            ("show", Some(m)) => {
                let url = value_of_url(m)?;
                let service_id_str = value_of_service_id(m)?;
                let service_id = service_id_str.as_deref();
                let pike_client = client_factory.get_pike_client(url);
                role::do_show_role(
                    pike_client,
                    m.value_of("org_id").unwrap().into(),
                    m.value_of("name").unwrap().into(),
                    service_id,
                )?
            }
            ("list", Some(m)) => {
                let url = value_of_url(m)?;
                let service_id_str = value_of_service_id(m)?;
                let service_id = service_id_str.as_deref();
                let pike_client = client_factory.get_pike_client(url);
                role::do_list_roles(
                    pike_client,
                    m.value_of("org_id").unwrap().into(),
                    service_id,
                )?
            }
            _ => return Err(CliError::UserError("Subcommand not recognized".into())),
        },
        #[cfg(feature = "schema")]
        ("schema", Some(m)) => match m.subcommand() {
            ("create", Some(m)) => {
                let url = value_of_url(m)?;
                let service_id_str = value_of_service_id(m)?;
                let service_id = service_id_str.as_deref();
                let schema_client = client_factory.get_schema_client(url);
                let key = value_of_key(m)?;
                let signer = signing::load_signer(key)?;
                let wait = value_t!(m, "wait", u64).unwrap_or(0);

                info!("Submitting request to create schema...");
                schema::do_create_schemas(
                    schema_client,
                    signer,
                    wait,
                    m.value_of("path").unwrap(),
                    service_id,
                )?;
            }
            ("update", Some(m)) => {
                let url = value_of_url(m)?;
                let service_id_str = value_of_service_id(m)?;
                let service_id = service_id_str.as_deref();
                let schema_client = client_factory.get_schema_client(url);
                let key = value_of_key(m)?;
                let signer = signing::load_signer(key)?;
                let wait = value_t!(m, "wait", u64).unwrap_or(0);

                info!("Submitting request to update schema...");
                schema::do_update_schemas(
                    schema_client,
                    signer,
                    wait,
                    m.value_of("path").unwrap(),
                    service_id,
                )?;
            }
            ("list", Some(m)) => {
                let url = value_of_url(m)?;
                let service_id_str = value_of_service_id(m)?;
                let service_id = service_id_str.as_deref();
                let schema_client = client_factory.get_schema_client(url);
                schema::do_list_schemas(schema_client, service_id)?
            }
            ("show", Some(m)) => {
                let url = value_of_url(m)?;
                let service_id_str = value_of_service_id(m)?;
                let service_id = service_id_str.as_deref();
                let schema_client = client_factory.get_schema_client(url);
                schema::do_show_schema(
                    schema_client,
                    m.value_of("name").unwrap().into(),
                    service_id,
                )?
            }
            _ => return Err(CliError::UserError("Subcommand not recognized".into())),
        },
        #[cfg(feature = "database")]
        ("database", Some(m)) => match m.subcommand() {
            ("migrate", Some(m)) => database::run_migrations(
                m.value_of("connect")
                    .unwrap_or("postgres://grid:grid_example@localhost/grid"),
            )?,
            _ => return Err(CliError::UserError("Subcommand not recognized".into())),
        },
        ("keygen", Some(m)) => {
            let key_name = m.value_of("key_name").map(String::from).unwrap_or_else(|| {
                if m.is_present("system") {
                    DEFAULT_SYSTEM_KEY_NAME.to_string()
                } else {
                    whoami::username()
                }
            });

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

            let conflict_strategy = if m.is_present("force") {
                keygen::ConflictStrategy::Force
            } else if m.is_present("skip") {
                keygen::ConflictStrategy::Skip
            } else {
                keygen::ConflictStrategy::Error
            };

            keygen::generate_keys(key_name, conflict_strategy, key_dir)?
        }
        #[cfg(feature = "product")]
        ("product", Some(m)) => match m.subcommand() {
            ("create", Some(m)) if m.is_present("file") => {
                let url = value_of_url(m)?;
                let service_id_str = value_of_service_id(m)?;
                let service_id = service_id_str.as_deref();
                let product_client = client_factory.get_product_client(url.clone());
                let schema_client = client_factory.get_schema_client(url);
                let key = value_of_key(m)?;
                let signer = signing::load_signer(key)?;
                let wait = value_t!(m, "wait", u64).unwrap_or(0);

                let actions = product::create_product_payloads_from_file(
                    m.values_of("file").unwrap().collect(),
                    schema_client,
                    service_id,
                    m.value_of("owner"),
                )?;

                info!("Submitting request to create product...");
                product::do_create_products(product_client, signer, wait, actions, service_id)?;
            }
            ("create", Some(m)) => {
                let url = value_of_url(m)?;
                let service_id_str = value_of_service_id(m)?;
                let service_id = service_id_str.as_deref();
                let product_client = client_factory.get_product_client(url.clone());
                let schema_client = client_factory.get_schema_client(url);
                let key = value_of_key(m)?;
                let signer = signing::load_signer(key)?;
                let wait = value_t!(m, "wait", u64).unwrap_or(0);

                let namespace = match m.value_of("product_namespace").unwrap_or("GS1") {
                    "GS1" => ProductNamespace::Gs1,
                    unknown => {
                        return Err(CliError::UserError(format!(
                            "Unrecognized namespace {}",
                            unknown
                        )))
                    }
                };

                let properties = parse_properties(
                    schema_client,
                    m.value_of("product_namespace").unwrap_or("gs1_product"),
                    service_id,
                    m,
                )?;

                let action = ProductCreateActionBuilder::new()
                    .with_product_id(m.value_of("product_id").unwrap().into())
                    .with_owner(m.value_of("owner").unwrap().into())
                    .with_product_namespace(namespace)
                    .with_properties(properties)
                    .build()
                    .map_err(|err| CliError::UserError(format!("{}", err)))?;

                info!("Submitting request to create product...");
                product::do_create_products(
                    product_client,
                    signer,
                    wait,
                    vec![action],
                    service_id,
                )?;
            }
            ("update", Some(m)) if m.is_present("file") => {
                let url = value_of_url(m)?;
                let service_id_str = value_of_service_id(m)?;
                let service_id = service_id_str.as_deref();
                let product_client = client_factory.get_product_client(url.clone());
                let schema_client = client_factory.get_schema_client(url);
                let key = value_of_key(m)?;
                let signer = signing::load_signer(key)?;
                let wait = value_t!(m, "wait", u64).unwrap_or(0);

                let actions = product::update_product_payloads_from_file(
                    m.values_of("file").unwrap().collect(),
                    schema_client,
                    service_id,
                )?;

                info!("Submitting request to update product...");
                product::do_update_products(product_client, signer, wait, actions, service_id)?;
            }
            ("update", Some(m)) => {
                let url = value_of_url(m)?;
                let service_id_str = value_of_service_id(m)?;
                let service_id = service_id_str.as_deref();
                let product_client = client_factory.get_product_client(url.clone());
                let schema_client = client_factory.get_schema_client(url);
                let key = value_of_key(m)?;
                let signer = signing::load_signer(key)?;
                let wait = value_t!(m, "wait", u64).unwrap_or(0);

                let namespace = match m.value_of("product_namespace").unwrap_or("GS1") {
                    "GS1" => ProductNamespace::Gs1,
                    unknown => {
                        return Err(CliError::UserError(format!(
                            "Unrecognized namespace {}",
                            unknown
                        )))
                    }
                };

                let properties = parse_properties(
                    schema_client,
                    m.value_of("product_namespace").unwrap_or("gs1_product"),
                    service_id,
                    m,
                )?;

                let action = ProductUpdateActionBuilder::new()
                    .with_product_id(m.value_of("product_id").unwrap().into())
                    .with_product_namespace(namespace)
                    .with_properties(properties)
                    .build()
                    .map_err(|err| CliError::UserError(format!("{}", err)))?;

                info!("Submitting request to update product...");
                product::do_update_products(
                    product_client,
                    signer,
                    wait,
                    vec![action],
                    service_id,
                )?;
            }
            ("delete", Some(m)) => {
                let url = value_of_url(m)?;
                let service_id_str = value_of_service_id(m)?;
                let service_id = service_id_str.as_deref();
                let product_client = client_factory.get_product_client(url);
                let key = value_of_key(m)?;
                let signer = signing::load_signer(key)?;
                let wait = value_t!(m, "wait", u64).unwrap_or(0);

                let namespace = match m.value_of("product_namespace").unwrap_or("GS1") {
                    "GS1" => ProductNamespace::Gs1,
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
                product::do_delete_products(product_client, signer, wait, action, service_id)?;
            }
            ("list", Some(m)) => {
                let url = value_of_url(m)?;
                let service_id_str = value_of_service_id(m)?;
                let service_id = service_id_str.as_deref();
                let product_client = client_factory.get_product_client(url);
                product::do_list_products(product_client, service_id)?
            }
            ("show", Some(m)) => {
                let url = value_of_url(m)?;
                let service_id_str = value_of_service_id(m)?;
                let service_id = service_id_str.as_deref();
                let product_client = client_factory.get_product_client(url);
                product::do_show_products(
                    product_client,
                    m.value_of("product_id").unwrap().into(),
                    service_id,
                )?
            }
            _ => return Err(CliError::UserError("Subcommand not recognized".into())),
        },
        #[cfg(feature = "location")]
        ("location", Some(m)) => match m.subcommand() {
            ("create", Some(m)) if m.is_present("file") => {
                let url = value_of_url(m)?;
                let service_id_str = value_of_service_id(m)?;
                let service_id = service_id_str.as_deref();
                let location_client = client_factory.get_location_client(url.clone());
                let schema_client = client_factory.get_schema_client(url);
                let key = value_of_key(m)?;
                let signer = signing::load_signer(key)?;
                let wait = value_t!(m, "wait", u64).unwrap_or(0);

                let actions = location::create_location_payloads_from_file(
                    m.value_of("file").unwrap(),
                    schema_client,
                    service_id,
                )?;

                info!("Submitting request to create location...");
                location::do_create_location(location_client, signer, wait, actions, service_id)?;
            }
            ("create", Some(m)) => {
                let url = value_of_url(m)?;
                let service_id_str = value_of_service_id(m)?;
                let service_id = service_id_str.as_deref();
                let location_client = client_factory.get_location_client(url.clone());
                let schema_client = client_factory.get_schema_client(url);
                let key = value_of_key(m)?;
                let signer = signing::load_signer(key)?;
                let wait = value_t!(m, "wait", u64).unwrap_or(0);

                let namespace = match m.value_of("location_namespace").unwrap_or("GS1") {
                    "GS1" => LocationNamespace::Gs1,
                    unknown => {
                        return Err(CliError::UserError(format!(
                            "Unrecognized namespace {}",
                            unknown
                        )))
                    }
                };

                let properties = parse_properties(
                    schema_client,
                    m.value_of("location_namespace").unwrap_or("gs1_location"),
                    service_id,
                    m,
                )?;

                let action = LocationCreateActionBuilder::new()
                    .with_location_id(m.value_of("location_id").unwrap().into())
                    .with_owner(m.value_of("owner").unwrap().into())
                    .with_namespace(namespace)
                    .with_properties(properties)
                    .build()
                    .map_err(|err| CliError::UserError(format!("{}", err)))?;

                info!("Submitting request to create location...");
                location::do_create_location(
                    location_client,
                    signer,
                    wait,
                    vec![action],
                    service_id,
                )?;
            }
            ("update", Some(m)) if m.is_present("file") => {
                let url = value_of_url(m)?;
                let service_id_str = value_of_service_id(m)?;
                let service_id = service_id_str.as_deref();
                let location_client = client_factory.get_location_client(url.clone());
                let schema_client = client_factory.get_schema_client(url);
                let key = value_of_key(m)?;
                let signer = signing::load_signer(key)?;
                let wait = value_t!(m, "wait", u64).unwrap_or(0);

                let actions = location::update_location_payloads_from_file(
                    m.value_of("file").unwrap(),
                    schema_client,
                    service_id,
                )?;

                info!("Submitting request to update location...");
                location::do_update_location(location_client, signer, wait, actions, service_id)?;
            }
            ("update", Some(m)) => {
                let url = value_of_url(m)?;
                let service_id_str = value_of_service_id(m)?;
                let service_id = service_id_str.as_deref();
                let location_client = client_factory.get_location_client(url.clone());
                let schema_client = client_factory.get_schema_client(url);
                let key = value_of_key(m)?;
                let signer = signing::load_signer(key)?;
                let wait = value_t!(m, "wait", u64).unwrap_or(0);

                let namespace = match m.value_of("location_namespace").unwrap_or("GS1") {
                    "GS1" => LocationNamespace::Gs1,
                    unknown => {
                        return Err(CliError::UserError(format!(
                            "Unrecognized namespace {}",
                            unknown
                        )))
                    }
                };

                let properties = parse_properties(
                    schema_client,
                    m.value_of("location_namespace").unwrap_or("gs1_location"),
                    service_id,
                    m,
                )?;

                let action = LocationUpdateActionBuilder::new()
                    .with_location_id(m.value_of("location_id").unwrap().into())
                    .with_namespace(namespace)
                    .with_properties(properties)
                    .build()
                    .map_err(|err| CliError::UserError(format!("{}", err)))?;

                info!("Submitting request to update location...");
                location::do_update_location(
                    location_client,
                    signer,
                    wait,
                    vec![action],
                    service_id,
                )?;
            }
            ("delete", Some(m)) => {
                let url = value_of_url(m)?;
                let service_id_str = value_of_service_id(m)?;
                let service_id = service_id_str.as_deref();
                let location_client = client_factory.get_location_client(url);
                let key = value_of_key(m)?;
                let signer = signing::load_signer(key)?;
                let wait = value_t!(m, "wait", u64).unwrap_or(0);

                let namespace = match m.value_of("location_namespace").unwrap_or("GS1") {
                    "GS1" => LocationNamespace::Gs1,
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
                location::do_delete_location(location_client, signer, wait, action, service_id)?;
            }
            ("list", Some(m)) => {
                let url = value_of_url(m)?;
                let service_id_str = value_of_service_id(m)?;
                let service_id = service_id_str.as_deref();
                let location_client = client_factory.get_location_client(url);
                location::do_list_locations(location_client, service_id)?
            }
            ("show", Some(m)) => {
                let url = value_of_url(m)?;
                let service_id_str = value_of_service_id(m)?;
                let service_id = service_id_str.as_deref();
                let location_client = client_factory.get_location_client(url);
                location::do_show_location(
                    location_client,
                    m.value_of("location_id").unwrap(),
                    service_id,
                )?
            }
            _ => return Err(CliError::UserError("Subcommand not recognized".into())),
        },
        #[cfg(feature = "purchase-order")]
        ("po", Some(m)) => match m.subcommand() {
            ("create", Some(m)) => {
                let url = value_of_url(m)?;
                let service_id = value_of_service_id(m)?;
                let purchase_order_client = client_factory.get_purchase_order_client(url);
                let key = value_of_key(m)?;
                let signer = signing::load_signer(key)?;
                let wait = value_t!(m, "wait", u64).unwrap_or(0);

                let uid = m
                    .value_of("uid")
                    .map(String::from)
                    .unwrap_or_else(purchase_order::generate_purchase_order_uid);

                let alternate_ids: Vec<String> = m
                    .values_of("alternate_id")
                    .unwrap_or_default()
                    .map(String::from)
                    .collect();

                if !alternate_ids.is_empty() {
                    for id in &alternate_ids {
                        validate_alt_id_format(id)?;
                    }
                    purchase_order::do_check_alternate_ids_are_unique(
                        &*purchase_order_client,
                        alternate_ids.to_vec(),
                        service_id.as_deref(),
                    )?;
                }

                let client_alternate_ids: Vec<POClientAlternateId> = alternate_ids
                    .iter()
                    .map(|id| purchase_order::make_alternate_id_from_str(&uid, id))
                    .collect::<Result<_, _>>()?;

                let protocol_alternate_ids: Vec<POProtocolAlternateId> = client_alternate_ids
                    .into_iter()
                    .map(|id| id.try_into())
                    .collect::<Result<_, _>>()?;

                let payload = CreatePurchaseOrderPayloadBuilder::new()
                    .with_uid(uid)
                    .with_created_at(
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .map(|d| d.as_secs())
                            .map_err(|err| CliError::PayloadError(format!("{}", err)))?,
                    )
                    .with_buyer_org_id(m.value_of("buyer_org_id").unwrap().into())
                    .with_seller_org_id(m.value_of("seller_org_id").unwrap().into())
                    .with_alternate_ids(protocol_alternate_ids)
                    .with_workflow_state(m.value_of("workflow_state").unwrap().into())
                    .build()
                    .map_err(|err| {
                        CliError::UserError(format!("Could not build Purchase Order: {}", err))
                    })?;

                info!("Submitting request to create purchase order...");
                purchase_order::do_create_purchase_order(
                    &*purchase_order_client,
                    signer,
                    wait,
                    payload,
                    service_id.as_deref(),
                )?;
            }
            ("update", Some(m)) => {
                let url = value_of_url(m)?;
                let service_id = value_of_service_id(m)?;
                let purchase_order_client = client_factory.get_purchase_order_client(url);
                let key = value_of_key(m)?;
                let signer = signing::load_signer(key)?;
                let wait = value_t!(m, "wait", u64).unwrap_or(0);

                let uid = m.value_of("id").map(String::from).unwrap();

                let po = purchase_order::do_fetch_purchase_order(
                    &*purchase_order_client,
                    &uid,
                    service_id.as_deref(),
                )?;

                if let Some(po) = po {
                    let workflow_state = m.value_of("workflow_state").unwrap_or(&po.workflow_state);

                    let mut is_closed = po.is_closed;
                    if m.is_present("is_closed") {
                        is_closed = true;
                    }

                    let mut accepted_version = po.accepted_version_id;

                    if m.is_present("accepted_version") {
                        accepted_version = m
                            .value_of("accepted_version")
                            .map(String::from)
                            .map(Some)
                            .unwrap();
                    }

                    let mut alternate_ids = po.alternate_ids.clone();

                    if m.is_present("add_id") {
                        let adds: Vec<String> =
                            m.values_of("add_id").unwrap().map(String::from).collect();
                        for id in &adds {
                            validate_alt_id_format(id)?;
                        }
                        purchase_order::do_check_alternate_ids_are_unique(
                            &*purchase_order_client,
                            adds.to_vec(),
                            service_id.as_deref(),
                        )?;

                        for a in adds {
                            alternate_ids
                                .push(purchase_order::make_alternate_id_from_str(&uid, &a)?);
                        }
                    }

                    if m.is_present("rm_id") {
                        let rms: Vec<&str> = m.values_of("rm_id").unwrap().collect();

                        for r in rms {
                            let converted = purchase_order::make_alternate_id_from_str(&uid, r)?;
                            alternate_ids.retain(|i| {
                                i.alternate_id_type != converted.alternate_id_type
                                    || i.alternate_id != converted.alternate_id
                            });
                        }
                    }

                    let mut converted_ids = Vec::new();

                    for id in alternate_ids {
                        let converted = id
                            .try_into()
                            .map_err(|err| CliError::PayloadError(format!("{}", err)))?;
                        converted_ids.push(converted);
                    }

                    let payload = UpdatePurchaseOrderPayloadBuilder::new()
                        .with_uid(uid)
                        .with_workflow_state(workflow_state.to_string())
                        .with_is_closed(is_closed)
                        .with_accepted_version_number(
                            accepted_version.as_deref().map(|s| s.to_string()),
                        )
                        .with_alternate_ids(converted_ids)
                        .build()
                        .map_err(|err| {
                            CliError::UserError(format!("Could not build Purchase Order: {}", err))
                        })?;

                    info!("Submitting request to update purchase order...");
                    purchase_order::do_update_purchase_order(
                        &*purchase_order_client,
                        signer,
                        wait,
                        payload,
                        service_id.as_deref(),
                    )?;
                } else {
                    CliError::UserError(format!("Could not fetch Purchase Order {}", &uid));
                }
            }
            ("list", Some(m)) => {
                let url = value_of_url(m)?;
                let service_id = value_of_service_id(m)?;
                let purchase_order_client = client_factory.get_purchase_order_client(url);

                let filter = ListPOFilters {
                    is_open: if m.is_present("open") {
                        Some(true)
                    } else if m.is_present("closed") {
                        Some(false)
                    } else {
                        None
                    },
                    has_accepted_version: if m.is_present("accepted") {
                        Some(true)
                    } else if m.is_present("not_accepted") {
                        Some(false)
                    } else {
                        None
                    },
                    buyer_org_id: if m.is_present("buyer_org") {
                        Some(m.value_of("buyer_org").unwrap().to_string())
                    } else {
                        None
                    },
                    seller_org_id: if m.is_present("seller_org") {
                        Some(m.value_of("seller_org").unwrap().to_string())
                    } else {
                        None
                    },
                    alternate_ids: None,
                };
                let format = m.value_of("format");

                purchase_order::do_list_purchase_orders(
                    &*purchase_order_client,
                    Some(filter),
                    service_id,
                    format,
                )?
            }
            ("show", Some(m)) => {
                let url = value_of_url(m)?;
                let service_id = value_of_service_id(m)?;
                let purchase_order_client = client_factory.get_purchase_order_client(url);
                let purchase_order_id = m.value_of("id").unwrap();
                let format = m.value_of("format");
                purchase_order::do_show_purchase_order(
                    &*purchase_order_client,
                    purchase_order_id.to_string(),
                    service_id,
                    format,
                )?
            }
            ("version", Some(m)) => match m.subcommand() {
                ("create", Some(m)) => {
                    let url = value_of_url(m)?;
                    let service_id = value_of_service_id(m)?;
                    let purchase_order_client = client_factory.get_purchase_order_client(url);
                    let key = value_of_key(m)?;
                    let signer = signing::load_signer(key)?;
                    let wait = value_t!(m, "wait", u64).unwrap_or(0);

                    let order_xml_path = m.value_of("order_xml").unwrap();
                    let data_validation_dir = env::var(GRID_ORDER_SCHEMA_DIR)
                        .unwrap_or_else(|_| DEFAULT_SCHEMA_DIR.to_string() + "/po/gs1/ecom");
                    let mut xml_str = String::new();
                    std::fs::File::open(order_xml_path)?.read_to_string(&mut xml_str)?;
                    validate_order_xml_3_4(&xml_str, false, &data_validation_dir)?;
                    info!("Purchase order was valid.");

                    let version_id = m.value_of("version_id").unwrap();

                    let po = m.value_of("po").unwrap();

                    let workflow_state = m.value_of("workflow_state").unwrap();

                    let revision_id = purchase_order::get_latest_revision_id(
                        &*purchase_order_client,
                        po,
                        version_id,
                        service_id.as_deref(),
                    )? + 1;

                    let draft = !m.is_present("not_draft");

                    let action =
                        CreateVersionPayloadBuilder::new()
                            .with_version_id(version_id.to_string())
                            .with_po_uid(po.to_string())
                            .with_workflow_state(workflow_state.to_string())
                            .with_is_draft(draft)
                            .with_revision(
                                PayloadRevisionBuilder::new()
                                    .with_revision_id(revision_id.try_into().map_err(|err| {
                                        CliError::PayloadError(format!("{}", err))
                                    })?)
                                    .with_submitter(
                                        signer
                                            .public_key()
                                            .map_err(|err| CliError::UserError(format!("{}", err)))?
                                            .as_hex(),
                                    )
                                    .with_created_at(
                                        SystemTime::now()
                                            .duration_since(UNIX_EPOCH)
                                            .map(|d| d.as_secs())
                                            .map_err(|err| {
                                                CliError::PayloadError(format!("{}", err))
                                            })?,
                                    )
                                    .with_order_xml_v3_4(xml_str)
                                    .build()
                                    .map_err(|err| {
                                        CliError::UserError(format!(
                                            "Could not build PO revision: {}",
                                            err
                                        ))
                                    })?,
                            )
                            .build()
                            .map_err(|err| {
                                CliError::UserError(format!("Could not build PO version: {}", err))
                            })?;

                    info!("Submitting request to create purchase order version...");
                    purchase_order::do_create_version(
                        &*purchase_order_client,
                        signer,
                        wait,
                        action,
                        service_id.as_deref(),
                    )?;
                }
                ("list", Some(m)) => {
                    let url = value_of_url(m)?;
                    let service_id = value_of_service_id(m)?;
                    let purchase_order_client = client_factory.get_purchase_order_client(url);

                    let po_uid = m.value_of("po_uid").unwrap();

                    let mut accepted_filter = None;
                    let mut draft_filter = None;

                    if m.is_present("accepted") {
                        accepted_filter = Some(true);
                    } else if m.is_present("not_accepted") {
                        accepted_filter = Some(false);
                    }

                    if m.is_present("draft") {
                        draft_filter = Some(true);
                    } else if m.is_present("not_draft") {
                        draft_filter = Some(false);
                    }

                    let format = Some(m.value_of("format").unwrap());

                    purchase_order::do_list_versions(
                        &*purchase_order_client,
                        po_uid,
                        accepted_filter,
                        draft_filter,
                        format,
                        service_id.as_deref(),
                    )?
                }
                ("show", Some(m)) => {
                    let url = value_of_url(m)?;
                    let service_id = value_of_service_id(m)?;
                    let purchase_order_client = client_factory.get_purchase_order_client(url);

                    let po_uid = m.value_of("po_uid").unwrap();

                    let version = m.value_of("version_id").unwrap();

                    purchase_order::do_show_version(
                        &*purchase_order_client,
                        po_uid,
                        version,
                        service_id.as_deref(),
                    )?;
                }
                ("update", Some(m)) => {
                    let url = value_of_url(m)?;
                    let service_id = value_of_service_id(m)?;
                    let purchase_order_client = client_factory.get_purchase_order_client(url);
                    let key = value_of_key(m)?;
                    let signer = signing::load_signer(key)?;

                    let wait = value_t!(m, "wait", u64).unwrap_or(0);

                    let version_id = m.value_of("version_id").unwrap();

                    let po = m.value_of("po").unwrap();

                    let version = purchase_order::get_purchase_order_version(
                        &*purchase_order_client,
                        po,
                        version_id,
                        service_id.as_deref(),
                    )?;

                    let current_revision = purchase_order::get_current_revision_for_version(
                        &*purchase_order_client,
                        po,
                        &version,
                        service_id.as_deref(),
                    )?;

                    let workflow_state = m
                        .value_of("workflow_state")
                        .unwrap_or(&version.workflow_state);

                    let mut current_revision_id: u64 = version.current_revision_id;

                    let mut new_xml = false;
                    let mut xml_str = current_revision.order_xml_v3_4.to_string();
                    if m.is_present("order_xml") {
                        new_xml = true;
                        let order_xml_path = m.value_of("order_xml").unwrap();
                        let data_validation_dir = env::var(GRID_ORDER_SCHEMA_DIR)
                            .unwrap_or_else(|_| DEFAULT_SCHEMA_DIR.to_string() + "/po/gs1/ecom");
                        xml_str = String::new();
                        std::fs::File::open(order_xml_path)?.read_to_string(&mut xml_str)?;
                        validate_order_xml_3_4(&xml_str, false, &data_validation_dir)?;
                        info!("Purchase order was valid.");

                        current_revision_id = u64::try_from(
                            purchase_order::get_latest_revision_id(
                                &*purchase_order_client,
                                po,
                                version_id,
                                service_id.as_deref(),
                            )? + 1,
                        )
                        .map_err(|err| CliError::PayloadError(format!("{}", err)))?;
                    }

                    let mut draft = version.is_draft;

                    if m.is_present("draft") {
                        draft = true;
                    } else if m.is_present("not_draft") {
                        draft = false;
                    }

                    let created_at = current_revision
                        .created_at
                        .try_into()
                        .map_err(|err| CliError::PayloadError(format!("{}", err)))?;

                    let mut payload_revision = PayloadRevisionBuilder::new()
                        .with_revision_id(current_revision.revision_id)
                        .with_submitter(current_revision.submitter.to_string())
                        .with_created_at(created_at)
                        .with_order_xml_v3_4(current_revision.order_xml_v3_4)
                        .build()
                        .map_err(|err| {
                            CliError::UserError(format!("Could not build PO revision: {}", err))
                        })?;

                    if new_xml {
                        payload_revision = PayloadRevisionBuilder::new()
                            .with_revision_id(current_revision_id)
                            .with_submitter(
                                signer
                                    .public_key()
                                    .map_err(|err| CliError::UserError(format!("{}", err)))?
                                    .as_hex(),
                            )
                            .with_created_at(
                                SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .map(|d| d.as_secs())
                                    .map_err(|err| CliError::PayloadError(format!("{}", err)))?,
                            )
                            .with_order_xml_v3_4(xml_str)
                            .build()
                            .map_err(|err| {
                                CliError::UserError(format!("Could not build PO revision: {}", err))
                            })?;
                    }

                    let action = UpdateVersionPayloadBuilder::new()
                        .with_version_id(version_id.to_string())
                        .with_po_uid(po.to_string())
                        .with_workflow_state(workflow_state.to_string())
                        .with_is_draft(draft)
                        .with_revision(payload_revision)
                        .build()
                        .map_err(|err| {
                            CliError::UserError(format!("Could not build PO version: {}", err))
                        })?;

                    info!("Submitting request to update purchase order version...");
                    purchase_order::do_update_version(
                        &*purchase_order_client,
                        signer,
                        wait,
                        action,
                        service_id.as_deref(),
                    )?;
                }
                _ => return Err(CliError::UserError("Subcommand not recognized".into())),
            },
            ("revision", Some(m)) => match m.subcommand() {
                ("list", Some(m)) => {
                    let url = value_of_url(m)?;
                    let service_id = value_of_service_id(m)?;
                    let purchase_order_client = client_factory.get_purchase_order_client(url);

                    let po_uid = m.value_of("po_uid").unwrap();

                    let version = m.value_of("version_id").unwrap();

                    purchase_order::do_list_revisions(
                        &*purchase_order_client,
                        po_uid,
                        version,
                        service_id.as_deref(),
                    )?
                }
                ("show", Some(m)) => {
                    let url = value_of_url(m)?;
                    let service_id = value_of_service_id(m)?;
                    let purchase_order_client = client_factory.get_purchase_order_client(url);

                    let po_uid = m.value_of("po_uid").unwrap();

                    let version = m.value_of("version_id").unwrap();

                    let revision_str = m.value_of("revision_number").unwrap();

                    let revision = revision_str
                        .parse::<u64>()
                        .map_err(|err| CliError::UserError(format!("{}", err)))?;

                    purchase_order::do_show_revision(
                        &*purchase_order_client,
                        po_uid,
                        version,
                        revision,
                        service_id.as_deref(),
                    )?;
                }
                _ => return Err(CliError::UserError("Subcommand not recognized".into())),
            },
            _ => return Err(CliError::UserError("Subcommand not recognized".into())),
        },
        _ => return Err(CliError::UserError("Subcommand not recognized".into())),
    }

    Ok(())
}

#[cfg(any(
    feature = "location",
    feature = "pike",
    feature = "product",
    feature = "schema",
    feature = "purchase-order",
))]
fn value_of_url(matches: &ArgMatches) -> Result<String, CliError> {
    let url = matches
        .value_of("url")
        .map(String::from)
        .or_else(|| env::var(GRID_DAEMON_ENDPOINT).ok())
        .unwrap_or_else(|| String::from("http://localhost:8000"));
    Ok(url)
}

#[cfg(any(
    feature = "location",
    feature = "pike",
    feature = "product",
    feature = "schema",
    feature = "purchase-order",
))]
fn value_of_service_id(matches: &ArgMatches) -> Result<Option<String>, CliError> {
    let service_id_string = matches
        .value_of("service_id")
        .map(String::from)
        .or_else(|| env::var(GRID_SERVICE_ID).ok());
    Ok(service_id_string)
}

#[cfg(any(
    feature = "location",
    feature = "pike",
    feature = "product",
    feature = "schema",
    feature = "purchase-order",
))]
fn value_of_key(matches: &ArgMatches) -> Result<Option<String>, CliError> {
    let key = matches
        .value_of("key")
        .map(String::from)
        .or_else(|| env::var(GRID_DAEMON_KEY).ok());
    Ok(key)
}

#[cfg(feature = "pike")]
fn parse_alternate_ids(matches: &ArgMatches) -> Result<Vec<AlternateId>, CliError> {
    let ids = matches
        .values_of("alternate_ids")
        .unwrap_or_default()
        .map(String::from)
        .collect::<Vec<String>>();

    let mut alternate_ids = Vec::new();

    for id in ids {
        let entries = id.split(':').map(String::from).collect::<Vec<String>>();

        let (id_type, alt_id) = if entries.len() != 2 {
            return Err(CliError::UserError(format!(
                "Alternate ID malformed: {}",
                id
            )));
        } else {
            (entries[0].clone(), entries[1].clone())
        };

        alternate_ids.push(
            AlternateIdBuilder::new()
                .with_id_type(id_type)
                .with_id(alt_id)
                .build()
                .map_err(|err| CliError::UserError(format!("Alternate ID malformed: {}", err)))?,
        )
    }

    Ok(alternate_ids)
}

#[cfg(feature = "pike")]
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

#[cfg(any(feature = "location", feature = "product",))]
fn parse_properties(
    client: Box<dyn SchemaClient>,
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

    let schemas = client.get_schema(String::from(namespace), service_id)?;

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
            grid_schema_client::DataType::Number => {
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
            grid_schema_client::DataType::Enum => {
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
            grid_schema_client::DataType::String => {
                let property_value = PropertyValueBuilder::new()
                    .with_name(property.name)
                    .with_data_type(property.data_type.into())
                    .with_string_value(value.into())
                    .build()
                    .map_err(|err| CliError::UserError(format!("{}", err)))?;

                property_values.push(property_value);
            }
            grid_schema_client::DataType::LatLong => {
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
            grid_schema_client::DataType::Boolean => {
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
            grid_schema_client::DataType::Bytes => {
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
            grid_schema_client::DataType::Struct => {
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
