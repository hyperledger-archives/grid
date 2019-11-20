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

use std::{error, fmt, io};

#[derive(Clone, Debug, PartialEq)]
pub enum ConnectionManagerError {
    StartUpError(String),
    HeartbeatError(String),
    CreateConnectionError(String),
    SendMessageError(String),
    SendTimeoutError(String),
    ConnectionCreationError(String),
    ConnectionRemovalError(String),
    StatePoisoned,
}

impl error::Error for ConnectionManagerError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            ConnectionManagerError::StartUpError(_) => None,
            ConnectionManagerError::HeartbeatError(_) => None,
            ConnectionManagerError::CreateConnectionError(_) => None,
            ConnectionManagerError::SendMessageError(_) => None,
            ConnectionManagerError::SendTimeoutError(_) => None,
            ConnectionManagerError::ConnectionCreationError(_) => None,
            ConnectionManagerError::ConnectionRemovalError(_) => None,
            ConnectionManagerError::StatePoisoned => None,
        }
    }
}

impl fmt::Display for ConnectionManagerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConnectionManagerError::StartUpError(err) => write!(f, "{}", err),
            ConnectionManagerError::HeartbeatError(ref s) => write!(f, "{}", s),
            ConnectionManagerError::CreateConnectionError(ref s) => write!(f, "{}", s),
            ConnectionManagerError::SendMessageError(ref s) => write!(f, "{}", s),
            ConnectionManagerError::SendTimeoutError(ref s) => write!(f, "{}", s),
            ConnectionManagerError::ConnectionCreationError(ref s) => write!(f, "{}", s),
            ConnectionManagerError::ConnectionRemovalError(ref s) => write!(f, "{}", s),
            ConnectionManagerError::StatePoisoned => {
                write!(f, "Connection state has been poisoned")
            }
        }
    }
}

impl From<io::Error> for ConnectionManagerError {
    fn from(err: io::Error) -> Self {
        ConnectionManagerError::StartUpError(err.to_string())
    }
}
