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

//! Errors that can occur during the signing process.

use std::error::Error as StdError;

#[derive(Debug)]
pub enum Error {
    SigningError(String),
    SignatureVerificationError(String),
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::SigningError(ref msg) => msg,
            Error::SignatureVerificationError(ref msg) => msg,
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::SigningError(_) => None,
            Error::SignatureVerificationError(_) => None,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::SigningError(ref s) => write!(f, "failed to sign message: {}", s),
            Error::SignatureVerificationError(ref s) => {
                write!(f, "failed to verify signature: {}", s)
            }
        }
    }
}
