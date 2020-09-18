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

/// Represents TrackAndTraceStore errors
#[derive(Debug)]
pub enum TrackAndTraceStoreError {
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

impl Error for TrackAndTraceStoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            TrackAndTraceStoreError::OperationError {
                source: Some(source),
                ..
            } => Some(&**source),
            TrackAndTraceStoreError::OperationError { source: None, .. } => None,
            TrackAndTraceStoreError::QueryError { source, .. } => Some(&**source),
            TrackAndTraceStoreError::StorageError {
                source: Some(source),
                ..
            } => Some(&**source),
            TrackAndTraceStoreError::StorageError { source: None, .. } => None,
            TrackAndTraceStoreError::ConnectionError(err) => Some(&**err),
            TrackAndTraceStoreError::DuplicateError {
                source: Some(source),
                ..
            } => Some(&**source),
            TrackAndTraceStoreError::DuplicateError { source: None, .. } => None,
            TrackAndTraceStoreError::NotFoundError(_) => None,
        }
    }
}

impl fmt::Display for TrackAndTraceStoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TrackAndTraceStoreError::OperationError {
                context,
                source: Some(source),
            } => write!(f, "failed to perform operation: {}: {}", context, source),
            TrackAndTraceStoreError::OperationError {
                context,
                source: None,
            } => write!(f, "failed to perform operation: {}", context),
            TrackAndTraceStoreError::QueryError { context, source } => {
                write!(f, "failed query: {}: {}", context, source)
            }
            TrackAndTraceStoreError::StorageError {
                context,
                source: Some(source),
            } => write!(
                f,
                "the underlying storage returned an error: {}: {}",
                context, source
            ),
            TrackAndTraceStoreError::StorageError {
                context,
                source: None,
            } => write!(f, "the underlying storage returned an error: {}", context),
            TrackAndTraceStoreError::ConnectionError(err) => {
                write!(f, "failed to connect to underlying storage: {}", err)
            }
            TrackAndTraceStoreError::DuplicateError {
                context,
                source: Some(source),
            } => write!(f, "Element already exists: {}: {}", context, source),
            TrackAndTraceStoreError::DuplicateError {
                context,
                source: None,
            } => write!(f, "The element already exists: {}", context),
            TrackAndTraceStoreError::NotFoundError(ref s) => write!(f, "Element not found: {}", s),
        }
    }
}
