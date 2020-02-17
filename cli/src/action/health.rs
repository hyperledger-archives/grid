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

use clap::ArgMatches;
use reqwest;
use serde_json::Value;

use super::Action;
use crate::action::DEFAULT_ENDPOINT;
use crate::error::CliError;

pub struct StatusAction;

impl Action for StatusAction {
    fn run<'a>(&mut self, arg_matches: Option<&ArgMatches<'a>>) -> Result<(), CliError> {
        let url = if let Some(args) = arg_matches {
            args.value_of("url").unwrap_or(DEFAULT_ENDPOINT)
        } else {
            DEFAULT_ENDPOINT
        };

        let status: Value = reqwest::blocking::get(&format!("{}/health/status", url))
            .and_then(|res| res.json())
            .map_err(|err| CliError::ActionError(format!("{:?}", err)))?;

        println!(
            "{}",
            serde_json::to_string_pretty(&status)
                .map_err(|_| CliError::ActionError("Failed to serialize response".into()))?
        );
        Ok(())
    }
}
