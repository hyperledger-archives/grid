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

use crate::error::CliError;
use crate::store::default_value::{
    DefaultStoreError, DefaultValue, DefaultValueStore, FileBackedDefaultStore,
};

const MANAGEMENT_TYPE_KEY: &str = "management_type";
const SERVICE_TYPE_KEY: &str = "service_type";

use super::Action;

pub struct SetDefaultValueAction;

impl Action for SetDefaultValueAction {
    fn run<'a>(&mut self, arg_matches: Option<&ArgMatches<'a>>) -> Result<(), CliError> {
        let args = arg_matches.ok_or_else(|| CliError::RequiresArgs)?;

        let name = match args.value_of("name") {
            Some(key) => key,
            None => return Err(CliError::ActionError("name is required".into())),
        };

        let key = get_key(name)?;

        let value = match args.value_of("value") {
            Some(value) => value,
            None => return Err(CliError::ActionError("value is required".into())),
        };

        let store = get_default_value_store();

        if !args.is_present("force") && store.get_default_value(key)?.is_some() {
            return Err(CliError::ActionError(format!(
                "Default value for {} is already in use",
                key
            )));
        }

        let default_value = DefaultValue::new(key, value);
        store.set_default_value(&default_value)?;

        Ok(())
    }
}

fn get_key(name: &str) -> Result<&str, CliError> {
    match name {
        "service-type" => Ok(SERVICE_TYPE_KEY),
        "management-type" => Ok(MANAGEMENT_TYPE_KEY),
        _ => Err(CliError::ActionError(format!(
            "{} is not a valid default name",
            name
        ))),
    }
}

fn get_default_value_store() -> FileBackedDefaultStore {
    FileBackedDefaultStore::default()
}

impl From<DefaultStoreError> for CliError {
    fn from(err: DefaultStoreError) -> Self {
        CliError::ActionError(format!("Failed to perform defaults operation: {}", err))
    }
}
