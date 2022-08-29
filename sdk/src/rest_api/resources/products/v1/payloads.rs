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
    product::store::{LatLongValue, Product, PropertyValue},
    rest_api::resources::paging::v1::Paging,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductSlice {
    pub product_id: String,
    pub product_address: String,
    pub product_namespace: String,
    pub owner: String,
    pub properties: Vec<ProductPropertyValueSlice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_updated: Option<i64>,
}

impl From<Product> for ProductSlice {
    fn from(product: Product) -> Self {
        Self {
            product_id: product.product_id().to_string(),
            product_address: product.product_address().to_string(),
            product_namespace: product.product_namespace().to_string(),
            owner: product.owner().to_string(),
            properties: product
                .properties()
                .into_iter()
                .map(ProductPropertyValueSlice::from)
                .collect(),
            service_id: product.service_id().map(String::from),
            last_updated: product.last_updated().cloned(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductListSlice {
    pub data: Vec<ProductSlice>,
    pub paging: Paging,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductPropertyValueSlice {
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
    pub struct_values: Vec<ProductPropertyValueSlice>,
    pub lat_long_value: Option<LatLongSlice>,
}

impl From<PropertyValue> for ProductPropertyValueSlice {
    fn from(property_value: PropertyValue) -> Self {
        Self {
            name: property_value.property_name().to_string(),
            data_type: property_value.data_type().to_string(),
            service_id: property_value.service_id().map(String::from),
            bytes_value: property_value.bytes_value(),
            boolean_value: property_value.boolean_value(),
            number_value: property_value.number_value(),
            string_value: property_value.string_value().map(String::from),
            enum_value: property_value.enum_value(),
            struct_values: property_value
                .struct_values()
                .into_iter()
                .map(ProductPropertyValueSlice::from)
                .collect(),
            lat_long_value: property_value.lat_long_value().map(LatLongSlice::from),
        }
    }
}

impl From<&PropertyValue> for ProductPropertyValueSlice {
    fn from(property_value: &PropertyValue) -> Self {
        Self {
            name: property_value.property_name().to_string(),
            data_type: property_value.data_type().to_string(),
            service_id: property_value.service_id().map(String::from),
            bytes_value: property_value.bytes_value(),
            boolean_value: property_value.boolean_value(),
            number_value: property_value.number_value(),
            string_value: property_value.string_value().map(String::from),
            enum_value: property_value.enum_value(),
            struct_values: property_value
                .struct_values()
                .into_iter()
                .map(ProductPropertyValueSlice::from)
                .collect(),
            lat_long_value: property_value.lat_long_value().map(LatLongSlice::from),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct LatLongSlice {
    pub latitude: i64,
    pub longitude: i64,
}

impl From<LatLongValue> for LatLongSlice {
    fn from(value: LatLongValue) -> Self {
        LatLongSlice {
            latitude: value.latitude,
            longitude: value.longitude,
        }
    }
}
