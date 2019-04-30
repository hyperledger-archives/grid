// Copyright 2019 Cargill Incorporated
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

use crate::database::{
    helpers as db,
    models::{GridPropertyDefinition, GridSchema},
};
use crate::rest_api::{error::RestApiResponseError, routes::DbExecutor};

use actix::{Handler, Message, SyncContext};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct GridSchemaSlice {
    pub name: String,
    pub description: String,
    pub owner: String,
    pub properties: Vec<GridPropertyDefinitionSlice>,
}

impl GridSchemaSlice {
    pub fn from_schema(schema: &GridSchema, properties: Vec<GridPropertyDefinition>) -> Self {
        Self {
            name: schema.name.clone(),
            description: schema.description.clone(),
            owner: schema.owner.clone(),
            properties: properties
                .iter()
                .map(|prop| GridPropertyDefinitionSlice::from_definition(prop))
                .collect(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GridPropertyDefinitionSlice {
    pub name: String,
    pub schema_name: String,
    pub data_type: String,
    pub required: bool,
    pub description: String,
    pub number_exponent: i64,
    pub enum_options: Vec<String>,
    pub struct_properties: Vec<String>,
}

impl GridPropertyDefinitionSlice {
    pub fn from_definition(definition: &GridPropertyDefinition) -> Self {
        Self {
            name: definition.name.clone(),
            schema_name: definition.schema_name.clone(),
            data_type: definition.data_type.clone(),
            required: definition.required,
            description: definition.description.clone(),
            number_exponent: definition.number_exponent,
            enum_options: definition.enum_options.clone(),
            struct_properties: definition.struct_properties.clone(),
        }
    }
}

struct ListGridSchemas;

impl Message for ListGridSchemas {
    type Result = Result<Vec<GridSchemaSlice>, RestApiResponseError>;
}

impl Handler<ListGridSchemas> for DbExecutor {
    type Result = Result<Vec<GridSchemaSlice>, RestApiResponseError>;

    fn handle(&mut self, _msg: ListGridSchemas, _: &mut SyncContext<Self>) -> Self::Result {
        let mut properties = db::list_grid_property_definitions(&*self.connection_pool.get()?)?
            .into_iter()
            .fold(HashMap::new(), |mut acc, definition| {
                acc.entry(definition.schema_name.to_string())
                    .or_insert_with(|| vec![])
                    .push(definition);
                acc
            });

        let fetched_schemas = db::list_grid_schemas(&*self.connection_pool.get()?)?
            .iter()
            .map(|schema| {
                GridSchemaSlice::from_schema(
                    schema,
                    properties.remove(&schema.name).unwrap_or_else(|| vec![]),
                )
            })
            .collect();
        Ok(fetched_schemas)
    }
}
