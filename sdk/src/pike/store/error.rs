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

/// Represents PikeStore errors
#[derive(Debug)]
pub enum PikeStoreError {
    InternalError(InternalError),
    ConstraintViolationError(ConstraintViolationError),
    ResourceTemporarilyUnavailableError(ResourceTemporarilyUnavailableError),
    NotFoundError(String),
}

impl Error for PikeStoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            PikeStoreError::InternalError(err) => Some(err),
            PikeStoreError::ConstraintViolationError(err) => Some(err),
            PikeStoreError::ResourceTemporarilyUnavailableError(err) => Some(err),
            PikeStoreError::NotFoundError(_) => None,
        }
    }
}

impl fmt::Display for PikeStoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PikeStoreError::InternalError(err) => err.fmt(f),
            PikeStoreError::ConstraintViolationError(err) => err.fmt(f),
            PikeStoreError::ResourceTemporarilyUnavailableError(err) => err.fmt(f),
            PikeStoreError::NotFoundError(ref s) => write!(f, "Resource not found: {}", s),
        }
    }
}
