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
use crate::grid_db::agents::store::diesel::{
    schema::{agent, role},
    Agent, AgentStoreError,
};

use crate::grid_db::agents::store::diesel::models::{AgentModel, RoleModel};
use crate::grid_db::commits::MAX_COMMIT_NUM;
use diesel::prelude::*;

pub(in crate::grid_db::agents::store::diesel) trait AgentStoreListAgentsOperation {
    fn list_agents(&self, service_id: Option<String>) -> Result<Vec<Agent>, AgentStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> AgentStoreListAgentsOperation for AgentStoreOperations<'a, diesel::pg::PgConnection> {
    fn list_agents(&self, service_id: Option<String>) -> Result<Vec<Agent>, AgentStoreError> {
        self.conn
            .build_transaction()
            .read_write()
            .run::<_, AgentStoreError, _>(|| {
                let agent_models: Vec<AgentModel> = agent::table
                    .select(agent::all_columns)
                    .filter(
                        agent::service_id
                            .eq(&service_id)
                            .and(agent::end_commit_num.eq(MAX_COMMIT_NUM)),
                    )
                    .load::<AgentModel>(self.conn)
                    .map(Some)
                    .map_err(|err| AgentStoreError::OperationError {
                        context: "Failed to fetch agents".to_string(),
                        source: Some(Box::new(err)),
                    })?
                    .ok_or_else(|| {
                        AgentStoreError::NotFoundError(
                            "Could not get all agents from storage".to_string(),
                        )
                    })?
                    .into_iter()
                    .collect();

                let mut agents = Vec::new();

                for a in agent_models {
                    let roles: Vec<RoleModel> = role::table
                        .select(role::all_columns)
                        .filter(
                            role::service_id
                                .eq(&service_id)
                                .and(role::public_key.eq(&a.public_key))
                                .and(role::end_commit_num.eq(MAX_COMMIT_NUM)),
                        )
                        .load::<RoleModel>(self.conn)
                        .map(Some)
                        .map_err(|err| AgentStoreError::OperationError {
                            context: "Failed to fetch roles".to_string(),
                            source: Some(Box::new(err)),
                        })?
                        .ok_or_else(|| {
                            AgentStoreError::NotFoundError(
                                "Could not get all roles from storage".to_string(),
                            )
                        })?
                        .into_iter()
                        .collect();

                    agents.push(Agent::from((a, roles)));
                }

                Ok(agents)
            })
    }
}
