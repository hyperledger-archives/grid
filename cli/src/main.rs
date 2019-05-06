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

mod actions;
mod error;
mod http;
mod key;
mod transaction;

use clap::ArgMatches;
use grid_sdk::protocol::pike::{
    payload::{
        CreateAgentActionBuilder, CreateOrganizationActionBuilder, UpdateAgentActionBuilder,
        UpdateOrganizationActionBuilder,
    },
    state::{KeyValueEntry, KeyValueEntryBuilder},
};
use simple_logger;

use crate::error::CliError;

use actions::{agents, organizations as orgs, schemas};

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn run() -> Result<(), CliError> {
    let matches = clap_app!(myapp =>
        (name: APP_NAME)
        (version: VERSION)
        (author: "Contributors to Hyperledger Grid")
        (about: "Command line for Hyperledger Grid")
        (@arg url: --url  +takes_value "URL for the REST API")
        (@arg wait: --wait +takes_value "How long to wait for transaction to be committed")
        (@arg key: -k +takes_value "base name for private key file")
        (@arg verbose: -v +multiple "Log verbosely")
        (@subcommand agent =>
            (about: "Update or create agent")
            (@setting SubcommandRequiredElseHelp)
            (@subcommand create =>
                (about: "Create an agent")
                (@arg org_id: +takes_value +required "organization ID")
                (@arg public_key: +takes_value +required "public key")
                (@arg active: "Is user active")
                (@arg roles: --roles +takes_value +multiple "Roles assigned to agent")
                (@arg metadata: --metadata +takes_value +multiple
                    "Comma-separated key value pairs stored in metadata")
            )
            (@subcommand update =>
                (about: "Update an agent")
                (@arg org_id: +takes_value +required "organization ID")
                (@arg public_key: +takes_value +required "public key")
                (@arg active: "Is user active")
                (@arg roles: --roles +takes_value +multiple "Roles assigned to agent")
                (@arg metadata: --metadata +takes_value +multiple
                    "Comma-separated key value pairs stored in metadata")
            )
        )
        (@subcommand organization =>
            (about: "Update or create organization")
            (@setting SubcommandRequiredElseHelp)
            (@subcommand create =>
                (about: "Create an organization")
                (@arg org_id: +required +takes_value "Unique ID for organization")
                (@arg name: +required +takes_value "Name of the organization")
                (@arg address: +takes_value "Physical address for organization")
                (@arg metadata: --metadata +takes_value +multiple
                    "Comma-separated key value pairs stored in metadata")
            )
            (@subcommand update =>
                (about: "Update an organization")
                (@arg org_id: +required +takes_value "Unique ID for organization")
                (@arg name: +required +takes_value "Name of the organization")
                (@arg address: +takes_value "Physical address for organization")
                (@arg metadata: --metadata +takes_value +multiple
                    "Comma-separated key value pairs stored in metadata")
            )
        )
        (@subcommand schema =>
            (about: "Update or create schemas")
            (@setting SubcommandRequiredElseHelp)
            (@subcommand create =>
                (about: "Create schemas from a yaml file")
                (@arg path: +takes_value +required "Path to yaml file containing a list of schema definitions")
            )
            (@subcommand update =>
                (about: "Update schemas from a yaml file")
                (@arg path: +takes_value +required "Path to yaml file containing a list of schema definitions")
            )
        )
    )
    .get_matches();

    match matches.occurrences_of("verbose") {
        0 => simple_logger::init_with_level(log::Level::Warn),
        1 => simple_logger::init_with_level(log::Level::Info),
        _ => simple_logger::init_with_level(log::Level::Debug),
    }?;

    let url = matches.value_of("url").unwrap_or("http://localhost:8000");

    let key = matches.value_of("key").map(ToString::to_string);

    let wait = value_t!(matches, "wait", u64).unwrap_or(0);

    match matches.subcommand() {
        ("agent", Some(m)) => match m.subcommand() {
            ("create", Some(m)) => {
                let create_agent = CreateAgentActionBuilder::new()
                    .with_org_id(m.value_of("org_id").unwrap().into())
                    .with_public_key(m.value_of("public_key").unwrap().into())
                    .with_active(m.is_present("active"))
                    .with_roles(
                        m.values_of("roles")
                            .unwrap_or_default()
                            .map(String::from)
                            .collect::<Vec<String>>(),
                    )
                    .with_metadata(parse_metadata(&m)?)
                    .build()
                    .map_err(|err| CliError::UserError(format!("{}", err)))?;

                agents::do_create_agent(&url, key, wait, create_agent)?
            }
            ("update", Some(m)) => {
                let update_agent = UpdateAgentActionBuilder::new()
                    .with_org_id(m.value_of("org_id").unwrap().into())
                    .with_public_key(m.value_of("public_key").unwrap().into())
                    .with_active(m.is_present("active"))
                    .with_roles(
                        m.values_of("roles")
                            .unwrap_or_default()
                            .map(String::from)
                            .collect::<Vec<String>>(),
                    )
                    .with_metadata(parse_metadata(&m)?)
                    .build()
                    .map_err(|err| CliError::UserError(format!("{}", err)))?;

                agents::do_update_agent(&url, key, wait, update_agent)?
            }
            _ => return Err(CliError::UserError("Subcommand not recognized".into())),
        },
        ("organization", Some(m)) => match m.subcommand() {
            ("create", Some(m)) => {
                let create_org = CreateOrganizationActionBuilder::new()
                    .with_org_id(m.value_of("org_id").unwrap().into())
                    .with_name(m.value_of("name").unwrap().into())
                    .with_address(m.value_of("address").unwrap().into())
                    .with_metadata(parse_metadata(&m)?)
                    .build()
                    .map_err(|err| CliError::UserError(format!("{}", err)))?;

                orgs::do_create_organization(&url, key, wait, create_org)?
            }
            ("update", Some(m)) => {
                let update_org = UpdateOrganizationActionBuilder::new()
                    .with_org_id(m.value_of("org_id").unwrap().into())
                    .with_name(m.value_of("name").unwrap().into())
                    .with_address(m.value_of("address").unwrap().into())
                    .with_metadata(parse_metadata(&m)?)
                    .build()
                    .map_err(|err| CliError::UserError(format!("{}", err)))?;

                orgs::do_update_organization(&url, key, wait, update_org)?
            }
            _ => return Err(CliError::UserError("Subcommand not recognized".into())),
        },
        ("schema", Some(m)) => match m.subcommand() {
            ("create", Some(m)) => {
                schemas::do_create_schemas(&url, key, wait, m.value_of("path").unwrap())?
            }
            ("update", Some(m)) => {
                schemas::do_update_schemas(&url, key, wait, m.value_of("path").unwrap())?
            }
            _ => return Err(CliError::UserError("Subcommand not recognized".into())),
        },
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

fn main() {
    if let Err(e) = run() {
        error!("{:?}", e);
        std::process::exit(1);
    }
}
