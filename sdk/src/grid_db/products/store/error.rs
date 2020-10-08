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
pub enum ProductStoreError {
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

impl Error for ProductStoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ProductStoreError::OperationError {
                source: Some(source),
                ..
            } => Some(&**source),
            ProductStoreError::OperationError { source: None, .. } => None,
            ProductStoreError::QueryError { source, .. } => Some(&**source),
            ProductStoreError::StorageError {
                source: Some(source),
                ..
            } => Some(&**source),
            ProductStoreError::StorageError { source: None, .. } => None,
            ProductStoreError::ConnectionError(err) => Some(&**err),
            ProductStoreError::DuplicateError {
                source: Some(source),
                ..
            } => Some(&**source),
            ProductStoreError::DuplicateError { source: None, .. } => None,
            ProductStoreError::NotFoundError(_) => None,
        }
    }
}

impl fmt::Display for ProductStoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProductStoreError::OperationError {
                context,
                source: Some(source),
            } => write!(f, "Failed to perform operation: {}: {}", context, source),
            ProductStoreError::OperationError {
                context,
                source: None,
            } => write!(f, "Failed to perform operation: {}", context),
            ProductStoreError::QueryError { context, source } => {
                write!(f, "Failed query: {}: {}", context, source)
            }
            ProductStoreError::StorageError {
                context,
                source: Some(source),
            } => write!(
                f,
                "The underlying storage returned an error: {}: {}",
                context, source
            ),
            ProductStoreError::StorageError {
                context,
                source: None,
            } => write!(f, "The underlying storage returned an error: {}", context),
            ProductStoreError::ConnectionError(err) => {
                write!(f, "Failed to connect to underlying storage: {}", err)
            }
            ProductStoreError::DuplicateError {
                context,
                source: Some(source),
            } => write!(f, "Element already exists: {}: {}", context, source),
            ProductStoreError::DuplicateError {
                context,
                source: None,
            } => write!(f, "The element already exists: {}", context),
            ProductStoreError::NotFoundError(ref s) => write!(f, "Element not found: {}", s),
        }
    }
}

#[cfg(feature = "diesel")]
impl From<error::DatabaseError> for ProductStoreError {
    fn from(err: error::DatabaseError) -> ProductStoreError {
        ProductStoreError::ConnectionError(Box::new(err))
    }
}

#[cfg(feature = "diesel")]
impl From<diesel::result::Error> for ProductStoreError {
    fn from(err: diesel::result::Error) -> ProductStoreError {
        ProductStoreError::QueryError {
            context: "Diesel query failed".to_string(),
            source: Box::new(err),
        }
    }
}

#[cfg(feature = "diesel")]
impl From<diesel::r2d2::PoolError> for ProductStoreError {
    fn from(err: diesel::r2d2::PoolError) -> ProductStoreError {
        ProductStoreError::ConnectionError(Box::new(err))
    }
}
