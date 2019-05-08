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

use super::models::{
    GridPropertyDefinition, GridSchema, NewGridPropertyDefinition, NewGridPropertyValue,
    NewGridSchema,
};
use super::schema::{grid_property_definition, grid_property_value, grid_schema};
use super::MAX_BLOCK_NUM;

use diesel::{
    dsl::{insert_into, update},
    pg::PgConnection,
    prelude::*,
    result::Error::NotFound,
    QueryResult,
};

pub fn insert_grid_schemas(conn: &PgConnection, schemas: &[NewGridSchema]) -> QueryResult<()> {
    for schema in schemas {
        update_grid_schema_end_block_num(conn, &schema.name, schema.start_block_num)?;
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
        update_definition_end_block_num(conn, &definition.name, definition.start_block_num)?;
    }

    insert_into(grid_property_definition::table)
        .values(definitions)
        .execute(conn)
        .map(|_| ())
}

pub fn update_grid_schema_end_block_num(
    conn: &PgConnection,
    name: &str,
    current_block_num: i64,
) -> QueryResult<()> {
    update(grid_schema::table)
        .filter(
            grid_schema::name
                .eq(name)
                .and(grid_schema::end_block_num.eq(MAX_BLOCK_NUM)),
        )
        .set(grid_schema::end_block_num.eq(current_block_num))
        .execute(conn)
        .map(|_| ())
}

pub fn update_definition_end_block_num(
    conn: &PgConnection,
    name: &str,
    current_block_num: i64,
) -> QueryResult<()> {
    update(grid_property_definition::table)
        .filter(
            grid_property_definition::name
                .eq(name)
                .and(grid_property_definition::end_block_num.eq(MAX_BLOCK_NUM)),
        )
        .set(grid_property_definition::end_block_num.eq(current_block_num))
        .execute(conn)
        .map(|_| ())
}

pub fn list_grid_schemas(conn: &PgConnection) -> QueryResult<Vec<GridSchema>> {
    grid_schema::table
        .select(grid_schema::all_columns)
        .filter(grid_schema::end_block_num.eq(MAX_BLOCK_NUM))
        .load::<GridSchema>(conn)
}

pub fn list_grid_property_definitions(
    conn: &PgConnection,
) -> QueryResult<Vec<GridPropertyDefinition>> {
    grid_property_definition::table
        .select(grid_property_definition::all_columns)
        .filter(grid_property_definition::end_block_num.eq(MAX_BLOCK_NUM))
        .load::<GridPropertyDefinition>(conn)
}

pub fn fetch_grid_schema(conn: &PgConnection, name: &str) -> QueryResult<Option<GridSchema>> {
    grid_schema::table
        .select(grid_schema::all_columns)
        .filter(
            grid_schema::name
                .eq(name)
                .and(grid_schema::end_block_num.eq(MAX_BLOCK_NUM)),
        )
        .first(conn)
        .map(Some)
        .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
}

pub fn list_grid_property_definitions_with_schema_name(
    conn: &PgConnection,
    schema_name: &str,
) -> QueryResult<Vec<GridPropertyDefinition>> {
    grid_property_definition::table
        .select(grid_property_definition::all_columns)
        .filter(
            grid_property_definition::schema_name
                .eq(schema_name)
                .and(grid_property_definition::end_block_num.eq(MAX_BLOCK_NUM)),
        )
        .load::<GridPropertyDefinition>(conn)
}

pub fn insert_grid_property_values(
    conn: &PgConnection,
    values: &[NewGridPropertyValue],
) -> QueryResult<()> {
    for value in values {
        update_grid_property_value_end_block_num(conn, &value.name, value.start_block_num)?;
    }

    insert_into(grid_property_value::table)
        .values(values)
        .execute(conn)
        .map(|_| ())
}

pub fn update_grid_property_value_end_block_num(
    conn: &PgConnection,
    property_name: &str,
    current_block_num: i64,
) -> QueryResult<()> {
    update(grid_property_value::table)
        .filter(
            grid_property_value::name
                .eq(property_name)
                .and(grid_property_value::end_block_num.eq(MAX_BLOCK_NUM)),
        )
        .set(grid_property_value::end_block_num.eq(current_block_num))
        .execute(conn)
        .map(|_| ())
}
