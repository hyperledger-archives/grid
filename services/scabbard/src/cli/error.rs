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
use protobuf::error::ProtobufError;
use sabre_sdk::{
    protocol::payload::{
        CreateContractActionBuildError, CreateContractRegistryActionBuildError,
        CreateNamespaceRegistryActionBuildError, CreateNamespaceRegistryPermissionActionBuildError,
        CreateSmartPermissionActionBuildError, DeleteContractActionBuildError,
        DeleteContractRegistryActionBuildError, DeleteNamespaceRegistryActionBuildError,
        DeleteNamespaceRegistryPermissionActionBuildError, DeleteSmartPermissionActionBuildError,
        ExecuteContractActionBuildError, SabrePayloadBuildError,
        UpdateContractRegistryOwnersActionBuildError,
        UpdateNamespaceRegistryOwnersActionBuildError, UpdateSmartPermissionActionBuildError,
    },
    protos::ProtoConversionError,
};
use splinter::{service::scabbard::client::Error as ClientError, signing::Error as SigningError};

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

impl From<ProtobufError> for CliError {
    fn from(err: ProtobufError) -> Self {
        Self::action_error_with_source("protobuf serialization failed", err.into())
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
        Self::action_error_with_source("signing failed", err.into())
    }
}

impl From<ClientError> for CliError {
    fn from(err: ClientError) -> Self {
        Self::action_error_with_source("scabbard client encountered an error", err.into())
    }
}

macro_rules! impl_sabre_action_builder_errors {
    ($($x:ty),*) => {
        $(
            impl From<$x> for CliError {
                fn from(e: $x) -> Self {
                    CliError::action_error_with_source("failed to build Sabre action", Box::new(e))
                }
            }
        )*
    };
}

impl_sabre_action_builder_errors!(
    CreateContractActionBuildError,
    DeleteContractActionBuildError,
    ExecuteContractActionBuildError,
    CreateContractRegistryActionBuildError,
    DeleteContractRegistryActionBuildError,
    UpdateContractRegistryOwnersActionBuildError,
    CreateNamespaceRegistryActionBuildError,
    DeleteNamespaceRegistryActionBuildError,
    UpdateNamespaceRegistryOwnersActionBuildError,
    CreateNamespaceRegistryPermissionActionBuildError,
    DeleteNamespaceRegistryPermissionActionBuildError,
    CreateSmartPermissionActionBuildError,
    UpdateSmartPermissionActionBuildError,
    DeleteSmartPermissionActionBuildError
);
