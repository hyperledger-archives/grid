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

use super::super::models::KeyModel;
use super::super::schema::keys;
use super::KeyStoreOperations;
use crate::biome::key_management::{store::KeyStoreError, Key};

use diesel::{prelude::*, result::Error::NotFound};

pub(in super::super) trait KeyStoreListKeysOperation {
    fn list_keys(&self) -> Result<Vec<Key>, KeyStoreError>;
}

impl<'a, C> KeyStoreListKeysOperation for KeyStoreOperations<'a, C>
where
    C: diesel::Connection,
    <C as diesel::Connection>::Backend: diesel::backend::SupportsDefaultKeyword,
    <C as diesel::Connection>::Backend: 'static,
    String: diesel::deserialize::FromSql<diesel::sql_types::Text, C::Backend>,
{
    fn list_keys(&self) -> Result<Vec<Key>, KeyStoreError> {
        let keys = keys::table
            .load::<KeyModel>(self.conn)
            .map(Some)
            .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
            .map_err(|err| KeyStoreError::OperationError {
                context: "Failed to get keys".to_string(),
                source: Box::new(err),
            })?
            .ok_or_else(|| {
                KeyStoreError::NotFoundError("Could not get all keys from storage".to_string())
            })?
            .into_iter()
            .map(Key::from)
            .collect();
        Ok(keys)
    }
}

pub(in super::super) trait KeyStoreListKeysWithUserIDOperation {
    fn list_keys_with_user_id(&self, user_id: &str) -> Result<Vec<Key>, KeyStoreError>;
}

impl<'a, C> KeyStoreListKeysWithUserIDOperation for KeyStoreOperations<'a, C>
where
    C: diesel::Connection,
    <C as diesel::Connection>::Backend: diesel::backend::SupportsDefaultKeyword,
    <C as diesel::Connection>::Backend: 'static,
    String: diesel::deserialize::FromSql<diesel::sql_types::Text, C::Backend>,
{
    fn list_keys_with_user_id(&self, user_id: &str) -> Result<Vec<Key>, KeyStoreError> {
        let keys = keys::table
            .filter(keys::user_id.eq(user_id))
            .load::<KeyModel>(self.conn)
            .map(Some)
            .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
            .map_err(|err| KeyStoreError::OperationError {
                context: "Failed to get keys with user ID".to_string(),
                source: Box::new(err),
            })?
            .ok_or_else(|| {
                KeyStoreError::NotFoundError(
                    "Could not get all keys with user ID from storage".to_string(),
                )
            })?
            .into_iter()
            .map(Key::from)
            .collect();
        Ok(keys)
    }
}
