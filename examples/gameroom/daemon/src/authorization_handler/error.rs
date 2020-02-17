/*
 * Copyright 2018-2020 Cargill Incorporated
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * -----------------------------------------------------------------------------
 */

use std::error::Error;
use std::fmt;

use sabre_sdk::protocol::payload::{
    CreateContractActionBuildError, CreateContractRegistryActionBuildError,
    CreateNamespaceRegistryActionBuildError, CreateNamespaceRegistryPermissionActionBuildError,
    SabrePayloadBuildError,
};
use sabre_sdk::protos::ProtoConversionError as SabreProtoConversionError;
use sawtooth_sdk::signing::Error as SigningError;
use splinter::events;

use crate::application_metadata::ApplicationMetadataError;

#[derive(Debug)]
pub enum AppAuthHandlerError {
    IOError(std::io::Error),
    InvalidMessageError(String),
    DatabaseError(String),
    ReactorError(events::ReactorError),
    WebSocketError(events::WebSocketError),
    SabreError(String),
    SawtoothError(String),
    SigningError(String),
    BatchSubmitError(String),
}

impl Error for AppAuthHandlerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            AppAuthHandlerError::IOError(err) => Some(err),
            AppAuthHandlerError::InvalidMessageError(_) => None,
            AppAuthHandlerError::DatabaseError(_) => None,
            AppAuthHandlerError::ReactorError(err) => Some(err),
            AppAuthHandlerError::SabreError(_) => None,
            AppAuthHandlerError::SawtoothError(_) => None,
            AppAuthHandlerError::SigningError(_) => None,
            AppAuthHandlerError::BatchSubmitError(_) => None,
            AppAuthHandlerError::WebSocketError(err) => Some(err),
        }
    }
}

impl fmt::Display for AppAuthHandlerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppAuthHandlerError::IOError(msg) => write!(f, "An I/O error occurred: {}", msg),
            AppAuthHandlerError::InvalidMessageError(msg) => {
                write!(f, "The client received an invalid message: {}", msg)
            }
            AppAuthHandlerError::DatabaseError(msg) => {
                write!(f, "The database returned an error: {}", msg)
            }
            AppAuthHandlerError::ReactorError(msg) => write!(f, "Reactor Error: {}", msg),
            AppAuthHandlerError::SabreError(msg) => write!(
                f,
                "An error occurred while building a Sabre payload: {}",
                msg
            ),
            AppAuthHandlerError::SawtoothError(msg) => write!(
                f,
                "An error occurred while building a transaction or batch: {}",
                msg
            ),
            AppAuthHandlerError::SigningError(msg) => {
                write!(f, "A signing error occurred: {}", msg)
            }
            AppAuthHandlerError::BatchSubmitError(msg) => write!(
                f,
                "An error occurred while submitting a batch to the scabbard service: {}",
                msg
            ),
            AppAuthHandlerError::WebSocketError(msg) => write!(f, "WebsocketError {}", msg),
        }
    }
}

impl From<std::io::Error> for AppAuthHandlerError {
    fn from(err: std::io::Error) -> AppAuthHandlerError {
        AppAuthHandlerError::IOError(err)
    }
}

impl From<serde_json::error::Error> for AppAuthHandlerError {
    fn from(err: serde_json::error::Error) -> AppAuthHandlerError {
        AppAuthHandlerError::InvalidMessageError(format!("{}", err))
    }
}

impl From<std::string::FromUtf8Error> for AppAuthHandlerError {
    fn from(err: std::string::FromUtf8Error) -> AppAuthHandlerError {
        AppAuthHandlerError::InvalidMessageError(format!("{}", err))
    }
}

impl From<ApplicationMetadataError> for AppAuthHandlerError {
    fn from(err: ApplicationMetadataError) -> AppAuthHandlerError {
        AppAuthHandlerError::InvalidMessageError(format!("{}", err))
    }
}

impl From<gameroom_database::DatabaseError> for AppAuthHandlerError {
    fn from(err: gameroom_database::DatabaseError) -> AppAuthHandlerError {
        AppAuthHandlerError::DatabaseError(format!("{}", err))
    }
}

impl From<diesel::result::Error> for AppAuthHandlerError {
    fn from(err: diesel::result::Error) -> Self {
        AppAuthHandlerError::DatabaseError(format!("Error performing query: {}", err))
    }
}

impl From<events::ReactorError> for AppAuthHandlerError {
    fn from(err: events::ReactorError) -> Self {
        AppAuthHandlerError::ReactorError(err)
    }
}

impl From<events::WebSocketError> for AppAuthHandlerError {
    fn from(err: events::WebSocketError) -> Self {
        AppAuthHandlerError::WebSocketError(err)
    }
}

macro_rules! impl_from_sabre_errors {
    ($($x:ty),*) => {
        $(
            impl From<$x> for AppAuthHandlerError {
                fn from(e: $x) -> Self {
                    AppAuthHandlerError::SabreError(e.to_string())
                }
            }
        )*
    };
}

impl_from_sabre_errors!(
    CreateContractActionBuildError,
    CreateContractRegistryActionBuildError,
    CreateNamespaceRegistryActionBuildError,
    CreateNamespaceRegistryPermissionActionBuildError,
    SabreProtoConversionError,
    SabrePayloadBuildError
);

impl From<SigningError> for AppAuthHandlerError {
    fn from(err: SigningError) -> Self {
        AppAuthHandlerError::SigningError(err.to_string())
    }
}
