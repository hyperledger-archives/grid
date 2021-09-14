// Copyright 2021 Cargill Incorporated
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

use crate::error::{InternalError, InvalidArgumentError};

/// An error that can occur in the validation of data
#[derive(Debug)]
pub enum DataValidationError {
    Internal(InternalError),
    InvalidArgument(InvalidArgumentError),
}

impl Error for DataValidationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            DataValidationError::Internal(err) => Some(err),
            DataValidationError::InvalidArgument(err) => Some(err),
        }
    }
}

impl fmt::Display for DataValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DataValidationError::Internal(err) => err.fmt(f),
            DataValidationError::InvalidArgument(err) => err.fmt(f),
        }
    }
}

impl From<quick_xml::Error> for DataValidationError {
    fn from(err: quick_xml::Error) -> Self {
        DataValidationError::Internal(InternalError::from_source(Box::new(err)))
    }
}

impl From<std::str::Utf8Error> for DataValidationError {
    fn from(err: std::str::Utf8Error) -> Self {
        DataValidationError::Internal(InternalError::from_source(Box::new(err)))
    }
}

impl From<crate::protocol::schema::state::PropertyValueBuildError> for DataValidationError {
    fn from(err: crate::protocol::schema::state::PropertyValueBuildError) -> Self {
        DataValidationError::Internal(InternalError::from_source(Box::new(err)))
    }
}

impl From<crate::protocol::errors::BuilderError> for DataValidationError {
    fn from(err: crate::protocol::errors::BuilderError) -> Self {
        DataValidationError::Internal(InternalError::from_source(Box::new(err)))
    }
}

impl From<std::io::Error> for DataValidationError {
    fn from(err: std::io::Error) -> Self {
        DataValidationError::Internal(InternalError::from_source(Box::new(err)))
    }
}
