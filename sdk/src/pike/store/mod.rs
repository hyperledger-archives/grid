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

//! Data store for writing and reading Pike smart contract state.
//!
//! The [`PikeStore`] trait provides the public interface for storing Pike agents, organizations
//! and roles Grid provides the following implementations:
//!
//! * [`DieselPikeStore`] - A database-backed store, powered by [`Diesel`], that currently
//!   supports SQLite databases (with the `sqlite` feature) and PostgreSQL databases (with the
//!   `postgres` feature).
//!
//! [`PikeStore`]: trait.PikeStore.html
//! [`DieselPikeStore`]: diesel/struct.DieselPikeStore.html
//! [`Diesel`]: https://crates.io/crates/diesel

#[cfg(feature = "diesel")]
pub(in crate) mod diesel;
mod error;

use crate::paging::Paging;

#[cfg(feature = "diesel")]
pub use self::diesel::DieselPikeStore;
pub use error::PikeStoreError;

/// Represents a Grid Agent
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Agent {
    pub public_key: String,
    pub org_id: String,
    pub active: bool,
    pub metadata: Vec<u8>,
    pub roles: Vec<String>,
    // The indicators of the start and stop for the slowly-changing dimensions.
    pub start_commit_num: i64,
    pub end_commit_num: i64,

    pub service_id: Option<String>,

    pub last_updated: Option<i64>,
}

/// Represents a Grid Role
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Role {
    pub name: String,
    pub org_id: String,
    pub description: String,
    pub active: bool,
    pub permissions: Vec<String>,
    pub allowed_organizations: Vec<String>,
    pub inherit_from: Vec<String>,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
    pub last_updated: Option<i64>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct RoleList {
    pub data: Vec<Role>,
    pub paging: Paging,
}

impl RoleList {
    pub fn new(data: Vec<Role>, paging: Paging) -> Self {
        Self { data, paging }
    }
}

/// Represents a Grid Organization
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Organization {
    pub org_id: String,
    pub name: String,
    pub locations: Vec<String>,
    pub alternate_ids: Vec<AlternateId>,
    pub metadata: Vec<OrganizationMetadata>,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
    pub last_updated: Option<i64>,
}

/// Represents a Grid Alternate ID
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct AlternateId {
    pub org_id: String,
    pub alternate_id_type: String,
    pub alternate_id: String,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

pub struct OrganizationList {
    pub data: Vec<Organization>,
    pub paging: Paging,
}

impl OrganizationList {
    pub fn new(data: Vec<Organization>, paging: Paging) -> Self {
        Self { data, paging }
    }
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct AgentList {
    pub data: Vec<Agent>,
    pub paging: Paging,
}

impl AgentList {
    pub fn new(data: Vec<Agent>, paging: Paging) -> Self {
        Self { data, paging }
    }
}

/// Represents a Grid Organization metadata key-value pair
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct OrganizationMetadata {
    pub key: String,
    pub value: String,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

pub trait PikeStore: Send + Sync {
    /// Adds an agent to the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `agent` - The agent to be added
    fn add_agent(&self, agent: Agent) -> Result<(), PikeStoreError>;

    /// Adds or updates a role to the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `role` - The role to be added
    fn add_role(&self, role: Role) -> Result<(), PikeStoreError>;

    ///  Lists agents from the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `service_id` - The service id to list agents for
    ///  * `offset` - The index of the first in storage to retrieve
    ///  * `limit` - The number of items to retrieve from the offset
    fn list_agents(
        &self,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<AgentList, PikeStoreError>;

    ///  Lists roles from the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `org_id` - The organization id to list roles for
    ///  * `service_id` - The service id to list roles for
    ///  * `offset` - The index of the first in storage to retrieve
    ///  * `limit` - The number of items to retrieve from the offset
    fn list_roles_for_organization(
        &self,
        org_id: &str,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<RoleList, PikeStoreError>;

    /// Fetches an agent from the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `pub_key` - This public key of the agent to fetch
    ///  * `service_id` - The service id of the agent to fetch
    fn get_agent(
        &self,
        pub_key: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Agent>, PikeStoreError>;

    /// Fetches a role from the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `name` - The role to fetch
    ///  * `service_id` - The service id of the role to fetch
    fn get_role(
        &self,
        name: &str,
        org_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Role>, PikeStoreError>;

    /// Updates an agent in the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `agent` - The updated agent to add
    fn update_agent(&self, agent: Agent) -> Result<(), PikeStoreError>;

    /// Deletes a role from the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `address` - The state address of the role to delete
    ///  * `current_commit_num` - The current commit number to update the chain record
    fn delete_role(&self, address: &str, current_commit_num: i64) -> Result<(), PikeStoreError>;

    /// Adds an organization to the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `org` - The Organization to be added
    fn add_organization(&self, org: Organization) -> Result<(), PikeStoreError>;

    ///  Lists organizations from the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `service_id` - The service ID to list organizations for
    ///  * `offset` - The index of the first in storage to retrieve
    ///  * `limit` - The number of items to retrieve from the offset
    fn list_organizations(
        &self,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<OrganizationList, PikeStoreError>;

    /// Fetches an organization from the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `org_id` - This organization ID to fetch
    ///  * `service_id` - The service ID of the organization to fetch
    fn get_organization(
        &self,
        org_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Organization>, PikeStoreError>;
}

impl<PS> PikeStore for Box<PS>
where
    PS: PikeStore + ?Sized,
{
    fn add_agent(&self, agent: Agent) -> Result<(), PikeStoreError> {
        (**self).add_agent(agent)
    }

    fn add_role(&self, role: Role) -> Result<(), PikeStoreError> {
        (**self).add_role(role)
    }

    fn list_agents(
        &self,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<AgentList, PikeStoreError> {
        (**self).list_agents(service_id, offset, limit)
    }

    fn list_roles_for_organization(
        &self,
        org_id: &str,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<RoleList, PikeStoreError> {
        (**self).list_roles_for_organization(org_id, service_id, offset, limit)
    }

    fn get_agent(
        &self,
        pub_key: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Agent>, PikeStoreError> {
        (**self).get_agent(pub_key, service_id)
    }

    fn get_role(
        &self,
        name: &str,
        org_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Role>, PikeStoreError> {
        (**self).get_role(name, org_id, service_id)
    }

    fn update_agent(&self, agent: Agent) -> Result<(), PikeStoreError> {
        (**self).update_agent(agent)
    }

    fn delete_role(&self, address: &str, current_commit_num: i64) -> Result<(), PikeStoreError> {
        (**self).delete_role(address, current_commit_num)
    }

    fn add_organization(&self, org: Organization) -> Result<(), PikeStoreError> {
        (**self).add_organization(org)
    }

    fn list_organizations(
        &self,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<OrganizationList, PikeStoreError> {
        (**self).list_organizations(service_id, offset, limit)
    }

    fn get_organization(
        &self,
        org_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Organization>, PikeStoreError> {
        (**self).get_organization(org_id, service_id)
    }
}
