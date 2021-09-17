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

/// Represents Store errors
#[derive(Debug)]
pub enum PurchaseOrderStoreError {
    InternalError(InternalError),
    ConstraintViolationError(ConstraintViolationError),
    ResourceTemporarilyUnavailableError(ResourceTemporarilyUnavailableError),
    NotFoundError(String),
}

impl Error for PurchaseOrderStoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            PurchaseOrderStoreError::InternalError(err) => Some(err),
            PurchaseOrderStoreError::ConstraintViolationError(err) => Some(err),
            PurchaseOrderStoreError::ResourceTemporarilyUnavailableError(err) => Some(err),
            PurchaseOrderStoreError::NotFoundError(_) => None,
        }
    }
}

impl fmt::Display for PurchaseOrderStoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PurchaseOrderStoreError::InternalError(err) => err.fmt(f),
            PurchaseOrderStoreError::ConstraintViolationError(err) => err.fmt(f),
            PurchaseOrderStoreError::ResourceTemporarilyUnavailableError(err) => err.fmt(f),
            PurchaseOrderStoreError::NotFoundError(ref s) => write!(f, "Element not found: {}", s),
        }
    }
}

#[cfg(feature = "diesel")]
impl From<diesel::result::Error> for PurchaseOrderStoreError {
    fn from(err: diesel::result::Error) -> Self {
        match err {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            ) => PurchaseOrderStoreError::ConstraintViolationError(
                ConstraintViolationError::from_source_with_violation_type(
                    ConstraintViolationType::Unique,
                    Box::new(err),
                ),
            ),
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::ForeignKeyViolation,
                _,
            ) => PurchaseOrderStoreError::ConstraintViolationError(
                ConstraintViolationError::from_source_with_violation_type(
                    ConstraintViolationType::ForeignKey,
                    Box::new(err),
                ),
            ),
            _ => PurchaseOrderStoreError::InternalError(InternalError::from_source(Box::new(err))),
        }
    }
}

#[cfg(feature = "diesel")]
impl From<diesel::r2d2::PoolError> for PurchaseOrderStoreError {
    fn from(err: diesel::r2d2::PoolError) -> PurchaseOrderStoreError {
        PurchaseOrderStoreError::ResourceTemporarilyUnavailableError(
            ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
        )
    }
}

/// Represents PurchaseOrderBuilder errors
#[derive(Debug)]
pub enum PurchaseOrderBuilderError {
    /// Returned when a required field was not set
    MissingRequiredField(String),
    /// Returned when an error occurs building the PO
    BuildError(Box<dyn Error>),
}

impl Error for PurchaseOrderBuilderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            PurchaseOrderBuilderError::MissingRequiredField(_) => None,
            PurchaseOrderBuilderError::BuildError(err) => Some(&**err),
        }
    }
}

impl fmt::Display for PurchaseOrderBuilderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PurchaseOrderBuilderError::MissingRequiredField(ref s) => {
                write!(f, "Missing required field: {}", s)
            }
            PurchaseOrderBuilderError::BuildError(ref s) => {
                write!(f, "Failed to build purchase order object: {}", s)
            }
        }
    }
}
