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

#[cfg(feature = "diesel")]
pub mod diesel;
mod error;

use crate::paging::Paging;

pub use error::PurchaseOrderStoreError;

/// Represents a list of Grid Purchase Orders
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct PurchaseOrderList {
    pub data: Vec<PurchaseOrder>,
    pub paging: Paging,
}

impl PurchaseOrderList {
    pub fn new(data: Vec<PurchaseOrder>, paging: Paging) -> Self {
        Self { data, paging }
    }
}

/// Represents a Grid Purchase Order
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct PurchaseOrder {
    pub uuid: String,
    pub org_id: String,
    pub workflow_status: String,
    pub is_closed: bool,
    pub accepted_version_id: String,
    pub versions: Vec<PurchaseOrderVersion>,
    pub created_at: i64,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

/// Represents a Grid Purchase Order Version
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct PurchaseOrderVersion {
    pub version_id: String,
    pub is_draft: bool,
    pub current_revision_id: String,
    pub revisions: Vec<PurchaseOrderVersionRevision>,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

/// Represents a Grid Purchase Order Version Revision
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct PurchaseOrderVersionRevision {
    pub revision_id: String,
    pub order_xml_v3_4: String,
    pub submitter: String,
    pub created_at: i64,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

/// Represents a list of Grid Purchase Order Alternate IDs
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct PurchaseOrderAlternateIdList {
    pub alternate_ids: Vec<PurchaseOrderAlternateId>,
}

/// Represents a Grid Purchase Order Alternate ID
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct PurchaseOrderAlternateId {
    pub purchase_order_uuid: String,
    pub org_id: String,
    pub id_type: String,
    pub id: String,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

pub trait PurchaseOrderStore: Send + Sync {
    /// Adds a purchase order to the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `order` - The purchase order to be added
    fn add_purchase_order(&self, order: PurchaseOrder) -> Result<(), PurchaseOrderStoreError>;

    /// Lists purchase orders from the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `org_id` - The organization to fetch for
    ///  * `service_id` - The service id
    ///  * `offset` - The index of the first in storage to retrieve
    ///  * `limit` - The number of items to retrieve from the offset
    fn list_purchase_orders(
        &self,
        org_id: Option<String>,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<PurchaseOrderList, PurchaseOrderStoreError>;

    /// Fetches a purchase order from the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `uuid`   - The uuid of the purchase order
    ///  * `service_id` - The service id
    fn get_purchase_order(
        &self,
        uuid: &str,
        service_id: Option<&str>,
    ) -> Result<Option<PurchaseOrder>, PurchaseOrderStoreError>;

    /// Adds an alternate id to the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `alternate_id` - The alternate_id to be added
    fn add_alternate_id(
        &self,
        alternate_id: PurchaseOrderAlternateId,
    ) -> Result<(), PurchaseOrderStoreError>;

    /// Lists alternate IDs for a purchase order from the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `purchase_order_uuid` - The purchase order to fetch alternate IDs for
    ///  * `org_id` - The organization to fetch for
    ///  * `service_id` - The service id
    ///  * `offset` - The index of the first in storage to retrieve
    ///  * `limit` - The number of items to retrieve from the offset
    fn list_alternate_ids_for_purchase_order(
        &self,
        purchase_order_uuid: &str,
        org_id: &str,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<PurchaseOrderAlternateIdList, PurchaseOrderStoreError>;
}

impl<PS> PurchaseOrderStore for Box<PS>
where
    PS: PurchaseOrderStore + ?Sized,
{
    fn add_purchase_order(&self, order: PurchaseOrder) -> Result<(), PurchaseOrderStoreError> {
        (**self).add_purchase_order(order)
    }

    fn list_purchase_orders(
        &self,
        org_id: Option<String>,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<PurchaseOrderList, PurchaseOrderStoreError> {
        (**self).list_purchase_orders(org_id, service_id, offset, limit)
    }

    fn get_purchase_order(
        &self,
        uuid: &str,
        service_id: Option<&str>,
    ) -> Result<Option<PurchaseOrder>, PurchaseOrderStoreError> {
        (**self).get_purchase_order(uuid, service_id)
    }

    fn add_alternate_id(
        &self,
        alternate_id: PurchaseOrderAlternateId,
    ) -> Result<(), PurchaseOrderStoreError> {
        (**self).add_alternate_id(alternate_id)
    }

    fn list_alternate_ids_for_purchase_order(
        &self,
        purchase_order_uuid: &str,
        org_id: &str,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<PurchaseOrderAlternateIdList, PurchaseOrderStoreError> {
        (**self).list_alternate_ids_for_purchase_order(
            purchase_order_uuid,
            org_id,
            service_id,
            offset,
            limit,
        )
    }
}
