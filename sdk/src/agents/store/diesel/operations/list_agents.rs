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

use super::AgentStoreOperations;
use crate::{
    agents::store::diesel::{
        schema::{agent, role},
        Agent, AgentList, AgentStoreError,
    },
    paging::Paging,
};

use crate::agents::store::diesel::models::{AgentModel, RoleModel};
use crate::commits::MAX_COMMIT_NUM;
use crate::error::InternalError;
use diesel::prelude::*;

pub(in crate::agents::store::diesel) trait AgentStoreListAgentsOperation {
    fn list_agents(
        &self,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<AgentList, AgentStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> AgentStoreListAgentsOperation for AgentStoreOperations<'a, diesel::pg::PgConnection> {
    fn list_agents(
        &self,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<AgentList, AgentStoreError> {
        self.conn
            .build_transaction()
            .read_write()
            .run::<_, AgentStoreError, _>(|| {
                let mut query = agent::table
                    .into_boxed()
                    .select(agent::all_columns)
                    .offset(offset)
                    .limit(limit)
                    .filter(agent::end_commit_num.eq(MAX_COMMIT_NUM));

                if let Some(service_id) = service_id {
                    query = query.filter(agent::service_id.eq(service_id));
                } else {
                    query = query.filter(agent::service_id.is_null());
                }

                let agent_models = query.load::<AgentModel>(self.conn).map_err(|err| {
                    AgentStoreError::InternalError(InternalError::from_source(Box::new(err)))
                })?;

                let mut count_query = agent::table.into_boxed().select(agent::all_columns);

                if let Some(service_id) = service_id {
                    count_query = count_query.filter(agent::service_id.eq(service_id));
                } else {
                    count_query = count_query.filter(agent::service_id.is_null());
                }

                let total = count_query.count().get_result(self.conn)?;

                let mut agents = Vec::new();

                for a in agent_models {
                    let mut query = role::table.into_boxed().select(role::all_columns).filter(
                        role::public_key
                            .eq(&a.public_key)
                            .and(role::end_commit_num.eq(MAX_COMMIT_NUM)),
                    );

                    if let Some(service_id) = service_id {
                        query = query.filter(role::service_id.eq(service_id));
                    } else {
                        query = query.filter(role::service_id.is_null());
                    }

                    let roles = query.load::<RoleModel>(self.conn).map_err(|err| {
                        AgentStoreError::InternalError(InternalError::from_source(Box::new(err)))
                    })?;

                    agents.push(Agent::from((a, roles)));
                }

                Ok(AgentList::new(agents, Paging::new(offset, limit, total)))
            })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> AgentStoreListAgentsOperation
    for AgentStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn list_agents(
        &self,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<AgentList, AgentStoreError> {
        self.conn
            .immediate_transaction::<_, AgentStoreError, _>(|| {
                let mut query = agent::table
                    .into_boxed()
                    .select(agent::all_columns)
                    .offset(offset)
                    .limit(limit)
                    .filter(agent::end_commit_num.eq(MAX_COMMIT_NUM));

                if let Some(service_id) = service_id {
                    query = query.filter(agent::service_id.eq(service_id));
                } else {
                    query = query.filter(agent::service_id.is_null());
                }

                let agent_models = query.load::<AgentModel>(self.conn).map_err(|err| {
                    AgentStoreError::InternalError(InternalError::from_source(Box::new(err)))
                })?;

                let mut count_query = agent::table.into_boxed().select(agent::all_columns);

                if let Some(service_id) = service_id {
                    count_query = count_query.filter(agent::service_id.eq(service_id));
                } else {
                    count_query = count_query.filter(agent::service_id.is_null());
                }

                let total = count_query.count().get_result(self.conn)?;

                let mut agents = Vec::new();

                for a in agent_models {
                    let mut query = role::table.into_boxed().select(role::all_columns).filter(
                        role::public_key
                            .eq(&a.public_key)
                            .and(role::end_commit_num.eq(MAX_COMMIT_NUM)),
                    );

                    if let Some(service_id) = service_id {
                        query = query.filter(role::service_id.eq(service_id));
                    } else {
                        query = query.filter(role::service_id.is_null());
                    }

                    let roles = query.load::<RoleModel>(self.conn).map_err(|err| {
                        AgentStoreError::InternalError(InternalError::from_source(Box::new(err)))
                    })?;

                    agents.push(Agent::from((a, roles)));
                }

                Ok(AgentList::new(agents, Paging::new(offset, limit, total)))
            })
    }
}
