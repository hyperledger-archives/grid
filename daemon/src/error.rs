// Copyright 2019 Bitwise IO, Inc.
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

#[derive(Debug)]
pub enum DaemonError {
    LoggingInitializationError(Box<log::SetLoggerError>),
}

impl Error for DaemonError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            DaemonError::LoggingInitializationError(err) => Some(err),
        }
    }
}

impl fmt::Display for DaemonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DaemonError::LoggingInitializationError(e) => {
                write!(f, "Logging initialization error: {}", e)
            }
        }
    }
}

impl From<log::SetLoggerError> for DaemonError {
    fn from(err: log::SetLoggerError) -> DaemonError {
        DaemonError::LoggingInitializationError(Box::new(err))
    }
}
