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

use crate::client::product::{
    LatLong as ClientLatLong, Product as ClientProduct, PropertyValue as ClientPropertyValue,
};

#[derive(Debug, Deserialize)]
pub struct Product {
    pub product_id: String,
    pub product_namespace: String,
    pub owner: String,
    pub properties: Vec<PropertyValue>,
}

impl From<&Product> for ClientProduct {
    fn from(d: &Product) -> Self {
        Self {
            product_id: d.product_id.to_string(),
            product_namespace: d.product_namespace.to_string(),
            owner: d.owner.to_string(),
            properties: d.properties.iter().map(ClientPropertyValue::from).collect(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
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

impl From<&PropertyValue> for ClientPropertyValue {
    fn from(d: &PropertyValue) -> Self {
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
            lat_long_value: d.lat_long_value.as_ref().map(ClientLatLong::from),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct LatLong {
    pub latitude: i64,
    pub longitude: i64,
}

impl From<&LatLong> for ClientLatLong {
    fn from(d: &LatLong) -> Self {
        Self {
            latitude: d.latitude,
            longitude: d.longitude,
        }
    }
}
