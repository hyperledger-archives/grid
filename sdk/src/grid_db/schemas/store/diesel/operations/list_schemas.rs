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

use crate::grid_db::schemas::{
    store::{
        diesel::{
            models::{GridPropertyDefinition, GridSchema},
            schema::{grid_property_definition, grid_schema},
        },
        error::SchemaStoreError,
        PropertyDefinition, Schema,
    },
    MAX_COMMIT_NUM,
};
use diesel::prelude::*;

pub(in crate::grid_db::schemas) trait ListSchemasOperation {
    fn list_schemas(&self, service_id: Option<&str>) -> Result<Vec<Schema>, SchemaStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> ListSchemasOperation for SchemaStoreOperations<'a, diesel::pg::PgConnection> {
    fn list_schemas(&self, service_id: Option<&str>) -> Result<Vec<Schema>, SchemaStoreError> {
        let db_schemas = pg::fetch_grid_schemas(&*self.conn, service_id)?;

        let mut schemas = Vec::new();

        for schema in db_schemas {
            let roots = pg::get_root_definitions(&*self.conn, &schema.name)?;

            let properties = pg::get_property_definitions_for_schema(&*self.conn, roots)?;

            schemas.push(Schema::from((schema, properties)));
        }

        Ok(schemas)
    }
}

#[cfg(feature = "sqlite")]
impl<'a> ListSchemasOperation for SchemaStoreOperations<'a, diesel::sqlite::SqliteConnection> {
    fn list_schemas(&self, service_id: Option<&str>) -> Result<Vec<Schema>, SchemaStoreError> {
        let db_schemas = sqlite::fetch_grid_schemas(&*self.conn, service_id)?;

        let mut schemas = Vec::new();

        for schema in db_schemas {
            let roots = sqlite::get_root_definitions(&*self.conn, &schema.name)?;

            let properties = sqlite::get_property_definitions_for_schema(&*self.conn, roots)?;

            schemas.push(Schema::from((schema, properties)));
        }

        Ok(schemas)
    }
}

#[cfg(feature = "postgres")]
mod pg {
    use super::*;

    pub fn fetch_grid_schemas(
        conn: &PgConnection,
        service_id: Option<&str>,
    ) -> QueryResult<Vec<GridSchema>> {
        let mut query = grid_schema::table
            .into_boxed()
            .select(grid_schema::all_columns)
            .filter(grid_schema::end_commit_num.eq(MAX_COMMIT_NUM));

        if let Some(service_id) = service_id {
            query = query.filter(grid_schema::service_id.eq(service_id));
        } else {
            query = query.filter(grid_schema::service_id.is_null());
        }

        query.load(conn)
    }

    pub fn get_root_definitions(
        conn: &PgConnection,
        schema_name: &str,
    ) -> QueryResult<Vec<GridPropertyDefinition>> {
        grid_property_definition::table
            .select(grid_property_definition::all_columns)
            .filter(
                grid_property_definition::schema_name
                    .eq(schema_name)
                    .and(grid_property_definition::parent_name.is_null())
                    .and(grid_property_definition::end_commit_num.eq(MAX_COMMIT_NUM)),
            )
            .load::<GridPropertyDefinition>(conn)
    }

    pub fn get_property_definitions_for_schema(
        conn: &PgConnection,
        root_definitions: Vec<GridPropertyDefinition>,
    ) -> Result<Vec<PropertyDefinition>, SchemaStoreError> {
        let mut definitions = Vec::new();

        for root_def in root_definitions {
            let children = grid_property_definition::table
                .select(grid_property_definition::all_columns)
                .filter(grid_property_definition::parent_name.eq(&root_def.name))
                .load(conn)?;

            if children.is_empty() {
                definitions.push(PropertyDefinition::from(root_def));
            } else {
                definitions.push(PropertyDefinition::from((
                    root_def,
                    get_property_definitions_for_schema(conn, children)?,
                )));
            }
        }

        Ok(definitions)
    }
}

#[cfg(feature = "sqlite")]
mod sqlite {
    use super::*;

    pub fn fetch_grid_schemas(
        conn: &SqliteConnection,
        service_id: Option<&str>,
    ) -> QueryResult<Vec<GridSchema>> {
        let mut query = grid_schema::table
            .into_boxed()
            .select(grid_schema::all_columns)
            .filter(grid_schema::end_commit_num.eq(MAX_COMMIT_NUM));

        if let Some(service_id) = service_id {
            query = query.filter(grid_schema::service_id.eq(service_id));
        } else {
            query = query.filter(grid_schema::service_id.is_null());
        }

        query.load(conn)
    }

    pub fn get_root_definitions(
        conn: &SqliteConnection,
        schema_name: &str,
    ) -> QueryResult<Vec<GridPropertyDefinition>> {
        grid_property_definition::table
            .select(grid_property_definition::all_columns)
            .filter(
                grid_property_definition::schema_name
                    .eq(schema_name)
                    .and(grid_property_definition::parent_name.is_null())
                    .and(grid_property_definition::end_commit_num.eq(MAX_COMMIT_NUM)),
            )
            .load::<GridPropertyDefinition>(conn)
    }

    pub fn get_property_definitions_for_schema(
        conn: &SqliteConnection,
        root_definitions: Vec<GridPropertyDefinition>,
    ) -> Result<Vec<PropertyDefinition>, SchemaStoreError> {
        let mut definitions = Vec::new();

        for root_def in root_definitions {
            let children = grid_property_definition::table
                .select(grid_property_definition::all_columns)
                .filter(grid_property_definition::parent_name.eq(&root_def.name))
                .load(conn)?;

            if children.is_empty() {
                definitions.push(PropertyDefinition::from(root_def));
            } else {
                definitions.push(PropertyDefinition::from((
                    root_def,
                    get_property_definitions_for_schema(conn, children)?,
                )));
            }
        }

        Ok(definitions)
    }
}
