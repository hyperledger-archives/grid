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

use crate::error::ClientError;
use serde::de::DeserializeOwned;

use protobuf::Message;
use reqwest::blocking::Client as BlockingClient;
use sawtooth_sdk::messages::batch::BatchList;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Instant;

use super::location::reqwest::ReqwestLocationClient;
use super::pike::reqwest::ReqwestPikeClient;
use super::product::reqwest::ReqwestProductClient;
#[cfg(feature = "purchase-order")]
use super::purchase_order;
#[cfg(feature = "purchase-order")]
use super::purchase_order::reqwest::ReqwestPurchaseOrderClient;
use super::schema::reqwest::ReqwestSchemaClient;
use super::ClientFactory;
use super::{location, pike, product, schema};

pub struct ReqwestClientFactory {}

impl ReqwestClientFactory {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for ReqwestClientFactory {
    fn default() -> Self {
        ReqwestClientFactory::new()
    }
}

impl ClientFactory for ReqwestClientFactory {
    /// Retrieves a client for listing and showing locations
    fn get_location_client(&self, url: String) -> Box<dyn location::LocationClient> {
        Box::new(ReqwestLocationClient::new(url))
    }

    /// Retrieves a client for listing and showing pike members
    fn get_pike_client(&self, url: String) -> Box<dyn pike::PikeClient> {
        Box::new(ReqwestPikeClient::new(url))
    }

    /// Retrieves a client for listing and showing products
    fn get_product_client(&self, url: String) -> Box<dyn product::ProductClient> {
        Box::new(ReqwestProductClient::new(url))
    }

    /// Retrieves a client for listing and showing
    /// purchase orders, revisions, and versions
    #[cfg(feature = "purchase-order")]
    fn get_purchase_order_client(
        &self,
        url: String,
    ) -> Box<dyn purchase_order::PurchaseOrderClient> {
        Box::new(ReqwestPurchaseOrderClient::new(url))
    }

    /// Retrieves a client for listing and showing schemas
    fn get_schema_client(&self, url: String) -> Box<dyn schema::SchemaClient> {
        Box::new(ReqwestSchemaClient::new(url))
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Paging {
    current: String,
    offset: i64,
    limit: i64,
    total: i64,
    first: String,
    prev: Option<String>,
    next: Option<String>,
    last: String,
}

#[derive(Debug, Deserialize)]
pub struct ListSlice<T> {
    pub data: Vec<T>,
    pub paging: Paging,
}

#[derive(Deserialize, Debug)]
pub struct BatchStatusLink {
    pub link: String,
}

#[derive(Deserialize, Debug)]
struct BatchStatusResponse {
    pub data: Vec<BatchStatus>,
    pub link: String,
}

#[derive(Deserialize, Debug)]
struct BatchStatus {
    pub id: String,
    pub invalid_transactions: Vec<HashMap<String, String>>,
    pub status: String,
}

/// Fetches and serializes `T` entities from REST API
///
/// # Arguments
///
/// * `url` - The base url of the request
/// * `route` - the route to find the entity
/// * `service_id` - optional - the service ID to fetch the entities from
pub fn fetch_entities_list<T: DeserializeOwned>(
    url: &str,
    route: String,
    service_id: Option<&str>,
    filters: Option<HashMap<&str, String>>,
) -> Result<Vec<T>, ClientError> {
    let client = BlockingClient::new();
    let mut final_url = format!("{}/{}", url, route);

    let mut query_params: Vec<(&str, String)> = service_id
        .into_iter()
        .map(|sid| ("service_id", sid.to_string()))
        .chain(filters.unwrap_or_default().into_iter())
        .collect();

    let mut entities: Vec<T> = Vec::new();

    loop {
        let response = client.get(&final_url).query(&query_params).send()?;

        if !response.status().is_success() {
            return Err(ClientError::DaemonError(response.text()?));
        }

        let mut entity_list_slice = response.json::<ListSlice<T>>()?;

        entities.append(&mut entity_list_slice.data);

        if let Some(next) = entity_list_slice.paging.next {
            final_url = next;
            query_params.clear();
        } else {
            break;
        }
    }

    Ok(entities)
}

/// Fetches and serializes single `T` Entity from REST API
///
/// # Arguments
///
/// * `url` - the base url of the request
/// * `route` - the identifying route where to find the entity
/// * `service_id` - optional - the service ID to fetch the entity from
pub fn fetch_entity<T: DeserializeOwned>(
    url: &str,
    route: String,
    service_id: Option<&str>,
) -> Result<T, ClientError> {
    let client = BlockingClient::new();
    let final_url = format!("{}/{}", url, route);

    let query_params: Vec<(&str, String)> = service_id
        .into_iter()
        .map(|sid| ("service_id", sid.to_string()))
        .collect();

    let response = client.get(&final_url).query(&query_params).send()?;

    if !response.status().is_success() {
        return Err(ClientError::DaemonError(response.text()?));
    }

    let agent = response.json::<T>()?;

    Ok(agent)
}

pub fn post_batches(
    url: &str,
    wait: u64,
    batch_list: &BatchList,
    service_id: Option<&str>,
) -> Result<(), ClientError> {
    let bytes = batch_list.write_to_bytes().map_err(|_err| {
        ClientError::DaemonError("Failed to convert batch list to bytes".to_string())
    })?;

    let mut wait_time = wait;

    let mut url = format!("{}/batches", url);

    if let Some(service_id) = service_id {
        url.push_str(&format!("?service_id={}", service_id));
    }

    let client = BlockingClient::new();

    let response = client
        .post(&url)
        .header("GridProtocolVersion", "1")
        .body(bytes)
        .send()
        .map_err(|_err| ClientError::DaemonError("Failed to post batch list".to_string()))?;

    if !response.status().is_success() {
        return Err(ClientError::DaemonError(response.text().map_err(|_| {
            ClientError::DaemonError("Unable to convert error response to text".to_string())
        })?));
    }

    let batch_link = response.json::<BatchStatusLink>().map_err(|_err| {
        ClientError::DaemonError("Unable to get batch status link from response".to_string())
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

        let response = client
            .get(&url)
            .send()
            .map_err(|_err| ClientError::DaemonError("Unable to get batch status".to_string()))?;

        if !response.status().is_success() {
            return Err(ClientError::DaemonError(response.text().map_err(|_| {
                ClientError::DaemonError("Unable to convert error response to text".to_string())
            })?));
        }

        let batch_status = response.json::<BatchStatusResponse>().map_err(|_err| {
            ClientError::DaemonError("Unable to get batch status response".to_string())
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
