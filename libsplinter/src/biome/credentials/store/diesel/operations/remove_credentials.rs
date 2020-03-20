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

use super::super::super::UserCredentialsModel;
use super::super::{schema::user_credentials, CredentialsStoreError};
use super::CredentialsStoreOperations;
use diesel::{dsl::delete, prelude::*, result::Error::NotFound};

pub(in super::super) trait CredentialsStoreRemoveCredentialsOperation {
    fn remove_credentials(&self, user_id: &str) -> Result<(), CredentialsStoreError>;
}

impl<'a, C> CredentialsStoreRemoveCredentialsOperation for CredentialsStoreOperations<'a, C>
where
    C: diesel::Connection,
    <C as diesel::Connection>::Backend: diesel::backend::SupportsDefaultKeyword,
    <C as diesel::Connection>::Backend: 'static,
    i64: diesel::deserialize::FromSql<diesel::sql_types::BigInt, C::Backend>,
    String: diesel::deserialize::FromSql<diesel::sql_types::Text, C::Backend>,
{
    fn remove_credentials(&self, user_id: &str) -> Result<(), CredentialsStoreError> {
        let credentials = user_credentials::table
            .filter(user_credentials::user_id.eq(user_id))
            .first::<UserCredentialsModel>(self.conn)
            .map(Some)
            .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
            .map_err(|err| CredentialsStoreError::QueryError {
                context: "Failed check for existing user id".to_string(),
                source: Box::new(err),
            })?;
        if credentials.is_none() {
            return Err(CredentialsStoreError::NotFoundError(format!(
                "Credentials not found for user id: {}",
                user_id
            )));
        }

        delete(user_credentials::table.filter(user_credentials::user_id.eq(user_id)))
            .execute(self.conn)
            .map(|_| ())
            .map_err(|err| CredentialsStoreError::OperationError {
                context: "Failed to delete credentials".to_string(),
                source: Box::new(err),
            })?;
        Ok(())
    }
}
