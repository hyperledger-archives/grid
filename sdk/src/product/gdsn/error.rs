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

/// An error that can occur in the handling of GDSN XML data
#[derive(Debug)]
pub enum ProductGdsnError {
    Internal(InternalError),
    InvalidArgument(InvalidArgumentError),
}

impl Error for ProductGdsnError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ProductGdsnError::Internal(err) => Some(err),
            ProductGdsnError::InvalidArgument(err) => Some(err),
        }
    }
}

impl fmt::Display for ProductGdsnError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProductGdsnError::Internal(err) => err.fmt(f),
            ProductGdsnError::InvalidArgument(err) => err.fmt(f),
        }
    }
}

impl From<quick_xml::Error> for ProductGdsnError {
    fn from(err: quick_xml::Error) -> Self {
        ProductGdsnError::Internal(InternalError::from_source(Box::new(err)))
    }
}

impl From<std::str::Utf8Error> for ProductGdsnError {
    fn from(err: std::str::Utf8Error) -> Self {
        ProductGdsnError::Internal(InternalError::from_source(Box::new(err)))
    }
}

impl From<crate::protocol::schema::state::PropertyValueBuildError> for ProductGdsnError {
    fn from(err: crate::protocol::schema::state::PropertyValueBuildError) -> Self {
        ProductGdsnError::Internal(InternalError::from_source(Box::new(err)))
    }
}

impl From<crate::protocol::errors::BuilderError> for ProductGdsnError {
    fn from(err: crate::protocol::errors::BuilderError) -> Self {
        ProductGdsnError::Internal(InternalError::from_source(Box::new(err)))
    }
}
