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
use std::io::Error as IoError;

#[derive(Clone, Debug)]
pub enum CliError {
    UserError(String),
    IoError(String),
    InvalidSubcommand,
}

impl Error for CliError {
    fn cause(&self) -> Option<&dyn Error> {
        None
    }

    fn description(&self) -> &str {
        match self {
            CliError::UserError(_) => "User provided invalid input",
            CliError::InvalidSubcommand => "Subcommand is not supported",
            CliError::IoError(_) => "Received IO Error",
        }
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CliError::UserError(msg) => write!(f, "UserError: {}", msg),
            CliError::InvalidSubcommand => write!(f, "Subcommand is not supported"),
            CliError::IoError(msg) => write!(f, "IoError: {}", msg),
        }
    }
}

impl From<IoError> for CliError {
    fn from(err: IoError) -> Self {
        CliError::IoError(format!("{}", err))
    }
}
