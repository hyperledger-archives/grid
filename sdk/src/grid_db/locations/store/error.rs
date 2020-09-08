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

/// Represents LocationStore errors
#[derive(Debug)]
pub enum LocationStoreError {
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

impl Error for LocationStoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            LocationStoreError::OperationError {
                source: Some(source),
                ..
            } => Some(&**source),
            LocationStoreError::OperationError { source: None, .. } => None,
            LocationStoreError::QueryError { source, .. } => Some(&**source),
            LocationStoreError::StorageError {
                source: Some(source),
                ..
            } => Some(&**source),
            LocationStoreError::StorageError { source: None, .. } => None,
            LocationStoreError::ConnectionError(err) => Some(&**err),
            LocationStoreError::DuplicateError {
                source: Some(source),
                ..
            } => Some(&**source),
            LocationStoreError::DuplicateError { source: None, .. } => None,
            LocationStoreError::NotFoundError(_) => None,
        }
    }
}

impl fmt::Display for LocationStoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LocationStoreError::OperationError {
                context,
                source: Some(source),
            } => write!(f, "failed to perform operation: {}: {}", context, source),
            LocationStoreError::OperationError {
                context,
                source: None,
            } => write!(f, "failed to perform operation: {}", context),
            LocationStoreError::QueryError { context, source } => {
                write!(f, "failed query: {}: {}", context, source)
            }
            LocationStoreError::StorageError {
                context,
                source: Some(source),
            } => write!(
                f,
                "the underlying storage returned an error: {}: {}",
                context, source
            ),
            LocationStoreError::StorageError {
                context,
                source: None,
            } => write!(f, "the underlying storage returned an error: {}", context),
            LocationStoreError::ConnectionError(err) => {
                write!(f, "failed to connect to underlying storage: {}", err)
            }
            LocationStoreError::DuplicateError {
                context,
                source: Some(source),
            } => write!(f, "Commit already exists: {}: {}", context, source),
            LocationStoreError::DuplicateError {
                context,
                source: None,
            } => write!(f, "The commit already exists: {}", context),
            LocationStoreError::NotFoundError(ref s) => write!(f, "Commit not found: {}", s),
        }
    }
}

#[cfg(feature = "diesel")]
impl From<error::DatabaseError> for LocationStoreError {
    fn from(err: error::DatabaseError) -> LocationStoreError {
        LocationStoreError::ConnectionError(Box::new(err))
    }
}

#[cfg(feature = "diesel")]
impl From<diesel::result::Error> for LocationStoreError {
    fn from(err: diesel::result::Error) -> LocationStoreError {
        LocationStoreError::QueryError {
            context: "Diesel query failed".to_string(),
            source: Box::new(err),
        }
    }
}

#[cfg(feature = "diesel")]
impl From<diesel::r2d2::PoolError> for LocationStoreError {
    fn from(err: diesel::r2d2::PoolError) -> LocationStoreError {
        LocationStoreError::ConnectionError(Box::new(err))
    }
}
