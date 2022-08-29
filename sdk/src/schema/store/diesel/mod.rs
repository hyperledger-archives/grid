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

pub(in crate::schema) mod models;
mod operations;
pub(crate) mod schema;

use crate::error::ResourceTemporarilyUnavailableError;

use models::{GridPropertyDefinition, GridSchema, NewGridPropertyDefinition, NewGridSchema};
use operations::{
    add_schema::AddSchemaOperation,
    get_property_definition_by_name::GetPropertyDefinitionByNameOperation,
    get_schema::GetSchemaOperation, list_property_definitions::ListPropertyDefinitionsOperation,
    list_property_definitions_with_schema_name::ListPropertyDefinitionsWithSchemaNameOperation,
    list_schemas::ListSchemasOperation, SchemaStoreOperations,
};

use diesel::connection::AnsiTransactionManager;
use diesel::r2d2::{ConnectionManager, Pool};

use super::{PropertyDefinition, Schema, SchemaList, SchemaStore, SchemaStoreError};

/// Manages creating commits in the database
#[derive(Clone)]
pub struct DieselSchemaStore<C: diesel::Connection + 'static> {
    connection_pool: Pool<ConnectionManager<C>>,
}

impl<C: diesel::Connection> DieselSchemaStore<C> {
    pub fn new(connection_pool: Pool<ConnectionManager<C>>) -> Self {
        DieselSchemaStore { connection_pool }
    }
}

#[cfg(feature = "postgres")]
impl SchemaStore for DieselSchemaStore<diesel::pg::PgConnection> {
    fn add_schema(&self, schema: Schema) -> Result<(), SchemaStoreError> {
        SchemaStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            SchemaStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .add_schema(schema)
    }

