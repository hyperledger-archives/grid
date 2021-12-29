// Copyright 2019 - 2021 Cargill Incorporated
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

use grid_sdk::protos;
use sawtooth_sdk::signing;
use std::error::Error as StdError;
use std::io;

#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
pub enum CliError {
    /// A general error encountered by a subcommand.
    #[cfg(any(
        feature = "location",
        feature = "pike",
        feature = "product",
        feature = "purchase-order",
        feature = "schema",
        feature = "database",
    ))]
    ActionError(String),
    LoggingInitializationError(Box<flexi_logger::FlexiLoggerError>),
    InvalidYamlError(String),
    #[cfg(any(
        feature = "location",
        feature = "pike",
        feature = "product",
        feature = "purchase-order",
        feature = "schema",
    ))]
    PayloadError(String),
    UserError(String),
    SigningError(signing::Error),
    IoError(io::Error),
    ProtobufError(protobuf::ProtobufError),
    GridProtoError(protos::ProtoConversionError),
    SabreProtoError(sabre_sdk::protos::ProtoConversionError),
    #[cfg(any(
        feature = "location",
        feature = "pike",
        feature = "product",
        feature = "purchase-order",
        feature = "schema",
    ))]
    DaemonError(String),
    #[cfg(any(
        feature = "location",
        feature = "pike",
        feature = "product",
        feature = "purchase-order",
        feature = "schema",
    ))]
    InternalError(String),
}

impl StdError for CliError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            #[cfg(any(
                feature = "location",
                feature = "pike",
                feature = "product",
                feature = "purchase-order",
                feature = "schema",
                feature = "database",
            ))]
            CliError::ActionError(_) => None,
            CliError::LoggingInitializationError(err) => Some(err),
            CliError::InvalidYamlError(_) => None,
            #[cfg(any(
                feature = "location",
                feature = "pike",
                feature = "product",
                feature = "purchase-order",
                feature = "schema",
            ))]
            CliError::PayloadError(_) => None,
            CliError::UserError(_) => None,
            CliError::IoError(err) => Some(err),
            CliError::ProtobufError(err) => Some(err),
            CliError::SigningError(err) => Some(err),
            CliError::GridProtoError(err) => Some(err),
            CliError::SabreProtoError(err) => Some(err),
            #[cfg(any(
                feature = "location",
                feature = "pike",
                feature = "product",
                feature = "purchase-order",
                feature = "schema",
            ))]
            CliError::DaemonError(_) => None,
            #[cfg(any(
                feature = "location",
                feature = "pike",
                feature = "product",
                feature = "purchase-order",
                feature = "schema",
            ))]
            CliError::InternalError(_) => None,
        }
    }
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            #[cfg(any(
                feature = "location",
                feature = "pike",
                feature = "product",
                feature = "purchase-order",
                feature = "schema",
                feature = "database",
            ))]
            CliError::ActionError(ref err) => write!(f, "Subcommand encountered an error: {}", err),
            CliError::UserError(ref err) => write!(f, "Error: {}", err),
            CliError::InvalidYamlError(ref err) => write!(f, "InvalidYamlError: {}", err),
            #[cfg(any(
                feature = "location",
                feature = "pike",
                feature = "product",
                feature = "purchase-order",
                feature = "schema",
            ))]
            CliError::PayloadError(ref err) => write!(f, "PayloadError: {}", err),
            CliError::IoError(ref err) => write!(f, "IoError: {}", err),
            CliError::SigningError(ref err) => write!(f, "SigningError: {}", err),
            CliError::ProtobufError(ref err) => write!(f, "ProtobufError: {}", err),
            CliError::LoggingInitializationError(ref err) => {
                write!(f, "LoggingInitializationError: {}", err)
            }
            CliError::GridProtoError(ref err) => write!(f, "Grid Proto Error: {}", err),
            CliError::SabreProtoError(ref err) => write!(f, "Sabre Proto Error: {}", err),
            #[cfg(any(
                feature = "location",
                feature = "pike",
                feature = "product",
                feature = "purchase-order",
                feature = "schema",
            ))]
            CliError::DaemonError(ref err) => write!(f, "{}", err.replace("\"", "")),
            #[cfg(any(
                feature = "location",
                feature = "pike",
                feature = "product",
                feature = "purchase-order",
                feature = "schema",
            ))]
            CliError::InternalError(ref err) => write!(f, "{}", err.replace("\"", "")),
        }
    }
}

impl From<flexi_logger::FlexiLoggerError> for CliError {
    fn from(err: flexi_logger::FlexiLoggerError) -> Self {
        CliError::LoggingInitializationError(Box::new(err))
    }
}

impl From<signing::Error> for CliError {
    fn from(err: signing::Error) -> Self {
        CliError::SigningError(err)
    }
}

impl From<io::Error> for CliError {
    fn from(err: io::Error) -> Self {
        CliError::IoError(err)
    }
}
impl From<serde_yaml::Error> for CliError {
    fn from(err: serde_yaml::Error) -> Self {
        CliError::InvalidYamlError(err.to_string())
    }
}

impl From<protobuf::ProtobufError> for CliError {
    fn from(err: protobuf::ProtobufError) -> Self {
        CliError::ProtobufError(err)
    }
}

impl From<protos::ProtoConversionError> for CliError {
    fn from(err: protos::ProtoConversionError) -> Self {
        CliError::GridProtoError(err)
    }
}

impl From<sabre_sdk::protos::ProtoConversionError> for CliError {
    fn from(err: sabre_sdk::protos::ProtoConversionError) -> Self {
        CliError::SabreProtoError(err)
    }
}

#[cfg(any(feature = "product", feature = "product-gdsn",))]
impl From<grid_sdk::product::gdsn::ProductGdsnError> for CliError {
    fn from(err: grid_sdk::product::gdsn::ProductGdsnError) -> Self {
        CliError::UserError(err.to_string())
    }
}

#[cfg(any(
    feature = "location",
    feature = "pike",
    feature = "product",
    feature = "purchase-order",
    feature = "schema",
))]
impl From<grid_sdk::error::ClientError> for CliError {
    fn from(client_error: grid_sdk::error::ClientError) -> Self {
        match client_error {
            grid_sdk::error::ClientError::IoError(err) => CliError::IoError(err),
            grid_sdk::error::ClientError::DaemonError(err) => CliError::DaemonError(err),
            grid_sdk::error::ClientError::InternalError(err) => CliError::InternalError(err),
        }
    }
}

#[cfg(any(feature = "purchase-order", feature = "product"))]
impl From<grid_sdk::data_validation::DataValidationError> for CliError {
    fn from(err: grid_sdk::data_validation::DataValidationError) -> Self {
        CliError::UserError(err.to_string())
    }
}
