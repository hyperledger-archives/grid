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

use std::collections::HashMap;
use std::time::SystemTime;

use crate::client::reqwest::{fetch_entities_list, fetch_entity, post_batches};
use crate::client::Client;
use crate::error::ClientError;

use super::{
    PurchaseOrder, PurchaseOrderClient, PurchaseOrderFilter, PurchaseOrderRevision,
    PurchaseOrderVersion,
};

use sawtooth_sdk::messages::batch::BatchList;

const PURCHASE_ORDER_ROUTE: &str = "purchase_order";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct PurchaseOrderDto {
    buyer_org_id: String,
    seller_org_id: String,
    purchase_order_uid: String,
    workflow_status: String,
    is_closed: bool,
    accepted_version_id: Option<String>,
    versions: Vec<String>,
    created_at: SystemTime,
    workflow_type: String,
    start_commit_num: i64,
    end_commit_num: i64,
}

impl From<&PurchaseOrderDto> for PurchaseOrder {
    fn from(d: &PurchaseOrderDto) -> Self {
        Self {
            buyer_org_id: d.buyer_org_id.to_string(),
            seller_org_id: d.seller_org_id.to_string(),
            purchase_order_uid: d.purchase_order_uid.to_string(),
            workflow_status: d.workflow_status.to_string(),
            is_closed: d.is_closed,
            accepted_version_id: d.accepted_version_id.as_ref().map(String::from),
            versions: d.versions.iter().map(String::from).collect(),
            created_at: d.created_at,
            workflow_type: d.workflow_type.to_string(),
            start_commit_num: d.start_commit_num,
            end_commit_num: d.end_commit_num,
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
    start_commit_num: i64,
    end_commit_num: i64,
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
            start_commit_num: d.start_commit_num,
            end_commit_num: d.end_commit_num,
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
    fn get_purchase_order(
        &self,
        id: String,
        service_id: Option<&str>,
    ) -> Result<Option<PurchaseOrder>, ClientError> {
        let dto = fetch_entity::<PurchaseOrderDto>(
            &self.url,
            format!("{}/{}", PURCHASE_ORDER_ROUTE, id),
            service_id,
        )?;
        Ok(Some(PurchaseOrder::from(&dto)))
    }

    /// Retrieves the purchase order version with the given `version_id` of the purchase order
    /// with the given `id`
    fn get_purchase_order_version(
        &self,
        _id: String,
        _version_id: String,
        _service_id: Option<&str>,
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
        _service_id: Option<&str>,
    ) -> Result<Option<PurchaseOrderRevision>, ClientError> {
        unimplemented!()
    }

    /// lists purchase orders.
    fn list_purchase_orders(
        &self,
        filter: Option<PurchaseOrderFilter>,
        service_id: Option<&str>,
    ) -> Result<Vec<PurchaseOrder>, ClientError> {
        let mut filter_map = HashMap::new();
        let org_id_placeholder: String;
        if let Some(filter) = filter {
            if let Some(org_id) = filter.org_id {
                org_id_placeholder = org_id;
                filter_map.insert("org_id", org_id_placeholder);
            }
            if let Some(is_closed) = filter.is_closed {
                filter_map.insert("is_closed", is_closed.to_string());
            }
            if let Some(is_accepted) = filter.is_accepted {
                filter_map.insert("is_accepted", is_accepted.to_string());
            }
        }
        let dto_vec = fetch_entities_list::<PurchaseOrderDto>(
            &self.url,
            PURCHASE_ORDER_ROUTE.to_string(),
            service_id,
            Some(filter_map),
        )?;
        Ok(dto_vec.iter().map(PurchaseOrder::from).collect())
    }

    /// lists the purchase order versions of a specific purchase order.
    fn list_purchase_order_versions(
        &self,
        _id: String,
        _service_id: Option<&str>,
    ) -> Result<Vec<PurchaseOrderVersion>, ClientError> {
        unimplemented!()
    }

    /// lists the purchase order revisions of a specific purchase order version.
    fn list_purchase_order_revisions(
        &self,
        _id: String,
        _version_id: String,
        _service_id: Option<&str>,
    ) -> Result<Vec<PurchaseOrderRevision>, ClientError> {
        unimplemented!()
    }
}
