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

/// Represents CommitStore errors
#[derive(Debug)]
pub enum CommitStoreError {
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

impl Error for CommitStoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CommitStoreError::OperationError {
                source: Some(source),
                ..
            } => Some(&**source),
            CommitStoreError::OperationError { source: None, .. } => None,
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
            CommitStoreError::OperationError {
                context,
                source: Some(source),
            } => write!(f, "failed to perform operation: {}: {}", context, source),
            CommitStoreError::OperationError {
                context,
                source: None,
            } => write!(f, "failed to perform operation: {}", context),
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

/// Represents CommitEvent errors
#[derive(Debug)]
pub enum CommitEventError {
    /// Represents CRUD operations failures
    OperationError {
        context: String,
        source: Option<Box<dyn Error>>,
    },
    /// Represents an issue receiving events
    ConnectionError(String),
}

impl Error for CommitEventError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CommitEventError::OperationError {
                source: Some(source),
                ..
            } => Some(&**source),
            CommitEventError::OperationError { source: None, .. } => None,
            CommitEventError::ConnectionError(_err) => None,
        }
    }
}

impl fmt::Display for CommitEventError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CommitEventError::OperationError {
                context,
                source: Some(source),
            } => write!(f, "failed to perform operation: {}: {}", context, source),
            CommitEventError::OperationError {
                context,
                source: None,
            } => write!(f, "failed to perform operation: {}", context),
            CommitEventError::ConnectionError(err) => write!(f, "Event Error: {}", err),
        }
    }
}
