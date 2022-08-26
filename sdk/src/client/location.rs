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

use super::{schema::DataType, Client};

/// The client representation of a Grid Location
#[derive(Debug, PartialEq, Eq)]
pub struct Location {
    pub location_id: String,
    pub location_namespace: String,
    pub owner: String,
    pub properties: Vec<LocationPropertyValue>,
    pub service_id: Option<String>,
}

/// The client representation of a Grid Location property value
#[derive(Debug, PartialEq, Eq)]
pub struct LocationPropertyValue {
    pub name: String,
    pub data_type: DataType,
    pub service_id: Option<String>,
    pub bytes_value: Option<Vec<u8>>,
    pub boolean_value: Option<bool>,
    pub number_value: Option<i64>,
    pub string_value: Option<String>,
    pub enum_value: Option<i32>,
    pub struct_values: Option<Vec<String>>,
    pub lat_long_value: Option<LatLong>,
}

/// The client representation of a Grid Location lat/long value
#[derive(Debug, PartialEq, Eq)]
pub struct LatLong {
    pub latitude: i64,
    pub longitude: i64,
}

pub trait LocationClient: Client {
    /// Fetches an agent based on its identified
    ///
    /// # Arguments
    ///
    /// * `id` - the location's identifier
    /// * `service_id` - optional - the service ID to fetch the location from
    fn get_location(&self, id: String, service_id: Option<&str>) -> Result<Location, ClientError>;

    /// Fetches locations
    ///
    /// # Arguments
    ///
    /// * `service_id` - optional - the service ID to fetch locations from
    fn list_locations(&self, service_id: Option<&str>) -> Result<Vec<Location>, ClientError>;
}
