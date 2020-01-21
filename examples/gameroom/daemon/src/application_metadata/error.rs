/*
 * Copyright 2018-2020 Cargill Incorporated
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * -----------------------------------------------------------------------------
 */

use serde_json::error::Error as SerdeError;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ApplicationMetadataError {
    SerializationError(SerdeError),
    DeserializationError(SerdeError),
}

impl Error for ApplicationMetadataError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ApplicationMetadataError::SerializationError(err) => Some(err),
            ApplicationMetadataError::DeserializationError(err) => Some(err),
        }
    }
}

impl fmt::Display for ApplicationMetadataError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ApplicationMetadataError::SerializationError(e) => {
                write!(f, "Failed to serialize ApplicationMetadata: {}", e)
            }
            ApplicationMetadataError::DeserializationError(e) => {
                write!(f, "Failed to deserialize ApplicationMetadata: {}", e)
            }
        }
    }
}
