// Copyright 2019 Bitwise IO, Inc.
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

use std::error::Error;
use std::fmt;

use log;

use crate::rest_api::RestApiError;

#[derive(Debug)]
pub enum DaemonError {
    LoggingInitializationError(Box<log::SetLoggerError>),
    ConfigurationError(Box<ConfigurationError>),
    RestApiError(RestApiError),
}

impl Error for DaemonError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            DaemonError::LoggingInitializationError(err) => Some(err),
            DaemonError::ConfigurationError(err) => Some(err),
            DaemonError::RestApiError(err) => Some(err),
        }
    }
}

impl fmt::Display for DaemonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DaemonError::LoggingInitializationError(e) => {
                write!(f, "Logging initialization error: {}", e)
            }
            DaemonError::ConfigurationError(e) => write!(f, "Configuration error: {}", e),
            DaemonError::RestApiError(e) => write!(f, "Rest API error: {}", e),
        }
    }
}

impl From<log::SetLoggerError> for DaemonError {
    fn from(err: log::SetLoggerError) -> DaemonError {
        DaemonError::LoggingInitializationError(Box::new(err))
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

impl From<ConfigurationError> for DaemonError {
    fn from(err: ConfigurationError) -> Self {
        DaemonError::ConfigurationError(Box::new(err))
    }
}

impl From<RestApiError> for DaemonError {
    fn from(err: RestApiError) -> DaemonError {
        DaemonError::RestApiError(err)
    }
}
