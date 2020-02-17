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
    CommandFailed(String),
    InvalidSubcommand(String),
    MissingArgument(String),
}

impl Error for CliError {}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::CommandFailed(err) => write!(f, "command failed to execute: {}", err),
            Self::InvalidSubcommand(subcmd) => write!(f, "invalid subcommand provided: {}", subcmd),
            Self::MissingArgument(arg) => write!(f, "missing required argument: {}", arg),
        }
    }
}
