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

use super::super::super::{UserCredentials, UserCredentialsModel};
use super::super::schema::user_credentials;
use super::super::CredentialsStoreError;
use super::CredentialsStoreOperations;
use diesel::{prelude::*, result::Error::NotFound};

pub(in super::super) trait CredentialsStoreFetchCredentialByUsernameOperation {
    fn fetch_credential_by_username(
        &self,
        username: &str,
    ) -> Result<UserCredentials, CredentialsStoreError>;
}

impl<'a, C> CredentialsStoreFetchCredentialByUsernameOperation for CredentialsStoreOperations<'a, C>
where
    C: diesel::Connection,
    <C as diesel::Connection>::Backend: diesel::backend::SupportsDefaultKeyword,
    <C as diesel::Connection>::Backend: 'static,
    i64: diesel::deserialize::FromSql<diesel::sql_types::BigInt, C::Backend>,
    String: diesel::deserialize::FromSql<diesel::sql_types::Text, C::Backend>,
{
    fn fetch_credential_by_username(
        &self,
        username: &str,
    ) -> Result<UserCredentials, CredentialsStoreError> {
        let credentials = user_credentials::table
            .filter(user_credentials::username.eq(username))
            .first::<UserCredentialsModel>(self.conn)
            .map(Some)
            .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
            .map_err(|err| CredentialsStoreError::QueryError {
                context: "Failed to fetch credentials by username".to_string(),
                source: Box::new(err),
            })?
            .ok_or_else(|| {
                CredentialsStoreError::NotFoundError(format!(
                    "Failed to find credentials: {}",
                    username
                ))
            })?;
        Ok(UserCredentials::from(credentials))
    }
}
