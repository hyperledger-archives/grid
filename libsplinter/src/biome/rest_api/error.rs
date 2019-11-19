// Copyright 2019 Cargill Incorporated
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

/// Error for BiomeRestResourceManagerBuilder
#[derive(Debug)]
pub enum BiomeRestResourceManagerBuilderError {
    /// Returned if a required field is missing
    MissingRequiredField(String),
}

impl Error for BiomeRestResourceManagerBuilderError {}

impl fmt::Display for BiomeRestResourceManagerBuilderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BiomeRestResourceManagerBuilderError::MissingRequiredField(ref s) => {
                write!(f, "failed to build BiomeRestResourceManager: {}", s)
            }
        }
    }
}
