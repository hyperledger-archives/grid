// Copyright 2019 Cargill Incorporated
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

use super::database::helpers::{fetch_credential_by_username, insert_credential};
use super::{CredentialsStore, CredentialsStoreError, UserCredentials};

use crate::database::ConnectionPool;

/// Manages creating, updating and fetching UserCredentials from the databae
pub struct SplinterCredentialsStore {
    connection_pool: ConnectionPool,
}

impl SplinterCredentialsStore {
    /// Creates a new SplinterCredentialsStore
    ///
    /// # Arguments
    ///
    ///  * `connection_pool`: connection pool to the database
    pub fn new(connection_pool: ConnectionPool) -> SplinterCredentialsStore {
        SplinterCredentialsStore { connection_pool }
    }
}

impl CredentialsStore<UserCredentials> for SplinterCredentialsStore {
    fn add_credentials(&self, credentials: UserCredentials) -> Result<(), CredentialsStoreError> {
        let duplicate_credentials =
            fetch_credential_by_username(&*self.connection_pool.get()?, &credentials.username)
                .map_err(|err| CredentialsStoreError::QueryError {
                    context: "Failed check for existing username".to_string(),
                    source: Box::new(err),
                })?;
        if duplicate_credentials.is_some() {
            return Err(CredentialsStoreError::DuplicateError(format!(
                "Username already in use: {}",
                credentials.username
            )));
        }
        insert_credential(&*self.connection_pool.get()?, credentials.into()).map_err(|err| {
            CredentialsStoreError::OperationError {
                context: "Failed to add credentials".to_string(),
                source: Box::new(err),
            }
        })?;
        Ok(())
    }

    fn update_credentials(
        &self,
        _user_id: &str,
        _username: &str,
        _password: &str,
    ) -> Result<(), CredentialsStoreError> {
        unimplemented!()
    }

    fn remove_credentials(&self, _user_id: &str) -> Result<UserCredentials, CredentialsStoreError> {
        unimplemented!()
    }

    fn fetch_credential_by_user_id(
        &self,
        _user_id: &str,
    ) -> Result<UserCredentials, CredentialsStoreError> {
        unimplemented!()
    }

    fn fetch_credential_by_username(
        &self,
        username: &str,
    ) -> Result<UserCredentials, CredentialsStoreError> {
        let credentials = fetch_credential_by_username(&*self.connection_pool.get()?, &username)
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
