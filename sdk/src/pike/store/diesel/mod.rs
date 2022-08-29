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

//! Database backend support for the PikeStore, powered by
//! [`Diesel`](https://crates.io/crates/diesel).
//!
//! This module contains the [`DieselPikeStore`], which provides an implementation of the
//! [`PikeStore`] trait.
//!
//! [`DieselPikeStore`]: struct.DieselPikeStore.html
//! [`PikeStore`]: ../trait.PikeStore.html

pub mod models;
mod operations;
pub(crate) mod schema;

use diesel::connection::AnsiTransactionManager;
use diesel::r2d2::{ConnectionManager, Pool};

use super::{
    Agent, AgentList, AlternateId, Organization, OrganizationList, OrganizationMetadata, PikeStore,
    PikeStoreError, Role, RoleList,
};
use crate::error::ResourceTemporarilyUnavailableError;
use models::{
    make_allowed_orgs_models, make_alternate_id_models, make_inherit_from_models,
    make_location_association_models, make_org_metadata_models, make_permissions_models,
    make_role_association_models,
};
use operations::add_agent::PikeStoreAddAgentOperation as _;
use operations::add_organization::PikeStoreAddOrganizationOperation as _;
use operations::add_role::PikeStoreAddRoleOperation as _;
use operations::delete_role::PikeStoreDeleteRoleOperation as _;
use operations::get_agent::PikeStoreGetAgentOperation as _;
use operations::get_organization::PikeStoreGetOrganizationOperation as _;
use operations::get_role::PikeStoreGetRoleOperation as _;
use operations::list_agents::PikeStoreListAgentsOperation as _;
use operations::list_organizations::PikeStoreListOrganizationsOperation as _;
use operations::list_roles_for_organization::PikeStoreListRolesForOrganizationOperation as _;
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
        .add_agent(agent.clone().into(), make_role_association_models(&agent))
    }

    fn add_role(&self, role: Role) -> Result<(), PikeStoreError> {
        PikeStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            PikeStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .add_role(
            role.clone().into(),
            make_inherit_from_models(&role),
            make_permissions_models(&role),
            make_allowed_orgs_models(&role),
        )
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

    fn list_roles_for_organization(
        &self,
        org_id: &str,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<RoleList, PikeStoreError> {
        PikeStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            PikeStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .list_roles_for_organization(org_id, service_id, offset, limit)
    }

    fn get_agent(
        &self,
        pub_key: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Agent>, PikeStoreError> {
        PikeStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            PikeStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .get_agent(pub_key, service_id)
    }

    fn get_role(
        &self,
        name: &str,
        org_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Role>, PikeStoreError> {
        PikeStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            PikeStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .get_role(name, org_id, service_id)
    }

    fn update_agent(&self, agent: Agent) -> Result<(), PikeStoreError> {
        PikeStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            PikeStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .update_agent(agent.clone().into(), make_role_association_models(&agent))
    }

    fn add_organization(&self, org: Organization) -> Result<(), PikeStoreError> {
        PikeStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            PikeStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .add_organization(
            org.clone().into(),
            make_location_association_models(&org),
            make_alternate_id_models(&org),
            make_org_metadata_models(&org),
        )
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

    fn get_organization(
        &self,
        org_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Organization>, PikeStoreError> {
        PikeStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            PikeStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .get_organization(org_id, service_id)
    }

    fn delete_role(&self, address: &str, current_commit_num: i64) -> Result<(), PikeStoreError> {
        PikeStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            PikeStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .delete_role(address, current_commit_num)
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
        .add_agent(agent.clone().into(), make_role_association_models(&agent))
    }

    fn add_role(&self, role: Role) -> Result<(), PikeStoreError> {
        PikeStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            PikeStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .add_role(
            role.clone().into(),
            make_inherit_from_models(&role),
            make_permissions_models(&role),
            make_allowed_orgs_models(&role),
        )
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

    fn list_roles_for_organization(
        &self,
        org_id: &str,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<RoleList, PikeStoreError> {
        PikeStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            PikeStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .list_roles_for_organization(org_id, service_id, offset, limit)
    }

    fn get_agent(
        &self,
        pub_key: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Agent>, PikeStoreError> {
        PikeStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            PikeStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .get_agent(pub_key, service_id)
    }

    fn get_role(
        &self,
        name: &str,
        org_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Role>, PikeStoreError> {
        PikeStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            PikeStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .get_role(name, org_id, service_id)
    }

    fn update_agent(&self, agent: Agent) -> Result<(), PikeStoreError> {
        PikeStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            PikeStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .update_agent(agent.clone().into(), make_role_association_models(&agent))
    }

    fn add_organization(&self, org: Organization) -> Result<(), PikeStoreError> {
        PikeStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            PikeStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .add_organization(
            org.clone().into(),
            make_location_association_models(&org),
            make_alternate_id_models(&org),
            make_org_metadata_models(&org),
        )
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

    fn get_organization(
        &self,
        org_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Organization>, PikeStoreError> {
        PikeStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            PikeStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .get_organization(org_id, service_id)
    }

    fn delete_role(&self, address: &str, current_commit_num: i64) -> Result<(), PikeStoreError> {
        PikeStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            PikeStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .delete_role(address, current_commit_num)
    }
}

pub struct DieselConnectionPikeStore<'a, C: diesel::Connection + 'static>
where
    C: diesel::Connection<TransactionManager = AnsiTransactionManager> + 'static,
    C::Backend: diesel::backend::UsesAnsiSavepointSyntax,
{
    connection: &'a C,
}

impl<'a, C> DieselConnectionPikeStore<'a, C>
where
    C: diesel::Connection<TransactionManager = AnsiTransactionManager> + 'static,
    C::Backend: diesel::backend::UsesAnsiSavepointSyntax,
{
    pub fn new(connection: &'a C) -> Self {
        DieselConnectionPikeStore { connection }
    }
}

#[cfg(feature = "postgres")]
impl<'a> PikeStore for DieselConnectionPikeStore<'a, diesel::pg::PgConnection> {
    fn add_agent(&self, agent: Agent) -> Result<(), PikeStoreError> {
        PikeStoreOperations::new(self.connection)
            .add_agent(agent.clone().into(), make_role_association_models(&agent))
    }

    fn add_role(&self, role: Role) -> Result<(), PikeStoreError> {
        PikeStoreOperations::new(self.connection).add_role(
            role.clone().into(),
            make_inherit_from_models(&role),
            make_permissions_models(&role),
            make_allowed_orgs_models(&role),
        )
    }

    fn list_agents(
        &self,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<AgentList, PikeStoreError> {
        PikeStoreOperations::new(self.connection).list_agents(service_id, offset, limit)
    }

    fn list_roles_for_organization(
        &self,
        org_id: &str,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<RoleList, PikeStoreError> {
        PikeStoreOperations::new(self.connection)
            .list_roles_for_organization(org_id, service_id, offset, limit)
    }

    fn get_agent(
        &self,
        pub_key: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Agent>, PikeStoreError> {
        PikeStoreOperations::new(self.connection).get_agent(pub_key, service_id)
    }

    fn get_role(
        &self,
        name: &str,
        org_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Role>, PikeStoreError> {
        PikeStoreOperations::new(self.connection).get_role(name, org_id, service_id)
    }

    fn update_agent(&self, agent: Agent) -> Result<(), PikeStoreError> {
        PikeStoreOperations::new(self.connection)
            .update_agent(agent.clone().into(), make_role_association_models(&agent))
    }

    fn add_organization(&self, org: Organization) -> Result<(), PikeStoreError> {
        PikeStoreOperations::new(self.connection).add_organization(
            org.clone().into(),
            make_location_association_models(&org),
            make_alternate_id_models(&org),
            make_org_metadata_models(&org),
        )
    }

    fn list_organizations(
        &self,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<OrganizationList, PikeStoreError> {
        PikeStoreOperations::new(self.connection).list_organizations(service_id, offset, limit)
    }

    fn get_organization(
        &self,
        org_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Organization>, PikeStoreError> {
        PikeStoreOperations::new(self.connection).get_organization(org_id, service_id)
    }

    fn delete_role(&self, address: &str, current_commit_num: i64) -> Result<(), PikeStoreError> {
        PikeStoreOperations::new(self.connection).delete_role(address, current_commit_num)
    }
}

#[cfg(feature = "sqlite")]
impl<'a> PikeStore for DieselConnectionPikeStore<'a, diesel::sqlite::SqliteConnection> {
    fn add_agent(&self, agent: Agent) -> Result<(), PikeStoreError> {
        PikeStoreOperations::new(self.connection)
            .add_agent(agent.clone().into(), make_role_association_models(&agent))
    }

    fn add_role(&self, role: Role) -> Result<(), PikeStoreError> {
        PikeStoreOperations::new(self.connection).add_role(
            role.clone().into(),
            make_inherit_from_models(&role),
            make_permissions_models(&role),
            make_allowed_orgs_models(&role),
        )
    }

    fn list_agents(
        &self,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<AgentList, PikeStoreError> {
        PikeStoreOperations::new(self.connection).list_agents(service_id, offset, limit)
    }

    fn list_roles_for_organization(
        &self,
        org_id: &str,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<RoleList, PikeStoreError> {
        PikeStoreOperations::new(self.connection)
            .list_roles_for_organization(org_id, service_id, offset, limit)
    }

    fn get_agent(
        &self,
        pub_key: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Agent>, PikeStoreError> {
        PikeStoreOperations::new(self.connection).get_agent(pub_key, service_id)
    }

    fn get_role(
        &self,
        name: &str,
        org_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Role>, PikeStoreError> {
        PikeStoreOperations::new(self.connection).get_role(name, org_id, service_id)
    }

    fn update_agent(&self, agent: Agent) -> Result<(), PikeStoreError> {
        PikeStoreOperations::new(self.connection)
            .update_agent(agent.clone().into(), make_role_association_models(&agent))
    }

    fn add_organization(&self, org: Organization) -> Result<(), PikeStoreError> {
        PikeStoreOperations::new(self.connection).add_organization(
            org.clone().into(),
            make_location_association_models(&org),
            make_alternate_id_models(&org),
            make_org_metadata_models(&org),
        )
    }

    fn list_organizations(
        &self,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<OrganizationList, PikeStoreError> {
        PikeStoreOperations::new(self.connection).list_organizations(service_id, offset, limit)
    }

    fn get_organization(
        &self,
        org_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Organization>, PikeStoreError> {
        PikeStoreOperations::new(self.connection).get_organization(org_id, service_id)
    }

    fn delete_role(&self, address: &str, current_commit_num: i64) -> Result<(), PikeStoreError> {
        PikeStoreOperations::new(self.connection).delete_role(address, current_commit_num)
    }
}
