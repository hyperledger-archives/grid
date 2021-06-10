// Copyright 2018-2020 Cargill Incorporated
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

#[cfg(feature = "diesel")]
pub(in crate) mod diesel;
pub mod error;

use crate::paging::Paging;

#[cfg(feature = "diesel")]
pub use self::diesel::DieselProductStore;
pub use error::ProductStoreError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub product_id: String,
    pub product_address: String,
    pub product_namespace: String,
    pub owner: String,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
    pub last_updated: Option<i64>,
    pub properties: Vec<PropertyValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyValue {
    pub product_id: String,
    pub product_address: String,
    pub property_name: String,
    pub data_type: String,
    pub bytes_value: Option<Vec<u8>>,
    pub boolean_value: Option<bool>,
    pub number_value: Option<i64>,
    pub string_value: Option<String>,
    pub enum_value: Option<i32>,
    pub struct_values: Vec<PropertyValue>,
    pub lat_long_value: Option<LatLongValue>,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ProductList {
    pub data: Vec<Product>,
    pub paging: Paging,
}

impl ProductList {
    pub fn new(data: Vec<Product>, paging: Paging) -> Self {
        Self { data, paging }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatLongValue {
    pub latitude: i64,
    pub longitude: i64,
}

pub trait ProductStore: Send + Sync {
    /// Adds a product to the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `product` - The product to be added
    fn add_product(&self, product: Product) -> Result<(), ProductStoreError>;

    /// Gets a product from the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `product_id` - The ID of the product to be fetched
    ///  * `service_id` - The service ID to fetch the product for
    fn get_product(
        &self,
        product_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Product>, ProductStoreError>;

    /// Gets a list of products from the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `service_id` - The service ID to fetch the product for
    ///  * `offset` - The index of the first in storage to retrieve
    ///  * `limit` - The number of items to retrieve from the offset
    fn list_products(
        &self,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<ProductList, ProductStoreError>;

    /// Updates a product in the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `product` - The updated product
    ///  * `service_id` - The service ID to fetch the product for
    ///  * `current_commit_num` - The current commit height
    fn update_product(
        &self,
        product_id: &str,
        service_id: Option<&str>,
        current_commit_num: i64,
    ) -> Result<(), ProductStoreError>;

    /// Deletes a product from the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `address` - The address of the record to be deleted
    ///  * `current_commit_num` - The current commit height
    fn delete_product(
        &self,
        address: &str,
        current_commit_num: i64,
    ) -> Result<(), ProductStoreError>;
}
