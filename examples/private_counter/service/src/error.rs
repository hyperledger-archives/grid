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
use std::io::Error as IoError;

use splinter::consensus::error::ProposalManagerError;

#[derive(Clone, Debug)]
pub enum HandleError {
    IoError(String),
    ServiceError(ServiceError),
}

impl Error for HandleError {
    fn cause(&self) -> Option<&dyn Error> {
        match self {
            HandleError::IoError(_) => None,
            HandleError::ServiceError(err) => Some(err),
        }
    }

    fn description(&self) -> &str {
        match self {
            HandleError::IoError(_) => "Received IO Error",
            HandleError::ServiceError(_) => "Encountered Service Error",
        }
    }
}

impl fmt::Display for HandleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HandleError::IoError(msg) => write!(f, "IoError: {}", msg),
            HandleError::ServiceError(err) => write!(f, "Service Error: {}", err),
        }
    }
}

impl From<IoError> for HandleError {
    fn from(err: IoError) -> Self {
        HandleError::IoError(format!("{}", err))
    }
}

impl From<ServiceError> for HandleError {
    fn from(err: ServiceError) -> Self {
        HandleError::ServiceError(err)
    }
}

#[derive(Clone, Debug)]
pub struct ServiceError(pub String);

impl Error for ServiceError {
    fn cause(&self) -> Option<&dyn Error> {
        None
    }
    fn description(&self) -> &str {
        "A Service Error"
    }
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<protobuf::ProtobufError> for ServiceError {
    fn from(err: protobuf::ProtobufError) -> Self {
        ServiceError(format!("Protocol Buffer Error: {}", err))
    }
}

impl From<String> for ServiceError {
    fn from(s: String) -> Self {
        ServiceError(s)
    }
}

impl<T> From<crossbeam_channel::SendError<T>> for ServiceError {
    fn from(err: crossbeam_channel::SendError<T>) -> Self {
        ServiceError(format!("Unable to send: {}", err))
    }
}

impl<T> From<std::sync::mpsc::SendError<T>> for ServiceError {
    fn from(err: std::sync::mpsc::SendError<T>) -> Self {
        ServiceError(format!("Unable to send: {}", err))
    }
}

impl From<ServiceError> for ProposalManagerError {
    fn from(err: ServiceError) -> Self {
        ProposalManagerError::Internal(Box::new(err))
    }
}
