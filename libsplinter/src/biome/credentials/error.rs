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

use bcrypt::BcryptError;

use std::error::Error;
use std::fmt;

/// Represents UserCredentialsBuilder errors
#[derive(Debug)]
pub enum UserCredentialsBuilderError {
    /// Returned when a required field was not set
    MissingRequiredField(String),
    /// Returned when a error occurs while attempting to encrypt the password
    EncryptionError(Box<dyn Error>),
}

impl Error for UserCredentialsBuilderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            UserCredentialsBuilderError::MissingRequiredField(_) => None,
            UserCredentialsBuilderError::EncryptionError(err) => Some(&**err),
        }
    }
}

impl fmt::Display for UserCredentialsBuilderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            UserCredentialsBuilderError::MissingRequiredField(ref s) => {
                write!(f, "failed to build user credentials: {}", s)
            }
            UserCredentialsBuilderError::EncryptionError(ref s) => {
                write!(f, "failed encrypt password: {}", s)
            }
        }
    }
}

impl From<BcryptError> for UserCredentialsBuilderError {
    fn from(err: BcryptError) -> UserCredentialsBuilderError {
        UserCredentialsBuilderError::EncryptionError(Box::new(err))
    }
}
