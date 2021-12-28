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
use std::collections::VecDeque;

use protobuf::Message;
use reqwest::blocking::Client as BlockingClient;
use sawtooth_sdk::messages::batch::BatchList;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Instant;

#[cfg(feature = "location")]
mod location;
#[cfg(feature = "location")]
pub use location::*;
#[cfg(feature = "pike")]
mod pike;
#[cfg(feature = "pike")]
pub use pike::*;
#[cfg(feature = "product")]
mod product;
#[cfg(feature = "product")]
pub use product::*;
#[cfg(feature = "purchase-order")]
mod purchase_order;
#[cfg(feature = "purchase-order")]
pub use purchase_order::*;
#[cfg(feature = "schema")]
mod schema;
#[cfg(feature = "location")]
use super::location as client_location;
#[cfg(feature = "pike")]
use super::pike as client_pike;
#[cfg(feature = "product")]
use super::product as client_product;
#[cfg(feature = "purchase-order")]
use super::purchase_order as client_purchase_order;
#[cfg(feature = "schema")]
use super::schema as client_schema;
use super::ClientFactory;
#[cfg(feature = "schema")]
pub use schema::*;

/// This is the abstraction of the `ClientFactory` struct for the
/// reqwest-backed implementation. This provides methods to return the reqwest
/// clients for various Grid features.
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
    #[cfg(feature = "location")]
    fn get_location_client(&self, url: String) -> Box<dyn client_location::LocationClient> {
        Box::new(ReqwestLocationClient::new(url))
    }

    /// Retrieves a client for listing and showing pike members
    #[cfg(feature = "pike")]
    fn get_pike_client(&self, url: String) -> Box<dyn client_pike::PikeClient> {
        Box::new(ReqwestPikeClient::new(url))
    }

    /// Retrieves a client for listing and showing products
    #[cfg(feature = "product")]
    fn get_product_client(&self, url: String) -> Box<dyn client_product::ProductClient> {
        Box::new(ReqwestProductClient::new(url))
    }

    /// Retrieves a client for listing and showing
    /// purchase orders, revisions, and versions
    #[cfg(feature = "purchase-order")]
    fn get_purchase_order_client(
        &self,
        url: String,
    ) -> Box<dyn client_purchase_order::PurchaseOrderClient> {
        Box::new(ReqwestPurchaseOrderClient::new(url))
    }

    /// Retrieves a client for listing and showing schemas
    #[cfg(feature = "schema")]
    fn get_schema_client(&self, url: String) -> Box<dyn client_schema::SchemaClient> {
        Box::new(ReqwestSchemaClient::new(url))
    }
}

/// Reqwest client representation of response paging
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

pub struct PagingIter<T>
where
    T: for<'a> serde::de::Deserialize<'a> + Sized,
{
    next: Option<String>,
    cache: VecDeque<T>,
    initial_query: Option<Vec<(String, String)>>,
}

impl<T> PagingIter<T>
where
    T: for<'a> serde::de::Deserialize<'a> + Sized,
{
    /// Create a new 'PagingIter' which will make a call to the REST API and load the initial
    /// cache with the first page of items.
    fn new(url: &str, query_params: Option<Vec<(String, String)>>) -> Result<Self, ClientError> {
        let mut new_iter = Self {
            next: Some(url.to_string()),
            cache: VecDeque::with_capacity(0),
            initial_query: query_params,
        };
        new_iter.reload_cache()?;
        Ok(new_iter)
    }

    // If another page of items exists, use the 'next' URL from the current page and
    /// reload the cache with the next page of items.
    fn reload_cache(&mut self) -> Result<(), ClientError> {
        if let Some(url) = &self.next.take() {
            let mut request = BlockingClient::new().get(url);
            if let Some(query) = &self.initial_query.take() {
                request = request.query(&query);
            }
            let response = request.send()?;
            if !response.status().is_success() {
                return Err(ClientError::InternalError(response.text()?));
            }

            let page: Page<T> = response.json::<Page<T>>()?;

            self.cache = page.data.into();
            self.next = page.paging.next.map(String::from);
            self.initial_query = None;
        }
        Ok(())
    }
}

impl<T> Iterator for PagingIter<T>
where
    T: for<'a> serde::de::Deserialize<'a> + Sized,
{
    type Item = Result<T, ClientError>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.cache.is_empty() && self.next.is_some() {
            if let Err(err) = self.reload_cache() {
                return Some(Err(err));
            }
        };
        self.cache.pop_front().map(Ok)
    }
}

/// A struct that represents a page of items, used for deserializing JSON objects.
#[derive(Debug, Deserialize)]
struct Page<T: Sized> {
    #[serde(bound(deserialize = "T: Deserialize<'de>"))]
    data: Vec<T>,
    paging: Paging,
}

/// Reqwest client representation of a slice of a list response
#[derive(Debug, Deserialize)]
pub struct ListSlice<T> {
    pub data: Vec<T>,
    pub paging: Paging,
}

/// Reqwest client representation of a link to a batch status
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

/// Fetches and deserializes `T` entities from REST API
///
/// # Arguments
///
/// * `url` - The base url of the request
/// * `route` - the route to find the entity
/// * `service_id` - optional - the service ID to fetch the entities from
/// * `filters` - optional - filters for the resource being fetched
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
            return Err(ClientError::InternalError(response.text()?));
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

pub fn fetch_entities_list_stream<T: 'static + DeserializeOwned + std::fmt::Debug>(
    url: &str,
    route: String,
    service_id: Option<&str>,
    filters: Option<HashMap<String, String>>,
) -> Result<Box<dyn Iterator<Item = Result<T, ClientError>>>, ClientError> {
    let final_url = format!("{}/{}", url, route);

    let query_params: Vec<(String, String)> = service_id
        .into_iter()
        .map(|sid| ("service_id".to_string(), sid.to_string()))
        .chain(filters.unwrap_or_default().into_iter())
        .collect();

    Ok(Box::new(PagingIter::new(&final_url, Some(query_params))?))
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
        return Err(ClientError::InternalError(response.text()?));
    }

    let agent = response.json::<T>()?;

    Ok(agent)
}

/// Submits a list of batches
///
/// # Arguments
///
/// * `url` - the base url of the request
/// * `wait` - duration to wait for batch status response
/// * `batch_list` - the list of batches to submit
/// * `service_id` - optional - the service ID to submit batches to if running
///   on splinter
pub fn post_batches(
    url: &str,
    wait: u64,
    batch_list: &BatchList,
    service_id: Option<&str>,
) -> Result<(), ClientError> {
    let bytes = batch_list.write_to_bytes().map_err(|_err| {
        ClientError::InternalError("Failed to convert batch list to bytes".to_string())
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
        .map_err(|_err| ClientError::InternalError("Failed to post batch list".to_string()))?;

    if !response.status().is_success() {
        return Err(ClientError::InternalError(response.text().map_err(
            |_| ClientError::InternalError("Unable to convert error response to text".to_string()),
        )?));
    }

    let batch_link = response.json::<BatchStatusLink>().map_err(|_err| {
        ClientError::InternalError("Unable to get batch status link from response".to_string())
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
            .map_err(|_err| ClientError::InternalError("Unable to get batch status".to_string()))?;

        if !response.status().is_success() {
            return Err(ClientError::InternalError(response.text().map_err(
                |_| {
                    ClientError::InternalError(
                        "Unable to convert error response to text".to_string(),
                    )
                },
            )?));
        }

        let batch_status = response.json::<BatchStatusResponse>().map_err(|_err| {
            ClientError::InternalError("Unable to get batch status response".to_string())
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
