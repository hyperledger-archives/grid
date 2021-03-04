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
    rest_api::resources::paging::v1::Paging,
    schemas::store::{PropertyDefinition, Schema},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct SchemaSlice {
    pub name: String,
    pub description: String,
    pub owner: String,
    pub properties: Vec<PropertyDefinitionSlice>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SchemaListSlice {
    pub data: Vec<SchemaSlice>,
    pub paging: Paging,
}

impl From<Schema> for SchemaSlice {
    fn from(schema: Schema) -> Self {
        Self {
            name: schema.name.clone(),
            description: schema.description.clone(),
            owner: schema.owner.clone(),
            properties: schema
                .properties
                .into_iter()
                .map(PropertyDefinitionSlice::from)
                .collect(),
            service_id: schema.service_id,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PropertyDefinitionSlice {
    pub name: String,
    pub schema_name: String,
    pub data_type: String,
    pub required: bool,
    pub description: String,
    pub number_exponent: i64,
    pub enum_options: Vec<String>,
    pub struct_properties: Vec<PropertyDefinitionSlice>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_id: Option<String>,
}

impl From<PropertyDefinition> for PropertyDefinitionSlice {
    fn from(definition: PropertyDefinition) -> Self {
        Self {
            name: definition.name.clone(),
            schema_name: definition.schema_name.clone(),
            data_type: definition.data_type.clone(),
            required: definition.required,
            description: definition.description.clone(),
            number_exponent: definition.number_exponent,
            enum_options: definition.enum_options.clone(),
            struct_properties: definition
                .struct_properties
                .into_iter()
                .map(PropertyDefinitionSlice::from)
                .collect(),
            service_id: definition.service_id,
        }
    }
}
