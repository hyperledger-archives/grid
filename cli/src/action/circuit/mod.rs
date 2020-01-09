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

mod api;
mod payload;

use std::fs::File;
use std::io::Read;

use clap::ArgMatches;
use splinter::admin::messages::CreateCircuit;

use crate::error::CliError;

use super::Action;

pub struct CircuitCreateAction;

impl Action for CircuitCreateAction {
    fn run<'a>(&mut self, arg_matches: Option<&ArgMatches<'a>>) -> Result<(), CliError> {
        let args = arg_matches.ok_or_else(|| CliError::RequiresArgs)?;
        let url = args.value_of("url").unwrap_or("http://localhost:8085");
        let key = args
            .value_of("private_key_file")
            .unwrap_or("./splinter.priv");
        let path = match args.value_of("path") {
            Some(path) => path,
            None => return Err(CliError::ActionError("Path is required".into())),
        };

        create_circuit_proposal(url, key, path)
    }
}

fn create_circuit_proposal(
    url: &str,
    private_key_file: &str,
    proposal_path: &str,
) -> Result<(), CliError> {
    let client = api::SplinterRestClient::new(url);
    let requester_node = client.fetch_node_id()?;
    let private_key_hex = read_private_key(private_key_file)?;

    let proposal_file = File::open(proposal_path).map_err(|err| {
        CliError::EnvironmentError(format!("Unable to open {}: {}", proposal_path, err))
    })?;

    let create_request: CreateCircuit = serde_yaml::from_reader(proposal_file).map_err(|err| {
        CliError::EnvironmentError(format!("Unable to parse {}: {}", proposal_path, err))
    })?;

    let signed_payload =
        payload::make_signed_payload(&requester_node, &private_key_hex, create_request)?;

    client.submit_admin_payload(signed_payload)
}

/// Reads a private key from the given file name.
pub fn read_private_key(file_name: &str) -> Result<String, CliError> {
    let mut file = File::open(file_name).map_err(|err| {
        CliError::EnvironmentError(format!("Unable to open {}: {}", file_name, err))
    })?;

    let mut buf = String::new();

    file.read_to_string(&mut buf).map_err(|err| {
        CliError::EnvironmentError(format!("Unable to read {}: {}", file_name, err))
    })?;

    Ok(buf)
}

enum Vote {
    Accept,
    Reject,
}

pub struct CircuitVoteAction;

impl Action for CircuitVoteAction {
    fn run<'a>(&mut self, arg_matches: Option<&ArgMatches<'a>>) -> Result<(), CliError> {
        let args = arg_matches.ok_or_else(|| CliError::RequiresArgs)?;
        let url = args.value_of("url").unwrap_or("http://localhost:8085");
        let key = args.value_of("private_key_file").unwrap_or("splinter");
        let circuit_id = match args.value_of("circuit_id") {
            Some(circuit_id) => circuit_id,
            None => return Err(CliError::ActionError("Circuit id is required".into())),
        };

        // accept or reject must be present
        let vote = {
            if args.is_present("accept") {
                Vote::Accept
            } else {
                Vote::Reject
            }
        };

        vote_on_circuit_proposal(url, key, circuit_id, vote)
    }
}

fn vote_on_circuit_proposal(
    _url: &str,
    _key: &str,
    _path: &str,
    _vote: Vote,
) -> Result<(), CliError> {
    unimplemented!()
}
