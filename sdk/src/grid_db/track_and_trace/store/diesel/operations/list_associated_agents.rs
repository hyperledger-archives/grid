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
    schema::associated_agent, TrackAndTraceStoreError,
};

use crate::grid_db::commits::MAX_COMMIT_NUM;
use crate::grid_db::track_and_trace::store::diesel::models::AssociatedAgentModel;
use crate::grid_db::track_and_trace::store::AssociatedAgent;

use diesel::prelude::*;

pub(in crate::grid_db::track_and_trace::store::diesel) trait TrackAndTraceStoreListAssociatedAgentsOperation
{
    fn list_associated_agents(
        &self,
        record_ids: &[String],
        service_id: Option<&str>,
    ) -> Result<Vec<AssociatedAgent>, TrackAndTraceStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> TrackAndTraceStoreListAssociatedAgentsOperation
    for TrackAndTraceStoreOperations<'a, diesel::pg::PgConnection>
{
    fn list_associated_agents(
        &self,
        record_ids: &[String],
        service_id: Option<&str>,
    ) -> Result<Vec<AssociatedAgent>, TrackAndTraceStoreError> {
        let mut query = associated_agent::table
            .into_boxed()
            .select(associated_agent::all_columns)
            .filter(
                associated_agent::end_commit_num
                    .eq(MAX_COMMIT_NUM)
                    .and(associated_agent::record_id.eq_any(record_ids)),
            );

        if let Some(service_id) = service_id {
            query = query.filter(associated_agent::service_id.eq(service_id));
        } else {
            query = query.filter(associated_agent::service_id.is_null());
        }

        let models: Vec<AssociatedAgentModel> = query
            .load::<AssociatedAgentModel>(self.conn)
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

        Ok(models.into_iter().map(AssociatedAgent::from).collect())
    }
}

#[cfg(feature = "sqlite")]
impl<'a> TrackAndTraceStoreListAssociatedAgentsOperation
    for TrackAndTraceStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn list_associated_agents(
        &self,
        record_ids: &[String],
        service_id: Option<&str>,
    ) -> Result<Vec<AssociatedAgent>, TrackAndTraceStoreError> {
        let mut query = associated_agent::table
            .into_boxed()
            .select(associated_agent::all_columns)
            .filter(
                associated_agent::end_commit_num
                    .eq(MAX_COMMIT_NUM)
                    .and(associated_agent::record_id.eq_any(record_ids)),
            );

        if let Some(service_id) = service_id {
            query = query.filter(associated_agent::service_id.eq(service_id));
        } else {
            query = query.filter(associated_agent::service_id.is_null());
        }

        let models: Vec<AssociatedAgentModel> = query
            .load::<AssociatedAgentModel>(self.conn)
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

        Ok(models.into_iter().map(AssociatedAgent::from).collect())
    }
}
