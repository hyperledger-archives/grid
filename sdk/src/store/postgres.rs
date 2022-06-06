// Copyright 2018-2021 Cargill Incorporated
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
    connection::TransactionManager,
    pg::PgConnection,
    r2d2::{ConnectionManager, Pool, PooledConnection},
    Connection,
};

#[cfg(feature = "batch-tracking")]
use crate::batch_tracking::store::{
    diesel::{DieselBatchTrackingStore, DieselConnectionBatchTrackingStore},
    BatchTrackingStore,
};
#[cfg(feature = "batch-store")]
use crate::batches::store::{BatchStore, DieselBatchStore, DieselConnectionBatchStore};
use crate::commits::store::{CommitStore, DieselCommitStore, DieselConnectionCommitStore};
use crate::error::InternalError;
#[cfg(feature = "location")]
use crate::location::store::{DieselConnectionLocationStore, DieselLocationStore, LocationStore};
#[cfg(feature = "pike")]
use crate::pike::store::{DieselConnectionPikeStore, DieselPikeStore, PikeStore};
#[cfg(feature = "product")]
use crate::product::store::{DieselConnectionProductStore, DieselProductStore, ProductStore};
#[cfg(feature = "purchase-order")]
use crate::purchase_order::store::{
    DieselConnectionPurchaseOrderStore, DieselPurchaseOrderStore, PurchaseOrderStore,
};
#[cfg(feature = "schema")]
use crate::schema::store::{DieselConnectionSchemaStore, DieselSchemaStore, SchemaStore};
#[cfg(feature = "track-and-trace")]
use crate::track_and_trace::store::{
    DieselConnectionTrackAndTraceStore, DieselTrackAndTraceStore, TrackAndTraceStore,
};

use super::{InContextStoreFactory, StoreFactory, TransactionalStoreFactory};

/// A `StoryFactory` backed by a PostgreSQL database.
#[derive(Clone)]
pub struct PgStoreFactory {
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl PgStoreFactory {
    pub fn new(pool: Pool<ConnectionManager<PgConnection>>) -> Self {
        Self { pool }
    }
}

impl StoreFactory for PgStoreFactory {
    fn get_grid_commit_store<'a>(&'a self) -> Box<dyn CommitStore + 'a> {
        Box::new(DieselCommitStore::new(self.pool.clone()))
    }

    #[cfg(feature = "pike")]
    fn get_grid_pike_store<'a>(&'a self) -> Box<dyn PikeStore + 'a> {
        Box::new(DieselPikeStore::new(self.pool.clone()))
    }

    #[cfg(feature = "location")]
    fn get_grid_location_store<'a>(&'a self) -> Box<dyn LocationStore + 'a> {
        Box::new(DieselLocationStore::new(self.pool.clone()))
    }

    #[cfg(feature = "product")]
    fn get_grid_product_store<'a>(&'a self) -> Box<dyn ProductStore + 'a> {
        Box::new(DieselProductStore::new(self.pool.clone()))
    }

    #[cfg(feature = "schema")]
    fn get_grid_schema_store<'a>(&'a self) -> Box<dyn SchemaStore + 'a> {
        Box::new(DieselSchemaStore::new(self.pool.clone()))
    }

    #[cfg(feature = "track-and-trace")]
    fn get_grid_track_and_trace_store<'a>(&'a self) -> Box<dyn TrackAndTraceStore + 'a> {
        Box::new(DieselTrackAndTraceStore::new(self.pool.clone()))
    }

    #[cfg(feature = "batch-store")]
    fn get_batch_store<'a>(&'a self) -> Box<dyn BatchStore + 'a> {
        Box::new(DieselBatchStore::new(self.pool.clone()))
    }

    #[cfg(feature = "purchase-order")]
    fn get_grid_purchase_order_store<'a>(&'a self) -> Box<dyn PurchaseOrderStore + 'a> {
        Box::new(DieselPurchaseOrderStore::new(self.pool.clone()))
    }

    #[cfg(feature = "batch-tracking")]
    fn get_batch_tracking_store<'a>(&'a self) -> Box<dyn BatchTrackingStore + 'a> {
        Box::new(DieselBatchTrackingStore::new(self.pool.clone()))
    }
}

