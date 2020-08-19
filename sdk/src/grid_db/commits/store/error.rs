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

#[cfg(feature = "diesel")]
use crate::database::error;

#[cfg(feature = "sawtooth-compat")]
use sawtooth_sdk::messaging::stream::{ReceiveError, SendError};

/// Represents CommitStore errors
#[derive(Debug)]
pub enum CommitStoreError {
    /// Represents CRUD operations failures
    OperationError {
        context: String,
        source: Box<dyn Error>,
    },
    /// Represents database query failures
    QueryError {
        context: String,
        source: Box<dyn Error>,
    },
    /// Represents general failures in the database
    StorageError {
        context: String,
        source: Option<Box<dyn Error>>,
    },
    DuplicateError {
        context: String,
        source: Option<Box<dyn Error>>,
    },
    /// Represents an issue connecting to the database
    ConnectionError(Box<dyn Error>),
    NotFoundError(String),
}

impl Error for CommitStoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CommitStoreError::OperationError { source, .. } => Some(&**source),
            CommitStoreError::QueryError { source, .. } => Some(&**source),
            CommitStoreError::StorageError {
                source: Some(source),
                ..
            } => Some(&**source),
            CommitStoreError::StorageError { source: None, .. } => None,
            CommitStoreError::ConnectionError(err) => Some(&**err),
            CommitStoreError::DuplicateError {
                source: Some(source),
                ..
            } => Some(&**source),
            CommitStoreError::DuplicateError { source: None, .. } => None,
            CommitStoreError::NotFoundError(_) => None,
        }
    }
}

impl fmt::Display for CommitStoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CommitStoreError::OperationError { context, source } => {
                write!(f, "failed to perform operation: {}: {}", context, source)
            }
            CommitStoreError::QueryError { context, source } => {
                write!(f, "failed query: {}: {}", context, source)
            }
            CommitStoreError::StorageError {
                context,
                source: Some(source),
            } => write!(
                f,
                "the underlying storage returned an error: {}: {}",
                context, source
            ),
            CommitStoreError::StorageError {
                context,
                source: None,
            } => write!(f, "the underlying storage returned an error: {}", context),
            CommitStoreError::ConnectionError(err) => {
                write!(f, "failed to connect to underlying storage: {}", err)
            }
            CommitStoreError::DuplicateError {
                context,
                source: Some(source),
            } => write!(f, "Commit already exists: {}: {}", context, source),
            CommitStoreError::DuplicateError {
                context,
                source: None,
            } => write!(f, "The commit already exists: {}", context),
            CommitStoreError::NotFoundError(ref s) => write!(f, "Commit not found: {}", s),
        }
    }
}

#[cfg(feature = "diesel")]
impl From<error::DatabaseError> for CommitStoreError {
    fn from(err: error::DatabaseError) -> CommitStoreError {
        CommitStoreError::ConnectionError(Box::new(err))
    }
}

#[cfg(feature = "diesel")]
impl From<diesel::result::Error> for CommitStoreError {
    fn from(err: diesel::result::Error) -> CommitStoreError {
        CommitStoreError::QueryError {
            context: "Diesel query failed".to_string(),
            source: Box::new(err),
        }
    }
}

/// Represents CommitEvent errors
#[derive(Debug)]
pub enum CommitEventError {
    /// Represents CRUD operations failures
    OperationError {
        context: String,
        source: Box<dyn Error>,
    },
    /// Represents an issue receiving events
    ConnectionError(String),
}

#[cfg(feature = "diesel")]
impl From<error::DatabaseError> for CommitEventError {
    fn from(err: error::DatabaseError) -> CommitEventError {
        CommitEventError::ConnectionError(format!("{}", err))
    }
}

impl Error for CommitEventError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CommitEventError::OperationError { source, .. } => Some(&**source),
            CommitEventError::ConnectionError(_err) => None,
        }
    }
}

impl fmt::Display for CommitEventError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CommitEventError::OperationError { context, source } => {
                write!(f, "failed to perform operation: {}: {}", context, source)
            }
            CommitEventError::ConnectionError(err) => write!(f, "Event Error: {}", err),
        }
    }
}

#[derive(Debug)]
pub struct EventError(pub String);

impl Error for EventError {}

impl fmt::Display for EventError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Event Error: {}", self.0)
    }
}

#[derive(Debug)]
pub enum EventIoError {
    ConnectionError(String),
    InvalidMessage(String),
}

impl Error for EventIoError {}

impl fmt::Display for EventIoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ConnectionError(err) => {
                write!(f, "event connection encountered an error: {}", err)
            }
            Self::InvalidMessage(err) => write!(f, "connection received invalid message: {}", err),
        }
    }
}

#[cfg(feature = "sawtooth-compat")]
impl From<ReceiveError> for EventIoError {
    fn from(err: ReceiveError) -> Self {
        EventIoError::ConnectionError(format!("Unable to receive message: {}", &err))
    }
}

#[cfg(feature = "sawtooth-compat")]
impl From<SendError> for EventIoError {
    fn from(err: SendError) -> Self {
        EventIoError::ConnectionError(format!("Unable to send message: {}", &err))
    }
}
