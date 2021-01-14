// Copyright 2018-2021 Cargill Incorporated
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

use crate::error::{ConstraintViolationError, InternalError, ResourceTemporarilyUnavailableError};

/// Represents AgentStore errors
#[derive(Debug)]
pub enum AgentStoreError {
    InternalError(InternalError),
    ConstraintViolationError(ConstraintViolationError),
    ResourceTemporarilyUnavailableError(ResourceTemporarilyUnavailableError),
    NotFoundError(String),
}

impl Error for AgentStoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            AgentStoreError::InternalError(err) => Some(err),
            AgentStoreError::ConstraintViolationError(err) => Some(err),
            AgentStoreError::ResourceTemporarilyUnavailableError(err) => Some(err),
            AgentStoreError::NotFoundError(_) => None,
        }
    }
}

impl fmt::Display for AgentStoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AgentStoreError::InternalError(err) => err.fmt(f),
            AgentStoreError::ConstraintViolationError(err) => err.fmt(f),
            AgentStoreError::ResourceTemporarilyUnavailableError(err) => err.fmt(f),
            AgentStoreError::NotFoundError(ref s) => write!(f, "Agent not found: {}", s),
        }
    }
}