    fn get_schema(
        &self,
        name: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Schema>, SchemaStoreError> {
        SchemaStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            SchemaStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .get_schema(name, service_id)
    }

    fn list_schemas(
        &self,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<SchemaList, SchemaStoreError> {
        SchemaStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            SchemaStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .list_schemas(service_id, offset, limit)
    }

    fn list_property_definitions(
        &self,
        service_id: Option<&str>,
    ) -> Result<Vec<PropertyDefinition>, SchemaStoreError> {
        SchemaStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            SchemaStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .list_property_definitions(service_id)
    }

    fn list_property_definitions_with_schema_name(
        &self,
        schema_name: &str,
        service_id: Option<&str>,
    ) -> Result<Vec<PropertyDefinition>, SchemaStoreError> {
        SchemaStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            SchemaStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .list_property_definitions_with_schema_name(schema_name, service_id)
    }

    fn get_property_definition_by_name(
        &self,
        schema_name: &str,
        definition_name: &str,
        service_id: Option<&str>,
    ) -> Result<Option<PropertyDefinition>, SchemaStoreError> {
        SchemaStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            SchemaStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .get_property_definition_by_name(schema_name, definition_name, service_id)
    }
}

#[cfg(feature = "sqlite")]
impl SchemaStore for DieselSchemaStore<diesel::sqlite::SqliteConnection> {
    fn add_schema(&self, schema: Schema) -> Result<(), SchemaStoreError> {
        SchemaStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            SchemaStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .add_schema(schema)
    }

    fn get_schema(
        &self,
        name: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Schema>, SchemaStoreError> {
        SchemaStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            SchemaStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .get_schema(name, service_id)
    }

    fn list_schemas(
        &self,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<SchemaList, SchemaStoreError> {
        SchemaStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            SchemaStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .list_schemas(service_id, offset, limit)
    }

    fn list_property_definitions(
        &self,
        service_id: Option<&str>,
    ) -> Result<Vec<PropertyDefinition>, SchemaStoreError> {
        SchemaStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            SchemaStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .list_property_definitions(service_id)
    }

    fn list_property_definitions_with_schema_name(
        &self,
        schema_name: &str,
        service_id: Option<&str>,
    ) -> Result<Vec<PropertyDefinition>, SchemaStoreError> {
        SchemaStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            SchemaStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .list_property_definitions_with_schema_name(schema_name, service_id)
    }

    fn get_property_definition_by_name(
        &self,
        schema_name: &str,
        definition_name: &str,
        service_id: Option<&str>,
    ) -> Result<Option<PropertyDefinition>, SchemaStoreError> {
        SchemaStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            SchemaStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .get_property_definition_by_name(schema_name, definition_name, service_id)
    }
}

pub struct DieselConnectionSchemaStore<'a, C>
where
    C: diesel::Connection<TransactionManager = AnsiTransactionManager> + 'static,
    C::Backend: diesel::backend::UsesAnsiSavepointSyntax,
{
    connection: &'a C,
}

impl<'a, C> DieselConnectionSchemaStore<'a, C>
where
    C: diesel::Connection<TransactionManager = AnsiTransactionManager> + 'static,
    C::Backend: diesel::backend::UsesAnsiSavepointSyntax,
{
    pub fn new(connection: &'a C) -> Self {
        DieselConnectionSchemaStore { connection }
    }
}

#[cfg(feature = "postgres")]
impl<'a> SchemaStore for DieselConnectionSchemaStore<'a, diesel::pg::PgConnection> {
    fn add_schema(&self, schema: Schema) -> Result<(), SchemaStoreError> {
        SchemaStoreOperations::new(self.connection).add_schema(schema)
    }

    fn get_schema(
        &self,
        name: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Schema>, SchemaStoreError> {
        SchemaStoreOperations::new(self.connection).get_schema(name, service_id)
    }

    fn list_schemas(
        &self,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<SchemaList, SchemaStoreError> {
        SchemaStoreOperations::new(self.connection).list_schemas(service_id, offset, limit)
    }

    fn list_property_definitions(
        &self,
        service_id: Option<&str>,
    ) -> Result<Vec<PropertyDefinition>, SchemaStoreError> {
        SchemaStoreOperations::new(self.connection).list_property_definitions(service_id)
    }

    fn list_property_definitions_with_schema_name(
        &self,
        schema_name: &str,
        service_id: Option<&str>,
    ) -> Result<Vec<PropertyDefinition>, SchemaStoreError> {
        SchemaStoreOperations::new(self.connection)
            .list_property_definitions_with_schema_name(schema_name, service_id)
    }

    fn get_property_definition_by_name(
        &self,
        schema_name: &str,
        definition_name: &str,
        service_id: Option<&str>,
    ) -> Result<Option<PropertyDefinition>, SchemaStoreError> {
        SchemaStoreOperations::new(self.connection).get_property_definition_by_name(
            schema_name,
            definition_name,
            service_id,
        )
    }
}

#[cfg(feature = "sqlite")]
impl<'a> SchemaStore for DieselConnectionSchemaStore<'a, diesel::sqlite::SqliteConnection> {
    fn add_schema(&self, schema: Schema) -> Result<(), SchemaStoreError> {
        SchemaStoreOperations::new(self.connection).add_schema(schema)
    }

    fn get_schema(
        &self,
        name: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Schema>, SchemaStoreError> {
        SchemaStoreOperations::new(self.connection).get_schema(name, service_id)
    }

    fn list_schemas(
        &self,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<SchemaList, SchemaStoreError> {
        SchemaStoreOperations::new(self.connection).list_schemas(service_id, offset, limit)
    }

    fn list_property_definitions(
        &self,
        service_id: Option<&str>,
    ) -> Result<Vec<PropertyDefinition>, SchemaStoreError> {
        SchemaStoreOperations::new(self.connection).list_property_definitions(service_id)
    }

    fn list_property_definitions_with_schema_name(
        &self,
        schema_name: &str,
        service_id: Option<&str>,
    ) -> Result<Vec<PropertyDefinition>, SchemaStoreError> {
        SchemaStoreOperations::new(self.connection)
            .list_property_definitions_with_schema_name(schema_name, service_id)
    }

    fn get_property_definition_by_name(
        &self,
        schema_name: &str,
        definition_name: &str,
        service_id: Option<&str>,
    ) -> Result<Option<PropertyDefinition>, SchemaStoreError> {
        SchemaStoreOperations::new(self.connection).get_property_definition_by_name(
            schema_name,
            definition_name,
            service_id,
        )
    }
}

impl From<Schema> for (NewGridSchema, Vec<NewGridPropertyDefinition>) {
    fn from(schema: Schema) -> Self {
        let new_schema = NewGridSchema {
            name: schema.name.clone(),
            description: schema.description.clone(),
            owner: schema.owner.clone(),
            service_id: schema.service_id.clone(),
            start_commit_num: schema.start_commit_num,
            end_commit_num: schema.end_commit_num,
        };

        let properties = make_property_definitions(&schema.properties, None);

        (new_schema, properties)
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
            last_updated: model.last_updated.map(|d| d.timestamp()),
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
            data_type: def.data_type.clone(),
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
