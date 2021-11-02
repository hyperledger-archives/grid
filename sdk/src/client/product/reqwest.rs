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

use super::{LatLong, Product, ProductClient, PropertyValue};
use crate::client::reqwest::{fetch_entities_list, fetch_entity, post_batches};
use crate::client::Client;
use crate::error::ClientError;

use sawtooth_sdk::messages::batch::BatchList;

const PRODUCT_ROUTE: &str = "product";

#[derive(Debug, Deserialize)]
pub struct ProductDto {
    pub product_id: String,
    pub product_namespace: String,
    pub owner: String,
    pub properties: Vec<PropertyValueDto>,
}

impl From<&ProductDto> for Product {
    fn from(d: &ProductDto) -> Self {
        Self {
            product_id: d.product_id.to_string(),
            product_namespace: d.product_namespace.to_string(),
            owner: d.owner.to_string(),
            properties: d.properties.iter().map(PropertyValue::from).collect(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct PropertyValueDto {
    pub name: String,
    pub data_type: String,
    pub bytes_value: Option<Vec<u8>>,
    pub boolean_value: Option<bool>,
    pub number_value: Option<i64>,
    pub string_value: Option<String>,
    pub enum_value: Option<u32>,
    pub struct_values: Option<Vec<String>>,
    pub lat_long_value: Option<LatLongDto>,
}

impl From<&PropertyValueDto> for PropertyValue {
    fn from(d: &PropertyValueDto) -> Self {
        Self {
            name: d.name.to_string(),
            data_type: d.data_type.to_string(),
            bytes_value: d.bytes_value.as_ref().map(|x| x.to_vec()),
            boolean_value: d.boolean_value,
            number_value: d.number_value,
            string_value: d.string_value.as_ref().map(String::from),
            enum_value: d.enum_value,
            struct_values: d
                .struct_values
                .as_ref()
                .map(|x| x.iter().map(String::from).collect()),
            lat_long_value: d.lat_long_value.as_ref().map(LatLong::from),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct LatLongDto {
    latitude: i64,
    longitude: i64,
}

impl From<&LatLongDto> for LatLong {
    fn from(d: &LatLongDto) -> Self {
        Self {
            latitude: d.latitude,
            longitude: d.longitude,
        }
    }
}

/// The Reqwest implementation of the Product client
pub struct ReqwestProductClient {
    url: String,
}

impl ReqwestProductClient {
    pub fn new(url: String) -> Self {
        Self { url }
    }
}

impl Client for ReqwestProductClient {
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

impl ProductClient for ReqwestProductClient {
    /// Fetches single product by identifier
    ///
    /// # Arguments
    ///
    /// * `product_id` - the product's identifier
    /// * `service_id` - optional - the service ID to fetch the product from
    fn get_product(&self, id: String, service_id: Option<&str>) -> Result<Product, ClientError> {
        let dto =
            fetch_entity::<ProductDto>(&self.url, format!("{}/{}", PRODUCT_ROUTE, id), service_id)?;
        Ok(Product::from(&dto))
    }

    /// Fetches all products for a service
    ///
    /// # Arguments
    ///
    /// * `service_id` - optional - the service ID to fetch the products from
    fn list_products(&self, service_id: Option<&str>) -> Result<Vec<Product>, ClientError> {
        let dto_vec = fetch_entities_list::<ProductDto>(
            &self.url,
            PRODUCT_ROUTE.to_string(),
            service_id,
            None,
        )?;
        Ok(dto_vec.iter().map(Product::from).collect())
    }
}
