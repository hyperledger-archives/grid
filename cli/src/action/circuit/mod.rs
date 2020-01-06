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

use clap::ArgMatches;

use crate::error::CliError;

use super::Action;

pub struct CircuitCreateAction;

impl Action for CircuitCreateAction {
    fn run<'a>(&mut self, arg_matches: Option<&ArgMatches<'a>>) -> Result<(), CliError> {
        let args = arg_matches.ok_or_else(|| CliError::RequiresArgs)?;
        let url = args.value_of("url").unwrap_or("http://localhost:8085");
        let key = args.value_of("private_key_file").unwrap_or("splinter");
        let path = match args.value_of("path") {
            Some(path) => path,
            None => return Err(CliError::ActionError("Path is required".into())),
        };

        create_circuit_proposal(url, key, path)
    }
}

fn create_circuit_proposal(_url: &str, _key: &str, _path: &str) -> Result<(), CliError> {
    unimplemented!()
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
