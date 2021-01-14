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

#[cfg(feature = "diesel")]
use crate::error::ConstraintViolationType;
use crate::error::{ConstraintViolationError, InternalError, ResourceTemporarilyUnavailableError};

/// Represents LocationStore errors
#[derive(Debug)]
pub enum LocationStoreError {
    InternalError(InternalError),
    ConstraintViolationError(ConstraintViolationError),
    ResourceTemporarilyUnavailableError(ResourceTemporarilyUnavailableError),
    NotFoundError(String),
}

impl Error for LocationStoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            LocationStoreError::InternalError(err) => Some(err),
            LocationStoreError::ConstraintViolationError(err) => Some(err),
            LocationStoreError::ResourceTemporarilyUnavailableError(err) => Some(err),
            LocationStoreError::NotFoundError(_) => None,
        }
    }
}

impl fmt::Display for LocationStoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LocationStoreError::InternalError(err) => err.fmt(f),
            LocationStoreError::ConstraintViolationError(err) => err.fmt(f),
            LocationStoreError::ResourceTemporarilyUnavailableError(err) => err.fmt(f),
            LocationStoreError::NotFoundError(ref s) => write!(f, "Commit not found: {}", s),
        }
    }
}

#[cfg(feature = "diesel")]
impl From<diesel::result::Error> for LocationStoreError {
    fn from(err: diesel::result::Error) -> Self {
        match err {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            ) => LocationStoreError::ConstraintViolationError(
                ConstraintViolationError::from_source_with_violation_type(
                    ConstraintViolationType::Unique,
                    Box::new(err),
                ),
            ),
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::ForeignKeyViolation,
                _,
            ) => LocationStoreError::ConstraintViolationError(
                ConstraintViolationError::from_source_with_violation_type(
                    ConstraintViolationType::ForeignKey,
                    Box::new(err),
                ),
            ),
            _ => LocationStoreError::InternalError(InternalError::from_source(Box::new(err))),
        }
    }
}

#[cfg(feature = "diesel")]
impl From<diesel::r2d2::PoolError> for LocationStoreError {
    fn from(err: diesel::r2d2::PoolError) -> LocationStoreError {
        LocationStoreError::ResourceTemporarilyUnavailableError(
            ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
        )
    }
}
