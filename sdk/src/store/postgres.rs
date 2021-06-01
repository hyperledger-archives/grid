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
    pg::PgConnection,
    r2d2::{ConnectionManager, Pool},
};

#[cfg(feature = "batch-store")]
use crate::batches::store::{BatchStore, DieselBatchStore};
use crate::commits::store::{CommitStore, DieselCommitStore};
#[cfg(any(feature = "location-store-postgres", feature = "location-store-sqlite"))]
use crate::location::store::{DieselLocationStore, LocationStore};
#[cfg(feature = "pike")]
use crate::pike::store::{DieselPikeStore, PikeStore};
#[cfg(any(feature = "product-store-postgres", feature = "product-store-sqlite"))]
use crate::product::store::{DieselProductStore, ProductStore};
#[cfg(any(feature = "schema-store-postgres", feature = "schema-store-sqlite"))]
use crate::schema::store::{DieselSchemaStore, SchemaStore};
#[cfg(feature = "track-and-trace")]
use crate::track_and_trace::store::{DieselTrackAndTraceStore, TrackAndTraceStore};

use super::StoreFactory;

/// A `StoryFactory` backed by a PostgreSQL database.
pub struct PgStoreFactory {
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl PgStoreFactory {
    pub fn new(pool: Pool<ConnectionManager<PgConnection>>) -> Self {
        Self { pool }
    }
}

impl StoreFactory for PgStoreFactory {
    fn get_grid_commit_store(&self) -> Box<dyn CommitStore> {
        Box::new(DieselCommitStore::new(self.pool.clone()))
    }

    #[cfg(feature = "pike")]
    fn get_grid_pike_store(&self) -> Box<dyn PikeStore> {
        Box::new(DieselPikeStore::new(self.pool.clone()))
    }

    #[cfg(all(feature = "diesel", feature = "location-store-postgres"))]
    fn get_grid_location_store(&self) -> Box<dyn LocationStore> {
        Box::new(DieselLocationStore::new(self.pool.clone()))
    }

    #[cfg(all(feature = "diesel", feature = "product-store-postgres"))]
    fn get_grid_product_store(&self) -> Box<dyn ProductStore> {
        Box::new(DieselProductStore::new(self.pool.clone()))
    }

    #[cfg(all(feature = "diesel", feature = "schema-store-postgres"))]
    fn get_grid_schema_store(&self) -> Box<dyn SchemaStore> {
        Box::new(DieselSchemaStore::new(self.pool.clone()))
    }

    #[cfg(feature = "track-and-trace")]
    fn get_grid_track_and_trace_store(&self) -> Box<dyn TrackAndTraceStore> {
        Box::new(DieselTrackAndTraceStore::new(self.pool.clone()))
    }

    #[cfg(feature = "batch-store")]
    fn get_batch_store(&self) -> Box<dyn BatchStore> {
        Box::new(DieselBatchStore::new(self.pool.clone()))
    }
}
