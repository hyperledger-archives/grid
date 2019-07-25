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

#[derive(Debug)]
pub enum GameroomDaemonError {
    LoggingInitializationError(log::SetLoggerError),
    ConfigurationError(Box<ConfigurationError>),
}

impl Error for GameroomDaemonError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            GameroomDaemonError::LoggingInitializationError(err) => Some(err),
            GameroomDaemonError::ConfigurationError(err) => Some(err),
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
        }
    }
}

impl From<log::SetLoggerError> for GameroomDaemonError {
    fn from(err: log::SetLoggerError) -> GameroomDaemonError {
        GameroomDaemonError::LoggingInitializationError(err)
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
