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

pub mod error;
pub mod postgres;

pub use error::KeyStoreError;

/// Defines methods for CRUD operations and fetching and listing keys
/// without defining a storage strategy
pub trait KeyStore<T> {
    /// Adds a key to the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `key` - The key to be added
    ///
    ///
    fn add_key(&self, key: T) -> Result<(), KeyStoreError>;

    /// Updates a key information in the underling storage
    ///
    /// # Arguments
    ///
    /// * `public_key`: The public key of the key record to be updated.
    /// * `user_id`: The ID owner of the key record to be updated.
    /// * `new_display_name`: The new display name of the key record.
    ///
    fn update_key(
        &self,
        public_key: &str,
        user_id: &str,
        new_display_name: &str,
    ) -> Result<(), KeyStoreError>;

    /// Removes a key from the underlying storage
    ///
    /// # Arguments
    ///
    /// * `public_key`: The public key of the key record to be removed.
    /// * `user_id`: The ID owner of the key record to be removed.
    ///
    fn remove_key(&self, public_key: &str, user_id: &str) -> Result<T, KeyStoreError>;

    /// Fetches a key from the underlying storage
    ///
    /// # Arguments
    ///
    /// * `public_key`: The public key of the key record to be removed.
    /// * `user_id`: The ID owner of the key record to be removed.
    ///
    fn fetch_key(&self, public_key: &str, user_id: &str) -> Result<T, KeyStoreError>;

    /// List all keys from the underlying storage
    ///
    /// # Arguments
    ///
    /// * `user_id`: The ID owner of the key records to list.
    ///
    fn list_keys(&self, user_id: Option<&str>) -> Result<Vec<T>, KeyStoreError>;
}
