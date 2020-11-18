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

#[derive(Debug)]
pub enum MigrationsError {
    InternalError(InternalError),
    ConstraintViolationError(ConstraintViolationError),
    ResourceTemporarilyUnavailableError(ResourceTemporarilyUnavailableError),
    MigrationError(Box<dyn Error>),
}

impl Error for MigrationsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            MigrationsError::InternalError(err) => Some(err),
            MigrationsError::ConstraintViolationError(err) => Some(err),
            MigrationsError::ResourceTemporarilyUnavailableError(err) => Some(err),
            MigrationsError::MigrationError(e) => Some(&**e),
        }
    }
}

impl fmt::Display for MigrationsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MigrationsError::InternalError(err) => err.fmt(f),
            MigrationsError::ConstraintViolationError(err) => err.fmt(f),
            MigrationsError::ResourceTemporarilyUnavailableError(err) => err.fmt(f),
            MigrationsError::MigrationError(e) => write!(f, "Unable to migrate database: {}", e),
        }
    }
}

#[cfg(feature = "diesel")]
impl From<diesel::ConnectionError> for MigrationsError {
    fn from(err: diesel::ConnectionError) -> Self {
        MigrationsError::ResourceTemporarilyUnavailableError(
            ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
        )
    }
}

#[cfg(feature = "diesel")]
impl From<diesel_migrations::RunMigrationsError> for MigrationsError {
    fn from(err: diesel_migrations::RunMigrationsError) -> Self {
        MigrationsError::MigrationError(Box::new(err))
    }
}

#[cfg(feature = "diesel")]
impl From<diesel::result::Error> for MigrationsError {
    fn from(err: diesel::result::Error) -> Self {
        match err {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            ) => MigrationsError::ConstraintViolationError(
                ConstraintViolationError::from_source_with_violation_type(
                    ConstraintViolationType::Unique,
                    Box::new(err),
                ),
            ),
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::ForeignKeyViolation,
                _,
            ) => MigrationsError::ConstraintViolationError(
                ConstraintViolationError::from_source_with_violation_type(
                    ConstraintViolationType::ForeignKey,
                    Box::new(err),
                ),
            ),
            _ => MigrationsError::InternalError(InternalError::from_source(Box::new(err))),
        }
    }
}
