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

#[cfg(feature = "diesel")]
use diesel::r2d2::PoolError;
#[cfg(feature = "diesel")]
use diesel::result::{DatabaseErrorKind, Error as diesel_error};
use std::error::Error;
use std::fmt;

#[cfg(feature = "diesel")]
use crate::error::ConstraintViolationType;
use crate::error::{ConstraintViolationError, InternalError, ResourceTemporarilyUnavailableError};

/// Represents BatchStore errors
#[derive(Debug)]
pub enum BatchStoreError {
    InternalError(InternalError),
    ConstraintViolationError(ConstraintViolationError),
    ResourceTemporarilyUnavailableError(ResourceTemporarilyUnavailableError),
    NotFoundError(String),
}

impl Error for BatchStoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            BatchStoreError::InternalError(err) => Some(err),
            BatchStoreError::ConstraintViolationError(err) => Some(err),
            BatchStoreError::ResourceTemporarilyUnavailableError(err) => Some(err),
            BatchStoreError::NotFoundError(_) => None,
        }
    }
}

impl fmt::Display for BatchStoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BatchStoreError::InternalError(err) => err.fmt(f),
            BatchStoreError::ConstraintViolationError(err) => err.fmt(f),
            BatchStoreError::ResourceTemporarilyUnavailableError(err) => err.fmt(f),
            BatchStoreError::NotFoundError(ref s) => write!(f, "Batch not found: {}", s),
        }
    }
}

#[cfg(feature = "diesel")]
impl From<diesel_error> for BatchStoreError {
    fn from(err: diesel_error) -> BatchStoreError {
        match err {
            diesel_error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                BatchStoreError::ConstraintViolationError(
                    ConstraintViolationError::from_source_with_violation_type(
                        ConstraintViolationType::Unique,
                        Box::new(err),
                    ),
                )
            }
            diesel_error::DatabaseError(DatabaseErrorKind::ForeignKeyViolation, _) => {
                BatchStoreError::ConstraintViolationError(
                    ConstraintViolationError::from_source_with_violation_type(
                        ConstraintViolationType::ForeignKey,
                        Box::new(err),
                    ),
                )
            }
            _ => BatchStoreError::InternalError(InternalError::from_source(Box::new(err))),
        }
    }
}

#[cfg(feature = "diesel")]
impl From<PoolError> for BatchStoreError {
    fn from(err: PoolError) -> BatchStoreError {
        BatchStoreError::ResourceTemporarilyUnavailableError(
            ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
        )
    }
}
