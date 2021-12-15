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

mod data;

use std::collections::HashMap;

use crate::client::reqwest::{fetch_entities_list, fetch_entity, post_batches};
use crate::client::Client;
use crate::error::ClientError;
use crate::purchase_order::store::{ListPOFilters, ListVersionFilters};

use crate::client::purchase_order::{
    PurchaseOrder, PurchaseOrderClient, PurchaseOrderRevision, PurchaseOrderVersion,
};

use sawtooth_sdk::messages::batch::BatchList;

const PO_ROUTE: &str = "purchase_order";
const VERSION_ROUTE: &str = "version";
const REVISION_ROUTE: &str = "revision";
const LATEST_ROUTE: &str = "latest";

/// The Reqwest implementation of the Purchase Order client
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
    /// * `service_id` - optional - the service ID to post batches to if running splinter
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
        let dto = fetch_entity::<data::PurchaseOrder>(
            &self.url,
            format!("{}/{}", PO_ROUTE, id),
            service_id,
        )?;
        Ok(Some(PurchaseOrder::from(&dto)))
    }

    /// Retrieves the purchase order version with the given `version_id` of the purchase order
    /// with the given `id`
    fn get_purchase_order_version(
        &self,
        id: String,
        version_id: String,
        service_id: Option<&str>,
    ) -> Result<Option<PurchaseOrderVersion>, ClientError> {
        let dto = fetch_entity::<data::PurchaseOrderVersion>(
            &self.url,
            format!("{}/{}/{}/{}", PO_ROUTE, id, VERSION_ROUTE, version_id),
            service_id,
        )?;

        Ok(Some(PurchaseOrderVersion::from(&dto)))
    }

    /// Retrieves the purchase order revision with the given `revision_id` of the purchase
    /// order version with the given `version_id` of the purchase order with the given `id`
    fn get_purchase_order_revision(
        &self,
        id: String,
        version_id: String,
        revision_id: u64,
        service_id: Option<&str>,
    ) -> Result<Option<PurchaseOrderRevision>, ClientError> {
        let dto = fetch_entity::<data::PurchaseOrderRevision>(
            &self.url,
            format!(
                "{}/{}/{}/{}/{}/{}",
                PO_ROUTE, id, VERSION_ROUTE, version_id, REVISION_ROUTE, revision_id
            ),
            service_id,
        )?;

        Ok(Some(PurchaseOrderRevision::from(&dto)))
    }

    /// Lists purchase orders.
    fn list_purchase_orders(
        &self,
        filters: Option<ListPOFilters>,
        service_id: Option<&str>,
    ) -> Result<Vec<PurchaseOrder>, ClientError> {
        let mut filter_map = HashMap::new();
        if let Some(filters) = filters {
            if let Some(is_open) = filters.is_open {
                filter_map.insert("is_open", is_open.to_string());
            }
            if let Some(has_accepted_version) = filters.has_accepted_version {
                filter_map.insert("has_accepted_version", has_accepted_version.to_string());
            }
            if let Some(buyer_org_id) = filters.buyer_org_id {
                filter_map.insert("buyer_org_id", buyer_org_id);
            }
            if let Some(seller_org_id) = filters.seller_org_id {
                filter_map.insert("seller_org_id", seller_org_id);
            }
            if let Some(alternate_ids) = filters.alternate_ids {
                filter_map.insert("alternate_ids", alternate_ids);
            }
        }
        let dto_vec = fetch_entities_list::<data::PurchaseOrder>(
            &self.url,
            PO_ROUTE.to_string(),
            service_id,
            Some(filter_map),
        )?;
        Ok(dto_vec.iter().map(PurchaseOrder::from).collect())
    }

    /// Lists the purchase order versions of a specific purchase order.
    fn list_purchase_order_versions(
        &self,
        id: String,
        filters: Option<ListVersionFilters>,
        service_id: Option<&str>,
    ) -> Result<Vec<PurchaseOrderVersion>, ClientError> {
        let mut filter_map = HashMap::new();
        if let Some(filters) = filters {
            if let Some(is_accepted) = filters.is_accepted {
                filter_map.insert("is_accepted", is_accepted.to_string());
            }
            if let Some(is_draft) = filters.is_draft {
                filter_map.insert("is_draft", is_draft.to_string());
            }
        }
        let dto = fetch_entities_list::<data::PurchaseOrderVersion>(
            &self.url,
            format!("{}/{}/{}", PO_ROUTE, id, VERSION_ROUTE,),
            service_id,
            Some(filter_map),
        )?;

        Ok(dto.iter().map(PurchaseOrderVersion::from).collect())
    }

    /// Lists the purchase order revisions of a specific purchase order version.
    fn list_purchase_order_revisions(
        &self,
        id: String,
        version_id: String,
        service_id: Option<&str>,
    ) -> Result<Vec<PurchaseOrderRevision>, ClientError> {
        let dto = fetch_entities_list::<data::PurchaseOrderRevision>(
            &self.url,
            format!(
                "{}/{}/{}/{}/{}",
                PO_ROUTE, id, VERSION_ROUTE, version_id, REVISION_ROUTE
            ),
            service_id,
            None,
        )?;

        Ok(dto.iter().map(PurchaseOrderRevision::from).collect())
    }

    fn get_latest_revision_id(
        &self,
        purchase_order_uid: String,
        version_id: String,
        service_id: Option<&str>,
    ) -> Result<Option<i64>, ClientError> {
        let dto = fetch_entity::<Option<i64>>(
            &self.url,
            format!(
                "{}/{}/{}/{}/{}/{}",
                PO_ROUTE,
                purchase_order_uid,
                VERSION_ROUTE,
                version_id,
                REVISION_ROUTE,
                LATEST_ROUTE,
            ),
            service_id,
        )?;

        Ok(dto)
    }
}
