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

use super::schema::{product, product_property_value};

#[derive(Clone, Insertable, Debug)]
#[table_name = "product"]
pub struct NewProduct {
    pub product_id: String,
    pub product_address: String,
    pub product_namespace: String,
    pub owner: String,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Queryable, Identifiable, Debug)]
#[table_name = "product"]
pub struct Product {
    pub id: i64,
    pub product_id: String,
    pub product_address: String,
    pub product_namespace: String,
    pub owner: String,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(AsChangeset, Clone, Insertable, Debug)]
#[table_name = "product_property_value"]
pub struct NewProductPropertyValue {
    pub product_id: String,
    pub product_address: String,
    pub property_name: String,
    pub parent_property: Option<String>,
    pub data_type: String,
    pub bytes_value: Option<Vec<u8>>,
    pub boolean_value: Option<bool>,
    pub number_value: Option<i64>,
    pub string_value: Option<String>,
    pub enum_value: Option<i32>,
    pub latitude_value: Option<i64>,
    pub longitude_value: Option<i64>,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Queryable, Identifiable, Debug)]
#[table_name = "product_property_value"]
pub struct ProductPropertyValue {
    pub id: i64,
    pub product_id: String,
    pub product_address: String,
    pub property_name: String,
    pub parent_property: Option<String>,
    pub data_type: String,
    pub bytes_value: Option<Vec<u8>>,
    pub boolean_value: Option<bool>,
    pub number_value: Option<i64>,
    pub string_value: Option<String>,
    pub enum_value: Option<i32>,
    pub latitude_value: Option<i64>,
    pub longitude_value: Option<i64>,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}
