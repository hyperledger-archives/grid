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

/// Error for a SecretManager
#[derive(Debug)]
pub enum SecretManagerError {
    /// Returned when the manager fails to update a secret
    UpdateSecretError(Box<dyn Error>),
    /// Returned when the manager fails to fetch a secret
    SecretError(Box<dyn Error>),
}

impl Error for SecretManagerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            SecretManagerError::UpdateSecretError(err) => Some(&**err),
            SecretManagerError::SecretError(err) => Some(&**err),
        }
    }
}

impl fmt::Display for SecretManagerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SecretManagerError::UpdateSecretError(ref s) => {
                write!(f, "failed to update secret: {}", s)
            }
            SecretManagerError::SecretError(ref s) => write!(f, "failed to fetch secret: {}", s),
        }
    }
}
