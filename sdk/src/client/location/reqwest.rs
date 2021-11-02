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

use crate::client::location::{LatLong, Location, LocationClient, LocationPropertyValue};
use crate::client::reqwest::{fetch_entities_list, fetch_entity, post_batches};
use crate::client::schema::{reqwest::DataTypeDto, DataType};
use crate::client::Client;
use crate::error::ClientError;

use sawtooth_sdk::messages::batch::BatchList;

const LOCATION_ROUTE: &str = "location";

#[derive(Debug, Deserialize)]
struct LocationDto {
    pub location_id: String,
    pub location_namespace: String,
    pub owner: String,
    pub properties: Vec<LocationPropertyValueDto>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_id: Option<String>,
}

impl From<&LocationDto> for Location {
    fn from(d: &LocationDto) -> Self {
        Self {
            location_id: d.location_id.to_string(),
            location_namespace: d.location_namespace.to_string(),
            owner: d.owner.to_string(),
            properties: d
                .properties
                .iter()
                .map(LocationPropertyValue::from)
                .collect(),
            service_id: d.service_id.as_ref().map(String::from),
        }
    }
}

#[derive(Debug, Deserialize)]
struct LocationPropertyValueDto {
    pub name: String,
    pub data_type: DataTypeDto,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_id: Option<String>,
    pub bytes_value: Option<Vec<u8>>,
    pub boolean_value: Option<bool>,
    pub number_value: Option<i64>,
    pub string_value: Option<String>,
    pub enum_value: Option<i32>,
    pub struct_values: Option<Vec<String>>,
    pub lat_long_value: Option<LatLongDto>,
}

impl From<&LocationPropertyValueDto> for LocationPropertyValue {
    fn from(d: &LocationPropertyValueDto) -> Self {
        Self {
            name: d.name.to_string(),
            data_type: DataType::from(&d.data_type),
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
            lat_long_value: d.lat_long_value.as_ref().map(LatLong::from),
        }
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
struct LatLongDto {
    pub latitude: i64,
    pub longitude: i64,
}

impl From<&LatLongDto> for LatLong {
    fn from(d: &LatLongDto) -> Self {
        Self {
            latitude: d.latitude,
            longitude: d.longitude,
        }
    }
}

/// The Reqwest implementation of the Location client
pub struct ReqwestLocationClient {
    url: String,
}

impl ReqwestLocationClient {
    pub fn new(url: String) -> Self {
        Self { url }
    }
}

impl Client for ReqwestLocationClient {
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

impl LocationClient for ReqwestLocationClient {
    /// Fetches a location based on its identifier
    ///
    /// # Arguments
    ///
    /// * `id` - the location's identifier
    /// * `service_id` - optional - the service ID to fetch the location from
    fn get_location(&self, id: String, service_id: Option<&str>) -> Result<Location, ClientError> {
        let dto = fetch_entity::<LocationDto>(
            &self.url,
            format!("{}/{}", LOCATION_ROUTE, id),
            service_id,
        )?;
        Ok(Location::from(&dto))
    }

    /// Fetches locations
    ///
    /// # Arguments
    ///
    /// * `service_id` - optional - the service ID to fetch locations from
    fn list_locations(&self, service_id: Option<&str>) -> Result<Vec<Location>, ClientError> {
        let dto_vec = fetch_entities_list::<LocationDto>(
            &self.url,
            LOCATION_ROUTE.to_string(),
            service_id,
            None,
        )?;
        Ok(dto_vec.iter().map(Location::from).collect())
    }
}
