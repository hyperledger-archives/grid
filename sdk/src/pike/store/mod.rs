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

mod builder;
#[cfg(feature = "diesel")]
pub(in crate) mod diesel;
mod error;

use crate::paging::Paging;

#[cfg(feature = "diesel")]
pub use self::diesel::DieselPikeStore;
pub use builder::{
    AgentBuilder, AlternateIdBuilder, OrganizationBuilder, OrganizationMetadataBuilder, RoleBuilder,
};
pub use error::PikeStoreError;

/// Represents a Grid Agent
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Agent {
    public_key: String,
    org_id: String,
    active: bool,
    metadata: Vec<u8>,
    roles: Vec<String>,
    // The indicators of the start and stop for the slowly-changing dimensions.
    start_commit_num: i64,
    end_commit_num: i64,

    service_id: Option<String>,

    last_updated: Option<i64>,
}

impl Agent {
    /// Returns the public key of the Agent
    pub fn public_key(&self) -> &str {
        &self.public_key
    }

    /// Returns the organization ID of the organization the Agent belongs to
    pub fn org_id(&self) -> &str {
        &self.org_id
    }

    /// Returns the `active` status of the Agent
    pub fn active(&self) -> bool {
        self.active
    }

    /// Returns the metadata of the Agent
    pub fn metadata(&self) -> &[u8] {
        &self.metadata
    }

    /// Returns the roles of the Agent
    pub fn roles(&self) -> &[String] {
        &self.roles
    }

    /// Returns the `start_commit_num` for this Agent
    pub fn start_commit_num(&self) -> &i64 {
        &self.start_commit_num
    }

    /// Returns the `end_commit_num` for this Agent
    pub fn end_commit_num(&self) -> &i64 {
        &self.end_commit_num
    }

    /// Returns the service ID for this Agent
    pub fn service_id(&self) -> Option<&str> {
        self.service_id.as_deref()
    }

    /// Returns the last updated timestamp for the Agent
    pub fn last_updated(&self) -> Option<&i64> {
        self.last_updated.as_ref()
    }
}

/// Represents a Grid Role
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Role {
    name: String,
    org_id: String,
    description: String,
    active: bool,
    permissions: Vec<String>,
    allowed_organizations: Vec<String>,
    inherit_from: Vec<String>,
    start_commit_num: i64,
    end_commit_num: i64,
    service_id: Option<String>,
    last_updated: Option<i64>,
}

impl Role {
    /// Return the name of this role
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Return the ID of the organization that created this role
    pub fn org_id(&self) -> &str {
        &self.org_id
    }

    /// Return the description of this role
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Return the `active` status of this role
    pub fn active(&self) -> bool {
        self.active
    }

    /// Return the permissions assigned to this role
    pub fn permissions(&self) -> &[String] {
        &self.permissions
    }

    /// Return a list of organizations that are allowed to use this role
    pub fn allowed_organizations(&self) -> &[String] {
        &self.allowed_organizations
    }

    /// Return a list of roles this role is able to inherit permissions from
    pub fn inherit_from(&self) -> &[String] {
        &self.inherit_from
    }

    /// Return the `start_commit_num` for this roole
    pub fn start_commit_num(&self) -> &i64 {
        &self.start_commit_num
    }

    /// Return the `end_commit_num` for this role
    pub fn end_commit_num(&self) -> &i64 {
        &self.end_commit_num
    }

    /// Return the service ID for this role
    pub fn service_id(&self) -> Option<&str> {
        self.service_id.as_deref()
    }

    /// Return the last updated timestamp for this role
    pub fn last_updated(&self) -> Option<&i64> {
        self.last_updated.as_ref()
    }
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
    org_id: String,
    name: String,
    locations: Vec<String>,
    alternate_ids: Vec<AlternateId>,
    metadata: Vec<OrganizationMetadata>,
    start_commit_num: i64,
    end_commit_num: i64,
    service_id: Option<String>,
    last_updated: Option<i64>,
}

impl Organization {
    /// Return the unique identifier of the organization
    pub fn org_id(&self) -> &str {
        &self.org_id
    }

    /// Return the name of the organization
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Return the locations of the organization
    pub fn locations(&self) -> &[String] {
        &self.locations
    }

    /// Return the Alternate IDs of the organization
    pub fn alternate_ids(&self) -> &[AlternateId] {
        &self.alternate_ids
    }

    /// Return the metadata of the organization
    pub fn metadata(&self) -> &[OrganizationMetadata] {
        &self.metadata
    }

    /// Return the start commit num of the organization
    pub fn start_commit_num(&self) -> &i64 {
        &self.start_commit_num
    }

    /// Return the end commit num of the organization
    pub fn end_commit_num(&self) -> &i64 {
        &self.end_commit_num
    }

    /// Return the service ID of the organization
    pub fn service_id(&self) -> Option<&str> {
        self.service_id.as_deref()
    }

    /// Return the last updated timestamp of the organization
    pub fn last_updated(&self) -> Option<&i64> {
        self.last_updated.as_ref()
    }
}

/// Represents a Grid Alternate ID
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct AlternateId {
    org_id: String,
    alternate_id_type: String,
    alternate_id: String,
    start_commit_num: i64,
    end_commit_num: i64,
    service_id: Option<String>,
}

impl AlternateId {
    /// Return the organization ID associated with the Alternate ID
    pub fn org_id(&self) -> &str {
        &self.org_id
    }

    /// Return the type of this Alternate ID
    pub fn alternate_id_type(&self) -> &str {
        &self.alternate_id_type
    }

    /// Return the unique identifier of the Alternate ID
    pub fn alternate_id(&self) -> &str {
        &self.alternate_id
    }

    /// Return the start commit num of the Alternate ID
    pub fn start_commit_num(&self) -> &i64 {
        &self.start_commit_num
    }

    /// Return the end commit num of the Alternate ID
    pub fn end_commit_num(&self) -> &i64 {
        &self.end_commit_num
    }

    /// Return the service ID associated with the Alternate ID
    pub fn service_id(&self) -> Option<&str> {
        self.service_id.as_deref()
    }
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
    key: String,
    value: String,
    start_commit_num: i64,
    end_commit_num: i64,
    service_id: Option<String>,
}

impl OrganizationMetadata {
    /// Return the key of the metadata's internal key-value pair
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Return the value of the metadata's internal key-value pair
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Return the start commit num of the metadata
    pub fn start_commit_num(&self) -> &i64 {
        &self.start_commit_num
    }

    /// Return the end commit num of the metadata
    pub fn end_commit_num(&self) -> &i64 {
        &self.end_commit_num
    }

    /// Return the service ID associated with the metadata
    pub fn service_id(&self) -> Option<&str> {
        self.service_id.as_deref()
    }
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
