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
pub enum GriddleError {
    InvalidArgumentError(String),
    MissingRequiredField(String),
    InternalError(String),
}

impl error::Error for GriddleError {}

impl fmt::Display for GriddleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GriddleError::InvalidArgumentError(msg) => f.write_str(msg),
            GriddleError::MissingRequiredField(msg) => {
                write!(f, "missing required field: {}", msg)
            }
            GriddleError::InternalError(msg) => f.write_str(msg),
        }
    }
}
