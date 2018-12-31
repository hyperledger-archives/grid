// Copyright 2018 Cargill Incorporated
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

use protobuf::error::ProtobufError;

#[derive(Responder, Clone, Debug)]
pub enum BatchSubmitError {
    #[response(status = 400)]
    InvalidBatchListFormat(String),
    #[response(status = 500)]
    Internal(String),
}

impl Error for BatchSubmitError {
    fn cause(&self) -> Option<&dyn Error> {
        None
    }

    fn description(&self) -> &str {
        match self {
            BatchSubmitError::InvalidBatchListFormat(_) => "Invalid format of BatchList input",
            BatchSubmitError::Internal(_) => "Internal Server Error",
        }
    }
}

impl fmt::Display for BatchSubmitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BatchSubmitError::InvalidBatchListFormat(err_msg) => {
                write!(f, "Invalid BatchList Format: {}", err_msg)
            }
            BatchSubmitError::Internal(err_msg) => write!(f, "Internal Server Error: {}", err_msg),
        }
    }
}

impl From<ProtobufError> for BatchSubmitError {
    fn from(err: ProtobufError) -> Self {
        BatchSubmitError::InvalidBatchListFormat(format!("{}", err))
    }
}

#[derive(Responder, Clone, Debug)]
pub enum BatchStatusesError {
    #[response(status = 500)]
    Internal(String),
}

impl Error for BatchStatusesError {
    fn cause(&self) -> Option<&dyn Error> {
        None
    }

    fn description(&self) -> &str {
        match self {
            BatchStatusesError::Internal(_) => "Internal Server Error",
        }
    }
}

impl fmt::Display for BatchStatusesError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BatchStatusesError::Internal(err_msg) => {
                write!(f, "Internal Server Error: {}", err_msg)
            }
        }
    }
}

#[derive(Responder, Clone, Debug)]
pub enum StateError {
    #[response(status = 400)]
    BadRequest(String),
    #[response(status = 404)]
    NotFound(String),
}

impl Error for StateError {
    fn cause(&self) -> Option<&dyn Error> {
        None
    }

    fn description(&self) -> &str {
        match self {
            StateError::BadRequest(_) => "Invalid input format",
            StateError::NotFound(_) => "The requested state could not be found",
        }
    }
}

impl fmt::Display for StateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StateError::BadRequest(msg) => write!(f, "Invalid input: {}", msg),
            StateError::NotFound(msg) => write!(f, "Not Found: {}", msg),
        }
    }
}
