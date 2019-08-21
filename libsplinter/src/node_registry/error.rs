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

#[derive(Debug)]
pub enum NodeRegistryError {
    /// This error is returned when a node is not found.
    NotFoundError(String),
    /// This error is returned when the user attempts to create a node with an identity that
    /// already exists.
    DuplicateNodeError(String),
    /// This error is returned when the user attempts to filter the nodes list using an invalid
    /// filter.
    InvalidFilterError(String),
    /// This error is returned when an internal error occurred
    InternalError(Box<dyn Error + Send>),
    /// This error is returned when the user cannot create a node in the registry
    UnableToCreateNode(String, Option<Box<dyn Error + Send>>),
}

impl Error for NodeRegistryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            NodeRegistryError::NotFoundError(_) => None,
            NodeRegistryError::DuplicateNodeError(_) => None,
            NodeRegistryError::InvalidFilterError(_) => None,
            NodeRegistryError::InternalError(err) => Some(err.as_ref()),
            // Unfortunately, have to match on both arms to return the expected result
            NodeRegistryError::UnableToCreateNode(_, Some(err)) => Some(err.as_ref()),
            NodeRegistryError::UnableToCreateNode(_, None) => None,
        }
    }
}

impl fmt::Display for NodeRegistryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NodeRegistryError::NotFoundError(e) => write!(f, "Node not found: {}", e),
            NodeRegistryError::DuplicateNodeError(e) => write!(f, "Duplicate identity: {}", e),
            NodeRegistryError::InvalidFilterError(e) => write!(f, "Invalid filter: {}", e),
            NodeRegistryError::InternalError(e) => write!(f, "Internal error: {}", e),
            NodeRegistryError::UnableToCreateNode(msg, err) => write!(
                f,
                "unable to add node: {}{}",
                msg,
                err.as_ref()
                    .map(|e| format!("; {}", e))
                    .unwrap_or_else(|| "".to_string())
            ),
        }
    }
}
