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

/// Represents AgentStore errors
#[derive(Debug)]
pub enum AgentStoreError {
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

impl Error for AgentStoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            AgentStoreError::OperationError {
                source: Some(source),
                ..
            } => Some(&**source),
            AgentStoreError::OperationError { source: None, .. } => None,
            AgentStoreError::QueryError { source, .. } => Some(&**source),
            AgentStoreError::StorageError {
                source: Some(source),
                ..
            } => Some(&**source),
            AgentStoreError::StorageError { source: None, .. } => None,
            AgentStoreError::ConnectionError(err) => Some(&**err),
            AgentStoreError::DuplicateError {
                source: Some(source),
                ..
            } => Some(&**source),
            AgentStoreError::DuplicateError { source: None, .. } => None,
            AgentStoreError::NotFoundError(_) => None,
        }
    }
}

impl fmt::Display for AgentStoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AgentStoreError::OperationError {
                context,
                source: Some(source),
            } => write!(f, "failed to perform operation: {}: {}", context, source),
            AgentStoreError::OperationError {
                context,
                source: None,
            } => write!(f, "failed to perform operation: {}", context),
            AgentStoreError::QueryError { context, source } => {
                write!(f, "failed query: {}: {}", context, source)
            }
            AgentStoreError::StorageError {
                context,
                source: Some(source),
            } => write!(
                f,
                "the underlying storage returned an error: {}: {}",
                context, source
            ),
            AgentStoreError::StorageError {
                context,
                source: None,
            } => write!(f, "the underlying storage returned an error: {}", context),
            AgentStoreError::ConnectionError(err) => {
                write!(f, "failed to connect to underlying storage: {}", err)
            }
            AgentStoreError::DuplicateError {
                context,
                source: Some(source),
            } => write!(f, "Agent already exists: {}: {}", context, source),
            AgentStoreError::DuplicateError {
                context,
                source: None,
            } => write!(f, "The agent already exists: {}", context),
            AgentStoreError::NotFoundError(ref s) => write!(f, "Agent not found: {}", s),
        }
    }
}
