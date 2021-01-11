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

pub mod models;
mod operations;
pub(in crate) mod schema;

use diesel::r2d2::{ConnectionManager, Pool};

use super::diesel::models::{AgentModel, NewAgentModel};
use super::{Agent, AgentStore, AgentStoreError};
use crate::commits::MAX_COMMIT_NUM;
use crate::error::{
    ConstraintViolationError, ConstraintViolationType, InternalError,
    ResourceTemporarilyUnavailableError,
};
use operations::add_agent::AgentStoreAddAgentOperation as _;
use operations::fetch_agent::AgentStoreFetchAgentOperation as _;
use operations::list_agents::AgentStoreListAgentsOperation as _;
use operations::update_agent::AgentStoreUpdateAgentOperation as _;
use operations::AgentStoreOperations;

/// Manages creating agents in the database
#[derive(Clone)]
pub struct DieselAgentStore<C: diesel::Connection + 'static> {
    connection_pool: Pool<ConnectionManager<C>>,
}

impl<C: diesel::Connection> DieselAgentStore<C> {
    /// Creates a new DieselAgentStore
    ///
    /// # Arguments
    ///
    ///  * `connection_pool`: connection pool to the database
    // Allow dead code if diesel feature is not enabled
    #[allow(dead_code)]
    pub fn new(connection_pool: Pool<ConnectionManager<C>>) -> Self {
        DieselAgentStore { connection_pool }
    }
}

#[cfg(feature = "postgres")]
impl AgentStore for DieselAgentStore<diesel::pg::PgConnection> {
    fn add_agent(&self, agent: Agent) -> Result<(), AgentStoreError> {
        AgentStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            AgentStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .add_agent(agent.clone().into())
    }

    fn list_agents(&self, service_id: Option<&str>) -> Result<Vec<Agent>, AgentStoreError> {
        AgentStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            AgentStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .list_agents(service_id)
    }

    fn fetch_agent(
        &self,
        pub_key: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Agent>, AgentStoreError> {
        AgentStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            AgentStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .fetch_agent(pub_key, service_id)
    }

    fn update_agent(&self, agent: Agent) -> Result<(), AgentStoreError> {
        AgentStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            AgentStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .update_agent(agent.clone().into())
    }
}

#[cfg(feature = "sqlite")]
impl AgentStore for DieselAgentStore<diesel::sqlite::SqliteConnection> {
    fn add_agent(&self, agent: Agent) -> Result<(), AgentStoreError> {
        AgentStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            AgentStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .add_agent(agent.clone().into())
    }

    fn list_agents(&self, service_id: Option<&str>) -> Result<Vec<Agent>, AgentStoreError> {
        AgentStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            AgentStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .list_agents(service_id)
    }

    fn fetch_agent(
        &self,
        pub_key: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Agent>, AgentStoreError> {
        AgentStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            AgentStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .fetch_agent(pub_key, service_id)
    }

    fn update_agent(&self, agent: Agent) -> Result<(), AgentStoreError> {
        AgentStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            AgentStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .update_agent(agent.clone().into())
    }
}

impl From<AgentModel> for Agent {
    fn from(agent_model: AgentModel) -> Self {
        Self {
            public_key: agent_model.public_key,
            org_id: agent_model.org_id,
            active: agent_model.active,
            metadata: agent_model.metadata,
            roles: agent_model.roles,
            start_commit_num: agent_model.start_commit_num,
            end_commit_num: agent_model.end_commit_num,
            service_id: agent_model.service_id,
        }
    }
}

impl Into<NewAgentModel> for Agent {
    fn into(self) -> NewAgentModel {
        NewAgentModel {
            public_key: self.public_key,
            org_id: self.org_id,
            active: self.active,
            roles: self.roles,
            metadata: self.metadata,
            start_commit_num: self.start_commit_num,
            end_commit_num: MAX_COMMIT_NUM,
            service_id: self.service_id,
        }
    }
}

impl From<diesel::result::Error> for AgentStoreError {
    fn from(err: diesel::result::Error) -> AgentStoreError {
        match err {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            ) => AgentStoreError::ConstraintViolationError(
                ConstraintViolationError::from_source_with_violation_type(
                    ConstraintViolationType::Unique,
                    Box::new(err),
                ),
            ),
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::ForeignKeyViolation,
                _,
            ) => AgentStoreError::ConstraintViolationError(
                ConstraintViolationError::from_source_with_violation_type(
                    ConstraintViolationType::ForeignKey,
                    Box::new(err),
                ),
            ),
            _ => AgentStoreError::InternalError(InternalError::from_source(Box::new(err))),
        }
    }
}

impl From<diesel::r2d2::PoolError> for AgentStoreError {
    fn from(err: diesel::r2d2::PoolError) -> AgentStoreError {
        AgentStoreError::ResourceTemporarilyUnavailableError(
            ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
        )
    }
}
