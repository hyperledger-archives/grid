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
    let key = buf.trim().to_string();

    Ok(key)
}

pub(self) enum Vote {
    Accept,
    Reject,
}

pub(self) struct CircuitVote {
    circuit_id: String,
    circuit_hash: String,
    vote: Vote,
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
    url: &str,
    key: &str,
    circuit_id: &str,
    vote: Vote,
) -> Result<(), CliError> {
    let client = api::SplinterRestClient::new(url);
    let private_key_hex = read_private_key(key)?;

    let requester_node = client.fetch_node_id()?;
    let proposal = client.fetch_proposal(circuit_id)?;

    if let Some(proposal) = proposal {
        let circuit_vote = CircuitVote {
            circuit_id: circuit_id.into(),
            circuit_hash: proposal.circuit_hash,
            vote,
        };

        let signed_payload =
            payload::make_signed_payload(&requester_node, &private_key_hex, circuit_vote)?;

        client.submit_admin_payload(signed_payload)
    } else {
        Err(CliError::ActionError(format!(
            "Proposal for {} does not exist",
            circuit_id
        )))
    }
}

pub struct CircuitListAction;

impl Action for CircuitListAction {
    fn run<'a>(&mut self, arg_matches: Option<&ArgMatches<'a>>) -> Result<(), CliError> {
        let args = arg_matches.ok_or_else(|| CliError::RequiresArgs)?;

        let url = args.value_of("url").unwrap_or("http://127.0.0.1:8080");

        let filter = args.value_of("member");

        list_circuits(url, filter)
    }
}

fn list_circuits(url: &str, filter: Option<&str>) -> Result<(), CliError> {
    let client = api::SplinterRestClient::new(url);

    let circuits = client.list_circuits(filter)?;
    println!(
        "{0: <80} | {1: <30}",
        "CIRCUIT ID", "CIRCUIT MANAGEMENT TYPE",
    );
    println!("{}", "-".repeat(110));
    circuits.data.iter().for_each(|circuit| {
        println!(
            "{0: <80} | {1: <30}",
            circuit.id, circuit.circuit_management_type,
        );
    });
    Ok(())
}

pub struct CircuitShowAction;

impl Action for CircuitShowAction {
    fn run<'a>(&mut self, arg_matches: Option<&ArgMatches<'a>>) -> Result<(), CliError> {
        let args = arg_matches.ok_or_else(|| CliError::RequiresArgs)?;

        let url = args.value_of("url").unwrap_or("http://127.0.0.1:8080");
        let circuit_id = args
            .value_of("circuit")
            .ok_or_else(|| CliError::ActionError("Circuit name must be provided".to_string()))?;

        // A value should always be passed because a default is defined
        let format = args.value_of("format").expect("format was not provided");

        show_circuit(url, circuit_id, format)
    }
}

fn show_circuit(url: &str, circuit_id: &str, format: &str) -> Result<(), CliError> {
    let client = api::SplinterRestClient::new(url);
    let circuit = client.fetch_circuit(circuit_id)?;
    let mut print_circuit = false;
    let mut print_proposal = false;
    if let Some(circuit) = circuit {
        print_circuit = true;
        match format {
            "json" => println!(
                "\n {}",
                serde_json::to_string(&circuit).map_err(|err| CliError::ActionError(format!(
                    "Cannot format circuit into json: {}",
                    err
                )))?
            ),
            // default is yaml
            _ => println!(
                "{}",
                serde_yaml::to_string(&circuit).map_err(|err| CliError::ActionError(format!(
                    "Cannot format circuit into yaml: {}",
                    err
                )))?
            ),
        }
    }

    let proposal = client.fetch_proposal(circuit_id)?;

    if let Some(proposal) = proposal {
        print_proposal = true;
        match format {
            "json" => println!(
                "\n {}",
                serde_json::to_string(&proposal).map_err(|err| CliError::ActionError(format!(
                    "Cannot format proposal into json: {}",
                    err
                )))?
            ),
            // default is yaml
            _ => println!(
                "{}",
                serde_yaml::to_string(&proposal).map_err(|err| CliError::ActionError(format!(
                    "Cannot format proposal into yaml: {}",
                    err
                )))?
            ),
        }
    }

    if !print_circuit && !print_proposal {
        return Err(CliError::ActionError(format!(
            "Proposal for {} does not exist",
            circuit_id
        )));
    }

    Ok(())
}

pub struct CircuitProposalsAction;

impl Action for CircuitProposalsAction {
    fn run<'a>(&mut self, arg_matches: Option<&ArgMatches<'a>>) -> Result<(), CliError> {
        let args = arg_matches.ok_or_else(|| CliError::RequiresArgs)?;

        let url = args.value_of("url").unwrap_or("http://127.0.0.1:8080");

        let filter = args.value_of("management_type");

        list_proposals(url, filter)
    }
}

fn list_proposals(url: &str, filter: Option<&str>) -> Result<(), CliError> {
    let client = api::SplinterRestClient::new(url);

    let proposals = client.list_proposals(filter)?;
    println!(
        "{0: <80} | {1: <30}",
        "CIRCUIT ID", "CIRCUIT MANAGEMENT TYPE",
    );
    println!("{}", "-".repeat(110));
    proposals.data.iter().for_each(|proposal| {
        println!(
            "{0: <80} | {1: <30}",
            proposal.circuit_id, proposal.circuit.circuit_management_type,
        );
    });
    Ok(())
}
