// Copyright 2022 Cargill Incorporated
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
use crate::proxy::response::ProxyResponse;

/// Represents Store errors
#[derive(Debug)]
pub enum ProxyError {
    InternalError(InternalError),
    InvalidArgumentError(InvalidArgumentError),
}

impl Error for ProxyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ProxyError::InternalError(err) => Some(err),
            ProxyError::InvalidArgumentError(err) => Some(err),
        }
    }
}

impl fmt::Display for ProxyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProxyError::InternalError(err) => err.fmt(f),
            ProxyError::InvalidArgumentError(err) => err.fmt(f),
        }
    }
}

impl From<ProxyError> for ProxyResponse {
    fn from(err: ProxyError) -> Self {
        match err {
            ProxyError::InternalError(err) => {
                ProxyResponse::new(500, format!("{err}").as_bytes().to_owned())
            }
            ProxyError::InvalidArgumentError(err) => {
                ProxyResponse::new(400, format!("{err}").as_bytes().to_owned())
            }
        }
    }
}
