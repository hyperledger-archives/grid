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

pub mod models;
mod operations;
pub(in crate) mod schema;

use diesel::r2d2::{ConnectionManager, Pool};

use super::{
    PurchaseOrder, PurchaseOrderAlternateId, PurchaseOrderAlternateIdList, PurchaseOrderList,
    PurchaseOrderStore, PurchaseOrderStoreError, PurchaseOrderVersion,
    PurchaseOrderVersionRevision,
};

use models::{make_purchase_order_version_revisions, make_purchase_order_versions};

use crate::error::ResourceTemporarilyUnavailableError;

use operations::add_alternate_id::PurchaseOrderStoreAddAlternateIdOperation as _;
use operations::add_purchase_order::PurchaseOrderStoreAddPurchaseOrderOperation as _;
use operations::get_purchase_order::PurchaseOrderStoreGetPurchaseOrderOperation as _;
use operations::list_alternate_ids_for_purchase_order::PurchaseOrderStoreListAlternateIdsForPurchaseOrderOperation as _;
use operations::list_purchase_orders::PurchaseOrderStoreListPurchaseOrdersOperation as _;
use operations::PurchaseOrderStoreOperations;

/// Manages creating agents in the database
#[derive(Clone)]
pub struct DieselPurchaseOrderStore<C: diesel::Connection + 'static> {
    connection_pool: Pool<ConnectionManager<C>>,
}

impl<C: diesel::Connection> DieselPurchaseOrderStore<C> {
    /// Creates a new DieselPurchaseOrderStore
    ///
    /// # Arguments
    ///
    ///  * `connection_pool`: connection pool to the database
    pub fn new(connection_pool: Pool<ConnectionManager<C>>) -> Self {
        DieselPurchaseOrderStore { connection_pool }
    }
}

#[cfg(feature = "postgres")]
impl PurchaseOrderStore for DieselPurchaseOrderStore<diesel::pg::PgConnection> {
    fn add_purchase_order(&self, order: PurchaseOrder) -> Result<(), PurchaseOrderStoreError> {
        PurchaseOrderStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            PurchaseOrderStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .add_purchase_order(
            order.clone().into(),
            make_purchase_order_versions(&order),
            make_purchase_order_version_revisions(&order),
        )
    }

    fn list_purchase_orders(
        &self,
        org_id: Option<String>,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<PurchaseOrderList, PurchaseOrderStoreError> {
        PurchaseOrderStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            PurchaseOrderStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .list_purchase_orders(org_id, service_id, offset, limit)
    }

    fn get_purchase_order(
        &self,
        uuid: &str,
        service_id: Option<&str>,
    ) -> Result<Option<PurchaseOrder>, PurchaseOrderStoreError> {
        PurchaseOrderStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            PurchaseOrderStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .get_purchase_order(uuid, service_id)
    }

    fn add_alternate_id(
        &self,
        alternate_id: PurchaseOrderAlternateId,
    ) -> Result<(), PurchaseOrderStoreError> {
        PurchaseOrderStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            PurchaseOrderStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .add_alternate_id(alternate_id.into())
    }

    fn list_alternate_ids_for_purchase_order(
        &self,
        purchase_order_uuid: &str,
        org_id: &str,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<PurchaseOrderAlternateIdList, PurchaseOrderStoreError> {
        PurchaseOrderStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            PurchaseOrderStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .list_alternate_ids_for_purchase_order(
            purchase_order_uuid,
            org_id,
            service_id,
            offset,
            limit,
        )
    }
}

#[cfg(feature = "sqlite")]
impl PurchaseOrderStore for DieselPurchaseOrderStore<diesel::sqlite::SqliteConnection> {
    fn add_purchase_order(&self, order: PurchaseOrder) -> Result<(), PurchaseOrderStoreError> {
        PurchaseOrderStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            PurchaseOrderStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .add_purchase_order(
            order.clone().into(),
            make_purchase_order_versions(&order),
            make_purchase_order_version_revisions(&order),
        )
    }

    fn list_purchase_orders(
        &self,
        org_id: Option<String>,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<PurchaseOrderList, PurchaseOrderStoreError> {
        PurchaseOrderStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            PurchaseOrderStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .list_purchase_orders(org_id, service_id, offset, limit)
    }

    fn get_purchase_order(
        &self,
        uuid: &str,
        service_id: Option<&str>,
    ) -> Result<Option<PurchaseOrder>, PurchaseOrderStoreError> {
        PurchaseOrderStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            PurchaseOrderStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .get_purchase_order(uuid, service_id)
    }

    fn add_alternate_id(
        &self,
        alternate_id: PurchaseOrderAlternateId,
    ) -> Result<(), PurchaseOrderStoreError> {
        PurchaseOrderStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            PurchaseOrderStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .add_alternate_id(alternate_id.into())
    }

    fn list_alternate_ids_for_purchase_order(
        &self,
        purchase_order_uuid: &str,
        org_id: &str,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<PurchaseOrderAlternateIdList, PurchaseOrderStoreError> {
        PurchaseOrderStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            PurchaseOrderStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .list_alternate_ids_for_purchase_order(
            purchase_order_uuid,
            org_id,
            service_id,
            offset,
            limit,
        )
    }
}
