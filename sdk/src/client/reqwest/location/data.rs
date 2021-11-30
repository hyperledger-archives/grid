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

//! This module provides the data types for the reqwest-backed client
//! implementation. These must be able to be converted into their
//! corresponding structs in the corresponding client module.

use crate::client::location::{
    LatLong as ClientLatLong, Location as ClientLocation,
    LocationPropertyValue as ClientLocationPropertyValue,
};
use crate::client::reqwest::schema::data::DataType;
use crate::client::schema::DataType as ClientDataType;

#[derive(Debug, Deserialize)]
pub struct Location {
    pub location_id: String,
    pub location_namespace: String,
    pub owner: String,
    pub properties: Vec<LocationPropertyValue>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_id: Option<String>,
}

impl From<&Location> for ClientLocation {
    fn from(d: &Location) -> Self {
        Self {
            location_id: d.location_id.to_string(),
            location_namespace: d.location_namespace.to_string(),
            owner: d.owner.to_string(),
            properties: d
                .properties
                .iter()
                .map(ClientLocationPropertyValue::from)
                .collect(),
            service_id: d.service_id.as_ref().map(String::from),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct LocationPropertyValue {
    pub name: String,
    pub data_type: DataType,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_id: Option<String>,
    pub bytes_value: Option<Vec<u8>>,
    pub boolean_value: Option<bool>,
    pub number_value: Option<i64>,
    pub string_value: Option<String>,
    pub enum_value: Option<i32>,
    pub struct_values: Option<Vec<String>>,
    pub lat_long_value: Option<LatLong>,
}

impl From<&LocationPropertyValue> for ClientLocationPropertyValue {
    fn from(d: &LocationPropertyValue) -> Self {
        Self {
            name: d.name.to_string(),
            data_type: ClientDataType::from(&d.data_type),
            service_id: d.service_id.as_ref().map(String::from),
            bytes_value: d.bytes_value.as_ref().map(|x| x.to_vec()),
            boolean_value: d.boolean_value,
            number_value: d.number_value,
            string_value: d.string_value.as_ref().map(String::from),
            enum_value: d.enum_value,
            struct_values: d
                .struct_values
                .as_ref()
                .map(|s| s.iter().map(String::from).collect()),
            lat_long_value: d.lat_long_value.as_ref().map(ClientLatLong::from),
        }
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
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
