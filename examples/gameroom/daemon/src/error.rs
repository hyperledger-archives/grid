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

use sawtooth_sdk::signing::Error as KeyGenError;

use crate::authorization_handler::AppAuthHandlerError;
use crate::rest_api::RestApiServerError;
use gameroom_database::DatabaseError;

#[derive(Debug)]
pub enum GameroomDaemonError {
    LoggingInitializationError(flexi_logger::FlexiLoggerError),
    ConfigurationError(Box<ConfigurationError>),
    DatabaseError(Box<DatabaseError>),
    RestApiError(RestApiServerError),
    AppAuthHandlerError(AppAuthHandlerError),
    KeyGenError(KeyGenError),
    GetNodeError(GetNodeError),
}

impl Error for GameroomDaemonError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            GameroomDaemonError::LoggingInitializationError(err) => Some(err),
            GameroomDaemonError::ConfigurationError(err) => Some(err),
            GameroomDaemonError::DatabaseError(err) => Some(&**err),
            GameroomDaemonError::RestApiError(err) => Some(err),
            GameroomDaemonError::AppAuthHandlerError(err) => Some(err),
            GameroomDaemonError::KeyGenError(err) => Some(err),
            GameroomDaemonError::GetNodeError(err) => Some(err),
        }
    }
}

impl fmt::Display for GameroomDaemonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GameroomDaemonError::LoggingInitializationError(e) => {
                write!(f, "Logging initialization error: {}", e)
            }
            GameroomDaemonError::ConfigurationError(e) => write!(f, "Coniguration error: {}", e),
            GameroomDaemonError::DatabaseError(e) => write!(f, "Database error: {}", e),
            GameroomDaemonError::RestApiError(e) => write!(f, "Rest API error: {}", e),
            GameroomDaemonError::AppAuthHandlerError(e) => write!(
                f,
                "The application authorization handler returned an error: {}",
                e
            ),
            GameroomDaemonError::KeyGenError(e) => write!(
                f,
                "an error occurred while generating a new key pair: {}",
                e
            ),
            GameroomDaemonError::GetNodeError(e) => write!(
                f,
                "an error occurred while getting splinterd node information: {}",
                e
            ),
        }
    }
}

impl From<flexi_logger::FlexiLoggerError> for GameroomDaemonError {
    fn from(err: flexi_logger::FlexiLoggerError) -> GameroomDaemonError {
        GameroomDaemonError::LoggingInitializationError(err)
    }
}

impl From<DatabaseError> for GameroomDaemonError {
    fn from(err: DatabaseError) -> GameroomDaemonError {
        GameroomDaemonError::DatabaseError(Box::new(err))
    }
}

impl From<RestApiServerError> for GameroomDaemonError {
    fn from(err: RestApiServerError) -> GameroomDaemonError {
        GameroomDaemonError::RestApiError(err)
    }
}

impl From<AppAuthHandlerError> for GameroomDaemonError {
    fn from(err: AppAuthHandlerError) -> GameroomDaemonError {
        GameroomDaemonError::AppAuthHandlerError(err)
    }
}

impl From<KeyGenError> for GameroomDaemonError {
    fn from(err: KeyGenError) -> GameroomDaemonError {
        GameroomDaemonError::KeyGenError(err)
    }
}

#[derive(Debug, PartialEq)]
pub enum ConfigurationError {
    MissingValue(String),
}

impl Error for ConfigurationError {}

impl fmt::Display for ConfigurationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigurationError::MissingValue(config_field_name) => {
                write!(f, "Missing configuration for {}", config_field_name)
            }
        }
    }
}

impl From<ConfigurationError> for GameroomDaemonError {
    fn from(err: ConfigurationError) -> Self {
        GameroomDaemonError::ConfigurationError(Box::new(err))
    }
}

#[derive(Debug, PartialEq)]
pub struct GetNodeError(pub String);

impl Error for GetNodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl fmt::Display for GetNodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<GetNodeError> for GameroomDaemonError {
    fn from(err: GetNodeError) -> Self {
        GameroomDaemonError::GetNodeError(err)
    }
}
