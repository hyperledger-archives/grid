// Copyright 2018-2021 Cargill Incorporated
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

use super::diesel::models::{
    AgentModel, NewAgentModel, NewOrganizationModel, NewRoleModel, OrganizationModel, RoleModel,
};
use super::{Agent, AgentList, Organization, OrganizationList, PikeStore, PikeStoreError, Role};
use crate::commits::MAX_COMMIT_NUM;
use crate::error::{
    ConstraintViolationError, ConstraintViolationType, InternalError,
    ResourceTemporarilyUnavailableError,
};
use operations::add_agent::PikeStoreAddAgentOperation as _;
use operations::add_organizations::PikeStoreAddOrganizationsOperation as _;
use operations::fetch_agent::PikeStoreFetchAgentOperation as _;
use operations::fetch_organization::PikeStoreFetchOrganizationOperation as _;
use operations::list_agents::PikeStoreListAgentsOperation as _;
use operations::list_organizations::PikeStoreListOrganizationsOperation as _;
use operations::update_agent::PikeStoreUpdateAgentOperation as _;
use operations::PikeStoreOperations;

/// Manages creating agents in the database
#[derive(Clone)]
pub struct DieselPikeStore<C: diesel::Connection + 'static> {
    connection_pool: Pool<ConnectionManager<C>>,
}

impl<C: diesel::Connection> DieselPikeStore<C> {
    /// Creates a new DieselPikeStore
    ///
    /// # Arguments
    ///
    ///  * `connection_pool`: connection pool to the database
    pub fn new(connection_pool: Pool<ConnectionManager<C>>) -> Self {
        DieselPikeStore { connection_pool }
    }
}

#[cfg(feature = "postgres")]
impl PikeStore for DieselPikeStore<diesel::pg::PgConnection> {
    fn add_agent(&self, agent: Agent) -> Result<(), PikeStoreError> {
        PikeStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            PikeStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .add_agent(agent.clone().into(), make_role_models(&agent))
    }

