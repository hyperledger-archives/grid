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

pub(in crate::grid_db) mod models;
mod operations;
pub(in crate::grid_db) mod schema;

use crate::database::DatabaseError;

use models::{GridPropertyDefinition, GridSchema, NewGridPropertyDefinition, NewGridSchema};
use operations::{
    add_schema::AddSchemaOperation, fetch_schema::FetchSchemaOperation,
    list_schemas::ListSchemasOperation, SchemaStoreOperations,
};

use diesel::r2d2::{ConnectionManager, Pool};

use super::{PropertyDefinition, Schema, SchemaStore, SchemaStoreError};

/// Manages creating commits in the database
#[derive(Clone)]
pub struct DieselSchemaStore<C: diesel::Connection + 'static> {
    connection_pool: Pool<ConnectionManager<C>>,
}

impl<C: diesel::Connection> DieselSchemaStore<C> {
    #[allow(dead_code)]
    pub fn new(connection_pool: Pool<ConnectionManager<C>>) -> Self {
        DieselSchemaStore { connection_pool }
    }
}

#[cfg(feature = "postgres")]
impl SchemaStore for DieselSchemaStore<diesel::pg::PgConnection> {
    fn add_schema(&self, schema: &Schema) -> Result<(), SchemaStoreError> {
        SchemaStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            DatabaseError::ConnectionError {
                context: "Could not get connection pool".to_string(),
                source: Box::new(err),
            }
        })?)
        .add_schema(schema)
    }

    fn fetch_schema(
        &self,
        name: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Schema>, SchemaStoreError> {
        SchemaStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            DatabaseError::ConnectionError {
                context: "Could not get connection pool".to_string(),
                source: Box::new(err),
            }
        })?)
        .fetch_schema(name, service_id)
    }

    fn list_schemas(&self, service_id: Option<&str>) -> Result<Vec<Schema>, SchemaStoreError> {
        SchemaStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            DatabaseError::ConnectionError {
                context: "Could not get connection pool".to_string(),
                source: Box::new(err),
            }
        })?)
        .list_schemas(service_id)
    }
}

impl Into<(NewGridSchema, Vec<NewGridPropertyDefinition>)> for &Schema {
    fn into(self) -> (NewGridSchema, Vec<NewGridPropertyDefinition>) {
        let schema = NewGridSchema {
            name: self.name.clone(),
            description: self.description.clone(),
            owner: self.owner.clone(),
            service_id: self.service_id.clone(),
            start_commit_num: self.start_commit_num,
            end_commit_num: self.end_commit_num,
        };

        let properties = make_property_definitions(&self.properties, None);

        (schema, properties)
    }
}

impl From<(GridSchema, Vec<PropertyDefinition>)> for Schema {
    fn from((model, properties): (GridSchema, Vec<PropertyDefinition>)) -> Self {
        Self {
            name: model.name,
            description: model.description,
            owner: model.owner,
            properties,
            service_id: model.service_id,
            start_commit_num: model.start_commit_num,
            end_commit_num: model.end_commit_num,
        }
    }
}

fn make_property_definitions(
    definitions: &[PropertyDefinition],
    parent_name: Option<String>,
) -> Vec<NewGridPropertyDefinition> {
    let mut properties = Vec::new();

    for def in definitions {
        properties.push(NewGridPropertyDefinition {
            name: def.name.to_string(),
            schema_name: def.schema_name.to_string(),
            data_type: format!("{:?}", def.data_type),
            required: def.required,
            description: def.description.to_string(),
            number_exponent: def.number_exponent,
            enum_options: def.enum_options.join(","),
            parent_name: parent_name.clone(),
            start_commit_num: def.start_commit_num,
            end_commit_num: def.end_commit_num,
            service_id: def.service_id.clone(),
        });

        if !def.struct_properties.is_empty() {
            properties.append(&mut make_property_definitions(
                &def.struct_properties,
                Some(def.name.clone()),
            ));
        }
    }

    properties
}

impl From<GridPropertyDefinition> for PropertyDefinition {
    fn from(model: GridPropertyDefinition) -> Self {
        Self {
            start_commit_num: model.start_commit_num,
            end_commit_num: model.end_commit_num,
            name: model.name,
            schema_name: model.schema_name,
            data_type: model.data_type,
            required: model.required,
            description: model.description,
            number_exponent: model.number_exponent,
            enum_options: model.enum_options.split(',').map(String::from).collect(),
            struct_properties: vec![],
            service_id: model.service_id,
        }
    }
}

impl From<(GridPropertyDefinition, Vec<PropertyDefinition>)> for PropertyDefinition {
    fn from((model, children): (GridPropertyDefinition, Vec<PropertyDefinition>)) -> Self {
        Self {
            start_commit_num: model.start_commit_num,
            end_commit_num: model.end_commit_num,
            name: model.name,
            schema_name: model.schema_name,
            data_type: model.data_type,
            required: model.required,
            description: model.description,
            number_exponent: model.number_exponent,
            enum_options: model.enum_options.split(',').map(String::from).collect(),
            struct_properties: children,
            service_id: model.service_id,
        }
    }
}

impl From<DatabaseError> for SchemaStoreError {
    fn from(err: DatabaseError) -> SchemaStoreError {
        SchemaStoreError::ConnectionError(Box::new(err))
    }
}

impl From<diesel::result::Error> for SchemaStoreError {
    fn from(err: diesel::result::Error) -> SchemaStoreError {
        SchemaStoreError::QueryError {
            context: "Diesel query failed".to_string(),
            source: Box::new(err),
        }
    }
}

impl From<diesel::r2d2::PoolError> for SchemaStoreError {
    fn from(err: diesel::r2d2::PoolError) -> SchemaStoreError {
        SchemaStoreError::ConnectionError(Box::new(err))
    }
}
