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

use crate::database::DatabaseError;
use crate::event::EventProcessorError;
use crate::rest_api::RestApiServerError;
#[cfg(feature = "splinter-support")]
use crate::splinter::error::AppAuthHandlerError;

#[derive(Debug)]
pub enum DaemonError {
    DatabaseError(Box<dyn Error>),
    LoggingInitializationError(Box<flexi_logger::FlexiLoggerError>),
    ConfigurationError(Box<ConfigurationError>),
    EventProcessorError(Box<EventProcessorError>),
    RestApiError(RestApiServerError),
    StartUpError(Box<dyn Error>),
    ShutdownError(String),
    UnsupportedEndpoint(String),
    #[cfg(feature = "splinter-support")]
    AppAuthHandlerError(AppAuthHandlerError),
}

impl Error for DaemonError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            DaemonError::DatabaseError(err) => Some(&**err),
            DaemonError::LoggingInitializationError(err) => Some(err),
            DaemonError::ConfigurationError(err) => Some(err),
            DaemonError::EventProcessorError(err) => Some(err),
            DaemonError::RestApiError(err) => Some(err),
            DaemonError::StartUpError(err) => Some(&**err),
            DaemonError::ShutdownError(_) => None,
            DaemonError::UnsupportedEndpoint(_) => None,
            #[cfg(feature = "splinter-support")]
            DaemonError::AppAuthHandlerError(err) => Some(err),
        }
    }
}

impl fmt::Display for DaemonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DaemonError::DatabaseError(e) => write!(f, "Database Error: {}", e),
            DaemonError::LoggingInitializationError(e) => {
                write!(f, "Logging initialization error: {}", e)
            }
            DaemonError::ConfigurationError(e) => write!(f, "Configuration error: {}", e),
            DaemonError::EventProcessorError(e) => write!(f, "Event Processor Error: {}", e),
            DaemonError::RestApiError(e) => write!(f, "Rest API error: {}", e),
            DaemonError::StartUpError(e) => write!(f, "Start-up error: {}", e),
            DaemonError::ShutdownError(msg) => write!(f, "Unable to cleanly shutdown: {}", msg),
            DaemonError::UnsupportedEndpoint(msg) => write!(f, "{}", msg),
            #[cfg(feature = "splinter-support")]
            DaemonError::AppAuthHandlerError(e) => {
                write!(f, "Application Authorization Handler Error: {}", e)
            }
        }
    }
}

impl From<flexi_logger::FlexiLoggerError> for DaemonError {
    fn from(err: flexi_logger::FlexiLoggerError) -> DaemonError {
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

impl From<RestApiServerError> for DaemonError {
    fn from(err: RestApiServerError) -> DaemonError {
        DaemonError::RestApiError(err)
    }
}

impl From<EventProcessorError> for DaemonError {
    fn from(err: EventProcessorError) -> Self {
        DaemonError::EventProcessorError(Box::new(err))
    }
}

impl From<DatabaseError> for DaemonError {
    fn from(err: DatabaseError) -> Self {
        DaemonError::DatabaseError(Box::new(err))
    }
}

#[cfg(feature = "splinter-support")]
impl From<AppAuthHandlerError> for DaemonError {
    fn from(err: AppAuthHandlerError) -> DaemonError {
        DaemonError::AppAuthHandlerError(err)
    }
}
