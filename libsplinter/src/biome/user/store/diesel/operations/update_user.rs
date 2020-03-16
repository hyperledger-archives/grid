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

use super::super::{SplinterUser, UserStoreError};
use super::UserStoreOperations;
use crate::biome::datastore::models::UserModel;
use crate::biome::datastore::schema::splinter_user;
use diesel::{dsl::update, prelude::*, result::Error::NotFound};

pub(in super::super) trait UserStoreUpdateUserOperation {
    fn update_user(&self, updated_user: SplinterUser) -> Result<(), UserStoreError>;
}

impl<'a, C> UserStoreUpdateUserOperation for UserStoreOperations<'a, C>
where
    C: diesel::Connection,
    <C as diesel::Connection>::Backend: diesel::backend::SupportsDefaultKeyword,
    <C as diesel::Connection>::Backend: 'static,
    String: diesel::deserialize::FromSql<diesel::sql_types::Text, C::Backend>,
{
    fn update_user(&self, updated_user: SplinterUser) -> Result<(), UserStoreError> {
        let id = updated_user.id();
        let user = splinter_user::table
            .find(&id)
            .first::<UserModel>(self.conn)
            .map(Some)
            .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
            .map_err(|err| UserStoreError::OperationError {
                context: "Failed to fetch user".to_string(),
                source: Box::new(err),
            })?;

        if user.is_none() {
            return Err(UserStoreError::NotFoundError(format!(
                "Failed to find user: {}",
                &id
            )));
        }

        update(splinter_user::table.filter(splinter_user::id.eq(&id)))
            .set((splinter_user::id.eq(&id),))
            .execute(self.conn)
            .map(|_| ())
            .map_err(|err| UserStoreError::OperationError {
                context: "Failed to update user".to_string(),
                source: Box::new(err),
            })?;
        Ok(())
    }
}
