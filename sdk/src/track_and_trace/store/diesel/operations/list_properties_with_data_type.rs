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

use super::TrackAndTraceStoreOperations;
use crate::track_and_trace::store::diesel::{
    make_property_with_data_type,
    schema::{grid_property_definition, property, record},
    TrackAndTraceStoreError,
};

use crate::commits::MAX_COMMIT_NUM;
use crate::error::InternalError;
use crate::track_and_trace::store::diesel::models::PropertyModel;
use crate::track_and_trace::store::Property;

use diesel::prelude::*;

pub(in crate::track_and_trace::store::diesel) trait TrackAndTraceStoreListPropertiesWithDataTypeOperation
{
    fn list_properties_with_data_type(
        &self,
        record_ids: &[String],
        service_id: Option<&str>,
    ) -> Result<Vec<(Property, Option<String>)>, TrackAndTraceStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> TrackAndTraceStoreListPropertiesWithDataTypeOperation
    for TrackAndTraceStoreOperations<'a, diesel::pg::PgConnection>
{
    fn list_properties_with_data_type(
        &self,
        record_ids: &[String],
        service_id: Option<&str>,
    ) -> Result<Vec<(Property, Option<String>)>, TrackAndTraceStoreError> {
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
                property::record_id
                    .eq_any(record_ids)
                    .and(property::end_commit_num.eq(MAX_COMMIT_NUM)),
            );

        if let Some(service_id) = service_id {
            query = query.filter(property::service_id.eq(service_id));
        } else {
            query = query.filter(property::service_id.is_null());
        }

        let models: Vec<(PropertyModel, Option<String>)> = query
            .select((
                property::all_columns,
                grid_property_definition::data_type.nullable(),
            ))
            .load::<(PropertyModel, Option<String>)>(self.conn)
            .map(Some)
            .map_err(|err| {
                TrackAndTraceStoreError::InternalError(InternalError::from_source(Box::new(err)))
            })?
            .ok_or_else(|| {
                TrackAndTraceStoreError::NotFoundError(
                    "Could not get all records from storage".to_string(),
                )
            })?
            .into_iter()
            .collect();

        Ok(models
            .into_iter()
            .map(make_property_with_data_type)
            .collect())
    }
}

#[cfg(feature = "sqlite")]
impl<'a> TrackAndTraceStoreListPropertiesWithDataTypeOperation
    for TrackAndTraceStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn list_properties_with_data_type(
        &self,
        record_ids: &[String],
        service_id: Option<&str>,
    ) -> Result<Vec<(Property, Option<String>)>, TrackAndTraceStoreError> {
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
                property::record_id
                    .eq_any(record_ids)
                    .and(property::end_commit_num.eq(MAX_COMMIT_NUM)),
            );

        if let Some(service_id) = service_id {
            query = query.filter(property::service_id.eq(service_id));
        } else {
            query = query.filter(property::service_id.is_null());
        }

        let models: Vec<(PropertyModel, Option<String>)> = query
            .select((
                property::all_columns,
                grid_property_definition::data_type.nullable(),
            ))
            .load::<(PropertyModel, Option<String>)>(self.conn)
            .map(Some)
            .map_err(|err| {
                TrackAndTraceStoreError::InternalError(InternalError::from_source(Box::new(err)))
            })?
            .ok_or_else(|| {
                TrackAndTraceStoreError::NotFoundError(
                    "Could not get all records from storage".to_string(),
                )
            })?
            .into_iter()
            .collect();

        Ok(models
            .into_iter()
            .map(make_property_with_data_type)
            .collect())
    }
}
