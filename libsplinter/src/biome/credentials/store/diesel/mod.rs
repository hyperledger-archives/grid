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

pub(in crate::biome) mod models;
mod operations;
#[cfg(feature = "postgres")]
pub(in super::super) mod postgres;
mod schema;

use super::{CredentialsStore, CredentialsStoreError, UserCredentials, UsernameId};
use crate::database::ConnectionPool;
use models::UserCredentialsModel;
use operations::get_usernames::CredentialsStoreGetUsernamesOperation as _;
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
