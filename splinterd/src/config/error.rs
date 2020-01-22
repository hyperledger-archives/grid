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
use std::io;

use toml::de::Error as TomlError;

#[derive(Debug)]
pub enum ConfigError {
    ReadError(io::Error),
    TomlParseError(TomlError),
}

impl From<io::Error> for ConfigError {
    fn from(e: io::Error) -> Self {
        ConfigError::ReadError(e)
    }
}

impl From<TomlError> for ConfigError {
    fn from(e: TomlError) -> Self {
        ConfigError::TomlParseError(e)
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ConfigError::ReadError(source) => Some(source),
            ConfigError::TomlParseError(source) => Some(source),
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::ReadError(source) => source.fmt(f),
            ConfigError::TomlParseError(source) => write!(f, "Invalid File Format: {}", source),
        }
    }
}
