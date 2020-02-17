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
// WI()HOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use diesel::result::{DatabaseErrorKind, Error as QueryError};

use super::super::database::postgres::helpers::{
    insert_key, list_keys, list_keys_with_user_id, update_key,
};
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
    // Allow dead code if biome-key-management feature is not enabled
    #[allow(dead_code)]
    pub fn new(connection_pool: ConnectionPool) -> Self {
        PostgresKeyStore { connection_pool }
    }
}

impl KeyStore<Key> for PostgresKeyStore {
    fn add_key(&self, key: Key) -> Result<(), KeyStoreError> {
        let key_model = key.into();
        insert_key(&*self.connection_pool.get()?, &key_model).map_err(|err| {
            if let QueryError::DatabaseError(db_err, _) = err {
                match db_err {
                    DatabaseErrorKind::UniqueViolation => {
                        return KeyStoreError::DuplicateKeyError(format!(
                            "Public key {} for user {} is already in database",
                            key_model.public_key, key_model.user_id
                        ));
                    }
                    DatabaseErrorKind::ForeignKeyViolation => {
                        return KeyStoreError::UserDoesNotExistError(format!(
                            "User with ID {} does not exist in database",
                            key_model.user_id
                        ));
                    }
                    _ => {
                        return KeyStoreError::OperationError {
                            context: "Failed to add key".to_string(),
                            source: Box::new(err),
                        }
                    }
                }
            }
            KeyStoreError::OperationError {
                context: "Failed to add key".to_string(),
                source: Box::new(err),
            }
        })?;
        Ok(())
    }

    fn update_key(
        &self,
        public_key: &str,
        user_id: &str,
        new_display_name: &str,
    ) -> Result<(), KeyStoreError> {
        let updated_row = update_key(
            &*self.connection_pool.get()?,
            &user_id,
            &public_key,
            &new_display_name,
        )
        .map_err(|err| KeyStoreError::OperationError {
            context: "Failed to update key".to_string(),
            source: Box::new(err),
        })?;
        if updated_row == 0 {
            return Err(KeyStoreError::NotFoundError(format!(
                "Key with public key {}, and user ID {} not found",
                public_key, user_id
            )));
        }
        Ok(())
    }

    fn remove_key(&self, _public_key: &str, _user_id: &str) -> Result<Key, KeyStoreError> {
        unimplemented!()
    }

    fn fetch_key(&self, _public_key: &str, _user_id: &str) -> Result<Key, KeyStoreError> {
        unimplemented!()
    }

    fn list_keys(&self, user_id: Option<&str>) -> Result<Vec<Key>, KeyStoreError> {
        let query_result = match user_id {
            Some(user_id) => list_keys_with_user_id(&*self.connection_pool.get()?, user_id)
                .map_err(|err| KeyStoreError::OperationError {
                    context: "Failed to retrieve keys".to_string(),
                    source: Box::new(err),
                })?,
            None => list_keys(&*self.connection_pool.get()?).map_err(|err| {
                KeyStoreError::OperationError {
                    context: "Failed to retrieve keys".to_string(),
                    source: Box::new(err),
                }
            })?,
        };
        let keys = query_result.into_iter().map(Key::from).collect();
        Ok(keys)
    }
}
