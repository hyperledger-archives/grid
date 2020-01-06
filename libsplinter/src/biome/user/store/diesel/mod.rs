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

pub(in crate::biome) mod models;
mod operations;
#[cfg(feature = "postgres")]
pub(in super::super) mod postgres;
mod schema;

use super::{SplinterUser, UserStore, UserStoreError};
use crate::database::ConnectionPool;
use operations::add_user::UserStoreAddUserOperation as _;
use operations::UserStoreOperations;

/// Manages creating, updating and fetching SplinterUser from the databae
pub struct SplinterUserStore {
    connection_pool: ConnectionPool,
}

impl SplinterUserStore {
    /// Creates a new SplinterUserStore
    ///
    /// # Arguments
    ///
    ///  * `connection_pool`: connection pool to the database
    pub fn new(connection_pool: ConnectionPool) -> SplinterUserStore {
        SplinterUserStore { connection_pool }
    }
}

impl UserStore<SplinterUser> for SplinterUserStore {
    fn add_user(&self, user: SplinterUser) -> Result<(), UserStoreError> {
        UserStoreOperations::new(&*self.connection_pool.get()?).add_user(user.into())
    }

    fn update_user(&self, _updated_user: SplinterUser) -> Result<(), UserStoreError> {
        unimplemented!()
    }

    fn remove_user(&self, _id: &str) -> Result<SplinterUser, UserStoreError> {
        unimplemented!()
    }

    fn fetch_user(&self, _id: &str) -> Result<SplinterUser, UserStoreError> {
        unimplemented!()
    }

    fn list_users(&self, _id: &str) -> Result<Vec<SplinterUser>, UserStoreError> {
        unimplemented!()
    }

    fn is_user(&self, _id: &str) -> Result<bool, UserStoreError> {
        unimplemented!()
    }
}
