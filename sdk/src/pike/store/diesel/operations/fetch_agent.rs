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
    schema::{agent, role},
    Agent, PikeStoreError,
};

use crate::commits::MAX_COMMIT_NUM;
use crate::error::InternalError;
use crate::pike::store::diesel::models::{AgentModel, RoleModel};
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
        self.conn
            .build_transaction()
            .read_write()
            .run::<_, PikeStoreError, _>(|| {
                let mut query = agent::table.into_boxed().select(agent::all_columns).filter(
                    agent::public_key
                        .eq(&pub_key)
                        .and(agent::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

                if let Some(service_id) = service_id {
                    query = query.filter(agent::service_id.eq(service_id));
                } else {
                    query = query.filter(agent::service_id.is_null());
                }

                let agent = query
                    .first::<AgentModel>(self.conn)
                    .map(Some)
                    .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
                    .map_err(|err| {
                        PikeStoreError::InternalError(InternalError::from_source(Box::new(err)))
                    })?;

                let mut query = role::table
                    .select(role::all_columns)
                    .into_boxed()
                    .select(role::all_columns)
                    .filter(
                        role::public_key
                            .eq(&pub_key)
                            .and(role::end_commit_num.eq(MAX_COMMIT_NUM)),
                    );

                if let Some(service_id) = service_id {
                    query = query.filter(role::service_id.eq(service_id));
                } else {
                    query = query.filter(role::service_id.is_null());
                }

                let roles = query.load::<RoleModel>(self.conn).map_err(|err| {
                    PikeStoreError::InternalError(InternalError::from_source(Box::new(err)))
                })?;

                Ok(agent.map(|agent| Agent::from((agent, roles))))
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
        self.conn.immediate_transaction::<_, PikeStoreError, _>(|| {
            let mut query = agent::table.into_boxed().select(agent::all_columns).filter(
                agent::public_key
                    .eq(&pub_key)
                    .and(agent::end_commit_num.eq(MAX_COMMIT_NUM)),
            );

            if let Some(service_id) = service_id {
                query = query.filter(agent::service_id.eq(service_id));
            } else {
                query = query.filter(agent::service_id.is_null());
            }

            let agent = query
                .first::<AgentModel>(self.conn)
                .map(Some)
                .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
                .map_err(|err| {
                    PikeStoreError::InternalError(InternalError::from_source(Box::new(err)))
                })?;

            let mut query = role::table
                .select(role::all_columns)
                .into_boxed()
                .select(role::all_columns)
                .filter(
                    role::public_key
                        .eq(&pub_key)
                        .and(role::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

            if let Some(service_id) = service_id {
                query = query.filter(role::service_id.eq(service_id));
            } else {
                query = query.filter(role::service_id.is_null());
            }

            let roles = query.load::<RoleModel>(self.conn).map_err(|err| {
                PikeStoreError::InternalError(InternalError::from_source(Box::new(err)))
            })?;

            Ok(agent.map(|agent| Agent::from((agent, roles))))
        })
    }
}
