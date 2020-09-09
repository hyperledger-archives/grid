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
    error::SchemaStoreError,
    store::{
        diesel::schema::{grid_property_definition, grid_schema},
        Schema,
    },
    MAX_COMMIT_NUM,
};
use diesel::{
    dsl::{insert_into, update},
    prelude::*,
};

pub(in crate::grid_db::schemas) trait AddSchemaOperation {
    fn add_schema(&self, schema: &Schema) -> Result<(), SchemaStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> AddSchemaOperation for SchemaStoreOperations<'a, diesel::pg::PgConnection> {
    fn add_schema(&self, schema: &Schema) -> Result<(), SchemaStoreError> {
        let (schema_model, definitions) = schema.into();

        self.conn.transaction::<_, SchemaStoreError, _>(|| {
            update_grid_schema_end_commit_num(
                &*self.conn,
                &schema_model.name,
                schema_model.service_id.as_deref(),
                schema_model.start_commit_num,
            )?;

            insert_into(grid_schema::table)
                .values(schema_model)
                .execute(&*self.conn)
                .map(|_| ())?;

            for definition in &definitions {
                update_definition_end_commit_num(
                    &*self.conn,
                    &definition.name,
                    definition.service_id.as_deref(),
                    &definition.schema_name,
                    definition.start_commit_num,
                )?;
            }

            insert_into(grid_property_definition::table)
                .values(definitions)
                .execute(&*self.conn)?;

            Ok(())
        })
    }
}

fn update_grid_schema_end_commit_num<C: diesel::Connection>(
    conn: &C,
    name: &str,
    service_id: Option<&str>,
    current_commit_num: i64,
) -> QueryResult<()> {
    let update = update(grid_schema::table);

    if let Some(service_id) = service_id {
        update
            .filter(
                grid_schema::name
                    .eq(name)
                    .and(grid_schema::service_id.eq(service_id))
                    .and(grid_schema::end_commit_num.eq(MAX_COMMIT_NUM)),
            )
            .set(grid_schema::end_commit_num.eq(current_commit_num))
            .execute(conn)
            .map(|_| ())
    } else {
        update
            .filter(
                grid_schema::name
                    .eq(name)
                    .and(grid_schema::end_commit_num.eq(MAX_COMMIT_NUM)),
            )
            .set(grid_schema::end_commit_num.eq(current_commit_num))
            .execute(conn)
            .map(|_| ())
    }
}

fn update_definition_end_commit_num<C: diesel::Connection>(
    conn: &C,
    name: &str,
    service_id: Option<&str>,
    schema_name: &str,
    current_commit_num: i64,
) -> QueryResult<()> {
    let update = update(grid_property_definition::table);

    if let Some(service_id) = service_id {
        update
            .filter(
                grid_property_definition::schema_name
                    .eq(schema_name)
                    .and(grid_property_definition::name.eq(name))
                    .and(grid_property_definition::service_id.eq(service_id))
                    .and(grid_property_definition::end_commit_num.eq(MAX_COMMIT_NUM)),
            )
            .set(grid_property_definition::end_commit_num.eq(current_commit_num))
            .execute(conn)
            .map(|_| ())
    } else {
        update
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
}
