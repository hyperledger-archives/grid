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
    schema::reported_value_reporter_to_agent_metadata, TrackAndTraceStoreError,
};

use crate::grid_db::commits::MAX_COMMIT_NUM;
use crate::grid_db::track_and_trace::store::diesel::models::ReportedValueReporterToAgentMetadataModel;
use crate::grid_db::track_and_trace::store::ReportedValueReporterToAgentMetadata;

use diesel::{prelude::*, result::Error::NotFound};

pub(in crate::grid_db::track_and_trace::store::diesel) trait TrackAndTraceStoreFetchReportedValueReporterToAgentMetadataOperation<
    C: Connection,
>
{
    fn fetch_reported_value_reporter_to_agent_metadata(
        &self,
        record_id: &str,
        property_name: &str,
        commit_height: Option<i64>,
        service_id: Option<&str>,
    ) -> Result<Option<ReportedValueReporterToAgentMetadata>, TrackAndTraceStoreError>;
    fn get_root_rvs(
        conn: &C,
        record_id: &str,
        property_name: &str,
        commit_height: Option<i64>,
        service_id: Option<&str>,
    ) -> QueryResult<Vec<ReportedValueReporterToAgentMetadataModel>>;
    fn get_rvs_for_rv(
        conn: &C,
        rvs: Vec<ReportedValueReporterToAgentMetadataModel>,
    ) -> Result<Vec<ReportedValueReporterToAgentMetadata>, TrackAndTraceStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a>
    TrackAndTraceStoreFetchReportedValueReporterToAgentMetadataOperation<diesel::pg::PgConnection>
    for TrackAndTraceStoreOperations<'a, diesel::pg::PgConnection>
{
    fn fetch_reported_value_reporter_to_agent_metadata(
        &self,
        record_id: &str,
        property_name: &str,
        commit_height: Option<i64>,
        service_id: Option<&str>,
    ) -> Result<Option<ReportedValueReporterToAgentMetadata>, TrackAndTraceStoreError> {
        let height = commit_height.unwrap_or(MAX_COMMIT_NUM);
        let mut query = reported_value_reporter_to_agent_metadata::table
            .into_boxed()
            .filter(
                reported_value_reporter_to_agent_metadata::property_name
                    .eq(property_name)
                    .and(reported_value_reporter_to_agent_metadata::record_id.eq(record_id))
                    .and(
                        reported_value_reporter_to_agent_metadata::reported_value_end_commit_num
                            .eq(height),
                    ),
            );

        if let Some(service_id) = service_id {
            query =
                query.filter(reported_value_reporter_to_agent_metadata::service_id.eq(service_id));
        } else {
            query = query.filter(reported_value_reporter_to_agent_metadata::service_id.is_null());
        }

        let val = query
            .first::<ReportedValueReporterToAgentMetadataModel>(self.conn)
            .map(Some)
            .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
            .map_err(|err| TrackAndTraceStoreError::QueryError {
                context: "Failed to fetch existing record".to_string(),
                source: Box::new(err),
            })?;

        let roots = Self::get_root_rvs(
            &*self.conn,
            &record_id,
            &property_name,
            commit_height,
            service_id,
        )?;

        let rvs = Self::get_rvs_for_rv(&*self.conn, roots)?;

        Ok(val.map(|v| ReportedValueReporterToAgentMetadata::from((v, rvs))))
    }

    fn get_root_rvs(
        conn: &PgConnection,
        record_id: &str,
        property_name: &str,
        commit_height: Option<i64>,
        service_id: Option<&str>,
    ) -> QueryResult<Vec<ReportedValueReporterToAgentMetadataModel>> {
        let mut query = reported_value_reporter_to_agent_metadata::table
            .into_boxed()
            .select(reported_value_reporter_to_agent_metadata::all_columns)
            .filter(
                reported_value_reporter_to_agent_metadata::record_id
                    .eq(record_id)
                    .and(reported_value_reporter_to_agent_metadata::parent_name.is_null())
                    .and(
                        reported_value_reporter_to_agent_metadata::property_name.eq(property_name),
                    ),
            );

        if let Some(service_id) = service_id {
            query =
                query.filter(reported_value_reporter_to_agent_metadata::service_id.eq(service_id));
        } else {
            query = query.filter(reported_value_reporter_to_agent_metadata::service_id.is_null());
        }

        if let Some(commit_height) = commit_height {
            query = query.filter(
                reported_value_reporter_to_agent_metadata::reported_value_end_commit_num
                    .eq(commit_height),
            );
        } else {
            query = query.filter(
                reported_value_reporter_to_agent_metadata::reported_value_end_commit_num
                    .eq(MAX_COMMIT_NUM),
            );
        }

        query.load::<ReportedValueReporterToAgentMetadataModel>(conn)
    }

    fn get_rvs_for_rv(
        conn: &PgConnection,
        rvs: Vec<ReportedValueReporterToAgentMetadataModel>,
    ) -> Result<Vec<ReportedValueReporterToAgentMetadata>, TrackAndTraceStoreError> {
        let mut values = Vec::new();

        for rv in rvs {
            let mut query = reported_value_reporter_to_agent_metadata::table
                .into_boxed()
                .select(reported_value_reporter_to_agent_metadata::all_columns)
                .filter(
                    reported_value_reporter_to_agent_metadata::parent_name
                        .eq(&rv.property_name)
                        .and(
                            reported_value_reporter_to_agent_metadata::record_id.eq(&rv.record_id),
                        ),
                );

            if let Some(service_id) = &rv.service_id {
                query = query
                    .filter(reported_value_reporter_to_agent_metadata::service_id.eq(service_id));
            } else {
                query =
                    query.filter(reported_value_reporter_to_agent_metadata::service_id.is_null());
            }

            let children = query.load::<ReportedValueReporterToAgentMetadataModel>(conn)?;

            if children.is_empty() {
                values.push(ReportedValueReporterToAgentMetadata::from(rv))
            } else {
                values.push(ReportedValueReporterToAgentMetadata::from((
                    rv,
                    Self::get_rvs_for_rv(conn, children)?,
                )));
            }
        }

        Ok(values)
    }
}

#[cfg(feature = "sqlite")]
impl<'a>
    TrackAndTraceStoreFetchReportedValueReporterToAgentMetadataOperation<
        diesel::sqlite::SqliteConnection,
    > for TrackAndTraceStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn fetch_reported_value_reporter_to_agent_metadata(
        &self,
        record_id: &str,
        property_name: &str,
        commit_height: Option<i64>,
        service_id: Option<&str>,
    ) -> Result<Option<ReportedValueReporterToAgentMetadata>, TrackAndTraceStoreError> {
        let height = commit_height.unwrap_or(MAX_COMMIT_NUM);
        let mut query = reported_value_reporter_to_agent_metadata::table
            .into_boxed()
            .filter(
                reported_value_reporter_to_agent_metadata::property_name
                    .eq(property_name)
                    .and(reported_value_reporter_to_agent_metadata::record_id.eq(record_id))
                    .and(
                        reported_value_reporter_to_agent_metadata::reported_value_end_commit_num
                            .eq(height),
                    ),
            );

        if let Some(service_id) = service_id {
            query =
                query.filter(reported_value_reporter_to_agent_metadata::service_id.eq(service_id));
        } else {
            query = query.filter(reported_value_reporter_to_agent_metadata::service_id.is_null());
        }

        let val = query
            .first::<ReportedValueReporterToAgentMetadataModel>(self.conn)
            .map(Some)
            .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
            .map_err(|err| TrackAndTraceStoreError::QueryError {
                context: "Failed to fetch existing record".to_string(),
                source: Box::new(err),
            })?;

        let roots = Self::get_root_rvs(
            &*self.conn,
            &record_id,
            &property_name,
            commit_height,
            service_id,
        )?;

        let rvs = Self::get_rvs_for_rv(&*self.conn, roots)?;

        Ok(val.map(|v| ReportedValueReporterToAgentMetadata::from((v, rvs))))
    }

    fn get_root_rvs(
        conn: &SqliteConnection,
        record_id: &str,
        property_name: &str,
        commit_height: Option<i64>,
        service_id: Option<&str>,
    ) -> QueryResult<Vec<ReportedValueReporterToAgentMetadataModel>> {
        let mut query = reported_value_reporter_to_agent_metadata::table
            .into_boxed()
            .select(reported_value_reporter_to_agent_metadata::all_columns)
            .filter(
                reported_value_reporter_to_agent_metadata::record_id
                    .eq(record_id)
                    .and(reported_value_reporter_to_agent_metadata::parent_name.is_null())
                    .and(
                        reported_value_reporter_to_agent_metadata::property_name.eq(property_name),
                    ),
            );

        if let Some(service_id) = service_id {
            query =
                query.filter(reported_value_reporter_to_agent_metadata::service_id.eq(service_id));
        } else {
            query = query.filter(reported_value_reporter_to_agent_metadata::service_id.is_null());
        }

        if let Some(commit_height) = commit_height {
            query = query.filter(
                reported_value_reporter_to_agent_metadata::reported_value_end_commit_num
                    .eq(commit_height),
            );
        } else {
            query = query.filter(
                reported_value_reporter_to_agent_metadata::reported_value_end_commit_num
                    .eq(MAX_COMMIT_NUM),
            );
        }

        query.load::<ReportedValueReporterToAgentMetadataModel>(conn)
    }

    fn get_rvs_for_rv(
        conn: &SqliteConnection,
        rvs: Vec<ReportedValueReporterToAgentMetadataModel>,
    ) -> Result<Vec<ReportedValueReporterToAgentMetadata>, TrackAndTraceStoreError> {
        let mut values = Vec::new();

        for rv in rvs {
            let mut query = reported_value_reporter_to_agent_metadata::table
                .into_boxed()
                .select(reported_value_reporter_to_agent_metadata::all_columns)
                .filter(
                    reported_value_reporter_to_agent_metadata::parent_name
                        .eq(&rv.property_name)
                        .and(
                            reported_value_reporter_to_agent_metadata::record_id.eq(&rv.record_id),
                        ),
                );

            if let Some(service_id) = &rv.service_id {
                query = query
                    .filter(reported_value_reporter_to_agent_metadata::service_id.eq(service_id));
            } else {
                query =
                    query.filter(reported_value_reporter_to_agent_metadata::service_id.is_null());
            }

            let children = query.load::<ReportedValueReporterToAgentMetadataModel>(conn)?;

            if children.is_empty() {
                values.push(ReportedValueReporterToAgentMetadata::from(rv))
            } else {
                values.push(ReportedValueReporterToAgentMetadata::from((
                    rv,
                    Self::get_rvs_for_rv(conn, children)?,
                )));
            }
        }

        Ok(values)
    }
}
