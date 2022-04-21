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

//! Provides errors when building transaction payloads

use serde_json::error;

use crate::rest_api::resources::error::ErrorResponse;

#[derive(Debug)]
pub enum BuilderError {
    MissingField(String),
    EmptyVec(String),
}

impl std::fmt::Display for BuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            BuilderError::MissingField(ref s) => write!(f, "MissingField: {}", s),
            BuilderError::EmptyVec(ref s) => write!(f, "EmptyVec: {}", s),
        }
    }
}

impl From<error::Error> for ErrorResponse {
    fn from(e: error::Error) -> Self {
        if e.is_io() {
            ErrorResponse::internal_error(Box::new(e))
        } else {
            ErrorResponse::new(400, &format!("Failed to convert JSON: {e}"))
        }
    }
}
