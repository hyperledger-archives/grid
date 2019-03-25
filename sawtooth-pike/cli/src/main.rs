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

extern crate addresser;
#[macro_use]
extern crate clap;
extern crate crypto;
extern crate futures;
extern crate hyper;
extern crate protobuf;
extern crate sawtooth_sdk;
extern crate tokio_core;
extern crate users;

mod error;
mod key;
mod payload;
mod protos;
mod transaction;
mod submit;

const APP_NAME: &'static str = env!("CARGO_PKG_NAME");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

use std::fs::File;
use std::io::prelude::*;

use sawtooth_sdk::signing;
use sawtooth_sdk::signing::PrivateKey;

use error::CliError;
use key::load_signing_key;
use payload::{
    create_agent_payload,
    create_org_payload,
    update_agent_payload,
    update_org_payload
};
use submit::submit_batch_list;

use protos::payload::PikePayload;
use protos::state::KeyValueEntry;

use protobuf::Message;

fn do_create(
        url: &str,
        private_key: &PrivateKey,
        payload: &PikePayload,
        output: &str) -> Result<(), CliError> {

    if !output.is_empty() {
        let mut buffer = File::create(output)?;
        let payload_bytes = payload.write_to_bytes()?;
        buffer.write_all(&payload_bytes).map_err(|err| CliError::IoError(err))?;
        return Ok(())
    }

    let context = signing::create_context("secp256k1")?;
    let public_key = context.get_public_key(private_key)?;
    let factory = signing::CryptoFactory::new(&*context);
    let signer = factory.new_signer(private_key);

    println!("creating resource {:?}", payload);

    let txn = transaction::create_transaction(&payload, &signer, &public_key.as_hex())?;
    let batch = transaction::create_batch(txn, &signer, &public_key.as_hex())?;
    let batch_list = transaction::create_batch_list_from_one(batch);

    submit_batch_list(
        &format!("{}/batches?wait=120", url),
        &batch_list)
}

