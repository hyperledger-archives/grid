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

use diesel::{
    r2d2::{ConnectionManager, Pool},
    sqlite::SqliteConnection,
};

use super::StoreFactory;

/// A `StoryFactory` backed by a SQLite database.
pub struct SqliteStoreFactory {
    pool: Pool<ConnectionManager<SqliteConnection>>,
}

impl SqliteStoreFactory {
    pub fn new(pool: Pool<ConnectionManager<SqliteConnection>>) -> Self {
        Self { pool }
    }
}

impl StoreFactory for SqliteStoreFactory {
    fn get_grid_commit_store(&self) -> Box<dyn crate::grid_db::CommitStore> {
        Box::new(crate::grid_db::DieselCommitStore::new(self.pool.clone()))
    }

    fn get_grid_organization_store(&self) -> Box<dyn crate::grid_db::OrganizationStore> {
        Box::new(crate::grid_db::DieselOrganizationStore::new(
            self.pool.clone(),
        ))
    }

    fn get_grid_location_store(&self) -> Box<dyn crate::grid_db::LocationStore> {
        Box::new(crate::grid_db::DieselLocationStore::new(self.pool.clone()))
    }

    fn get_grid_product_store(&self) -> Box<dyn crate::grid_db::ProductStore> {
        Box::new(crate::grid_db::DieselProductStore::new(self.pool.clone()))
    }
}
