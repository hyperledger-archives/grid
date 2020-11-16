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
use crate::track_and_trace::store::diesel::{schema::record, TrackAndTraceStoreError};

use crate::commits::MAX_COMMIT_NUM;
use crate::error::InternalError;
use crate::track_and_trace::store::diesel::models::RecordModel;
use crate::track_and_trace::store::Record;

use diesel::{prelude::*, result::Error::NotFound};

pub(in crate::track_and_trace::store::diesel) trait TrackAndTraceStoreFetchRecordOperation {
    fn fetch_record(
        &self,
        record_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Record>, TrackAndTraceStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> TrackAndTraceStoreFetchRecordOperation
    for TrackAndTraceStoreOperations<'a, diesel::pg::PgConnection>
{
    fn fetch_record(
        &self,
        record_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Record>, TrackAndTraceStoreError> {
        let mut query = record::table
            .into_boxed()
            .select(record::all_columns)
            .filter(
                record::record_id
                    .eq(record_id)
                    .and(record::end_commit_num.eq(MAX_COMMIT_NUM)),
            );

        if let Some(service_id) = service_id {
            query = query.filter(record::service_id.eq(service_id));
        } else {
            query = query.filter(record::service_id.is_null());
        }

        let rec = query
            .first::<RecordModel>(self.conn)
            .map(Some)
            .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
            .map_err(|err| {
                TrackAndTraceStoreError::InternalError(InternalError::from_source(Box::new(err)))
            })?;

        Ok(rec.map(Record::from))
    }
}

#[cfg(feature = "sqlite")]
impl<'a> TrackAndTraceStoreFetchRecordOperation
    for TrackAndTraceStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn fetch_record(
        &self,
        record_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Record>, TrackAndTraceStoreError> {
        let mut query = record::table
            .into_boxed()
            .select(record::all_columns)
            .filter(
                record::record_id
                    .eq(record_id)
                    .and(record::end_commit_num.eq(MAX_COMMIT_NUM)),
            );

        if let Some(service_id) = service_id {
            query = query.filter(record::service_id.eq(service_id));
        } else {
            query = query.filter(record::service_id.is_null());
        }

        let rec = query
            .first::<RecordModel>(self.conn)
            .map(Some)
            .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
            .map_err(|err| {
                TrackAndTraceStoreError::InternalError(InternalError::from_source(Box::new(err)))
            })?;

        Ok(rec.map(Record::from))
    }
}
