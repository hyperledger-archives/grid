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

use diesel;
use std::error::Error;
use std::fmt;

use gameroom_database::DatabaseError;

#[derive(Debug)]
pub enum RestApiServerError {
    StdError(std::io::Error),
}

impl Error for RestApiServerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            RestApiServerError::StdError(err) => Some(err),
        }
    }
}

impl fmt::Display for RestApiServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RestApiServerError::StdError(e) => write!(f, "Std Error: {}", e),
        }
    }
}

impl From<std::io::Error> for RestApiServerError {
    fn from(err: std::io::Error) -> RestApiServerError {
        RestApiServerError::StdError(err)
    }
}

#[derive(Debug)]
pub enum RestApiResponseError {
    DatabaseError(String),
}

impl Error for RestApiResponseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            RestApiResponseError::DatabaseError(_) => None,
        }
    }
}

impl fmt::Display for RestApiResponseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RestApiResponseError::DatabaseError(e) => write!(f, "Database error: {}", e),
        }
    }
}

impl From<DatabaseError> for RestApiResponseError {
    fn from(err: DatabaseError) -> RestApiResponseError {
        RestApiResponseError::DatabaseError(err.to_string())
    }
}

impl From<diesel::result::Error> for RestApiResponseError {
    fn from(err: diesel::result::Error) -> Self {
        RestApiResponseError::DatabaseError(err.to_string())
    }
}
