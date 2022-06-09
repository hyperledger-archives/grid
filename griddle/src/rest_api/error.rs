// Copyright 2018-2022 Cargill, Inc.
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

use grid_sdk::error::{InternalError, InvalidArgumentError, InvalidStateError};

#[derive(Debug)]
pub enum GriddleRestApiServerError {
    BindError(String),
    StartUpError(String),
    StdError(std::io::Error),
    InternalError(InternalError),
    InvalidArgument(InvalidArgumentError),
    InvalidState(InvalidStateError),
}

impl From<std::io::Error> for GriddleRestApiServerError {
    fn from(err: std::io::Error) -> GriddleRestApiServerError {
        GriddleRestApiServerError::StdError(err)
    }
}

impl Error for GriddleRestApiServerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            GriddleRestApiServerError::BindError(_) => None,
            GriddleRestApiServerError::StartUpError(_) => None,
            GriddleRestApiServerError::StdError(err) => Some(err),
            GriddleRestApiServerError::InternalError(err) => Some(err),
            GriddleRestApiServerError::InvalidArgument(err) => Some(err),
            GriddleRestApiServerError::InvalidState(err) => Some(err),
        }
    }
}

impl fmt::Display for GriddleRestApiServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GriddleRestApiServerError::BindError(e) => write!(f, "Griddle Bind Error: {}", e),
            GriddleRestApiServerError::StartUpError(e) => {
                write!(f, "Griddle Start-up Error: {}", e)
            }
            GriddleRestApiServerError::StdError(e) => write!(f, "Std Error in Griddle: {}", e),
            GriddleRestApiServerError::InternalError(e) => write!(f, "{}", e),
            GriddleRestApiServerError::InvalidArgument(e) => write!(f, "{}", e),
            GriddleRestApiServerError::InvalidState(e) => write!(f, "{}", e),
        }
    }
}
