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

//! Traits and implementations useful for interacting with the REST API.

#[cfg(feature = "client-reqwest")]
mod reqwest;

use sawtooth_sdk::messages::batch::BatchList;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

use crate::error::InternalError;

pub trait Client {
    /// Submits a list of batches
    ///
    /// # Arguments
    ///
    /// * `batches` - The `BatchList` to be submitted
    fn post_batches(
        &self,
        batch_list: &BatchList,
        service_id: Option<&str>,
        wait: u64,
    ) -> Result<(), InternalError>;

    /// Retrieves the purchase order with the specified `id`.
    ///
    /// # Arguments
    ///
    /// * `id` - The uuid of the `PurchaseOrder` to be retrieved
    #[cfg(feature = "purchase-order")]
    fn get_purchase_order(&self, id: String) -> Result<Option<PurchaseOrder>, InternalError>;

    /// Retrieves the purchase order version with the given `version_id` of the purchase
    /// order with the given `id`
    ///
    /// # Arguments
    ///
    /// * `id` - The uuid of the `PurchaseOrder` containing the `PurchaseOrderVersion` to be retrieved
    /// * `version_id` - The version id of the `PurchaseOrderVersion` to be retrieved
    #[cfg(feature = "purchase-order")]
    fn get_purchase_order_version(
        &self,
        id: String,
        version_id: String,
    ) -> Result<Option<PurchaseOrderVersion>, InternalError>;

    /// Retrieves the purchase order revision with the given `revision_id` of
    /// the purchase order version with the given `version_id` of the purchase order with the given `id`
    ///
    /// # Arguments
    ///
    /// * `id` - The uuid of the `PurchaseOrder` containing the `PurchaseOrderRevision` to be retrieved
    /// * `version_id` - The version id of the `PurchaseOrderVersion` containing the
    ///   `PurchaseOrderRevision` to be retrieved
    /// * `revision_id` - The revision number of the `PurchaseOrderRevision` to be retrieved
    #[cfg(feature = "purchase-order")]
    fn get_purchase_order_revision(
        &self,
        id: String,
        version_id: String,
        revision_id: String,
    ) -> Result<Option<PurchaseOrderRevision>, InternalError>;

    /// lists purchase orders.
    ///
    /// # Arguments
    ///
    /// * `filter` - Filter to apply to the list of `PurchaseOrder`s
    #[cfg(feature = "purchase-order")]
    fn list_purchase_orders(
        &self,
        filter: Option<&str>,
    ) -> Result<Vec<PurchaseOrder>, InternalError>;

    /// lists the purchase order versions of a specific purchase order.
    ///
    /// # Arguments
    ///
    /// * `id` - The uuid of the `PurchaseOrder` containing the `PurchaseOrderVersion`s to be listed
    /// * `filter` - Filter to apply to the list of purchase orders
    #[cfg(feature = "purchase-order")]
    fn list_purchase_order_versions(
        &self,
        id: String,
        filter: Option<&str>,
    ) -> Result<Vec<PurchaseOrderVersion>, InternalError>;

    /// lists the purchase order revisions of a specific purchase order version.
    ///
    /// # Arguments
    ///
    /// * `id` - The uuid of the `PurchaseOrder` containing the `PurchaseOrderRevision`s to be listed
    /// * `version_id` - The version id of the `PurchaseOrderVersion` containing
    ///   the `PurchaseOrderRevision`s to be listed
    #[cfg(feature = "purchase-order")]
    fn list_purchase_order_revisions(
        &self,
        id: String,
        version_id: String,
        filter: Option<&str>,
    ) -> Result<Vec<PurchaseOrderRevision>, InternalError>;
}

#[cfg(feature = "purchase-order")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PurchaseOrder {
    org_id: String,
    uuid: String,
    workflow_status: String,
    is_closed: bool,
    accepted_version_id: Option<String>,
    versions: Vec<PurchaseOrderVersion>,
    created_at: SystemTime,
}

#[cfg(feature = "purchase-order")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PurchaseOrderVersion {
    version_id: String,
    workflow_status: String,
    is_draft: bool,
    current_revision_id: u64,
    revisions: Vec<PurchaseOrderRevision>,
}

#[cfg(feature = "purchase-order")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PurchaseOrderRevision {
    revision_id: u64,
    order_xml_v3_4: String,
    submitter: String,
    created_at: u64,
}
