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

use crate::database::error::DatabaseError;

/// Represents KeyStore errors
#[derive(Debug)]
pub enum KeyStoreError {
    /// Represents CRUD operations failures
    OperationError {
        context: String,
        source: Box<dyn Error>,
    },
    /// Represents database query failures
    /// Disable warning, this will be used for any database fetch helpers
    #[allow(dead_code)]
    QueryError {
        context: String,
        source: Box<dyn Error>,
    },
    /// Represents general failures in the database
    StorageError {
        context: String,
        source: Box<dyn Error>,
    },
    /// Represents an issue connecting to the database
    ConnectionError(Box<dyn Error>),
}

impl Error for KeyStoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            KeyStoreError::OperationError { source, .. } => Some(&**source),
            KeyStoreError::QueryError { source, .. } => Some(&**source),
            KeyStoreError::StorageError { source, .. } => Some(&**source),
            KeyStoreError::ConnectionError(err) => Some(&**err),
        }
    }
}

impl fmt::Display for KeyStoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            KeyStoreError::OperationError { context, source } => {
                write!(f, "failed to perform operation: {}: {}", context, source)
            }
            KeyStoreError::QueryError { context, source } => {
                write!(f, "failed query: {}: {}", context, source)
            }
            KeyStoreError::StorageError { context, source } => write!(
                f,
                "the underlying storage returned an error: {}: {}",
                context, source
            ),
            KeyStoreError::ConnectionError(err) => {
                write!(f, "failed to connect to underlying storage: {}", err)
            }
        }
    }
}

impl From<DatabaseError> for KeyStoreError {
    fn from(err: DatabaseError) -> KeyStoreError {
        match err {
            DatabaseError::ConnectionError(_) => KeyStoreError::ConnectionError(Box::new(err)),
            _ => KeyStoreError::StorageError {
                context: "The database returned an error".to_string(),
                source: Box::new(err),
            },
        }
    }
}
