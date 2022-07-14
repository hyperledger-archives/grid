// Copyright 2018-2022 Cargill Incorporated
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

use std::error;
use std::fmt;

#[derive(Debug)]
/// General error type used when constructing a Griddle configuration object
pub enum GriddleConfigError {
    InvalidArgument(String),
    MissingValue(String),
}

impl error::Error for GriddleConfigError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            GriddleConfigError::InvalidArgument(_) => None,
            GriddleConfigError::MissingValue(_) => None,
        }
    }
}

impl fmt::Display for GriddleConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GriddleConfigError::InvalidArgument(msg) => {
                write!(f, "Unable to parse argument: {}", msg)
            }
            GriddleConfigError::MissingValue(msg) => {
                write!(f, "Configuration value must be set: {}", msg)
            }
        }
    }
}
