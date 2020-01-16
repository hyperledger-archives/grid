// Copyright 2019 Cargill Incorporated
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
use protobuf;
use reqwest;
use sabre_sdk;
use sawtooth_sdk::signing;
use serde_yaml;
use std;
use std::error::Error as StdError;
use std::io;

#[derive(Debug)]
pub enum CliError {
    LoggingInitializationError(Box<flexi_logger::FlexiLoggerError>),
    InvalidYamlError(String),
    PayloadError(String),
    UserError(String),
    SigningError(signing::Error),
    DatabaseError(String),
    IoError(io::Error),
    ProtobufError(protobuf::ProtobufError),
    ReqwestError(reqwest::Error),
    GridProtoError(protos::ProtoConversionError),
    SabreProtoError(sabre_sdk::protos::ProtoConversionError),
}

impl StdError for CliError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            CliError::LoggingInitializationError(err) => Some(err),
            CliError::InvalidYamlError(_) => None,
            CliError::PayloadError(_) => None,
            CliError::UserError(_) => None,
            CliError::DatabaseError(_) => None,
            CliError::IoError(err) => Some(err),
            CliError::ProtobufError(err) => Some(err),
            CliError::SigningError(err) => Some(err),
            CliError::ReqwestError(err) => Some(err),
            CliError::GridProtoError(err) => Some(err),
            CliError::SabreProtoError(err) => Some(err),
        }
    }
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            CliError::UserError(ref err) => write!(f, "Error: {}", err),
            CliError::InvalidYamlError(ref err) => write!(f, "InvalidYamlError: {}", err),
            CliError::PayloadError(ref err) => write!(f, "PayloadError: {}", err),
            CliError::IoError(ref err) => write!(f, "IoError: {}", err),
            CliError::DatabaseError(ref err) => write!(f, "DatabaseError: {}", err),
            CliError::SigningError(ref err) => write!(f, "SigningError: {}", err.description()),
            CliError::ProtobufError(ref err) => write!(f, "ProtobufError: {}", err.description()),
            CliError::LoggingInitializationError(ref err) => {
                write!(f, "LoggingInitializationError: {}", err.description())
            }
            CliError::ReqwestError(ref err) => write!(f, "Reqwest Error: {}", err),
            CliError::GridProtoError(ref err) => write!(f, "Grid Proto Error: {}", err),
            CliError::SabreProtoError(ref err) => write!(f, "Sabre Proto Error: {}", err),
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

impl From<reqwest::Error> for CliError {
    fn from(err: reqwest::Error) -> Self {
        CliError::ReqwestError(err)
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
