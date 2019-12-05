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

/// Error for BiomeRestResourceManagerBuilder
#[derive(Debug)]
pub enum BiomeRestResourceManagerBuilderError {
    /// Returned if a required field is missing
    MissingRequiredField(String),
    /// Returned if a required field is missing
    BuildingError(Box<dyn Error>),
}

impl Error for BiomeRestResourceManagerBuilderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            BiomeRestResourceManagerBuilderError::MissingRequiredField(_) => None,
            BiomeRestResourceManagerBuilderError::BuildingError(err) => Some(&**err),
        }
    }
}

impl fmt::Display for BiomeRestResourceManagerBuilderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BiomeRestResourceManagerBuilderError::MissingRequiredField(ref s) => {
                write!(f, "failed to build BiomeRestResourceManager: {}", s)
            }
            BiomeRestResourceManagerBuilderError::BuildingError(ref s) => {
                write!(f, "failed to build BiomeRestResourceManager: {}", s)
            }
        }
    }
}

impl From<BiomeRestConfigBuilderError> for BiomeRestResourceManagerBuilderError {
    fn from(err: BiomeRestConfigBuilderError) -> BiomeRestResourceManagerBuilderError {
        BiomeRestResourceManagerBuilderError::BuildingError(Box::new(err))
    }
}

/// Error for BiomeRestConfigBuilder
#[derive(Debug)]
pub enum BiomeRestConfigBuilderError {
    /// Returned if a required field is missing
    MissingRequiredField(String),
    /// Returned if a value provided is not valid
    InvalidValue(String),
}

impl Error for BiomeRestConfigBuilderError {}

impl fmt::Display for BiomeRestConfigBuilderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BiomeRestConfigBuilderError::MissingRequiredField(ref s) => {
                write!(f, "failed to build BiomeRestResourceManager: {}", s)
            }
            BiomeRestConfigBuilderError::InvalidValue(ref s) => {
                write!(f, "failed to build BiomeRestResourceManager: {}", s)
            }
        }
    }
}
