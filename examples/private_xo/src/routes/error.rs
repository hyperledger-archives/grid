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

use iron::prelude::*;
use iron::status;
use protobuf::error::ProtobufError;

#[derive(Clone, Debug)]
pub enum BatchSubmitError {
    InvalidBatchListFormat(String),
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

impl From<BatchSubmitError> for IronError {
    fn from(err: BatchSubmitError) -> Self {
        let status = match err {
            BatchSubmitError::InvalidBatchListFormat(_) => status::BadRequest,
            BatchSubmitError::Internal(_) => status::InternalServerError,
        };
        let msg = err.to_string();
        IronError {
            error: Box::new(err),
            response: Response::with((status, msg)),
        }
    }
}

#[derive(Clone, Debug)]
pub enum BatchStatusesError {
    MissingParameter(String),
    InvalidParameter(String),
    Internal(String),
}

impl Error for BatchStatusesError {}

impl fmt::Display for BatchStatusesError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BatchStatusesError::MissingParameter(err_msg) => {
                write!(f, "Missing paramter: {}", err_msg)
            }
            BatchStatusesError::InvalidParameter(err_msg) => {
                write!(f, "Invalid parameter: {}", err_msg)
            }
            BatchStatusesError::Internal(err_msg) => {
                write!(f, "Internal Server Error: {}", err_msg)
            }
        }
    }
}

impl From<BatchStatusesError> for IronError {
    fn from(err: BatchStatusesError) -> Self {
        let status = match err {
            BatchStatusesError::MissingParameter(_) => status::BadRequest,
            BatchStatusesError::InvalidParameter(_) => status::BadRequest,
            BatchStatusesError::Internal(_) => status::InternalServerError,
        };
        let msg = err.to_string();
        IronError {
            error: Box::new(err),
            response: Response::with((status, msg)),
        }
    }
}

#[derive(Clone, Debug)]
pub enum StateError {
    BadRequest(String),
    NotFound(String),
    Internal(String),
    ServiceUnavailable(String),
}

impl Error for StateError {
    fn cause(&self) -> Option<&dyn Error> {
        None
    }

    fn description(&self) -> &str {
        match self {
            StateError::BadRequest(_) => "Invalid input format",
            StateError::NotFound(_) => "The requested state could not be found",
            StateError::Internal(_) => "Internal Server Error",
            StateError::ServiceUnavailable(_) => "The system is not ready",
        }
    }
}

impl fmt::Display for StateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StateError::BadRequest(msg) => write!(f, "Invalid input: {}", msg),
            StateError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            StateError::Internal(err_msg) => write!(f, "Internal Server Error: {}", err_msg),
            StateError::ServiceUnavailable(msg) => write!(f, "Service Unavailable: {}", msg),
        }
    }
}

impl From<StateError> for IronError {
    fn from(err: StateError) -> Self {
        let status = match err {
            StateError::BadRequest(_) => status::BadRequest,
            StateError::NotFound(_) => status::NotFound,
            StateError::Internal(_) => status::InternalServerError,
            StateError::ServiceUnavailable(_) => status::ServiceUnavailable,
        };
        let msg = err.to_string();
        IronError {
            error: Box::new(err),
            response: Response::with((status, msg)),
        }
    }
}
