// Copyright 2019 Cargill Incorporated
// Copyright 2018 Intel Corporation
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
pub mod admin;
pub mod certs;
#[cfg(feature = "database")]
pub mod database;
#[cfg(feature = "health")]
pub mod health;

use std::collections::HashMap;
use std::ffi::CString;
use std::path::Path;

use clap::ArgMatches;
use flexi_logger::ReconfigurationHandle;
use libc;

use super::error::CliError;

/// A CLI Command Action.
///
/// An Action is a single subcommand for CLI operations.
pub trait Action {
    /// Run a CLI Action with the given args
    fn run<'a>(
        &mut self,
        arg_matches: Option<&ArgMatches<'a>>,
        logger_handle: &mut ReconfigurationHandle,
    ) -> Result<(), CliError>;
}

/// A collection of Subcommands associated with a single parent command.
#[derive(Default)]
pub struct SubcommandActions<'a> {
    actions: HashMap<String, Box<dyn Action + 'a>>,
}

impl<'a> SubcommandActions<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_command<'action: 'a, A: Action + 'action>(
        mut self,
        command: &str,
        action: A,
    ) -> Self {
        self.actions.insert(command.to_string(), Box::new(action));

        self
    }
}

impl<'s> Action for SubcommandActions<'s> {
    fn run<'a>(
        &mut self,
        arg_matches: Option<&ArgMatches<'a>>,
        logger_handle: &mut ReconfigurationHandle,
    ) -> Result<(), CliError> {
        let args = arg_matches.ok_or_else(|| CliError::RequiresArgs)?;

        let (subcommand, args) = args.subcommand();

        if let Some(action) = self.actions.get_mut(subcommand) {
            action.run(args, logger_handle)
        } else {
            Err(CliError::InvalidSubcommand)
        }
    }
}

fn chown(path: &Path, uid: u32, gid: u32) -> Result<(), CliError> {
    let pathstr = path
        .to_str()
        .ok_or_else(|| CliError::EnvironmentError(format!("Invalid path: {:?}", path)))?;
    let cpath =
        CString::new(pathstr).map_err(|err| CliError::EnvironmentError(format!("{}", err)))?;
    let result = unsafe { libc::chown(cpath.as_ptr(), uid, gid) };
    match result {
        0 => Ok(()),
        code => Err(CliError::EnvironmentError(format!(
            "Error chowning file {}: {}",
            pathstr, code
        ))),
    }
}
