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

/// Error for ClaimsBuilder
#[derive(Debug)]
pub enum ClaimsBuildError {
    /// Returned if a required field is missing
    MissingRequiredField(String),
    /// Returned if a invalid value was provided to the builder
    InvalidValue(String),
}

impl Error for ClaimsBuildError {}

impl fmt::Display for ClaimsBuildError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ClaimsBuildError::MissingRequiredField(ref s) => {
                write!(f, "failed to build claim: {}", s)
            }
            ClaimsBuildError::InvalidValue(ref s) => write!(f, "failed to build claim: {}", s),
        }
    }
}
