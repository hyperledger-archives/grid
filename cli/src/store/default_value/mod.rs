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

mod error;
mod yaml_store;

pub use error::DefaultStoreError;

pub use yaml_store::FileBackedDefaultStore;

pub struct DefaultValue {
    key: String,
    value: String,
}

impl DefaultValue {
    pub fn new(key: &str, value: &str) -> DefaultValue {
        DefaultValue {
            key: key.to_owned(),
            value: value.to_owned(),
        }
    }

    pub fn key(&self) -> String {
        self.key.to_owned()
    }

    pub fn value(&self) -> String {
        self.value.to_owned()
    }
}

pub trait DefaultValueStore {
    /// Set new default value
    fn set_default_value(&self, default_value: &DefaultValue) -> Result<(), DefaultStoreError>;

    /// Unset a default value
    fn unset_default_value(&self, default_key: &str) -> Result<(), DefaultStoreError>;

    /// List default values
    fn list_default_values(&self) -> Result<Vec<DefaultValue>, DefaultStoreError>;

    /// Get a default value
    fn get_default_value(&self, key: &str) -> Result<Option<DefaultValue>, DefaultStoreError>;
}
