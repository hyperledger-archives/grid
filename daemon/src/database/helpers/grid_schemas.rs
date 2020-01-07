/*
 * Copyright 2019 Cargill Incorporated
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * -----------------------------------------------------------------------------
 */

use super::models::{GridPropertyDefinition, GridSchema, NewGridPropertyDefinition, NewGridSchema};
use super::schema::{grid_property_definition, grid_schema};
use super::MAX_COMMIT_NUM;

use diesel::{
    dsl::{insert_into, update},
    pg::PgConnection,
    prelude::*,
    result::Error::NotFound,
    QueryResult,
};

pub fn insert_grid_schemas(conn: &PgConnection, schemas: &[NewGridSchema]) -> QueryResult<()> {
    for schema in schemas {
        update_grid_schema_end_commit_num(conn, &schema.name, schema.start_commit_num)?;
    }

    insert_into(grid_schema::table)
        .values(schemas)
        .execute(conn)
        .map(|_| ())
}

pub fn insert_grid_property_definitions(
    conn: &PgConnection,
    definitions: &[NewGridPropertyDefinition],
) -> QueryResult<()> {
    for definition in definitions {
        update_definition_end_commit_num(
            conn,
            &definition.name,
            &definition.schema_name,
            definition.start_commit_num,
        )?;
    }

    insert_into(grid_property_definition::table)
        .values(definitions)
        .execute(conn)
        .map(|_| ())
}

pub fn update_grid_schema_end_commit_num(
    conn: &PgConnection,
    name: &str,
    current_commit_num: i64,
) -> QueryResult<()> {
    update(grid_schema::table)
        .filter(
            grid_schema::name
                .eq(name)
                .and(grid_schema::end_commit_num.eq(MAX_COMMIT_NUM)),
        )
        .set(grid_schema::end_commit_num.eq(current_commit_num))
        .execute(conn)
        .map(|_| ())
}

pub fn update_definition_end_commit_num(
    conn: &PgConnection,
    name: &str,
    schema_name: &str,
    current_commit_num: i64,
) -> QueryResult<()> {
    update(grid_property_definition::table)
        .filter(
            grid_property_definition::schema_name
                .eq(schema_name)
                .and(grid_property_definition::name.eq(name))
                .and(grid_property_definition::end_commit_num.eq(MAX_COMMIT_NUM)),
        )
        .set(grid_property_definition::end_commit_num.eq(current_commit_num))
        .execute(conn)
        .map(|_| ())
}

pub fn list_grid_schemas(
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

    query.load::<GridSchema>(conn)
}

pub fn list_grid_property_definitions(
    conn: &PgConnection,
    service_id: Option<&str>,
) -> QueryResult<Vec<GridPropertyDefinition>> {
    let mut query = grid_property_definition::table
        .into_boxed()
        .select(grid_property_definition::all_columns)
        .filter(grid_property_definition::end_commit_num.eq(MAX_COMMIT_NUM));

    if let Some(service_id) = service_id {
        query = query.filter(grid_property_definition::service_id.eq(service_id));
    } else {
        query = query.filter(grid_property_definition::service_id.is_null());
    }

    query.load::<GridPropertyDefinition>(conn)
}

pub fn fetch_grid_schema(
    conn: &PgConnection,
    name: &str,
    service_id: Option<&str>,
) -> QueryResult<Option<GridSchema>> {
    let mut query = grid_schema::table
        .into_boxed()
        .select(grid_schema::all_columns)
        .filter(
            grid_schema::name
                .eq(name)
                .and(grid_schema::end_commit_num.eq(MAX_COMMIT_NUM)),
        );

    if let Some(service_id) = service_id {
        query = query.filter(grid_schema::service_id.eq(service_id));
    } else {
        query = query.filter(grid_schema::service_id.is_null());
    }

    query
        .first(conn)
        .map(Some)
        .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
}

pub fn list_grid_property_definitions_with_schema_name(
    conn: &PgConnection,
    schema_name: &str,
    service_id: Option<&str>,
) -> QueryResult<Vec<GridPropertyDefinition>> {
    let mut query = grid_property_definition::table
        .into_boxed()
        .select(grid_property_definition::all_columns)
        .filter(
            grid_property_definition::schema_name
                .eq(schema_name)
                .and(grid_property_definition::end_commit_num.eq(MAX_COMMIT_NUM)),
        );

    if let Some(service_id) = service_id {
        query = query.filter(grid_property_definition::service_id.eq(service_id));
    } else {
        query = query.filter(grid_property_definition::service_id.is_null());
    }
    query.load::<GridPropertyDefinition>(conn)
}
