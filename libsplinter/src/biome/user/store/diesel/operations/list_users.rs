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
use diesel::{prelude::*, result::Error::NotFound};

pub(in super::super) trait UserStoreListUsersOperation {
    fn list_users(&self) -> Result<Vec<SplinterUser>, UserStoreError>;
}

impl<'a, C> UserStoreListUsersOperation for UserStoreOperations<'a, C>
where
    C: diesel::Connection,
    <C as diesel::Connection>::Backend: diesel::backend::SupportsDefaultKeyword,
    <C as diesel::Connection>::Backend: 'static,
    String: diesel::deserialize::FromSql<diesel::sql_types::Text, C::Backend>,
{
    fn list_users(&self) -> Result<Vec<SplinterUser>, UserStoreError> {
        let users = splinter_user::table
            .select(splinter_user::all_columns)
            .load::<UserModel>(self.conn)
            .map(Some)
            .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
            .map_err(|err| UserStoreError::OperationError {
                context: "Failed to get users".to_string(),
                source: Box::new(err),
            })?
            .ok_or_else(|| {
                UserStoreError::NotFoundError("Could not get all users from storage".to_string())
            })?
            .into_iter()
            .map(SplinterUser::from)
            .collect();
        Ok(users)
    }
}
