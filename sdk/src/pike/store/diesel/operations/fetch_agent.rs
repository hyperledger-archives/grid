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
    Agent, PikeStoreError,
};

use crate::commits::MAX_COMMIT_NUM;
use crate::error::InternalError;
use crate::pike::store::diesel::models::{AgentModel, RoleAssociationModel};
use diesel::{prelude::*, result::Error::NotFound};

pub(in crate::pike::store::diesel) trait PikeStoreFetchAgentOperation {
    fn fetch_agent(
        &self,
        pub_key: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Agent>, PikeStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> PikeStoreFetchAgentOperation for PikeStoreOperations<'a, diesel::pg::PgConnection> {
    fn fetch_agent(
        &self,
        pub_key: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Agent>, PikeStoreError> {
        self.conn.transaction::<_, PikeStoreError, _>(|| {
            let mut query = pike_agent::table
                .into_boxed()
                .select(pike_agent::all_columns)
                .filter(
                    pike_agent::public_key
                        .eq(&pub_key)
                        .and(pike_agent::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

            if let Some(service_id) = service_id {
                query = query.filter(pike_agent::service_id.eq(service_id));
            } else {
                query = query.filter(pike_agent::service_id.is_null());
            }

            let agent = query
                .first::<AgentModel>(self.conn)
                .map(Some)
                .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
                .map_err(|err| {
                    PikeStoreError::InternalError(InternalError::from_source(Box::new(err)))
                })?;

            match agent {
                Some(agent) => {
                    let mut query = pike_agent_role_assoc::table
                        .into_boxed()
                        .select(pike_agent_role_assoc::all_columns)
                        .filter(
                            pike_agent_role_assoc::agent_public_key
                                .eq(&pub_key)
                                .and(pike_agent_role_assoc::org_id.eq(&agent.org_id))
                                .and(pike_agent_role_assoc::end_commit_num.eq(MAX_COMMIT_NUM)),
                        );

                    if let Some(service_id) = service_id {
                        query = query.filter(pike_agent_role_assoc::service_id.eq(service_id));
                    } else {
                        query = query.filter(pike_agent_role_assoc::service_id.is_null());
                    }

                    let roles = query
                        .load::<RoleAssociationModel>(self.conn)
                        .map_err(|err| {
                            PikeStoreError::InternalError(InternalError::from_source(Box::new(err)))
                        })?;

                    Ok(Some(Agent::from((agent, roles))))
                }
                None => Ok(None),
            }
        })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> PikeStoreFetchAgentOperation
    for PikeStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn fetch_agent(
        &self,
        pub_key: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Agent>, PikeStoreError> {
        self.conn.transaction::<_, PikeStoreError, _>(|| {
            let mut query = pike_agent::table
                .into_boxed()
                .select(pike_agent::all_columns)
                .filter(
                    pike_agent::public_key
                        .eq(&pub_key)
                        .and(pike_agent::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

            if let Some(service_id) = service_id {
                query = query.filter(pike_agent::service_id.eq(service_id));
            } else {
                query = query.filter(pike_agent::service_id.is_null());
            }

            let agent = query
                .first::<AgentModel>(self.conn)
                .map(Some)
                .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
                .map_err(|err| {
                    PikeStoreError::InternalError(InternalError::from_source(Box::new(err)))
                })?;

            match agent {
                Some(agent) => {
                    let mut query = pike_agent_role_assoc::table
                        .into_boxed()
                        .select(pike_agent_role_assoc::all_columns)
                        .filter(
                            pike_agent_role_assoc::agent_public_key
                                .eq(&pub_key)
                                .and(pike_agent_role_assoc::org_id.eq(&agent.org_id))
                                .and(pike_agent_role_assoc::end_commit_num.eq(MAX_COMMIT_NUM)),
                        );

                    if let Some(service_id) = service_id {
                        query = query.filter(pike_agent_role_assoc::service_id.eq(service_id));
                    } else {
                        query = query.filter(pike_agent_role_assoc::service_id.is_null());
                    }

                    let roles = query
                        .load::<RoleAssociationModel>(self.conn)
                        .map_err(|err| {
                            PikeStoreError::InternalError(InternalError::from_source(Box::new(err)))
                        })?;

                    Ok(Some(Agent::from((agent, roles))))
                }
                None => Ok(None),
            }
        })
    }
}
