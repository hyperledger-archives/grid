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

#[derive(Debug)]
pub enum RegistryConfig {
    File { registry_file: String },
    NoOp,
}

pub struct RegistryConfigBuilder {
    registry_backend: Option<String>,
    registry_file: Option<String>,
}

impl Default for RegistryConfigBuilder {
    fn default() -> Self {
        Self {
            registry_backend: Some("FILE".to_owned()),
            registry_file: None,
        }
    }
}

impl RegistryConfigBuilder {
    pub fn with_registry_backend(mut self, value: Option<String>) -> Self {
        self.registry_backend = value;
        self
    }

    pub fn with_registry_file(mut self, value: String) -> Self {
        self.registry_file = Some(value);
        self
    }

    pub fn build(self) -> Result<RegistryConfig, RegistryConfigError> {
        match self.registry_backend {
            Some(ref registry_type) if registry_type == "FILE" => self
                .registry_file
                .map(|registry_file| RegistryConfig::File { registry_file })
                .ok_or_else(|| {
                    RegistryConfigError::MissingValue(
                        "For registry_backend of type 'FILE' a path to the file must be provided."
                            .to_string(),
                    )
                }),
            None => Ok(RegistryConfig::NoOp),
            _ => Err(RegistryConfigError::InvalidType(
                "NodeRegistry type is not supported".to_string(),
            )),
        }
    }
}
#[derive(Debug, PartialEq)]
pub enum RegistryConfigError {
    MissingValue(String),
    InvalidType(String),
}

impl Error for RegistryConfigError {}

impl fmt::Display for RegistryConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RegistryConfigError::MissingValue(config_field_name) => {
                write!(f, "Missing configuration for {}", config_field_name)
            }
            RegistryConfigError::InvalidType(config_field_name) => write!(
                f,
                "NodeRegistry of type {} is not yet supported",
                config_field_name
            ),
        }
    }
}
