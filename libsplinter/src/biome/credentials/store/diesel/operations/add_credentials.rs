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

use super::super::super::{NewUserCredentialsModel, UserCredentialsModel};
use super::super::{schema::user_credentials, CredentialsStoreError, UserCredentials};
use super::CredentialsStoreOperations;
use diesel::{dsl::insert_into, prelude::*, result::Error::NotFound};

pub(in super::super) trait CredentialsStoreAddCredentialsOperation {
    fn add_credentials(&self, credentials: UserCredentials) -> Result<(), CredentialsStoreError>;
}

impl<'a, C> CredentialsStoreAddCredentialsOperation for CredentialsStoreOperations<'a, C>
where
    C: diesel::Connection,
    <C as diesel::Connection>::Backend: diesel::backend::SupportsDefaultKeyword,
    <C as diesel::Connection>::Backend: 'static,
    i64: diesel::deserialize::FromSql<diesel::sql_types::BigInt, C::Backend>,
    String: diesel::deserialize::FromSql<diesel::sql_types::Text, C::Backend>,
{
    fn add_credentials(&self, credentials: UserCredentials) -> Result<(), CredentialsStoreError> {
        let duplicate_credentials = user_credentials::table
            .filter(user_credentials::username.eq(&credentials.username))
            .first::<UserCredentialsModel>(self.conn)
            .map(Some)
            .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
            .map_err(|err| CredentialsStoreError::QueryError {
                context: "Failed check for existing username".to_string(),
                source: Box::new(err),
            })?;
        if duplicate_credentials.is_some() {
            return Err(CredentialsStoreError::DuplicateError(format!(
                "Username already in use: {}",
                &credentials.username
            )));
        }

        let new_credentials: NewUserCredentialsModel = credentials.into();

        insert_into(user_credentials::table)
            .values(new_credentials)
            .execute(self.conn)
            .map(|_| ())
            .map_err(|err| CredentialsStoreError::OperationError {
                context: "Failed to add credentials".to_string(),
                source: Box::new(err),
            })?;
        Ok(())
    }
}
