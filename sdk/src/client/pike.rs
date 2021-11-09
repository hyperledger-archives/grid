// Copyright 2021 Cargill Incorporated
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

use std::collections::HashMap;

use crate::error::ClientError;

use super::Client;

/// The client representation of a Pike Alternate ID
#[derive(Debug, PartialEq)]
pub struct AlternateId {
    pub id_type: String,
    pub id: String,
}

/// The client representation of Grid Pike Organization metadata
#[derive(Debug, PartialEq)]
pub struct OrganizationMetadata {
    pub key: String,
    pub value: String,
    pub service_id: Option<String>,
}

/// The client representation of a Grid Pike Organization
#[derive(Debug, PartialEq)]
pub struct PikeOrganization {
    pub org_id: String,
    pub name: String,
    pub locations: Vec<String>,
    pub alternate_ids: Vec<AlternateId>,
    pub metadata: Vec<OrganizationMetadata>,
    pub service_id: Option<String>,
}

/// The client representation of a Grid Pike Agent
#[derive(Debug, PartialEq)]
pub struct PikeAgent {
    pub public_key: String,
    pub org_id: String,
    pub active: bool,
    pub roles: Vec<String>,
    pub service_id: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// The client representation of a Grid Pike Role
#[derive(Debug, PartialEq)]
pub struct PikeRole {
    pub org_id: String,
    pub name: String,
    pub description: String,
    pub active: bool,
    pub permissions: Vec<String>,
    pub inherit_from: Vec<InheritFrom>,
    pub allowed_organizations: Vec<String>,
}

/// The client representation of an inherited role
#[derive(Debug, PartialEq)]
pub struct InheritFrom {
    pub role_name: String,
    pub org_id: String,
}

pub trait PikeClient: Client {
    /// Fetches an agent based on its identifier
    ///
    /// # Arguments
    ///
    /// * `id` - the agent identifier, also known as public key
    /// * `service_id` - optional - the service ID to fetch the agent from
    fn get_agent(&self, id: String, service_id: Option<&str>) -> Result<PikeAgent, ClientError>;

    /// Fetches agents
    ///
    /// # Arguments
    ///
    /// * `service_id` - optional - the service ID to fetch the agents from
    fn list_agents(&self, service_id: Option<&str>) -> Result<Vec<PikeAgent>, ClientError>;

    /// Fetches organization by ID
    ///
    /// # Arguments
    ///
    /// * `id` - the Organization ID
    /// * `service_id` - optional - the service ID to fetch the organization from
    fn get_organization(
        &self,
        id: String,
        service_id: Option<&str>,
    ) -> Result<PikeOrganization, ClientError>;

    /// Fetches all organizations
    ///
    /// # Arguments
    ///
    /// * `service_id` - optional - the service ID to fetch the organizations from
    fn list_organizations(
        &self,
        service_id: Option<&str>,
    ) -> Result<Vec<PikeOrganization>, ClientError>;

    /// Fetches a single role from an organization based on name
    ///
    /// # Arguments
    ///
    /// * `org_id` - identifier of the role's organization
    /// * `name` - the name of the role (identifier)
    /// * `service_id` - optional - the service ID to fetch the role from
    fn get_role(
        &self,
        org_id: String,
        name: String,
        service_id: Option<&str>,
    ) -> Result<PikeRole, ClientError>;

    /// Fetches a list of roles for the organization
    ///
    /// # Arguments
    ///
    /// * `org_id` - identifier of the role's organization
    /// * `service_id` - optional - the service ID to fetch the roles from
    fn list_roles(
        &self,
        org_id: String,
        service_id: Option<&str>,
    ) -> Result<Vec<PikeRole>, ClientError>;
}
