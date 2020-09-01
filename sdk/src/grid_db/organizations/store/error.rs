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

/// Represents OrganizationStore errors
#[derive(Debug)]
pub enum OrganizationStoreError {
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

impl Error for OrganizationStoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            OrganizationStoreError::OperationError {
                source: Some(source),
                ..
            } => Some(&**source),
            OrganizationStoreError::OperationError { source: None, .. } => None,
            OrganizationStoreError::QueryError { source, .. } => Some(&**source),
            OrganizationStoreError::StorageError {
                source: Some(source),
                ..
            } => Some(&**source),
            OrganizationStoreError::StorageError { source: None, .. } => None,
            OrganizationStoreError::ConnectionError(err) => Some(&**err),
            OrganizationStoreError::DuplicateError {
                source: Some(source),
                ..
            } => Some(&**source),
            OrganizationStoreError::DuplicateError { source: None, .. } => None,
            OrganizationStoreError::NotFoundError(_) => None,
        }
    }
}

impl fmt::Display for OrganizationStoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OrganizationStoreError::OperationError {
                context,
                source: Some(source),
            } => write!(f, "failed to perform operation: {}: {}", context, source),
            OrganizationStoreError::OperationError {
                context,
                source: None,
            } => write!(f, "failed to perform operation: {}", context),
            OrganizationStoreError::QueryError { context, source } => {
                write!(f, "failed query: {}: {}", context, source)
            }
            OrganizationStoreError::StorageError {
                context,
                source: Some(source),
            } => write!(
                f,
                "the underlying storage returned an error: {}: {}",
                context, source
            ),
            OrganizationStoreError::StorageError {
                context,
                source: None,
            } => write!(f, "the underlying storage returned an error: {}", context),
            OrganizationStoreError::ConnectionError(err) => {
                write!(f, "failed to connect to underlying storage: {}", err)
            }
            OrganizationStoreError::DuplicateError {
                context,
                source: Some(source),
            } => write!(f, "Organization already exists: {}: {}", context, source),
            OrganizationStoreError::DuplicateError {
                context,
                source: None,
            } => write!(f, "The organization already exists: {}", context),
            OrganizationStoreError::NotFoundError(ref s) => {
                write!(f, "Organization not found: {}", s)
            }
        }
    }
}

#[cfg(feature = "diesel")]
impl From<error::DatabaseError> for OrganizationStoreError {
    fn from(err: error::DatabaseError) -> OrganizationStoreError {
        OrganizationStoreError::ConnectionError(Box::new(err))
    }
}

#[cfg(feature = "diesel")]
impl From<diesel::result::Error> for OrganizationStoreError {
    fn from(err: diesel::result::Error) -> OrganizationStoreError {
        OrganizationStoreError::QueryError {
            context: "Diesel query failed".to_string(),
            source: Box::new(err),
        }
    }
}

#[cfg(feature = "diesel")]
impl From<diesel::r2d2::PoolError> for OrganizationStoreError {
    fn from(err: diesel::r2d2::PoolError) -> OrganizationStoreError {
        OrganizationStoreError::ConnectionError(Box::new(err))
    }
}
