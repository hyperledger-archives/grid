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
use crate::track_and_trace::store::diesel::{schema::record, TrackAndTraceStoreError};

use crate::commits::MAX_COMMIT_NUM;
use crate::error::InternalError;
use crate::track_and_trace::store::diesel::models::RecordModel;
use crate::track_and_trace::store::Record;

use diesel::prelude::*;

pub(in crate::track_and_trace::store::diesel) trait TrackAndTraceStoreListRecordsOperation {
    fn list_records(
        &self,
        service_id: Option<&str>,
    ) -> Result<Vec<Record>, TrackAndTraceStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> TrackAndTraceStoreListRecordsOperation
    for TrackAndTraceStoreOperations<'a, diesel::pg::PgConnection>
{
    fn list_records(
        &self,
        service_id: Option<&str>,
    ) -> Result<Vec<Record>, TrackAndTraceStoreError> {
        let mut query = record::table
            .into_boxed()
            .select(record::all_columns)
            .filter(record::end_commit_num.eq(MAX_COMMIT_NUM));

        if let Some(service_id) = service_id {
            query = query.filter(record::service_id.eq(service_id));
        } else {
            query = query.filter(record::service_id.is_null());
        }

        let models: Vec<RecordModel> = query
            .load::<RecordModel>(self.conn)
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

        Ok(models.into_iter().map(Record::from).collect())
    }
}

#[cfg(feature = "sqlite")]
impl<'a> TrackAndTraceStoreListRecordsOperation
    for TrackAndTraceStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn list_records(
        &self,
        service_id: Option<&str>,
    ) -> Result<Vec<Record>, TrackAndTraceStoreError> {
        let mut query = record::table
            .into_boxed()
            .select(record::all_columns)
            .filter(record::end_commit_num.eq(MAX_COMMIT_NUM));

        if let Some(service_id) = service_id {
            query = query.filter(record::service_id.eq(service_id));
        } else {
            query = query.filter(record::service_id.is_null());
        }

        let models: Vec<RecordModel> = query
            .load::<RecordModel>(self.conn)
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

        Ok(models.into_iter().map(Record::from).collect())
    }
}
