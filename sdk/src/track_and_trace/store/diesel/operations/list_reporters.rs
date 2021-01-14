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
use crate::track_and_trace::store::diesel::{schema::reporter, TrackAndTraceStoreError};

use crate::commits::MAX_COMMIT_NUM;
use crate::error::InternalError;
use crate::track_and_trace::store::diesel::models::ReporterModel;
use crate::track_and_trace::store::Reporter;

use diesel::prelude::*;

pub(in crate::track_and_trace::store::diesel) trait TrackAndTraceStoreListReportersOperation {
    fn list_reporters(
        &self,
        record_id: &str,
        property_name: &str,
        service_id: Option<&str>,
    ) -> Result<Vec<Reporter>, TrackAndTraceStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> TrackAndTraceStoreListReportersOperation
    for TrackAndTraceStoreOperations<'a, diesel::pg::PgConnection>
{
    fn list_reporters(
        &self,
        record_id: &str,
        property_name: &str,
        service_id: Option<&str>,
    ) -> Result<Vec<Reporter>, TrackAndTraceStoreError> {
        let mut query = reporter::table
            .into_boxed()
            .select(reporter::all_columns)
            .filter(
                reporter::property_name
                    .eq(property_name)
                    .and(reporter::record_id.eq(record_id))
                    .and(reporter::end_commit_num.eq(MAX_COMMIT_NUM)),
            );

        if let Some(service_id) = service_id {
            query = query.filter(reporter::service_id.eq(service_id));
        } else {
            query = query.filter(reporter::service_id.is_null());
        }

        let model: Vec<ReporterModel> = query
            .load::<ReporterModel>(self.conn)
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

        Ok(model.into_iter().map(Reporter::from).collect())
    }
}

#[cfg(feature = "sqlite")]
impl<'a> TrackAndTraceStoreListReportersOperation
    for TrackAndTraceStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn list_reporters(
        &self,
        record_id: &str,
        property_name: &str,
        service_id: Option<&str>,
    ) -> Result<Vec<Reporter>, TrackAndTraceStoreError> {
        let mut query = reporter::table
            .into_boxed()
            .select(reporter::all_columns)
            .filter(
                reporter::property_name
                    .eq(property_name)
                    .and(reporter::record_id.eq(record_id))
                    .and(reporter::end_commit_num.eq(MAX_COMMIT_NUM)),
            );

        if let Some(service_id) = service_id {
            query = query.filter(reporter::service_id.eq(service_id));
        } else {
            query = query.filter(reporter::service_id.is_null());
        }

        let model: Vec<ReporterModel> = query
            .load::<ReporterModel>(self.conn)
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

        Ok(model.into_iter().map(Reporter::from).collect())
    }
}
