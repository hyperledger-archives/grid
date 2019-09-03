/*
 * Copyright 2019 Cargill Incorporated
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

mod error;

pub use error::ApplicationMetadataError;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApplicationMetadata {
    alias: String,
}

impl ApplicationMetadata {
    pub fn new(alias: &str) -> ApplicationMetadata {
        ApplicationMetadata {
            alias: alias.to_string(),
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<ApplicationMetadata, ApplicationMetadataError> {
        serde_json::from_slice(bytes).map_err(ApplicationMetadataError::DeserializationError)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, ApplicationMetadataError> {
        serde_json::to_vec(self).map_err(ApplicationMetadataError::SerializationError)
    }

    pub fn alias(&self) -> &str {
        &self.alias
    }
}
