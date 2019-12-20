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
// WI()HOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::super::database::postgres::helpers::insert_key;
use super::super::store::{KeyStore, KeyStoreError};
use super::super::Key;
use crate::database::ConnectionPool;

/// Manages creating, updating and fetching keys from a PostgreSQL database.
pub struct PostgresKeyStore {
    connection_pool: ConnectionPool,
}

impl PostgresKeyStore {
    /// Creates a new PostgresKeyStore
    ///
    /// # Arguments
    ///
    ///  * `connection_pool`: connection pool to the PostgreSQL database
    ///
    pub fn new(connection_pool: ConnectionPool) -> Self {
        PostgresKeyStore { connection_pool }
    }
}

impl KeyStore<Key> for PostgresKeyStore {
    fn add_key(&self, key: Key) -> Result<(), KeyStoreError> {
        let key_model = key.into();
        insert_key(&*self.connection_pool.get()?, key_model).map_err(|err| {
            KeyStoreError::OperationError {
                context: "Failed to add key".to_string(),
                source: Box::new(err),
            }
        })?;
        Ok(())
    }

    fn update_key(&self, _updated_key: Key) -> Result<(), KeyStoreError> {
        unimplemented!()
    }

    fn remove_key(&self, _public_key: &str, _user_id: &str) -> Result<Key, KeyStoreError> {
        unimplemented!()
    }

    fn fetch_key(&self, _public_key: &str, _user_id: &str) -> Result<Key, KeyStoreError> {
        unimplemented!()
    }

    fn list_keys(&self, _user_id: Option<&str>) -> Result<Vec<Key>, KeyStoreError> {
        unimplemented!()
    }
}
