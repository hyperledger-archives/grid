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

/// Represents Store errors
#[derive(Debug)]
pub enum StoreError {
    /// Represents CRUD operations failures
    OperationError {
        context: String,
        source: Option<Box<dyn Error>>,
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

impl Error for StoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            StoreError::OperationError {
                source: Some(source),
                ..
            } => Some(&**source),
            StoreError::OperationError { source: None, .. } => None,
            StoreError::QueryError { source, .. } => Some(&**source),
            StoreError::StorageError {
                source: Some(source),
                ..
            } => Some(&**source),
            StoreError::StorageError { source: None, .. } => None,
            StoreError::ConnectionError(err) => Some(&**err),
            StoreError::DuplicateError {
                source: Some(source),
                ..
            } => Some(&**source),
            StoreError::DuplicateError { source: None, .. } => None,
            StoreError::NotFoundError(_) => None,
        }
    }
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StoreError::OperationError {
                context,
                source: Some(source),
            } => write!(f, "failed to perform operation: {}: {}", context, source),
            StoreError::OperationError {
                context,
                source: None,
            } => write!(f, "failed to perform operation: {}", context),
            StoreError::QueryError { context, source } => {
                write!(f, "failed query: {}: {}", context, source)
            }
            StoreError::StorageError {
                context,
                source: Some(source),
            } => write!(
                f,
                "the underlying storage returned an error: {}: {}",
                context, source
            ),
            StoreError::StorageError {
                context,
                source: None,
            } => write!(f, "the underlying storage returned an error: {}", context),
            StoreError::ConnectionError(err) => {
                write!(f, "failed to connect to underlying storage: {}", err)
            }
            StoreError::DuplicateError {
                context,
                source: Some(source),
            } => write!(f, "Commit already exists: {}: {}", context, source),
            StoreError::DuplicateError {
                context,
                source: None,
            } => write!(f, "The commit already exists: {}", context),
            StoreError::NotFoundError(ref s) => write!(f, "Commit not found: {}", s),
        }
    }
}

#[cfg(feature = "diesel")]
impl From<error::DatabaseError> for StoreError {
    fn from(err: error::DatabaseError) -> StoreError {
        StoreError::ConnectionError(Box::new(err))
    }
}

#[cfg(feature = "diesel")]
impl From<diesel::result::Error> for StoreError {
    fn from(err: diesel::result::Error) -> StoreError {
        StoreError::QueryError {
            context: "Diesel query failed".to_string(),
            source: Box::new(err),
        }
    }
}

#[cfg(feature = "diesel")]
impl From<diesel::r2d2::PoolError> for StoreError {
    fn from(err: diesel::r2d2::PoolError) -> StoreError {
        StoreError::ConnectionError(Box::new(err))
    }
}
