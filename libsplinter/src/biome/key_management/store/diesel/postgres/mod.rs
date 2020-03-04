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

//! Defines methods and utilities to interact with key management tables in the database.

embed_migrations!("./src/biome/key_management/store/diesel/postgres/migrations");

use diesel::pg::PgConnection;

use super::super::{KeyStore, KeyStoreError};
use super::operations::fetch_key::KeyStoreFetchKeyOperation as _;
use super::operations::insert_key::KeyStoreInsertKeyOperation as _;
use super::operations::list_keys::KeyStoreListKeysOperation as _;
use super::operations::list_keys::KeyStoreListKeysWithUserIDOperation as _;
use super::operations::update_key::KeyStoreUpdateKeyOperation as _;
use super::operations::KeyStoreOperations;

use crate::biome::key_management::Key;
use crate::database::error::DatabaseError;
use crate::database::ConnectionPool;

/// Run database migrations to create tables defined in the key management module
///
/// # Arguments
///
/// * `conn` - Connection to database
///
pub fn run_migrations(conn: &PgConnection) -> Result<(), DatabaseError> {
    embedded_migrations::run(conn).map_err(|err| DatabaseError::ConnectionError(Box::new(err)))?;

    info!("Successfully applied Biome key management migrations");

    Ok(())
}

/// Manages creating, updating and fetching keys from a PostgreSQL database.
pub struct PostgresKeyStore {
    pub connection_pool: ConnectionPool,
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
        KeyStoreOperations::new(&*self.connection_pool.get()?).insert_key(key)
    }

    fn update_key(
        &self,
        public_key: &str,
        user_id: &str,
        new_display_name: &str,
    ) -> Result<(), KeyStoreError> {
        KeyStoreOperations::new(&*self.connection_pool.get()?).update_key(
            public_key,
            user_id,
            new_display_name,
        )
    }

    fn remove_key(&self, _public_key: &str, _user_id: &str) -> Result<Key, KeyStoreError> {
        unimplemented!()
    }

    fn fetch_key(&self, public_key: &str, user_id: &str) -> Result<Key, KeyStoreError> {
        KeyStoreOperations::new(&*self.connection_pool.get()?).fetch_key(public_key, user_id)
    }

    fn list_keys(&self, user_id: Option<&str>) -> Result<Vec<Key>, KeyStoreError> {
        match user_id {
            Some(user_id) => KeyStoreOperations::new(&*self.connection_pool.get()?)
                .list_keys_with_user_id(user_id),
            None => KeyStoreOperations::new(&*self.connection_pool.get()?).list_keys(),
        }
    }
}
