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

use actix_web::Error as ActixError;

/// Error module for `rest_api`.
#[derive(Debug)]
pub enum RestApiServerError {
    StartUpError(String),
    MissingField(String),
    StdError(std::io::Error),
}

impl From<std::io::Error> for RestApiServerError {
    fn from(err: std::io::Error) -> RestApiServerError {
        RestApiServerError::StdError(err)
    }
}

impl Error for RestApiServerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            RestApiServerError::StartUpError(_) => None,
            RestApiServerError::StdError(err) => Some(err),
            RestApiServerError::MissingField(_) => None,
        }
    }
}

impl fmt::Display for RestApiServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RestApiServerError::StartUpError(e) => write!(f, "Start-up Error: {}", e),
            RestApiServerError::StdError(e) => write!(f, "Std Error: {}", e),
            RestApiServerError::MissingField(field) => {
                write!(f, "Missing required field: {}", field)
            }
        }
    }
}

#[derive(Debug)]
pub enum ResponseError {
    ActixError(ActixError),
    InternalError(String),
}

impl Error for ResponseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ResponseError::ActixError(err) => Some(err),
            ResponseError::InternalError(_) => None,
        }
    }
}

impl fmt::Display for ResponseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ResponseError::ActixError(err) => write!(
                f,
                "Failed to get response when setting up websocket: {}",
                err
            ),
            ResponseError::InternalError(msg) => f.write_str(&msg),
        }
    }
}

impl From<ActixError> for ResponseError {
    fn from(err: ActixError) -> Self {
        ResponseError::ActixError(err)
    }
}
