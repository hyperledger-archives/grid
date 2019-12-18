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
    InvalidNode(InvalidNodeError),
    /// This error is returned when an internal error occurred
    InternalError(Box<dyn Error + Send>),
    /// This error is returned when the user cannot create a node in the registry
    UnableToAddNode(String, Option<Box<dyn Error + Send>>),
}

impl Error for NodeRegistryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            NodeRegistryError::NotFoundError(_) => None,
            NodeRegistryError::InvalidNode(err) => Some(err),
            NodeRegistryError::InternalError(err) => Some(err.as_ref()),
            // Unfortunately, have to match on both arms to return the expected result
            NodeRegistryError::UnableToAddNode(_, Some(err)) => Some(err.as_ref()),
            NodeRegistryError::UnableToAddNode(_, None) => None,
        }
    }
}

impl fmt::Display for NodeRegistryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NodeRegistryError::NotFoundError(e) => write!(f, "Node not found: {}", e),
            NodeRegistryError::InvalidNode(e) => write!(f, "Invalid node: {}", e),
            NodeRegistryError::InternalError(e) => write!(f, "Internal error: {}", e),
            NodeRegistryError::UnableToAddNode(msg, err) => write!(
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

impl From<InvalidNodeError> for NodeRegistryError {
    fn from(err: InvalidNodeError) -> Self {
        NodeRegistryError::InvalidNode(err)
    }
}

#[derive(Debug)]
pub enum InvalidNodeError {
    DuplicateEndpoint(String),
    DuplicateIdentity(String),
    EmptyEndpoint,
    EmptyIdentity,
    EmptyDisplayName,
    InvalidIdentity(String, String), // (identity, message)
}

impl Error for InvalidNodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            InvalidNodeError::DuplicateEndpoint(_) => None,
            InvalidNodeError::DuplicateIdentity(_) => None,
            InvalidNodeError::EmptyEndpoint => None,
            InvalidNodeError::EmptyIdentity => None,
            InvalidNodeError::EmptyDisplayName => None,
            InvalidNodeError::InvalidIdentity(..) => None,
        }
    }
}

impl fmt::Display for InvalidNodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InvalidNodeError::DuplicateEndpoint(endpoint) => {
                write!(f, "another node with endpoint {} exists", endpoint)
            }
            InvalidNodeError::DuplicateIdentity(identity) => {
                write!(f, "another node with identity {} exists", identity)
            }
            InvalidNodeError::EmptyEndpoint => write!(f, "node must have non-empty endpoint"),
            InvalidNodeError::EmptyIdentity => write!(f, "node must have non-empty identity"),
            InvalidNodeError::EmptyDisplayName => {
                write!(f, "node must have non-empty display_name")
            }
            InvalidNodeError::InvalidIdentity(identity, msg) => {
                write!(f, "identity {} is invalid: {}", identity, msg)
            }
        }
    }
}
