// Copyright 2021 Cargill Incorporated
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

use std::time::SystemTime;

use crate::error::ClientError;

use super::Client;

#[cfg(feature = "client-reqwest")]
pub mod reqwest;

pub struct PurchaseOrder {
    pub buyer_org_id: String,
    pub seller_org_id: String,
    pub purchase_order_uid: String,
    pub workflow_status: String,
    pub is_closed: bool,
    pub accepted_version_id: Option<String>,
    pub versions: Vec<String>,
    pub created_at: SystemTime,
    pub workflow_type: String,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
}

pub struct PurchaseOrderVersion {
    pub version_id: String,
    pub workflow_status: String,
    pub is_draft: bool,
    pub current_revision_id: u64,
    pub revisions: Vec<PurchaseOrderRevision>,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
}

pub struct PurchaseOrderRevision {
    pub revision_id: u64,
    pub order_xml_v3_4: String,
    pub submitter: String,
    pub created_at: u64,
}

pub struct PurchaseOrderFilter {
    pub org_id: Option<String>,
    pub is_closed: Option<bool>,
    pub is_accepted: Option<bool>,
}

pub trait PurchaseOrderClient: Client {
    /// Retrieves the purchase order with the specified `id`.
    ///
    /// # Arguments
    ///
    /// * `id` - The id of the `PurchaseOrder` to be retrieved
    /// * `service_id` - optional service id if running splinter
    fn get_purchase_order(
        &self,
        id: String,
        service_id: Option<&str>,
    ) -> Result<Option<PurchaseOrder>, ClientError>;

    /// Retrieves the purchase order version with the given `version_id` of the purchase
    /// order with the given `id`
    ///
    /// # Arguments
    ///
    /// * `id` - The id of the `PurchaseOrder` containing the `PurchaseOrderVersion` to be retrieved
    /// * `version_id` - The version id of the `PurchaseOrderVersion` to be retrieved
    /// * `service_id` - optional service id if running splinter
    fn get_purchase_order_version(
        &self,
        id: String,
        version_id: String,
        service_id: Option<&str>,
    ) -> Result<Option<PurchaseOrderVersion>, ClientError>;

    /// Retrieves the purchase order revision with the given `revision_id` of
    /// the purchase order version with the given `version_id` of the purchase order with the given `id`
    ///
    /// # Arguments
    ///
    /// * `id` - The id of the `PurchaseOrder` containing the `PurchaseOrderRevision` to be retrieved
    /// * `version_id` - The version id of the `PurchaseOrderVersion` containing the
    ///   `PurchaseOrderRevision` to be retrieved
    /// * `revision_id` - The revision number of the `PurchaseOrderRevision` to be retrieved
    /// * `service_id` - optional service id if running splinter
    fn get_purchase_order_revision(
        &self,
        id: String,
        version_id: String,
        revision_id: String,
        service_id: Option<&str>,
    ) -> Result<Option<PurchaseOrderRevision>, ClientError>;

    /// lists purchase orders.
    ///
    /// # Arguments
    ///
    /// * `filter` - Filter to apply to the list of `PurchaseOrder`s
    /// * `service_id` - optional service id if running splinter
    fn list_purchase_orders(
        &self,
        filter: Option<PurchaseOrderFilter>,
        service_id: Option<&str>,
    ) -> Result<Vec<PurchaseOrder>, ClientError>;

    /// lists the purchase order versions of a specific purchase order.
    ///
    /// # Arguments
    ///
    /// * `id` - The id of the `PurchaseOrder` containing the `PurchaseOrderVersion`s to be listed
    /// * `filter` - Filter to apply to the list of purchase orders
    /// * `service_id` - optional service id if running splinter
    fn list_purchase_order_versions(
        &self,
        id: String,
        service_id: Option<&str>,
    ) -> Result<Vec<PurchaseOrderVersion>, ClientError>;

    /// lists the purchase order revisions of a specific purchase order version.
    ///
    /// # Arguments
    ///
    /// * `id` - The id of the `PurchaseOrder` containing the `PurchaseOrderRevision`s to be listed
    /// * `version_id` - The version id of the `PurchaseOrderVersion` containing
    ///   the `PurchaseOrderRevision`s to be listed
    /// * `service_id` - optional service id if running splinter
    fn list_purchase_order_revisions(
        &self,
        id: String,
        version_id: String,
        service_id: Option<&str>,
    ) -> Result<Vec<PurchaseOrderRevision>, ClientError>;
}
