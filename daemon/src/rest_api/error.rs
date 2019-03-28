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

#[derive(Debug)]
pub enum RestApiError {
    StartUpError(String),
    StdError(std::io::Error),
}

impl Error for RestApiError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            RestApiError::StartUpError(_) => None,
            RestApiError::StdError(err) => Some(err),
        }
    }
}

impl fmt::Display for RestApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RestApiError::StartUpError(e) => write!(f, "Start-up Error: {}", e),
            RestApiError::StdError(e) => write!(f, "Std Error: {}", e),
        }
    }
}

impl From<std::io::Error> for RestApiError {
    fn from(err: std::io::Error) -> RestApiError {
        RestApiError::StdError(err)
    }
}
