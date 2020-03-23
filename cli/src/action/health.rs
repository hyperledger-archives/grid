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

use super::{Action, DEFAULT_ENDPOINT, SPLINTER_REST_API_URL_ENV};

use crate::error::CliError;

pub struct StatusAction;

impl Action for StatusAction {
    fn run<'a>(&mut self, arg_matches: Option<&ArgMatches<'a>>) -> Result<(), CliError> {
        let url = arg_matches
            .and_then(|args| args.value_of("url"))
            .map(ToOwned::to_owned)
            .or_else(|| std::env::var(SPLINTER_REST_API_URL_ENV).ok())
            .unwrap_or_else(|| DEFAULT_ENDPOINT.to_string());

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
