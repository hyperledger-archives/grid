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

use diesel::prelude::*;

pub(in crate::grid_db::track_and_trace::store::diesel) trait TrackAndTraceStoreListReportedValueReporterToAgentMetadataOperation<
    C: Connection,
>
{
    fn list_reported_value_reporter_to_agent_metadata(
        &self,
        record_id: &str,
        property_name: &str,
        service_id: Option<String>,
    ) -> Result<Vec<ReportedValueReporterToAgentMetadata>, TrackAndTraceStoreError>;
    fn get_root_rvs(
        conn: &C,
        record_id: &str,
        property_name: &str,
        service_id: Option<String>,
    ) -> QueryResult<Vec<ReportedValueReporterToAgentMetadataModel>>;
    fn get_rvs_for_rv(
        conn: &C,
        rvs: Vec<ReportedValueReporterToAgentMetadataModel>,
    ) -> Result<Vec<ReportedValueReporterToAgentMetadata>, TrackAndTraceStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a>
    TrackAndTraceStoreListReportedValueReporterToAgentMetadataOperation<diesel::pg::PgConnection>
    for TrackAndTraceStoreOperations<'a, diesel::pg::PgConnection>
{
    fn list_reported_value_reporter_to_agent_metadata(
        &self,
        record_id: &str,
        property_name: &str,
        service_id: Option<String>,
    ) -> Result<Vec<ReportedValueReporterToAgentMetadata>, TrackAndTraceStoreError> {
        let query: Vec<ReportedValueReporterToAgentMetadataModel> = reported_value_reporter_to_agent_metadata::table
            .filter(
                reported_value_reporter_to_agent_metadata::property_name
                    .eq(property_name)
                    .and(reported_value_reporter_to_agent_metadata::record_id.eq(record_id))
                    .and(
                        reported_value_reporter_to_agent_metadata::reported_value_end_commit_num
                            .le(MAX_COMMIT_NUM),
                    ),
            )
            .load::<ReportedValueReporterToAgentMetadataModel>(self.conn)
            .map(Some)
            .map_err(|err| TrackAndTraceStoreError::OperationError {
                context: "Failed to fetch records".to_string(),
                source: Some(Box::new(err)),
            })?
            .ok_or_else(|| {
                TrackAndTraceStoreError::NotFoundError(
                    "Could not get all records from storage".to_string(),
                )
            })?
            .into_iter()
            .collect();

        let mut rvs = Vec::new();

        for model in query {
            let rv: ReportedValueReporterToAgentMetadataModel = model;
            let roots =
                Self::get_root_rvs(&*self.conn, &record_id, &property_name, service_id.clone())?;

            let children = Self::get_rvs_for_rv(&*self.conn, roots)?;

            rvs.push(ReportedValueReporterToAgentMetadata::from((rv, children)));
        }

        Ok(rvs)
    }

    fn get_root_rvs(
        conn: &PgConnection,
        record_id: &str,
        property_name: &str,
        service_id: Option<String>,
    ) -> QueryResult<Vec<ReportedValueReporterToAgentMetadataModel>> {
        let mut query = reported_value_reporter_to_agent_metadata::table
            .into_boxed()
            .select(reported_value_reporter_to_agent_metadata::all_columns)
            .filter(
                reported_value_reporter_to_agent_metadata::record_id
                    .eq(record_id)
                    .and(reported_value_reporter_to_agent_metadata::parent_name.is_null())
                    .and(
                        reported_value_reporter_to_agent_metadata::reported_value_end_commit_num
                            .le(MAX_COMMIT_NUM),
                    )
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
