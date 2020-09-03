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

#[cfg(feature = "diesel")]
impl From<error::DatabaseError> for CommitEventError {
    fn from(err: error::DatabaseError) -> CommitEventError {
        CommitEventError::ConnectionError(format!("{}", err))
    }
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
