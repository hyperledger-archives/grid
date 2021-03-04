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

use crate::{
    locations::store::{LatLongValue, Location, LocationAttribute},
    rest_api::resources::paging::v1::Paging,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct LocationSlice {
    pub location_id: String,
    pub location_namespace: String,
    pub owner: String,
    pub properties: Vec<LocationPropertyValueSlice>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_id: Option<String>,
}

impl From<Location> for LocationSlice {
    fn from(location: Location) -> Self {
        Self {
            location_id: location.location_id,
            location_namespace: location.location_namespace,
            owner: location.owner,
            properties: location
                .attributes
                .into_iter()
                .map(LocationPropertyValueSlice::from)
                .collect(),
            service_id: location.service_id,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LocationListSlice {
    pub data: Vec<LocationSlice>,
    pub paging: Paging,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LocationPropertyValueSlice {
    pub name: String,
    pub data_type: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_id: Option<String>,
    pub bytes_value: Option<Vec<u8>>,
    pub boolean_value: Option<bool>,
    pub number_value: Option<i64>,
    pub string_value: Option<String>,
    pub enum_value: Option<i32>,
    pub struct_values: Option<Vec<LocationPropertyValueSlice>>,
    pub lat_long_value: Option<LatLongSlice>,
}

impl From<LocationAttribute> for LocationPropertyValueSlice {
    fn from(attribute: LocationAttribute) -> Self {
        Self {
            name: attribute.property_name,
            data_type: attribute.data_type,
            service_id: attribute.service_id,
            bytes_value: attribute.bytes_value,
            boolean_value: attribute.boolean_value,
            number_value: attribute.number_value,
            string_value: attribute.string_value.clone(),
            enum_value: attribute.enum_value,
            struct_values: attribute.struct_values.map(|attrs| {
                attrs
                    .into_iter()
                    .map(LocationPropertyValueSlice::from)
                    .collect()
            }),
            lat_long_value: attribute.lat_long_value.map(LatLongSlice::from),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LatLongSlice {
    pub latitude: i64,
    pub longitude: i64,
}

impl From<LatLongValue> for LatLongSlice {
    fn from(lat_long_value: LatLongValue) -> Self {
        Self {
            latitude: lat_long_value.0,
            longitude: lat_long_value.1,
        }
    }
}
