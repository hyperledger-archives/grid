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

use super::TrackAndTraceStoreOperations;
use crate::grid_db::track_and_trace::store::diesel::{
    make_property_with_data_type,
    schema::{grid_property_definition, property, record},
    TrackAndTraceStoreError,
};

use crate::grid_db::commits::MAX_COMMIT_NUM;
use crate::grid_db::track_and_trace::store::diesel::models::PropertyModel;
use crate::grid_db::track_and_trace::store::Property;

use diesel::{prelude::*, result::Error::NotFound};

pub(in crate::grid_db::track_and_trace::store::diesel) trait TrackAndTraceStoreFetchPropertyWithDataTypeOperation
{
    fn fetch_property_with_data_type(
        &self,
        record_id: &str,
        property_name: &str,
        service_id: Option<String>,
    ) -> Result<Option<(Property, Option<String>)>, TrackAndTraceStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> TrackAndTraceStoreFetchPropertyWithDataTypeOperation
    for TrackAndTraceStoreOperations<'a, diesel::pg::PgConnection>
{
    fn fetch_property_with_data_type(
        &self,
        record_id: &str,
        property_name: &str,
        service_id: Option<String>,
    ) -> Result<Option<(Property, Option<String>)>, TrackAndTraceStoreError> {
        let mut query = property::table
            .into_boxed()
            .left_join(
                record::table.on(property::record_id
                    .eq(record::record_id)
                    .and(property::end_commit_num.eq(record::end_commit_num))),
            )
            .left_join(
                grid_property_definition::table.on(record::schema
                    .eq(grid_property_definition::schema_name)
                    .and(property::name.eq(grid_property_definition::name))
                    .and(property::end_commit_num.eq(record::end_commit_num))),
            )
            .filter(
                property::name
                    .eq(property_name)
                    .and(property::record_id.eq(record_id))
                    .and(property::end_commit_num.eq(MAX_COMMIT_NUM)),
            );

        if let Some(service_id) = service_id {
            query = query.filter(property::service_id.eq(service_id));
        } else {
            query = query.filter(property::service_id.is_null());
        }

        let prop = query
            .select((
                property::all_columns,
                grid_property_definition::data_type.nullable(),
            ))
            .first::<(PropertyModel, Option<String>)>(self.conn)
            .map(Some)
            .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
            .map_err(|err| TrackAndTraceStoreError::QueryError {
                context: "Failed to fetch existing record".to_string(),
                source: Box::new(err),
            })?;

        Ok(Some(make_property_with_data_type(prop.unwrap())))
    }
}

#[cfg(feature = "sqlite")]
impl<'a> TrackAndTraceStoreFetchPropertyWithDataTypeOperation
    for TrackAndTraceStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn fetch_property_with_data_type(
        &self,
        record_id: &str,
        property_name: &str,
        service_id: Option<String>,
    ) -> Result<Option<(Property, Option<String>)>, TrackAndTraceStoreError> {
        let mut query = property::table
            .into_boxed()
            .left_join(
                record::table.on(property::record_id
                    .eq(record::record_id)
                    .and(property::end_commit_num.eq(record::end_commit_num))),
            )
            .left_join(
                grid_property_definition::table.on(record::schema
                    .eq(grid_property_definition::schema_name)
                    .and(property::name.eq(grid_property_definition::name))
                    .and(property::end_commit_num.eq(record::end_commit_num))),
            )
            .filter(
                property::name
                    .eq(property_name)
                    .and(property::record_id.eq(record_id))
                    .and(property::end_commit_num.eq(MAX_COMMIT_NUM)),
            );

        if let Some(service_id) = service_id {
            query = query.filter(property::service_id.eq(service_id));
        } else {
            query = query.filter(property::service_id.is_null());
        }

        let prop = query
            .select((
                property::all_columns,
                grid_property_definition::data_type.nullable(),
            ))
            .first::<(PropertyModel, Option<String>)>(self.conn)
            .map(Some)
            .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
            .map_err(|err| TrackAndTraceStoreError::QueryError {
                context: "Failed to fetch existing record".to_string(),
                source: Box::new(err),
            })?;

        Ok(Some(make_property_with_data_type(prop.unwrap())))
    }
}
