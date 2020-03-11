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

use super::KeyStoreOperations;
use crate::biome::datastore::{models::KeyModel, schema::keys};
use crate::biome::key_management::{store::KeyStoreError, Key};

use diesel::{dsl::delete, prelude::*, result::Error::NotFound};

pub(in super::super) trait KeyStoreRemoveKeyOperation {
    fn remove_key(&self, public_key: &str, user_id: &str) -> Result<Key, KeyStoreError>;
}

impl<'a, C> KeyStoreRemoveKeyOperation for KeyStoreOperations<'a, C>
where
    C: diesel::Connection,
    <C as diesel::Connection>::Backend: diesel::backend::SupportsDefaultKeyword,
    <C as diesel::Connection>::Backend: 'static,
    String: diesel::deserialize::FromSql<diesel::sql_types::Text, C::Backend>,
{
    fn remove_key(&self, public_key: &str, user_id: &str) -> Result<Key, KeyStoreError> {
        let key = keys::table
            .filter(
                keys::public_key
                    .eq(public_key)
                    .and(keys::user_id.eq(user_id)),
            )
            .first::<KeyModel>(self.conn)
            .map(Some)
            .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
            .map_err(|err| KeyStoreError::QueryError {
                context: "Failed to fetch key by user ID and public key".to_string(),
                source: Box::new(err),
            })?
            .ok_or_else(|| {
                KeyStoreError::NotFoundError(format!(
                    "Failed to find key with public key: {} and user id: {}",
                    public_key, user_id
                ))
            })?;

        delete(
            keys::table.filter(
                keys::public_key
                    .eq(public_key)
                    .and(keys::user_id.eq(user_id)),
            ),
        )
        .execute(self.conn)
        .map(|_| ())
        .map_err(|err| KeyStoreError::OperationError {
            context: "Failed to delete key".to_string(),
            source: Box::new(err),
        })?;

        Ok(Key::from(key))
    }
}
