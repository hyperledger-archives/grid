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
use crate::error::ConstraintViolationType;
use crate::error::{ConstraintViolationError, InternalError, ResourceTemporarilyUnavailableError};

/// Represents CommitStore errors
#[derive(Debug)]
pub enum CommitStoreError {
    InternalError(InternalError),
    ConstraintViolationError(ConstraintViolationError),
    ResourceTemporarilyUnavailableError(ResourceTemporarilyUnavailableError),
    NotFoundError(String),
}

impl Error for CommitStoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CommitStoreError::InternalError(err) => Some(err),
            CommitStoreError::ConstraintViolationError(err) => Some(err),
            CommitStoreError::ResourceTemporarilyUnavailableError(err) => Some(err),
            CommitStoreError::NotFoundError(_) => None,
        }
    }
}

impl fmt::Display for CommitStoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CommitStoreError::InternalError(err) => err.fmt(f),
            CommitStoreError::ConstraintViolationError(err) => err.fmt(f),
            CommitStoreError::ResourceTemporarilyUnavailableError(err) => err.fmt(f),
            CommitStoreError::NotFoundError(ref s) => write!(f, "Commit not found: {}", s),
        }
    }
}

/// Represents CommitEvent errors
#[derive(Debug)]
pub enum CommitEventError {
    InternalError(InternalError),
    ConstraintViolationError(ConstraintViolationError),
    ResourceTemporarilyUnavailableError(ResourceTemporarilyUnavailableError),
}

impl Error for CommitEventError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CommitEventError::InternalError(err) => Some(err),
            CommitEventError::ConstraintViolationError(err) => Some(err),
            CommitEventError::ResourceTemporarilyUnavailableError(err) => Some(err),
        }
    }
}

impl fmt::Display for CommitEventError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CommitEventError::InternalError(err) => err.fmt(f),
            CommitEventError::ConstraintViolationError(err) => err.fmt(f),
            CommitEventError::ResourceTemporarilyUnavailableError(err) => err.fmt(f),
        }
    }
}

#[cfg(feature = "diesel")]
impl From<diesel::result::Error> for CommitEventError {
    fn from(err: diesel::result::Error) -> Self {
        match err {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            ) => CommitEventError::ConstraintViolationError(
                ConstraintViolationError::from_source_with_violation_type(
                    ConstraintViolationType::Unique,
                    Box::new(err),
                ),
            ),
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::ForeignKeyViolation,
                _,
            ) => CommitEventError::ConstraintViolationError(
                ConstraintViolationError::from_source_with_violation_type(
                    ConstraintViolationType::ForeignKey,
                    Box::new(err),
                ),
            ),
            _ => CommitEventError::InternalError(InternalError::from_source(Box::new(err))),
        }
    }
}

#[cfg(feature = "diesel")]
impl From<diesel::r2d2::PoolError> for CommitEventError {
    fn from(err: diesel::r2d2::PoolError) -> CommitEventError {
        CommitEventError::ResourceTemporarilyUnavailableError(
            ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
        )
    }
}
