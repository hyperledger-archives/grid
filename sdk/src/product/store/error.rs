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

/// Represents Store errors
#[derive(Debug)]
pub enum ProductStoreError {
    InternalError(InternalError),
    ConstraintViolationError(ConstraintViolationError),
    ResourceTemporarilyUnavailableError(ResourceTemporarilyUnavailableError),
    NotFoundError(String),
}

impl Error for ProductStoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ProductStoreError::InternalError(err) => Some(err),
            ProductStoreError::ConstraintViolationError(err) => Some(err),
            ProductStoreError::ResourceTemporarilyUnavailableError(err) => Some(err),
            ProductStoreError::NotFoundError(_) => None,
        }
    }
}

impl fmt::Display for ProductStoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProductStoreError::InternalError(err) => err.fmt(f),
            ProductStoreError::ConstraintViolationError(err) => err.fmt(f),
            ProductStoreError::ResourceTemporarilyUnavailableError(err) => err.fmt(f),
            ProductStoreError::NotFoundError(ref s) => write!(f, "Element not found: {}", s),
        }
    }
}

#[cfg(feature = "diesel")]
impl From<diesel::result::Error> for ProductStoreError {
    fn from(err: diesel::result::Error) -> Self {
        match err {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            ) => ProductStoreError::ConstraintViolationError(
                ConstraintViolationError::from_source_with_violation_type(
                    ConstraintViolationType::Unique,
                    Box::new(err),
                ),
            ),
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::ForeignKeyViolation,
                _,
            ) => ProductStoreError::ConstraintViolationError(
                ConstraintViolationError::from_source_with_violation_type(
                    ConstraintViolationType::ForeignKey,
                    Box::new(err),
                ),
            ),
            _ => ProductStoreError::InternalError(InternalError::from_source(Box::new(err))),
        }
    }
}

#[cfg(feature = "diesel")]
impl From<diesel::r2d2::PoolError> for ProductStoreError {
    fn from(err: diesel::r2d2::PoolError) -> ProductStoreError {
        ProductStoreError::ResourceTemporarilyUnavailableError(
            ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
        )
    }
}

/// Represents ProductBuilder errors
#[derive(Debug)]
pub enum ProductBuilderError {
    /// Returned when a required field was not set
    MissingRequiredField(String),
    /// Returned when an error occurs building the product
    BuildError(Box<dyn Error>),
}

impl Error for ProductBuilderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ProductBuilderError::MissingRequiredField(_) => None,
            ProductBuilderError::BuildError(err) => Some(&**err),
        }
    }
}

impl fmt::Display for ProductBuilderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ProductBuilderError::MissingRequiredField(ref s) => {
                write!(f, "failed to build product: {}", s)
            }
            ProductBuilderError::BuildError(ref s) => {
                write!(f, "failed to build product: {}", s)
            }
        }
    }
}
