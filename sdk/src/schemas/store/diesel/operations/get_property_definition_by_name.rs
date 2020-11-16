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
use diesel::{prelude::*, result::Error::NotFound};

pub(in crate::schemas) trait GetPropertyDefinitionByNameOperation {
    fn get_property_definition_by_name(
        &self,
        schema_name: &str,
        definition_name: &str,
        service_id: Option<&str>,
    ) -> Result<Option<PropertyDefinition>, SchemaStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> GetPropertyDefinitionByNameOperation
    for SchemaStoreOperations<'a, diesel::pg::PgConnection>
{
    fn get_property_definition_by_name(
        &self,
        schema_name: &str,
        definition_name: &str,
        service_id: Option<&str>,
    ) -> Result<Option<PropertyDefinition>, SchemaStoreError> {
        let mut query = grid_property_definition::table
            .into_boxed()
            .select(grid_property_definition::all_columns)
            .filter(
                grid_property_definition::schema_name
                    .eq(&schema_name)
                    .and(grid_property_definition::name.eq(&definition_name))
                    .and(grid_property_definition::end_commit_num.eq(MAX_COMMIT_NUM)),
            );

        if let Some(service_id) = service_id {
            query = query.filter(grid_property_definition::service_id.eq(service_id));
        } else {
            query = query.filter(grid_property_definition::service_id.is_null());
        }

        let defn = query
            .first::<GridPropertyDefinition>(self.conn)
            .map(PropertyDefinition::from)
            .map(Some)
            .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
            .map_err(|err| {
                SchemaStoreError::InternalError(InternalError::from_source(Box::new(err)))
            })?;

        Ok(defn)
    }
}

#[cfg(feature = "sqlite")]
impl<'a> GetPropertyDefinitionByNameOperation
    for SchemaStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn get_property_definition_by_name(
        &self,
        schema_name: &str,
        definition_name: &str,
        service_id: Option<&str>,
    ) -> Result<Option<PropertyDefinition>, SchemaStoreError> {
        let mut query = grid_property_definition::table
            .into_boxed()
            .select(grid_property_definition::all_columns)
            .filter(
                grid_property_definition::schema_name
                    .eq(&schema_name)
                    .and(grid_property_definition::name.eq(&definition_name))
                    .and(grid_property_definition::end_commit_num.eq(MAX_COMMIT_NUM)),
            );

        if let Some(service_id) = service_id {
            query = query.filter(grid_property_definition::service_id.eq(service_id));
        } else {
            query = query.filter(grid_property_definition::service_id.is_null());
        }

        let defn = query
            .first::<GridPropertyDefinition>(self.conn)
            .map(PropertyDefinition::from)
            .map(Some)
            .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
            .map_err(|err| {
                SchemaStoreError::InternalError(InternalError::from_source(Box::new(err)))
            })?;

        Ok(defn)
    }
}
