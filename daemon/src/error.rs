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

#[derive(Debug)]
pub struct DaemonError {
    message: Option<String>,
    source: Option<Box<dyn Error>>,
}

impl DaemonError {
    pub fn with_message(message: &str) -> Self {
        Self {
            message: Some(message.to_string()),
            source: None,
        }
    }

    pub fn from_source(source: Box<dyn Error>) -> Self {
        Self {
            message: None,
            source: Some(source),
        }
    }
}

impl fmt::Display for DaemonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match (&self.message, &self.source) {
            (Some(m), Some(s)) => write!(f, "{}: {}", m, s),
            (Some(m), _) => write!(f, "{}", m),
            (_, Some(s)) => write!(f, "{:?}", s),
            (None, None) => write!(f, "An internal error occured"),
        }
    }
}

impl Error for DaemonError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_ref().map(|s| s.as_ref())
    }
}

#[derive(Debug, PartialEq, Eq)]
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
        DaemonError::from_source(Box::new(err))
    }
}
