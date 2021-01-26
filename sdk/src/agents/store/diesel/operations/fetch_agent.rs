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
use crate::agents::store::diesel::{schema::agent, Agent, AgentStoreError};

use crate::agents::store::diesel::models::AgentModel;
use crate::commits::MAX_COMMIT_NUM;
use crate::error::InternalError;
use diesel::{prelude::*, result::Error::NotFound};

pub(in crate::agents::store::diesel) trait AgentStoreFetchAgentOperation {
    fn fetch_agent(
        &self,
        pub_key: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Agent>, AgentStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> AgentStoreFetchAgentOperation for AgentStoreOperations<'a, diesel::pg::PgConnection> {
    fn fetch_agent(
        &self,
        pub_key: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Agent>, AgentStoreError> {
        self.conn
            .build_transaction()
            .read_write()
            .run::<_, AgentStoreError, _>(|| {
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

                query
                    .first::<AgentModel>(self.conn)
                    .map(Agent::from)
                    .map(Some)
                    .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
                    .map_err(|err| {
                        AgentStoreError::InternalError(InternalError::from_source(Box::new(err)))
                    })
            })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> AgentStoreFetchAgentOperation
    for AgentStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn fetch_agent(
        &self,
        pub_key: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Agent>, AgentStoreError> {
        self.conn
            .immediate_transaction::<_, AgentStoreError, _>(|| {
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

                query
                    .first::<AgentModel>(self.conn)
                    .map(Agent::from)
                    .map(Some)
                    .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
                    .map_err(|err| {
                        AgentStoreError::InternalError(InternalError::from_source(Box::new(err)))
                    })
            })
    }
}
