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

use crate::client::schema::{
    DataType as ClientDataType, PropertyDefinition as ClientPropertyDefinition,
    Schema as ClientSchema,
};

#[derive(Debug, Deserialize)]
pub struct Schema {
    pub name: String,
    pub description: String,
    pub owner: String,
    pub properties: Vec<PropertyDefinition>,
}

impl From<&Schema> for ClientSchema {
    fn from(d: &Schema) -> Self {
        Self {
            name: d.name.to_string(),
            description: d.description.to_string(),
            owner: d.owner.to_string(),
            properties: d
                .properties
                .iter()
                .map(ClientPropertyDefinition::from)
                .collect(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct PropertyDefinition {
    pub name: String,
    pub schema_name: String,
    pub data_type: DataType,
    pub required: bool,
    pub description: String,
    pub number_exponent: i64,
    pub enum_options: Vec<String>,
    pub struct_properties: Vec<PropertyDefinition>,
}

impl From<&PropertyDefinition> for ClientPropertyDefinition {
    fn from(d: &PropertyDefinition) -> Self {
        Self {
            name: d.name.to_string(),
            schema_name: d.schema_name.to_string(),
            data_type: ClientDataType::from(&d.data_type),
            required: d.required,
            description: d.description.to_string(),
            number_exponent: d.number_exponent,
            enum_options: d.enum_options.iter().map(String::from).collect(),
            struct_properties: d
                .struct_properties
                .iter()
                .map(ClientPropertyDefinition::from)
                .collect(),
        }
    }
}

#[derive(Deserialize, Debug)]
pub enum DataType {
    Bytes,
    Boolean,
    Number,
    String,
    Enum,
    Struct,
    LatLong,
}

impl From<&DataType> for ClientDataType {
    fn from(d: &DataType) -> Self {
        match *d {
            DataType::Bytes => ClientDataType::Bytes,
            DataType::Boolean => ClientDataType::Boolean,
            DataType::Number => ClientDataType::Number,
            DataType::String => ClientDataType::String,
            DataType::Enum => ClientDataType::Enum,
            DataType::Struct => ClientDataType::Struct,
            DataType::LatLong => ClientDataType::LatLong,
        }
    }
}
