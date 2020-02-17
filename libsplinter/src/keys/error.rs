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

//! Key-related errors

use std::error::Error;
use std::fmt;

/// An error that can occur in the underlying `KeyRegistry` implementation.
#[derive(Debug)]
pub struct KeyRegistryError {
    pub context: String,
    pub source: Option<Box<dyn Error + Send>>,
}

impl Error for KeyRegistryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        if let Some(ref err) = self.source {
            Some(&**err)
        } else {
            None
        }
    }
}

impl fmt::Display for KeyRegistryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref err) = self.source {
            write!(f, "{}: {}", self.context, err)
        } else {
            f.write_str(&self.context)
        }
    }
}

/// An error that can occur in the underlying `KeyPermissions` implementation.
#[derive(Debug)]
pub struct KeyPermissionError {
    pub context: String,
    pub source: Option<Box<dyn Error>>,
}

impl std::error::Error for KeyPermissionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        if let Some(ref err) = self.source {
            Some(&**err)
        } else {
            None
        }
    }
}

impl std::fmt::Display for KeyPermissionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(ref err) = self.source {
            write!(f, "{}: {}", self.context, err)
        } else {
            f.write_str(&self.context)
        }
    }
}