fn run() -> Result<(), CliError> {
    let matches = clap_app!(myapp =>
        (name: APP_NAME)
        (version: VERSION)
        (author: "Cargill")
        (about: "Sawtooth Pike CLI")
        (@arg url: --url +takes_value "Rest api url")
        (@setting SubcommandRequiredElseHelp)
        (@subcommand agent =>
            (@setting SubcommandRequiredElseHelp)
            (about: "agent commamds")
            (@subcommand create =>
                (about: "create an agent")
                (@arg org_id: +required "Organization IDs")
                (@arg public_key: +required "Agents public_key")
                (@arg roles: --roles +takes_value +multiple "Roles assigned to agent")
                (@arg metadata: --metadata +takes_value +multiple
                    "Comma seperated key value pairs stored in metadata")
                (@arg key: -k +takes_value "Admin agent's key name")
                (@arg output: --output -o +takes_value "File name to write payload to.")
            )
            (@subcommand update =>
                (about: "update an agent")
                (@arg org_id: +required "Organization IDs")
                (@arg public_key: +required "Agents public_key")
                (@arg roles: --roles +takes_value +multiple "Roles assigned to agent")
                (@arg metadata: --metadata +takes_value +multiple
                    "Comma seperated key value pairs stored in metadata")
                (@arg key: -k +takes_value "Admin agent's key name")
                (@arg output: --output -o +takes_value "File name to write payload to.")
            )
        )
        (@subcommand org =>
            (@setting SubcommandRequiredElseHelp)
            (about: "organization commamds")
            (@subcommand create =>
                (about: "create an organization")
                (@arg id: +required "Unique ID for organization")
                (@arg name: +required "Name of the organization")
                (@arg address: "Physical address for organization")
                (@arg key: -k +takes_value "Agent's signing key")
                (@arg output: --output -o +takes_value "File name to write payload to.")
            )
            (@subcommand update =>
                (about: "update an organization")
                (@arg id: +required "Unique ID for organization")
                (@arg name: +required "Name of the organization")
                (@arg address: "Physical address for organization")
                (@arg key: -k +takes_value "Agent's key name")
                (@arg output: --output -o +takes_value "File name to write payload to.")
            )
        )
    ).get_matches();

    let url = matches
        .value_of("url")
        .unwrap_or("http://pike-api:9001");

    if let Some(matches) = matches.subcommand_matches("agent") {
        if let Some(matches) = matches.subcommand_matches("create") {
            let key_name = matches.value_of("key");
            let org_id = matches.value_of("org_id").unwrap();
            let public_key = matches.value_of("public_key").unwrap();
            let output = matches.value_of("output").unwrap_or("");
            let roles = matches
                .values_of("roles")
                .unwrap_or(clap::Values::default())
                .map(String::from)
                .collect();
            let metadata_as_strings: Vec<String> = matches
                .values_of("metadata")
                .unwrap_or(clap::Values::default())
                .map(String::from)
                .collect();

            let mut metadata = Vec::<KeyValueEntry>::new();
            for meta in metadata_as_strings {
                let key_val: Vec<&str> = meta.split(",").collect();
                if key_val.len() != 2 {
                    return Err(CliError::UserError(
                        "Metadata is formated incorrectly".to_string(),
                    ));
                }
                let key = match key_val.get(0) {
                    Some(key) => key.to_string(),
                    None => {
                        return Err(CliError::UserError(
                            "Metadata is formated incorrectly".to_string(),
                        ))
                    }
                };
                let value = match key_val.get(1) {
                    Some(value) => value.to_string(),
                    None => {
                        return Err(CliError::UserError(
                            "Metadata is formated incorrectly".to_string(),
                        ))
                    }
                };
                let mut entry = KeyValueEntry::new();
                entry.set_key(key);
                entry.set_value(value);
                metadata.push(entry.clone());
            }

            let private_key = load_signing_key(key_name)?;

            let context = signing::create_context("secp256k1")?;

            let payload = create_agent_payload(org_id, public_key, roles, metadata);
            do_create(&url, &private_key, &payload, &output)?;
        } else if let Some(matches) = matches.subcommand_matches("update") {
            let key_name = matches.value_of("key");
            let org_id = matches.value_of("org_id").unwrap();
            let public_key = matches.value_of("public_key").unwrap();
            let output = matches.value_of("output").unwrap_or("");
            let roles = matches
                .values_of("roles")
                .unwrap_or(clap::Values::default())
                .map(String::from)
                .collect();
            let metadata_as_strings: Vec<String> = matches
                .values_of("metadata")
                .unwrap_or(clap::Values::default())
                .map(String::from)
                .collect();
            let mut metadata = Vec::<KeyValueEntry>::new();
            for meta in metadata_as_strings {
                let key_val: Vec<&str> = meta.split(",").collect();
                if key_val.len() != 2 {
                    return Err(CliError::UserError(
                        "Metadata is formated incorrectly".to_string(),
                    ));
                }
                let key = match key_val.get(0) {
                    Some(key) => key.to_string(),
                    None => {
                        return Err(CliError::UserError(
                            "Metadata is formated incorrectly".to_string(),
                        ))
                    }
                };
                let value = match key_val.get(1) {
                    Some(value) => value.to_string(),
                    None => {
                        return Err(CliError::UserError(
                            "Metadata is formated incorrectly".to_string(),
                        ))
                    }
                };
                let mut entry = KeyValueEntry::new();
                entry.set_key(key);
                entry.set_value(value);
                metadata.push(entry.clone());
            }

            let private_key = load_signing_key(key_name)?;

            let context = signing::create_context("secp256k1")?;
            let payload = update_agent_payload(org_id, public_key, roles, metadata);
            do_create(&url, &private_key, &payload, &output)?;
        }
    }

    if let Some(matches) = matches.subcommand_matches("org") {
        if let Some(matches) = matches.subcommand_matches("create") {
            let name = matches.value_of("name").unwrap();
            let id = matches.value_of("id").unwrap();
            let address = matches.value_of("address");
            let key_name = matches.value_of("key");
            let output = matches.value_of("output").unwrap_or("");

            let private_key = load_signing_key(key_name)?;

            let payload = create_org_payload(id, name, address);

            do_create(&url, &private_key, &payload, &output)?;
        }
        else if let Some(matches) = matches.subcommand_matches("update") {

            let name = matches.value_of("name").unwrap();
            let id = matches.value_of("id").unwrap();
            let address = matches.value_of("address");
            let key_name = matches.value_of("key");
            let output = matches.value_of("output").unwrap_or("");

            let private_key = load_signing_key(key_name)?;

            let payload = update_org_payload(id, name, address);

            do_create(&url, &private_key, &payload, &output)?;
        }
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        println!("{}", e);
        std::process::exit(1);
    }
}
