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

//! A Reqwest-based implementation of Client

use protobuf::Message;
use reqwest::blocking::Client as BlockingClient;
use sawtooth_sdk::messages::batch::BatchList;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Instant;

use crate::error::InternalError;

use super::Client;
#[cfg(feature = "purchase-order")]
use super::{PurchaseOrder, PurchaseOrderRevision, PurchaseOrderVersion};

pub struct ReqwestClient {
    pub url: String,
}

impl ReqwestClient {
    pub fn _new(url: String) -> Self {
        ReqwestClient { url }
    }
}

impl Client for ReqwestClient {
    /// Submits a list of batches
    fn post_batches(
        &self,
        batch_list: &BatchList,
        service_id: Option<&str>,
        wait: u64,
    ) -> Result<(), InternalError> {
        let bytes = batch_list.write_to_bytes().map_err(|err| {
            InternalError::from_source_with_message(
                Box::new(err),
                "Failed to convert batch list to bytes".to_string(),
            )
        })?;

        let mut wait_time = wait;

        let mut url = format!("{}/batches", self.url);

        if let Some(service_id) = service_id {
            url.push_str(&format!("?service_id={}", service_id));
        }

        let client = BlockingClient::new();

        let response = client
            .post(&url)
            .header("GridProtocolVersion", "1")
            .body(bytes)
            .send()
            .map_err(|err| {
                InternalError::from_source_with_message(
                    Box::new(err),
                    "Failed to post batch list".to_string(),
                )
            })?;

        if !response.status().is_success() {
            return Err(InternalError::with_message(response.text().map_err(
                |_| {
                    InternalError::with_message(
                        "Unable to convert error response to text".to_string(),
                    )
                },
            )?));
        }

        let batch_link = response.json::<BatchStatusLink>().map_err(|err| {
            InternalError::from_source_with_message(
                Box::new(err),
                "Unable to get batch status link from response".to_string(),
            )
        })?;

        let params: Vec<&str> = batch_link.link.split('?').collect();

        let id_param: Vec<&str> = params[1].split('=').collect();

        let id = id_param[1];

        info!("Submitted batch: {}", id);

        while wait_time > 0 {
            let time = Instant::now();

            let url = if let Some(service_id) = service_id {
                format!(
                    "{}&wait={}&service_id={}",
                    batch_link.link, wait_time, service_id
                )
            } else {
                format!("{}&wait={}", batch_link.link, wait_time)
            };

            let response = client.get(&url).send().map_err(|err| {
                InternalError::from_source_with_message(
                    Box::new(err),
                    "Unable to get batch status".to_string(),
                )
            })?;

            if !response.status().is_success() {
                return Err(InternalError::with_message(response.text().map_err(
                    |_| {
                        InternalError::with_message(
                            "Unable to convert error response to text".to_string(),
                        )
                    },
                )?));
            }

            let batch_status = response.json::<BatchStatusResponse>().map_err(|err| {
                InternalError::from_source_with_message(
                    Box::new(err),
                    "Unable to get batch status response".to_string(),
                )
            })?;

            for t in &batch_status.data {
                if t.status == "Invalid" {
                    for i in &t.invalid_transactions {
                        error!(
                            "Error: {}",
                            i.get("message")
                                .unwrap_or(&"Batch contained invalid transactions".to_string())
                        );
                    }
                }
            }

            if batch_status.data.iter().all(|d| d.status == "Valid") {
                info!("Batch and transaction structure was valid. Batch queued.");
            }

            if batch_status.data.iter().all(|x| x.status != "PENDING") {
                break;
            }

            wait_time -= time.elapsed().as_secs()
        }

        Ok(())
    }

    /// Retrieves the purchase order with the specified `id`.
    #[cfg(feature = "purchase-order")]
    fn get_purchase_order(&self, _id: String) -> Result<Option<PurchaseOrder>, InternalError> {
        unimplemented!()
    }

    /// Retrieves the purchase order version with the given `version_id` of the purchase order
    /// with the given `id`
    #[cfg(feature = "purchase-order")]
    fn get_purchase_order_version(
        &self,
        _id: String,
        _version_id: String,
    ) -> Result<Option<PurchaseOrderVersion>, InternalError> {
        unimplemented!()
    }

    /// Retrieves the purchase order revision with the given `revision_id` of the purchase
    /// order version with the given `version_id` of the purchase order with the given `id`
    #[cfg(feature = "purchase-order")]
    fn get_purchase_order_revision(
        &self,
        _id: String,
        _version_id: String,
        _revision_id: String,
    ) -> Result<Option<PurchaseOrderRevision>, InternalError> {
        unimplemented!()
    }

    /// lists purchase orders.
    #[cfg(feature = "purchase-order")]
    fn list_purchase_orders(
        &self,
        _filter: Option<&str>,
    ) -> Result<Vec<PurchaseOrder>, InternalError> {
        unimplemented!()
    }

    /// lists the purchase order versions of a specific purchase order.
    #[cfg(feature = "purchase-order")]
    fn list_purchase_order_versions(
        &self,
        _id: String,
        _filter: Option<&str>,
    ) -> Result<Vec<PurchaseOrderVersion>, InternalError> {
        unimplemented!()
    }

    /// lists the purchase order revisions of a specific purchase order version.
    #[cfg(feature = "purchase-order")]
    fn list_purchase_order_revisions(
        &self,
        _id: String,
        _version_id: String,
        _filter: Option<&str>,
    ) -> Result<Vec<PurchaseOrderRevision>, InternalError> {
        unimplemented!()
    }
}

#[derive(Deserialize, Debug)]
pub struct BatchStatusLink {
    pub link: String,
}

#[derive(Deserialize, Debug)]
struct BatchStatusResponse {
    pub data: Vec<BatchStatus>,
}

#[derive(Deserialize, Debug)]
struct BatchStatus {
    pub invalid_transactions: Vec<HashMap<String, String>>,
    pub status: String,
}
