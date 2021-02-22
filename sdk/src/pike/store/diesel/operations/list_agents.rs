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

use super::PikeStoreOperations;
use crate::paging::Paging;
use crate::pike::store::diesel::{
    schema::{pike_agent, pike_role},
    Agent, AgentList, PikeStoreError,
};

use crate::commits::MAX_COMMIT_NUM;
use crate::error::InternalError;
use crate::pike::store::diesel::models::{AgentModel, RoleModel};
use diesel::prelude::*;

pub(in crate::pike::store::diesel) trait PikeStoreListAgentsOperation {
    fn list_agents(
        &self,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<AgentList, PikeStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> PikeStoreListAgentsOperation for PikeStoreOperations<'a, diesel::pg::PgConnection> {
    fn list_agents(
        &self,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<AgentList, PikeStoreError> {
        self.conn.transaction::<_, PikeStoreError, _>(|| {
            let mut query = pike_agent::table
                .into_boxed()
                .select(pike_agent::all_columns)
                .offset(offset)
                .limit(limit)
                .filter(pike_agent::end_commit_num.eq(MAX_COMMIT_NUM));

            if let Some(service_id) = service_id {
                query = query.filter(pike_agent::service_id.eq(service_id));
            } else {
                query = query.filter(pike_agent::service_id.is_null());
            }

            let agent_models = query.load::<AgentModel>(self.conn).map_err(|err| {
                PikeStoreError::InternalError(InternalError::from_source(Box::new(err)))
            })?;

            let mut count_query = pike_agent::table
                .into_boxed()
                .select(pike_agent::all_columns);

            if let Some(service_id) = service_id {
                count_query = count_query.filter(pike_agent::service_id.eq(service_id));
            } else {
                count_query = count_query.filter(pike_agent::service_id.is_null());
            }

            let total = count_query.count().get_result(self.conn)?;

            let mut agents = Vec::new();

            for a in agent_models {
                let mut query = pike_role::table
                    .into_boxed()
                    .select(pike_role::all_columns)
                    .filter(
                        pike_role::public_key
                            .eq(&a.public_key)
                            .and(pike_role::end_commit_num.eq(MAX_COMMIT_NUM)),
                    );

                if let Some(service_id) = service_id {
                    query = query.filter(pike_role::service_id.eq(service_id));
                } else {
                    query = query.filter(pike_role::service_id.is_null());
                }

                let roles = query.load::<RoleModel>(self.conn).map_err(|err| {
                    PikeStoreError::InternalError(InternalError::from_source(Box::new(err)))
                })?;

                agents.push(Agent::from((a, roles)));
            }

            Ok(AgentList::new(agents, Paging::new(offset, limit, total)))
        })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> PikeStoreListAgentsOperation
    for PikeStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn list_agents(
        &self,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<AgentList, PikeStoreError> {
        self.conn.transaction::<_, PikeStoreError, _>(|| {
            let mut query = pike_agent::table
                .into_boxed()
                .select(pike_agent::all_columns)
                .offset(offset)
                .limit(limit)
                .filter(pike_agent::end_commit_num.eq(MAX_COMMIT_NUM));

            if let Some(service_id) = service_id {
                query = query.filter(pike_agent::service_id.eq(service_id));
            } else {
                query = query.filter(pike_agent::service_id.is_null());
            }

            let agent_models = query.load::<AgentModel>(self.conn).map_err(|err| {
                PikeStoreError::InternalError(InternalError::from_source(Box::new(err)))
            })?;

            let mut count_query = pike_agent::table
                .into_boxed()
                .select(pike_agent::all_columns);

            if let Some(service_id) = service_id {
                count_query = count_query.filter(pike_agent::service_id.eq(service_id));
            } else {
                count_query = count_query.filter(pike_agent::service_id.is_null());
            }

            let total = count_query.count().get_result(self.conn)?;

            let mut agents = Vec::new();

            for a in agent_models {
                let mut query = pike_role::table
                    .into_boxed()
                    .select(pike_role::all_columns)
                    .filter(
                        pike_role::public_key
                            .eq(&a.public_key)
                            .and(pike_role::end_commit_num.eq(MAX_COMMIT_NUM)),
                    );

                if let Some(service_id) = service_id {
                    query = query.filter(pike_role::service_id.eq(service_id));
                } else {
                    query = query.filter(pike_role::service_id.is_null());
                }

                let roles = query.load::<RoleModel>(self.conn).map_err(|err| {
                    PikeStoreError::InternalError(InternalError::from_source(Box::new(err)))
                })?;

                agents.push(Agent::from((a, roles)));
            }

            Ok(AgentList::new(agents, Paging::new(offset, limit, total)))
        })
    }
}
