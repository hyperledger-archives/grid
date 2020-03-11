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
use crate::biome::datastore::schema::keys;
use crate::biome::key_management::store::KeyStoreError;

use diesel::prelude::*;

pub(in super::super) trait KeyStoreUpdateKeyOperation {
    fn update_key(
        &self,
        user_id: &str,
        public_key: &str,
        display_name: &str,
    ) -> Result<(), KeyStoreError>;
}

impl<'a, C> KeyStoreUpdateKeyOperation for KeyStoreOperations<'a, C>
where
    C: diesel::Connection,
    <C as diesel::Connection>::Backend: diesel::backend::SupportsDefaultKeyword,
    <C as diesel::Connection>::Backend: 'static,
    String: diesel::deserialize::FromSql<diesel::sql_types::Text, C::Backend>,
{
    fn update_key(
        &self,
        user_id: &str,
        public_key: &str,
        display_name: &str,
    ) -> Result<(), KeyStoreError> {
        match diesel::update(keys::table.find((public_key, user_id)))
            .set((keys::display_name.eq(display_name),))
            .execute(self.conn)
            .map_err(|err| KeyStoreError::OperationError {
                context: "Failed to update key".to_string(),
                source: Box::new(err),
            })? {
            0 => Err(KeyStoreError::NotFoundError(format!(
                "Key with public key {} and user ID {} not found",
                public_key, user_id
            ))),
            _ => Ok(()),
        }
    }
}
