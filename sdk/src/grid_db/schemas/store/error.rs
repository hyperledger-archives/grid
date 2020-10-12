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

/// Represents Store errors
#[derive(Debug)]
pub enum SchemaStoreError {
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

impl Error for SchemaStoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            SchemaStoreError::OperationError {
                source: Some(source),
                ..
            } => Some(&**source),
            SchemaStoreError::OperationError { source: None, .. } => None,
            SchemaStoreError::QueryError { source, .. } => Some(&**source),
            SchemaStoreError::StorageError {
                source: Some(source),
                ..
            } => Some(&**source),
            SchemaStoreError::StorageError { source: None, .. } => None,
            SchemaStoreError::ConnectionError(err) => Some(&**err),
            SchemaStoreError::DuplicateError {
                source: Some(source),
                ..
            } => Some(&**source),
            SchemaStoreError::DuplicateError { source: None, .. } => None,
            SchemaStoreError::NotFoundError(_) => None,
        }
    }
}

impl fmt::Display for SchemaStoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SchemaStoreError::OperationError {
                context,
                source: Some(source),
            } => write!(f, "Failed to perform operation: {}: {}", context, source),
            SchemaStoreError::OperationError {
                context,
                source: None,
            } => write!(f, "Failed to perform operation: {}", context),
            SchemaStoreError::QueryError { context, source } => {
                write!(f, "Failed query: {}: {}", context, source)
            }
            SchemaStoreError::StorageError {
                context,
                source: Some(source),
            } => write!(
                f,
                "The underlying storage returned an error: {}: {}",
                context, source
            ),
            SchemaStoreError::StorageError {
                context,
                source: None,
            } => write!(f, "The underlying storage returned an error: {}", context),
            SchemaStoreError::ConnectionError(err) => {
                write!(f, "Failed to connect to underlying storage: {}", err)
            }
            SchemaStoreError::DuplicateError {
                context,
                source: Some(source),
            } => write!(f, "Element already exists: {}: {}", context, source),
            SchemaStoreError::DuplicateError {
                context,
                source: None,
            } => write!(f, "The element already exists: {}", context),
            SchemaStoreError::NotFoundError(ref s) => write!(f, "Element not found: {}", s),
        }
    }
}
