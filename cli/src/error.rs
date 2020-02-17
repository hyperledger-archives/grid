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
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum CliError {
    RequiresArgs,
    InvalidSubcommand,
    ActionError(String),
    EnvironmentError(String),
    #[cfg(feature = "database")]
    DatabaseError(String),
}

impl Error for CliError {}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CliError::RequiresArgs => write!(f, "action requires arguments"),
            CliError::InvalidSubcommand => write!(f, "received invalid subcommand"),
            CliError::ActionError(msg) => write!(f, "action encountered an error: {}", msg),
            CliError::EnvironmentError(msg) => {
                write!(f, "action encountered an environment error: {}", msg)
            }
            #[cfg(feature = "database")]
            CliError::DatabaseError(msg) => write!(f, "database error: {}", msg),
        }
    }
}
