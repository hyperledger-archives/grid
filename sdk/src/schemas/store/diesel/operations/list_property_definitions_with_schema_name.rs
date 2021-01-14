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

use super::SchemaStoreOperations;

use crate::error::InternalError;
use crate::schemas::{
    store::{
        diesel::{models::GridPropertyDefinition, schema::grid_property_definition},
        error::SchemaStoreError,
        PropertyDefinition,
    },
    MAX_COMMIT_NUM,
};
use diesel::prelude::*;

pub(in crate::schemas) trait ListPropertyDefinitionsWithSchemaNameOperation {
    fn list_property_definitions_with_schema_name(
        &self,
        schema_name: &str,
        service_id: Option<&str>,
    ) -> Result<Vec<PropertyDefinition>, SchemaStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> ListPropertyDefinitionsWithSchemaNameOperation
    for SchemaStoreOperations<'a, diesel::pg::PgConnection>
{
    fn list_property_definitions_with_schema_name(
        &self,
        schema_name: &str,
        service_id: Option<&str>,
    ) -> Result<Vec<PropertyDefinition>, SchemaStoreError> {
        let mut query = grid_property_definition::table
            .into_boxed()
            .select(grid_property_definition::all_columns)
            .filter(
                grid_property_definition::schema_name
                    .eq(&schema_name)
                    .and(grid_property_definition::end_commit_num.eq(MAX_COMMIT_NUM)),
            );

        if let Some(service_id) = service_id {
            query = query.filter(grid_property_definition::service_id.eq(service_id));
        } else {
            query = query.filter(grid_property_definition::service_id.is_null());
        }

        let defns = query
            .load::<GridPropertyDefinition>(self.conn)
            .map(Some)
            .map_err(|err| {
                SchemaStoreError::InternalError(InternalError::from_source(Box::new(err)))
            })?
            .ok_or_else(|| {
                SchemaStoreError::NotFoundError(format!(
                    "Could not get all definitions from storage for schema: {}",
                    schema_name,
                ))
            })?
            .into_iter()
            .map(PropertyDefinition::from)
            .collect();

        Ok(defns)
    }
}

#[cfg(feature = "sqlite")]
impl<'a> ListPropertyDefinitionsWithSchemaNameOperation
    for SchemaStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn list_property_definitions_with_schema_name(
        &self,
        schema_name: &str,
        service_id: Option<&str>,
    ) -> Result<Vec<PropertyDefinition>, SchemaStoreError> {
        let mut query = grid_property_definition::table
            .into_boxed()
            .select(grid_property_definition::all_columns)
            .filter(
                grid_property_definition::schema_name
                    .eq(&schema_name)
                    .and(grid_property_definition::end_commit_num.eq(MAX_COMMIT_NUM)),
            );

        if let Some(service_id) = service_id {
            query = query.filter(grid_property_definition::service_id.eq(service_id));
        } else {
            query = query.filter(grid_property_definition::service_id.is_null());
        }

        let defns = query
            .load::<GridPropertyDefinition>(self.conn)
            .map(Some)
            .map_err(|err| {
                SchemaStoreError::InternalError(InternalError::from_source(Box::new(err)))
            })?
            .ok_or_else(|| {
                SchemaStoreError::NotFoundError(format!(
                    "Could not get all definitions from storage for schema: {}",
                    schema_name,
                ))
            })?
            .into_iter()
            .map(PropertyDefinition::from)
            .collect();

        Ok(defns)
    }
}
