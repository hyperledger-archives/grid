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
use crate::pike::store::diesel::{
    schema::{pike_agent, pike_agent_role_assoc},
    PikeStoreError,
};

use crate::commits::MAX_COMMIT_NUM;
use crate::error::InternalError;
use crate::pike::store::diesel::models::{
    AgentModel, NewAgentModel, NewRoleAssociationModel, RoleAssociationModel,
};
use diesel::{
    dsl::{insert_into, update},
    prelude::*,
    result::Error as dsl_error,
};

pub(in crate::pike::store::diesel) trait PikeStoreAddAgentOperation {
    fn add_agent(
        &self,
        agent: NewAgentModel,
        roles: Vec<NewRoleAssociationModel>,
    ) -> Result<(), PikeStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> PikeStoreAddAgentOperation for PikeStoreOperations<'a, diesel::pg::PgConnection> {
    fn add_agent(
        &self,
        agent: NewAgentModel,
        roles: Vec<NewRoleAssociationModel>,
    ) -> Result<(), PikeStoreError> {
        self.conn.transaction::<_, PikeStoreError, _>(|| {
            let duplicate_agent = pike_agent::table
                .filter(
                    pike_agent::public_key
                        .eq(&agent.public_key)
                        .and(pike_agent::service_id.eq(&agent.service_id))
                        .and(pike_agent::end_commit_num.eq(MAX_COMMIT_NUM)),
                )
                .first::<AgentModel>(self.conn)
                .map(Some)
                .or_else(|err| {
                    if err == dsl_error::NotFound {
                        Ok(None)
                    } else {
                        Err(err)
                    }
                })
                .map_err(|err| {
                    PikeStoreError::InternalError(InternalError::from_source(Box::new(err)))
                })?;

            if duplicate_agent.is_some() {
                update(pike_agent::table)
                    .filter(
                        pike_agent::public_key
                            .eq(&agent.public_key)
                            .and(pike_agent::service_id.eq(&agent.service_id))
                            .and(pike_agent::end_commit_num.eq(MAX_COMMIT_NUM)),
                    )
                    .set(pike_agent::end_commit_num.eq(agent.start_commit_num))
                    .execute(self.conn)
                    .map(|_| ())
                    .map_err(PikeStoreError::from)?;
            }

            insert_into(pike_agent::table)
                .values(&agent)
                .execute(self.conn)
                .map(|_| ())
                .map_err(PikeStoreError::from)?;

            for role in roles {
                let duplicate_role = pike_agent_role_assoc::table
                    .filter(
                        pike_agent_role_assoc::agent_public_key
                            .eq(&role.agent_public_key)
                            .and(pike_agent_role_assoc::agent_public_key.eq(&agent.public_key))
                            .and(pike_agent_role_assoc::org_id.eq(&agent.org_id))
                            .and(pike_agent_role_assoc::org_id.eq(&role.org_id))
                            .and(pike_agent_role_assoc::role_name.eq(&role.role_name))
                            .and(pike_agent_role_assoc::service_id.eq(&role.service_id))
                            .and(pike_agent_role_assoc::end_commit_num.eq(MAX_COMMIT_NUM)),
                    )
                    .first::<RoleAssociationModel>(self.conn)
                    .map(Some)
                    .or_else(|err| {
                        if err == dsl_error::NotFound {
                            Ok(None)
                        } else {
                            Err(err)
                        }
                    })
                    .map_err(|err| {
                        PikeStoreError::InternalError(InternalError::from_source(Box::new(err)))
                    })?;

                if duplicate_role.is_some() {
                    update(pike_agent_role_assoc::table)
                        .filter(
                            pike_agent_role_assoc::agent_public_key
                                .eq(&role.agent_public_key)
                                .and(pike_agent_role_assoc::agent_public_key.eq(&agent.public_key))
                                .and(pike_agent_role_assoc::org_id.eq(&agent.org_id))
                                .and(pike_agent_role_assoc::org_id.eq(&role.org_id))
                                .and(pike_agent_role_assoc::role_name.eq(&role.role_name))
                                .and(pike_agent_role_assoc::service_id.eq(&role.service_id))
                                .and(pike_agent_role_assoc::end_commit_num.eq(MAX_COMMIT_NUM)),
                        )
                        .set(pike_agent_role_assoc::end_commit_num.eq(&role.start_commit_num))
                        .execute(self.conn)
                        .map(|_| ())
                        .map_err(PikeStoreError::from)?;
                }

                insert_into(pike_agent_role_assoc::table)
                    .values(&role)
                    .execute(self.conn)
                    .map(|_| ())
                    .map_err(PikeStoreError::from)?;
            }

            Ok(())
        })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> PikeStoreAddAgentOperation for PikeStoreOperations<'a, diesel::sqlite::SqliteConnection> {
    fn add_agent(
        &self,
        agent: NewAgentModel,
        roles: Vec<NewRoleAssociationModel>,
    ) -> Result<(), PikeStoreError> {
        self.conn.transaction::<_, PikeStoreError, _>(|| {
            let duplicate_agent = pike_agent::table
                .filter(
                    pike_agent::public_key
                        .eq(&agent.public_key)
                        .and(pike_agent::service_id.eq(&agent.service_id))
                        .and(pike_agent::end_commit_num.eq(MAX_COMMIT_NUM)),
                )
                .first::<AgentModel>(self.conn)
                .map(Some)
                .or_else(|err| {
                    if err == dsl_error::NotFound {
                        Ok(None)
                    } else {
                        Err(err)
                    }
                })
                .map_err(|err| {
                    PikeStoreError::InternalError(InternalError::from_source(Box::new(err)))
                })?;

            if duplicate_agent.is_some() {
                update(pike_agent::table)
                    .filter(
                        pike_agent::public_key
                            .eq(&agent.public_key)
                            .and(pike_agent::service_id.eq(&agent.service_id))
                            .and(pike_agent::end_commit_num.eq(MAX_COMMIT_NUM)),
                    )
                    .set(pike_agent::end_commit_num.eq(agent.start_commit_num))
                    .execute(self.conn)
                    .map(|_| ())
                    .map_err(PikeStoreError::from)?;
            }

            insert_into(pike_agent::table)
                .values(&agent)
                .execute(self.conn)
                .map(|_| ())
                .map_err(PikeStoreError::from)?;

            for role in roles {
                let duplicate_role = pike_agent_role_assoc::table
                    .filter(
                        pike_agent_role_assoc::agent_public_key
                            .eq(&role.agent_public_key)
                            .and(pike_agent_role_assoc::agent_public_key.eq(&agent.public_key))
                            .and(pike_agent_role_assoc::org_id.eq(&agent.org_id))
                            .and(pike_agent_role_assoc::org_id.eq(&role.org_id))
                            .and(pike_agent_role_assoc::role_name.eq(&role.role_name))
                            .and(pike_agent_role_assoc::service_id.eq(&role.service_id))
                            .and(pike_agent_role_assoc::end_commit_num.eq(MAX_COMMIT_NUM)),
                    )
                    .first::<RoleAssociationModel>(self.conn)
                    .map(Some)
                    .or_else(|err| {
                        if err == dsl_error::NotFound {
                            Ok(None)
                        } else {
                            Err(err)
                        }
                    })
                    .map_err(|err| {
                        PikeStoreError::InternalError(InternalError::from_source(Box::new(err)))
                    })?;

                if duplicate_role.is_some() {
                    update(pike_agent_role_assoc::table)
                        .filter(
                            pike_agent_role_assoc::agent_public_key
                                .eq(&role.agent_public_key)
                                .and(pike_agent_role_assoc::agent_public_key.eq(&agent.public_key))
                                .and(pike_agent_role_assoc::org_id.eq(&agent.org_id))
                                .and(pike_agent_role_assoc::org_id.eq(&role.org_id))
                                .and(pike_agent_role_assoc::role_name.eq(&role.role_name))
                                .and(pike_agent_role_assoc::service_id.eq(&role.service_id))
                                .and(pike_agent_role_assoc::end_commit_num.eq(MAX_COMMIT_NUM)),
                        )
                        .set(pike_agent_role_assoc::end_commit_num.eq(&role.start_commit_num))
                        .execute(self.conn)
                        .map(|_| ())
                        .map_err(PikeStoreError::from)?;
                }

                insert_into(pike_agent_role_assoc::table)
                    .values(&role)
                    .execute(self.conn)
                    .map(|_| ())
                    .map_err(PikeStoreError::from)?;
            }

            Ok(())
        })
    }
}
