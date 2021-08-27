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

use crate::client::reqwest::post_batches;
use crate::client::Client;
use crate::error::ClientError;

use super::{PurchaseOrder, PurchaseOrderClient, PurchaseOrderRevision, PurchaseOrderVersion};

use sawtooth_sdk::messages::batch::BatchList;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct PurchaseOrderDto {
    org_id: String,
    uuid: String,
    workflow_status: String,
    is_closed: bool,
    accepted_version_id: Option<String>,
    versions: Vec<PurchaseOrderVersionDto>,
    created_at: SystemTime,
}

impl From<&PurchaseOrderDto> for PurchaseOrder {
    fn from(d: &PurchaseOrderDto) -> Self {
        Self {
            org_id: d.org_id.to_string(),
            uuid: d.uuid.to_string(),
            workflow_status: d.workflow_status.to_string(),
            is_closed: d.is_closed,
            accepted_version_id: d.accepted_version_id.as_ref().map(String::from),
            versions: d.versions.iter().map(PurchaseOrderVersion::from).collect(),
            created_at: d.created_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct PurchaseOrderVersionDto {
    version_id: String,
    workflow_status: String,
    is_draft: bool,
    current_revision_id: u64,
    revisions: Vec<PurchaseOrderRevisionDto>,
}

impl From<&PurchaseOrderVersionDto> for PurchaseOrderVersion {
    fn from(d: &PurchaseOrderVersionDto) -> Self {
        Self {
            version_id: d.version_id.to_string(),
            workflow_status: d.workflow_status.to_string(),
            is_draft: d.is_draft,
            current_revision_id: d.current_revision_id,
            revisions: d
                .revisions
                .iter()
                .map(PurchaseOrderRevision::from)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct PurchaseOrderRevisionDto {
    revision_id: u64,
    order_xml_v3_4: String,
    submitter: String,
    created_at: u64,
}

impl From<&PurchaseOrderRevisionDto> for PurchaseOrderRevision {
    fn from(d: &PurchaseOrderRevisionDto) -> Self {
        Self {
            revision_id: d.revision_id,
            order_xml_v3_4: d.order_xml_v3_4.to_string(),
            submitter: d.submitter.to_string(),
            created_at: d.created_at,
        }
    }
}

pub struct ReqwestPurchaseOrderClient {
    url: String,
}

impl ReqwestPurchaseOrderClient {
    pub fn new(url: String) -> Self {
        Self { url }
    }
}

impl Client for ReqwestPurchaseOrderClient {
    /// Submits a list of batches
    ///
    /// # Arguments
    ///
    /// * `wait` - wait time in seconds
    /// * `batch_list` - The `BatchList` to be submitted
    /// * `service_id` - optional service id if running splinter
    fn post_batches(
        &self,
        wait: u64,
        batch_list: &BatchList,
        service_id: Option<&str>,
    ) -> Result<(), ClientError> {
        post_batches(&self.url, wait, batch_list, service_id)
    }
}

impl PurchaseOrderClient for ReqwestPurchaseOrderClient {
    /// Retrieves the purchase order with the specified `id`.
    fn get_purchase_order(&self, _id: String) -> Result<Option<PurchaseOrder>, ClientError> {
        unimplemented!()
    }

    /// Retrieves the purchase order version with the given `version_id` of the purchase order
    /// with the given `id`
    fn get_purchase_order_version(
        &self,
        _id: String,
        _version_id: String,
    ) -> Result<Option<PurchaseOrderVersion>, ClientError> {
        unimplemented!()
    }

    /// Retrieves the purchase order revision with the given `revision_id` of the purchase
    /// order version with the given `version_id` of the purchase order with the given `id`
    fn get_purchase_order_revision(
        &self,
        _id: String,
        _version_id: String,
        _revision_id: String,
    ) -> Result<Option<PurchaseOrderRevision>, ClientError> {
        unimplemented!()
    }

    /// lists purchase orders.
    fn list_purchase_orders(
        &self,
        _filter: Option<&str>,
    ) -> Result<Vec<PurchaseOrder>, ClientError> {
        unimplemented!()
    }

    /// lists the purchase order versions of a specific purchase order.
    fn list_purchase_order_versions(
        &self,
        _id: String,
        _filter: Option<&str>,
    ) -> Result<Vec<PurchaseOrderVersion>, ClientError> {
        unimplemented!()
    }

    /// lists the purchase order revisions of a specific purchase order version.
    fn list_purchase_order_revisions(
        &self,
        _id: String,
        _version_id: String,
        _filter: Option<&str>,
    ) -> Result<Vec<PurchaseOrderRevision>, ClientError> {
        unimplemented!()
    }
}
