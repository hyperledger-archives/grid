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

use flexi_logger::FlexiLoggerError;
use sabre_sdk::{
    protocol::{
        payload::{ActionBuildError, SabrePayloadBuildError},
        AddressingError,
    },
    protos::ProtoConversionError,
};
use sawtooth_sdk::signing::Error as SigningError;
use splinter::service::scabbard::client::Error as ClientError;
use transact::{
    contract::archive::Error as ContractArchiveError,
    protocol::{batch::BatchBuildError, transaction::TransactionBuildError},
};

#[derive(Debug)]
pub enum CliError {
    ActionError {
        context: String,
        source: Option<Box<dyn Error>>,
    },
    InvalidArgument(String),
    InvalidSubcommand,
    LoggingSetupError(String),
    MissingArgument(String),
}

impl CliError {
    pub fn action_error(context: &str) -> Self {
        CliError::ActionError {
            context: context.into(),
            source: None,
        }
    }

    pub fn action_error_with_source(context: &str, err: Box<dyn Error>) -> Self {
        CliError::ActionError {
            context: context.into(),
            source: Some(err),
        }
    }
}

impl Error for CliError {}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CliError::ActionError { context, source } => {
                if let Some(ref err) = source {
                    write!(f, "{}: {}", context, err)
                } else {
                    f.write_str(&context)
                }
            }
            CliError::InvalidArgument(msg) => write!(f, "invalid argument: {}", msg),
            CliError::InvalidSubcommand => write!(f, "specified subcommand invalid"),
            CliError::LoggingSetupError(msg) => write!(f, "error setting up logging: {}", msg),
            CliError::MissingArgument(arg) => write!(f, "missing required argument: {}", arg),
        }
    }
}

impl From<FlexiLoggerError> for CliError {
    fn from(err: FlexiLoggerError) -> Self {
        Self::LoggingSetupError(err.to_string())
    }
}

impl From<AddressingError> for CliError {
    fn from(err: AddressingError) -> Self {
        Self::action_error_with_source("failed to compute Sabre address", err.into())
    }
}

impl From<ActionBuildError> for CliError {
    fn from(err: ActionBuildError) -> Self {
        Self::action_error_with_source("failed to build Sabre action", err.into())
    }
}

impl From<SabrePayloadBuildError> for CliError {
    fn from(err: SabrePayloadBuildError) -> Self {
        Self::action_error_with_source("failed to build Sabre payload", err.into())
    }
}

impl From<ProtoConversionError> for CliError {
    fn from(err: ProtoConversionError) -> Self {
        Self::action_error_with_source("failed to convert Sabre protobuf", err.into())
    }
}

impl From<SigningError> for CliError {
    fn from(err: SigningError) -> Self {
        Self::action_error_with_source("signer failed", err.into())
    }
}

impl From<ClientError> for CliError {
    fn from(err: ClientError) -> Self {
        Self::action_error_with_source("scabbard client encountered an error", err.into())
    }
}

impl From<ContractArchiveError> for CliError {
    fn from(err: ContractArchiveError) -> Self {
        Self::action_error_with_source("failed to load .scar file", err.into())
    }
}

impl From<BatchBuildError> for CliError {
    fn from(err: BatchBuildError) -> Self {
        Self::action_error_with_source("failed to build batch", err.into())
    }
}

impl From<TransactionBuildError> for CliError {
    fn from(err: TransactionBuildError) -> Self {
        Self::action_error_with_source("failed to build transaction", err.into())
    }
}
