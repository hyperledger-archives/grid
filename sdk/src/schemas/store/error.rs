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

use crate::error::{ConstraintViolationError, InternalError, ResourceTemporarilyUnavailableError};

/// Represents Store errors
#[derive(Debug)]
pub enum SchemaStoreError {
    InternalError(InternalError),
    ConstraintViolationError(ConstraintViolationError),
    ResourceTemporarilyUnavailableError(ResourceTemporarilyUnavailableError),
    NotFoundError(String),
}

impl Error for SchemaStoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            SchemaStoreError::InternalError(err) => Some(err),
            SchemaStoreError::ConstraintViolationError(err) => Some(err),
            SchemaStoreError::ResourceTemporarilyUnavailableError(err) => Some(err),
            SchemaStoreError::NotFoundError(_) => None,
        }
    }
}

impl fmt::Display for SchemaStoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SchemaStoreError::InternalError(err) => err.fmt(f),
            SchemaStoreError::ConstraintViolationError(err) => err.fmt(f),
            SchemaStoreError::ResourceTemporarilyUnavailableError(err) => err.fmt(f),
            SchemaStoreError::NotFoundError(ref s) => write!(f, "Element not found: {}", s),
        }
    }
}