impl TransactionalStoreFactory for PgStoreFactory {
    fn begin_transaction<'a>(&self) -> Result<Box<dyn InContextStoreFactory<'a>>, InternalError> {
        let conn = self
            .pool
            .get()
            .map_err(|err| InternalError::from_source(Box::new(err)))?;

        let store_factory = InContextPgStoreFactory::new(conn);
        store_factory
            .conn
            .transaction_manager()
            .begin_transaction(&store_factory.conn)
            .map_err(|err| InternalError::from_source(Box::new(err)))?;
        Ok(Box::new(store_factory))
    }

    fn clone_box(&self) -> Box<dyn TransactionalStoreFactory> {
        Box::new(self.clone())
    }
}

pub struct InContextPgStoreFactory {
    conn: PooledConnection<ConnectionManager<PgConnection>>,
}

impl InContextPgStoreFactory {
    fn new(conn: PooledConnection<ConnectionManager<PgConnection>>) -> Self {
        Self { conn }
    }
}

impl StoreFactory for InContextPgStoreFactory {
    fn get_grid_commit_store<'a>(&'a self) -> Box<dyn CommitStore + 'a> {
        Box::new(DieselConnectionCommitStore::new(&*self.conn))
    }

    #[cfg(feature = "pike")]
    fn get_grid_pike_store<'a>(&'a self) -> Box<dyn PikeStore + 'a> {
        Box::new(DieselConnectionPikeStore::new(&*self.conn))
    }

    #[cfg(feature = "location")]
    fn get_grid_location_store<'a>(&'a self) -> Box<dyn LocationStore + 'a> {
        Box::new(DieselConnectionLocationStore::new(&*self.conn))
    }

    #[cfg(feature = "product")]
    fn get_grid_product_store<'a>(&'a self) -> Box<dyn ProductStore + 'a> {
        Box::new(DieselConnectionProductStore::new(&*self.conn))
    }

    #[cfg(feature = "schema")]
    fn get_grid_schema_store<'a>(&'a self) -> Box<dyn SchemaStore + 'a> {
        Box::new(DieselConnectionSchemaStore::new(&*self.conn))
    }

    #[cfg(feature = "track-and-trace")]
    fn get_grid_track_and_trace_store<'a>(&'a self) -> Box<dyn TrackAndTraceStore + 'a> {
        Box::new(DieselConnectionTrackAndTraceStore::new(&*self.conn))
    }

    #[cfg(feature = "batch-store")]
    fn get_batch_store<'a>(&'a self) -> Box<dyn BatchStore + 'a> {
        Box::new(DieselConnectionBatchStore::new(&*self.conn))
    }

    #[cfg(feature = "purchase-order")]
    fn get_grid_purchase_order_store<'a>(&'a self) -> Box<dyn PurchaseOrderStore + 'a> {
        Box::new(DieselConnectionPurchaseOrderStore::new(&*self.conn))
    }

    #[cfg(feature = "batch-tracking")]
    fn get_batch_tracking_store<'a>(&'a self) -> Box<dyn BatchTrackingStore + 'a> {
        Box::new(DieselConnectionBatchTrackingStore::new(&*self.conn))
    }
}

impl<'a> InContextStoreFactory<'a> for InContextPgStoreFactory {
    fn commit(&self) -> Result<(), InternalError> {
        let transaction_manager = self.conn.transaction_manager();
        transaction_manager
            .commit_transaction(&self.conn)
            .map_err(|err| InternalError::from_source(Box::new(err)))?;

        Ok(())
    }

    fn rollback(&self) -> Result<(), InternalError> {
        let transaction_manager = self.conn.transaction_manager();
        transaction_manager
            .rollback_transaction(&self.conn)
            .map_err(|err| InternalError::from_source(Box::new(err)))?;

        Ok(())
    }
}
