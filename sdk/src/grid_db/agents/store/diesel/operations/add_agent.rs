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
    AgentStoreError,
};

use crate::grid_db::agents::store::diesel::models::{
    AgentModel, NewAgentModel, NewRoleModel, RoleModel,
};
use crate::grid_db::commits::MAX_COMMIT_NUM;
use diesel::{
    dsl::{insert_into, update},
    prelude::*,
    result::Error::NotFound,
};

pub(in crate::grid_db::agents::store::diesel) trait AgentStoreAddAgentOperation {
    fn add_agent(
        &self,
        agent: NewAgentModel,
        roles: Vec<NewRoleModel>,
    ) -> Result<(), AgentStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> AgentStoreAddAgentOperation for AgentStoreOperations<'a, diesel::pg::PgConnection> {
    fn add_agent(
        &self,
        agent: NewAgentModel,
        roles: Vec<NewRoleModel>,
    ) -> Result<(), AgentStoreError> {
        self.conn
            .build_transaction()
            .read_write()
            .run::<_, AgentStoreError, _>(|| {
                let duplicate_agent = agent::table
                    .filter(
                        agent::public_key
                            .eq(&agent.public_key)
                            .and(agent::service_id.eq(&agent.service_id))
                            .and(agent::end_commit_num.eq(MAX_COMMIT_NUM)),
                    )
                    .first::<AgentModel>(self.conn)
                    .map(Some)
                    .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
                    .map_err(|err| AgentStoreError::QueryError {
                        context: "Failed check for existing agent".to_string(),
                        source: Box::new(err),
                    })?;

                if duplicate_agent.is_some() {
                    update(agent::table)
                        .filter(
                            agent::public_key
                                .eq(&agent.public_key)
                                .and(agent::service_id.eq(&agent.service_id))
                                .and(agent::end_commit_num.eq(MAX_COMMIT_NUM)),
                        )
                        .set(agent::end_commit_num.eq(agent.start_commit_num))
                        .execute(self.conn)
                        .map(|_| ())
                        .map_err(|err| AgentStoreError::OperationError {
                            context: "Failed to update agent".to_string(),
                            source: Some(Box::new(err)),
                        })?;
                }

                insert_into(agent::table)
                    .values(&agent)
                    .execute(self.conn)
                    .map(|_| ())
                    .map_err(|err| AgentStoreError::OperationError {
                        context: "Failed to add agent".to_string(),
                        source: Some(Box::new(err)),
                    })?;

                for role in roles {
                    let duplicate_role = role::table
                        .filter(
                            role::public_key
                                .eq(&role.public_key)
                                .and(role::role_name.eq(&role.role_name))
                                .and(role::service_id.eq(&role.service_id))
                                .and(role::end_commit_num.eq(MAX_COMMIT_NUM)),
                        )
                        .first::<RoleModel>(self.conn)
                        .map(Some)
                        .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
                        .map_err(|err| AgentStoreError::QueryError {
                            context: "Failed check for existing role".to_string(),
                            source: Box::new(err),
                        })?;

                    if duplicate_role.is_some() {
                        update(role::table)
                            .filter(
                                role::public_key
                                    .eq(&role.public_key)
                                    .and(role::role_name.eq(&role.role_name))
                                    .and(role::service_id.eq(&role.service_id))
                                    .and(role::end_commit_num.eq(MAX_COMMIT_NUM)),
                            )
                            .set(role::end_commit_num.eq(role.start_commit_num))
                            .execute(self.conn)
                            .map(|_| ())
                            .map_err(|err| AgentStoreError::OperationError {
                                context: "Failed to update agent role".to_string(),
                                source: Some(Box::new(err)),
                            })?;
                    }

                    insert_into(role::table)
                        .values(&role)
                        .execute(self.conn)
                        .map(|_| ())
                        .map_err(|err| AgentStoreError::OperationError {
                            context: "Failed to add agent role".to_string(),
                            source: Some(Box::new(err)),
                        })?;
                }

                Ok(())
            })
    }
}
