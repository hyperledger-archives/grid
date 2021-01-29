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

#[cfg(feature = "database")]
use crate::database::DatabaseError;
#[cfg(feature = "event")]
use crate::event::EventProcessorError;
#[cfg(feature = "rest-api")]
use crate::rest_api::RestApiServerError;
#[cfg(feature = "splinter-support")]
use crate::splinter::app_auth_handler::error::AppAuthHandlerError;

#[derive(Debug)]
pub enum DaemonError {
    #[cfg(feature = "database")]
    DatabaseError {
        context: String,
        source: Box<dyn Error>,
    },
    LoggingInitializationError(Box<flexi_logger::FlexiLoggerError>),
    ConfigurationError(Box<ConfigurationError>),
    #[cfg(feature = "event")]
    EventProcessorError(Box<EventProcessorError>),
    #[cfg(feature = "rest-api")]
    RestApiError(RestApiServerError),
    // dead_code allowed because this isn't used when splinter and sawtooth
    // features are both off, as is the case with --no-default-features
    #[allow(dead_code)]
    StartUpError(Box<dyn Error>),
    // dead_code allowed because this isn't used when splinter and sawtooth
    // features are both off, as is the case with --no-default-features
    #[allow(dead_code)]
    ShutdownError(String),
    #[cfg(not(all(feature = "sawtooth-support", feature = "splinter-support")))]
    UnsupportedEndpoint(String),
    #[cfg(feature = "splinter-support")]
    AppAuthHandlerError(AppAuthHandlerError),
}

impl Error for DaemonError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            #[cfg(feature = "database")]
            DaemonError::DatabaseError { source, .. } => Some(&**source),
            DaemonError::LoggingInitializationError(err) => Some(err),
            DaemonError::ConfigurationError(err) => Some(err),
            #[cfg(feature = "event")]
            DaemonError::EventProcessorError(err) => Some(err),
            #[cfg(feature = "rest-api")]
            DaemonError::RestApiError(err) => Some(err),
            DaemonError::StartUpError(err) => Some(&**err),
            DaemonError::ShutdownError(_) => None,
            #[cfg(not(all(feature = "sawtooth-support", feature = "splinter-support")))]
            DaemonError::UnsupportedEndpoint(_) => None,
            #[cfg(feature = "splinter-support")]
            DaemonError::AppAuthHandlerError(err) => Some(err),
        }
    }
}

impl fmt::Display for DaemonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            #[cfg(feature = "database")]
            DaemonError::DatabaseError { context, source } => {
                write!(f, "Database Error: {}: {}", context, source)
            }
            DaemonError::LoggingInitializationError(e) => {
                write!(f, "Logging initialization error: {}", e)
            }
            DaemonError::ConfigurationError(e) => write!(f, "Configuration error: {}", e),
            #[cfg(feature = "event")]
            DaemonError::EventProcessorError(e) => write!(f, "Event Processor Error: {}", e),
            #[cfg(feature = "rest-api")]
            DaemonError::RestApiError(e) => write!(f, "Rest API error: {}", e),
            DaemonError::StartUpError(e) => write!(f, "Start-up error: {}", e),
            DaemonError::ShutdownError(msg) => write!(f, "Unable to cleanly shutdown: {}", msg),
            #[cfg(not(all(feature = "sawtooth-support", feature = "splinter-support")))]
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

#[cfg(feature = "rest-api")]
impl From<RestApiServerError> for DaemonError {
    fn from(err: RestApiServerError) -> DaemonError {
        DaemonError::RestApiError(err)
    }
}

#[cfg(feature = "event")]
impl From<EventProcessorError> for DaemonError {
    fn from(err: EventProcessorError) -> Self {
        DaemonError::EventProcessorError(Box::new(err))
    }
}

#[cfg(feature = "database")]
impl From<DatabaseError> for DaemonError {
    fn from(err: DatabaseError) -> Self {
        DaemonError::DatabaseError {
            context: "There was an issue connecting to the database".to_string(),
            source: Box::new(err),
        }
    }
}

#[cfg(feature = "splinter-support")]
impl From<AppAuthHandlerError> for DaemonError {
    fn from(err: AppAuthHandlerError) -> DaemonError {
        DaemonError::AppAuthHandlerError(err)
    }
}