    fn list_agents(
        &self,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<AgentList, PikeStoreError> {
        PikeStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            PikeStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .list_agents(service_id, offset, limit)
    }

    fn fetch_agent(
        &self,
        pub_key: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Agent>, PikeStoreError> {
        PikeStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            PikeStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .fetch_agent(pub_key, service_id)
    }

    fn update_agent(&self, agent: Agent) -> Result<(), PikeStoreError> {
        PikeStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            PikeStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .update_agent(agent.clone().into(), make_role_models(&agent))
    }

    fn add_organizations(&self, orgs: Vec<Organization>) -> Result<(), PikeStoreError> {
        PikeStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            PikeStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .add_organizations(orgs.iter().map(|org| org.clone().into()).collect())
    }

    fn list_organizations(
        &self,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<OrganizationList, PikeStoreError> {
        PikeStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            PikeStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .list_organizations(service_id, offset, limit)
    }

    fn fetch_organization(
        &self,
        org_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Organization>, PikeStoreError> {
        PikeStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            PikeStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .fetch_organization(org_id, service_id)
    }
}

#[cfg(feature = "sqlite")]
impl PikeStore for DieselPikeStore<diesel::sqlite::SqliteConnection> {
    fn add_agent(&self, agent: Agent) -> Result<(), PikeStoreError> {
        PikeStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            PikeStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .add_agent(agent.clone().into(), make_role_models(&agent))
    }

    fn list_agents(
        &self,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<AgentList, PikeStoreError> {
        PikeStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            PikeStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .list_agents(service_id, offset, limit)
    }

    fn fetch_agent(
        &self,
        pub_key: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Agent>, PikeStoreError> {
        PikeStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            PikeStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .fetch_agent(pub_key, service_id)
    }

    fn update_agent(&self, agent: Agent) -> Result<(), PikeStoreError> {
        PikeStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            PikeStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .update_agent(agent.clone().into(), make_role_models(&agent))
    }

    fn add_organizations(&self, orgs: Vec<Organization>) -> Result<(), PikeStoreError> {
        PikeStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            PikeStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .add_organizations(orgs.iter().map(|org| org.clone().into()).collect())
    }

    fn list_organizations(
        &self,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<OrganizationList, PikeStoreError> {
        PikeStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            PikeStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .list_organizations(service_id, offset, limit)
    }

    fn fetch_organization(
        &self,
        org_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Organization>, PikeStoreError> {
        PikeStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            PikeStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .fetch_organization(org_id, service_id)
    }
}

impl From<RoleModel> for Role {
    fn from(role: RoleModel) -> Self {
        Self {
            public_key: role.public_key,
            role_name: role.role_name,
            start_commit_num: role.start_commit_num,
            end_commit_num: role.end_commit_num,
            service_id: role.service_id,
        }
    }
}

impl From<(AgentModel, Vec<RoleModel>)> for Agent {
    fn from((agent_model, role_models): (AgentModel, Vec<RoleModel>)) -> Self {
        Self {
            public_key: agent_model.public_key,
            org_id: agent_model.org_id,
            active: agent_model.active,
            metadata: agent_model.metadata,
            roles: role_models
                .iter()
                .map(|role| role.role_name.to_string())
                .collect(),
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
            metadata: self.metadata,
            start_commit_num: self.start_commit_num,
            end_commit_num: MAX_COMMIT_NUM,
            service_id: self.service_id,
        }
    }
}

pub fn make_role_models(agent: &Agent) -> Vec<NewRoleModel> {
    let mut roles = Vec::new();

    for role in &agent.roles {
        roles.push(NewRoleModel {
            public_key: agent.public_key.to_string(),
            role_name: role.to_string(),
            start_commit_num: agent.start_commit_num,
            end_commit_num: agent.end_commit_num,
            service_id: agent.service_id.clone(),
        })
    }

    roles
}

impl From<OrganizationModel> for Organization {
    fn from(org: OrganizationModel) -> Self {
        Self {
            org_id: org.org_id,
            name: org.name,
            address: org.address,
            metadata: org.metadata,
            start_commit_num: org.start_commit_num,
            end_commit_num: org.end_commit_num,
            service_id: org.service_id,
        }
    }
}

impl From<NewOrganizationModel> for Organization {
    fn from(org: NewOrganizationModel) -> Self {
        Self {
            org_id: org.org_id,
            name: org.name,
            address: org.address,
            metadata: org.metadata,
            start_commit_num: org.start_commit_num,
            end_commit_num: org.end_commit_num,
            service_id: org.service_id,
        }
    }
}

impl Into<NewOrganizationModel> for Organization {
    fn into(self) -> NewOrganizationModel {
        NewOrganizationModel {
            org_id: self.org_id,
            name: self.name,
            address: self.address,
            metadata: self.metadata,
            start_commit_num: self.start_commit_num,
            end_commit_num: self.end_commit_num,
            service_id: self.service_id,
        }
    }
}

impl From<diesel::result::Error> for PikeStoreError {
    fn from(err: diesel::result::Error) -> PikeStoreError {
        match err {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            ) => PikeStoreError::ConstraintViolationError(
                ConstraintViolationError::from_source_with_violation_type(
                    ConstraintViolationType::Unique,
                    Box::new(err),
                ),
            ),
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::ForeignKeyViolation,
                _,
            ) => PikeStoreError::ConstraintViolationError(
                ConstraintViolationError::from_source_with_violation_type(
                    ConstraintViolationType::ForeignKey,
                    Box::new(err),
                ),
            ),
            _ => PikeStoreError::InternalError(InternalError::from_source(Box::new(err))),
        }
    }
}

impl From<diesel::r2d2::PoolError> for PikeStoreError {
    fn from(err: diesel::r2d2::PoolError) -> PikeStoreError {
        PikeStoreError::ResourceTemporarilyUnavailableError(
            ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
        )
    }
}
