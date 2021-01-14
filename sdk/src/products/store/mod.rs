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

#[cfg(feature = "diesel")]
pub mod diesel;
pub mod error;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatLongValue {
    pub latitude: i64,
    pub longitude: i64,
}

pub trait ProductStore: Send + Sync {
    fn add_product(&self, product: Product) -> Result<(), ProductStoreError>;

    fn fetch_product(
        &self,
        product_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Product>, ProductStoreError>;

    fn list_products(&self, service_id: Option<&str>) -> Result<Vec<Product>, ProductStoreError>;

    fn update_product(
        &self,
        product_id: &str,
        service_id: Option<&str>,
        current_commit_num: i64,
    ) -> Result<(), ProductStoreError>;

    fn delete_product(
        &self,
        address: &str,
        current_commit_num: i64,
    ) -> Result<(), ProductStoreError>;
}
