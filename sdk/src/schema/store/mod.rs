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

#[cfg(feature = "diesel")]
pub(in crate) mod diesel;
mod error;

use crate::paging::Paging;

#[cfg(feature = "diesel")]
pub use self::diesel::DieselSchemaStore;
pub use error::SchemaStoreError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Schema {
    pub name: String,
    pub description: String,
    pub owner: String,
    pub properties: Vec<PropertyDefinition>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_id: Option<String>,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub last_updated: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyDefinition {
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub name: String,
    pub schema_name: String,
    pub data_type: String,
    pub required: bool,
    pub description: String,
    pub number_exponent: i64,
    pub enum_options: Vec<String>,
    pub struct_properties: Vec<PropertyDefinition>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_id: Option<String>,
}

#[derive(Clone, Debug)]
pub struct SchemaList {
    pub data: Vec<Schema>,
    pub paging: Paging,
}

impl SchemaList {
    pub fn new(data: Vec<Schema>, paging: Paging) -> Self {
        Self { data, paging }
    }
}

pub trait SchemaStore: Send + Sync {
    /// Adds a new schema to underlying storage
    ///
    /// # Arguments
    ///
    ///  * `schema` - The new schema to be added
    fn add_schema(&self, schema: Schema) -> Result<(), SchemaStoreError>;

    /// Retrieve a schema from the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `name` - Name of schema being fetched
    ///  * `service_id` - Service ID needed for when the source of the schema is a splinter circuit
    fn get_schema(
        &self,
        name: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Schema>, SchemaStoreError>;

    /// List all schemas in underlying storage
    ///
    /// # Arguments
    ///
    ///  * `service_id` - Service ID needed for when the source of the schema
    ///  is a splinter circuit
    ///  * `offset` - The index of the first in storage to retrieve
    ///  * `limit` - The number of items to retrieve from the offset
    fn list_schemas(
        &self,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<SchemaList, SchemaStoreError>;

    /// List all property definitions in underlying storage
    ///
    /// # Arguments
    ///
    ///  * `service_id` - Service ID needed for when the source of the schema is a splinter circuit
    fn list_property_definitions(
        &self,
        service_id: Option<&str>,
    ) -> Result<Vec<PropertyDefinition>, SchemaStoreError>;

    /// List all schemas in underlying storage for a particular schema
    ///
    /// # Arguments
    ///
    ///  * `schema_name` - The name of the schema to list property definitions for
    ///  * `service_id` - Service ID needed for when the source of the schema is a splinter circuit
    fn list_property_definitions_with_schema_name(
        &self,
        schema_name: &str,
        service_id: Option<&str>,
    ) -> Result<Vec<PropertyDefinition>, SchemaStoreError>;

    /// Get a particular property definition for a particular schema
    ///
    /// # Arguments
    ///
    ///  * `schema_name` - The name of the schema to list property definitions for
    ///  * `definition_name` - The name of the property definition to fetch
    ///  * `service_id` - Service ID needed for when the source of the schema is a splinter circuit
    fn get_property_definition_by_name(
        &self,
        schema_name: &str,
        definition_name: &str,
        service_id: Option<&str>,
    ) -> Result<Option<PropertyDefinition>, SchemaStoreError>;
}

impl<SS> SchemaStore for Box<SS>
where
    SS: SchemaStore + ?Sized,
{
    fn add_schema(&self, schema: Schema) -> Result<(), SchemaStoreError> {
        (**self).add_schema(schema)
    }

    fn get_schema(
        &self,
        name: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Schema>, SchemaStoreError> {
        (**self).get_schema(name, service_id)
    }

    fn list_schemas(
        &self,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<SchemaList, SchemaStoreError> {
        (**self).list_schemas(service_id, offset, limit)
    }

    fn list_property_definitions(
        &self,
        service_id: Option<&str>,
    ) -> Result<Vec<PropertyDefinition>, SchemaStoreError> {
        (**self).list_property_definitions(service_id)
    }

    fn list_property_definitions_with_schema_name(
        &self,
        schema_name: &str,
        service_id: Option<&str>,
    ) -> Result<Vec<PropertyDefinition>, SchemaStoreError> {
        (**self).list_property_definitions_with_schema_name(schema_name, service_id)
    }

    fn get_property_definition_by_name(
        &self,
        schema_name: &str,
        definition_name: &str,
        service_id: Option<&str>,
    ) -> Result<Option<PropertyDefinition>, SchemaStoreError> {
        (**self).get_property_definition_by_name(schema_name, definition_name, service_id)
    }
}
