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

use super::super::super::UserCredentialsBuilder;
use super::super::CredentialsStoreError;
use super::CredentialsStoreOperations;
use crate::biome::datastore::models::UserCredentialsModel;
use crate::biome::datastore::schema::user_credentials;
use diesel::{dsl::update, prelude::*, result::Error::NotFound};

pub(in super::super) trait CredentialsStoreUpdateCredentialsOperation {
    fn update_credentials(
        &self,
        user_id: &str,
        username: &str,
        password: &str,
    ) -> Result<(), CredentialsStoreError>;
}

impl<'a, C> CredentialsStoreUpdateCredentialsOperation for CredentialsStoreOperations<'a, C>
where
    C: diesel::Connection,
    <C as diesel::Connection>::Backend: diesel::backend::SupportsDefaultKeyword,
    <C as diesel::Connection>::Backend: 'static,
    i64: diesel::deserialize::FromSql<diesel::sql_types::BigInt, C::Backend>,
    String: diesel::deserialize::FromSql<diesel::sql_types::Text, C::Backend>,
{
    fn update_credentials(
        &self,
        user_id: &str,
        username: &str,
        password: &str,
    ) -> Result<(), CredentialsStoreError> {
        let credentials_builder: UserCredentialsBuilder = Default::default();
        let credentials = credentials_builder
            .with_user_id(user_id)
            .with_username(username)
            .with_password(password)
            .build()
            .map_err(|err| CredentialsStoreError::OperationError {
                context: "Failed to build updated credentials".to_string(),
                source: Box::new(err),
            })?;
        let credential_exists = user_credentials::table
            .filter(user_credentials::user_id.eq(&credentials.user_id))
            .first::<UserCredentialsModel>(self.conn)
            .map(Some)
            .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
            .map_err(|err| CredentialsStoreError::QueryError {
                context: "Failed check for existing user id".to_string(),
                source: Box::new(err),
            })?;
        if credential_exists.is_none() {
            return Err(CredentialsStoreError::NotFoundError(format!(
                "Credentials not found for user id: {}",
                &credentials.user_id
            )));
        }
        update(user_credentials::table.filter(user_credentials::user_id.eq(&credentials.user_id)))
            .set((
                user_credentials::user_id.eq(&credentials.user_id),
                user_credentials::username.eq(&credentials.username),
                user_credentials::password.eq(&credentials.password),
            ))
            .execute(self.conn)
            .map(|_| ())
            .map_err(|err| CredentialsStoreError::OperationError {
                context: "Failed to update credentials".to_string(),
                source: Box::new(err),
            })?;
        Ok(())
    }
}
