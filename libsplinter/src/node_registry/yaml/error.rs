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
pub enum YamlNodeRegistryError {
    PoisonLockError(String),

    SerdeError(serde_yaml::Error),

    IoError(std::io::Error),
}

impl Error for YamlNodeRegistryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            YamlNodeRegistryError::PoisonLockError(_) => None,
            YamlNodeRegistryError::SerdeError(err) => Some(err),
            YamlNodeRegistryError::IoError(err) => Some(err),
        }
    }
}

impl fmt::Display for YamlNodeRegistryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            YamlNodeRegistryError::PoisonLockError(e) => write!(f, "Error locking file: {}", e),
            YamlNodeRegistryError::SerdeError(e) => write!(f, "Serde error: {}", e),
            YamlNodeRegistryError::IoError(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl From<std::io::Error> for YamlNodeRegistryError {
    fn from(err: std::io::Error) -> YamlNodeRegistryError {
        YamlNodeRegistryError::IoError(err)
    }
}

impl From<serde_yaml::Error> for YamlNodeRegistryError {
    fn from(err: serde_yaml::Error) -> YamlNodeRegistryError {
        YamlNodeRegistryError::SerdeError(err)
    }
}
