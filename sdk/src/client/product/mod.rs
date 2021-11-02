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

use crate::error::ClientError;

use super::Client;

#[cfg(feature = "client-reqwest")]
pub mod reqwest;

/// The client representation of Grid Product
#[derive(Debug, PartialEq)]
pub struct Product {
    pub product_id: String,
    pub product_namespace: String,
    pub owner: String,
    pub properties: Vec<PropertyValue>,
}

/// The client representation of Grid Product property value
#[derive(Debug, PartialEq)]
pub struct PropertyValue {
    pub name: String,
    pub data_type: String,
    pub bytes_value: Option<Vec<u8>>,
    pub boolean_value: Option<bool>,
    pub number_value: Option<i64>,
    pub string_value: Option<String>,
    pub enum_value: Option<u32>,
    pub struct_values: Option<Vec<String>>,
    pub lat_long_value: Option<LatLong>,
}

/// The client representation of Grid Product lat/long value
#[derive(Debug, PartialEq)]
pub struct LatLong {
    latitude: i64,
    longitude: i64,
}

pub trait ProductClient: Client {
    /// Fetches single product by identifier
    ///
    /// # Arguments
    ///
    /// * `product_id` - the product's identifier
    /// * `service_id` - optional - the service ID to fetch the product from
    fn get_product(
        &self,
        product_id: String,
        service_id: Option<&str>,
    ) -> Result<Product, ClientError>;

    /// Fetches all products for a service
    ///
    /// # Arguments
    ///
    /// * `service_id` - optional - the service ID to fetch the products from
    fn list_products(&self, service_id: Option<&str>) -> Result<Vec<Product>, ClientError>;
}
