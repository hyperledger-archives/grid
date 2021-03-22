// Copyright 2018-2021 Cargill Incorporated
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
pub enum BatchSubmitterError {
    BadRequestError(String),
    ConnectionError(String),
    InternalError(String),
    ResourceTemporarilyUnavailableError(String),
}

impl Error for BatchSubmitterError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl fmt::Display for BatchSubmitterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BatchSubmitterError::BadRequestError(err) => write!(f, "{}", err),
            BatchSubmitterError::ConnectionError(err) => write!(f, "{}", err),
            BatchSubmitterError::InternalError(err) => write!(f, "{}", err),
            BatchSubmitterError::ResourceTemporarilyUnavailableError(err) => write!(f, "{}", err),
        }
    }
}
