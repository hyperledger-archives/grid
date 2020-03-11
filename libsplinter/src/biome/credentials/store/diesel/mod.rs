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

mod operations;

use super::{CredentialsStore, CredentialsStoreError, UserCredentials, UsernameId};
use crate::biome::datastore::models::UserCredentialsModel;
use crate::database::ConnectionPool;
use operations::add_credentials::CredentialsStoreAddCredentialsOperation as _;
use operations::fetch_credential_by_id::CredentialsStoreFetchCredentialByIdOperation as _;
use operations::fetch_credential_by_username::CredentialsStoreFetchCredentialByUsernameOperation as _;
use operations::fetch_username::CredentialsStoreFetchUsernameOperation as _;
use operations::get_usernames::CredentialsStoreGetUsernamesOperation as _;
use operations::remove_credentials::CredentialsStoreRemoveCredentialsOperation as _;
use operations::update_credentials::CredentialsStoreUpdateCredentialsOperation as _;
use operations::CredentialsStoreOperations;

/// Manages creating, updating and fetching SplinterCredentials from the database
pub struct SplinterCredentialsStore {
    connection_pool: ConnectionPool,
}

impl SplinterCredentialsStore {
    /// Creates a new SplinterUserStore
    ///
    /// # Arguments
    ///
    ///  * `connection_pool`: connection pool to the database
    // Allow dead code if diesel feature is not enabled
    #[allow(dead_code)]
    pub fn new(connection_pool: ConnectionPool) -> SplinterCredentialsStore {
        SplinterCredentialsStore { connection_pool }
    }
}

impl CredentialsStore<UserCredentials> for SplinterCredentialsStore {
    fn add_credentials(&self, credentials: UserCredentials) -> Result<(), CredentialsStoreError> {
        CredentialsStoreOperations::new(&*self.connection_pool.get()?).add_credentials(credentials)
    }

    fn update_credentials(
        &self,
        user_id: &str,
        username: &str,
        password: &str,
    ) -> Result<(), CredentialsStoreError> {
        CredentialsStoreOperations::new(&*self.connection_pool.get()?)
            .update_credentials(user_id, username, password)
    }

    fn remove_credentials(&self, user_id: &str) -> Result<(), CredentialsStoreError> {
        CredentialsStoreOperations::new(&*self.connection_pool.get()?).remove_credentials(user_id)
    }

    fn fetch_credential_by_user_id(
        &self,
        user_id: &str,
    ) -> Result<UserCredentials, CredentialsStoreError> {
        CredentialsStoreOperations::new(&*self.connection_pool.get()?)
            .fetch_credential_by_id(user_id)
    }

    fn fetch_credential_by_username(
        &self,
        username: &str,
    ) -> Result<UserCredentials, CredentialsStoreError> {
        CredentialsStoreOperations::new(&*self.connection_pool.get()?)
            .fetch_credential_by_username(username)
    }

    fn fetch_username_by_id(&self, user_id: &str) -> Result<UsernameId, CredentialsStoreError> {
        CredentialsStoreOperations::new(&*self.connection_pool.get()?).fetch_username_by_id(user_id)
    }

    fn get_usernames(&self) -> Result<Vec<UsernameId>, CredentialsStoreError> {
        CredentialsStoreOperations::new(&*self.connection_pool.get()?).get_usernames()
    }
}

impl From<UserCredentialsModel> for UsernameId {
    fn from(user_credentials: UserCredentialsModel) -> Self {
        Self {
            user_id: user_credentials.user_id,
            username: user_credentials.username,
        }
    }
}

impl From<UserCredentialsModel> for UserCredentials {
    fn from(user_credentials: UserCredentialsModel) -> Self {
        Self {
            user_id: user_credentials.user_id,
            username: user_credentials.username,
            password: user_credentials.password,
        }
    }
}
