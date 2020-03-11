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

use diesel::{
    dsl::insert_into,
    prelude::*,
    result::{DatabaseErrorKind, Error as QueryError},
};

pub(in super::super) trait KeyStoreInsertKeyOperation {
    fn insert_key(&self, key: Key) -> Result<(), KeyStoreError>;
}

impl<'a, C> KeyStoreInsertKeyOperation for KeyStoreOperations<'a, C>
where
    C: diesel::Connection,
    <C as diesel::Connection>::Backend: diesel::backend::SupportsDefaultKeyword,
    <C as diesel::Connection>::Backend: 'static,
    String: diesel::deserialize::FromSql<diesel::sql_types::Text, C::Backend>,
{
    fn insert_key(&self, key: Key) -> Result<(), KeyStoreError> {
        let key_model: KeyModel = key.into();
        let public_key = key_model.public_key.clone();
        let user_id = key_model.user_id.clone();
        insert_into(keys::table)
            .values(vec![key_model])
            .execute(self.conn)
            .map_err(|err| {
                if let QueryError::DatabaseError(db_err, _) = err {
                    match db_err {
                        DatabaseErrorKind::UniqueViolation => {
                            return KeyStoreError::DuplicateKeyError(format!(
                                "Public key {} for user {} is already in database",
                                public_key, user_id
                            ));
                        }
                        DatabaseErrorKind::ForeignKeyViolation => {
                            return KeyStoreError::UserDoesNotExistError(format!(
                                "User with ID {} does not exist in database",
                                user_id
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
}
