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

use super::{
    AlternateId, InheritFrom, OrganizationMetadata, PikeAgent, PikeClient, PikeOrganization,
    PikeRole,
};
use crate::client::reqwest::{fetch_entities_list, fetch_entity, post_batches};
use crate::client::Client;
use crate::error::ClientError;

use sawtooth_sdk::messages::batch::BatchList;

const AGENT_ROUTE: &str = "agent";
const ORGANIZATION_ROUTE: &str = "organization";
const ROLE_ROUTE: &str = "role";

#[derive(Debug, Deserialize)]
struct AlternateIdDto {
    pub id_type: String,
    pub id: String,
}

impl From<&AlternateIdDto> for AlternateId {
    fn from(d: &AlternateIdDto) -> Self {
        Self {
            id_type: d.id_type.to_string(),
            id: d.id.to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct OrganizationMetadataDto {
    pub key: String,
    pub value: String,
    pub service_id: Option<String>,
}

impl From<&OrganizationMetadataDto> for OrganizationMetadata {
    fn from(d: &OrganizationMetadataDto) -> Self {
        Self {
            key: d.key.to_string(),
            value: d.value.to_string(),
            service_id: d.service_id.as_ref().map(String::from),
        }
    }
}

#[derive(Debug, Deserialize)]
struct PikeOrganizationDto {
    pub org_id: String,
    pub name: String,
    pub locations: Vec<String>,
    pub alternate_ids: Vec<AlternateIdDto>,
    pub metadata: Vec<OrganizationMetadataDto>,
    pub service_id: Option<String>,
}

impl From<&PikeOrganizationDto> for PikeOrganization {
    fn from(d: &PikeOrganizationDto) -> Self {
        Self {
            org_id: d.org_id.to_string(),
            name: d.name.to_string(),
            locations: d.locations.iter().map(String::from).collect(),
            alternate_ids: d.alternate_ids.iter().map(AlternateId::from).collect(),
            metadata: d.metadata.iter().map(OrganizationMetadata::from).collect(),
            service_id: d.service_id.as_ref().map(String::from),
        }
    }
}

#[derive(Debug, Deserialize)]
struct PikeAgentDto {
    pub public_key: String,
    pub org_id: String,
    pub active: bool,
    pub roles: Vec<String>,
    pub service_id: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl From<&PikeAgentDto> for PikeAgent {
    fn from(d: &PikeAgentDto) -> Self {
        Self {
            public_key: d.public_key.to_string(),
            org_id: d.org_id.to_string(),
            active: d.active,
            roles: d.roles.iter().map(String::from).collect(),
            service_id: d.service_id.as_ref().map(String::from),
            metadata: d.metadata.clone(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct PikeRoleDto {
    pub org_id: String,
    pub name: String,
    pub description: String,
    pub active: bool,
    pub permissions: Vec<String>,
    pub inherit_from: Vec<String>,
    pub allowed_organizations: Vec<String>,
}

impl From<&PikeRoleDto> for PikeRole {
    fn from(d: &PikeRoleDto) -> Self {
        Self {
            org_id: d.org_id.to_string(),
            name: d.name.to_string(),
            description: d.description.to_string(),
            active: d.active,
            permissions: d.permissions.iter().map(String::from).collect(),
            inherit_from: d
                .inherit_from
                .iter()
                .map(|i| InheritFrom::from((i, &d.org_id)))
                .collect(),
            allowed_organizations: d.allowed_organizations.iter().map(String::from).collect(),
        }
    }
}

impl From<(&String, &String)> for InheritFrom {
    fn from((role, org): (&String, &String)) -> Self {
        let mut ifoid = org.to_string();
        let mut ifname = role.to_string();
        if role.contains('.') {
            let inherit_from: Vec<&str> = role.split('.').collect();
            ifoid = inherit_from[0].to_string();
            ifname = inherit_from[1].to_string();
        }
        Self {
            role_name: ifname,
            org_id: ifoid,
        }
    }
}

/// The Reqwest implementation of the Pike client
pub struct ReqwestPikeClient {
    url: String,
}

impl ReqwestPikeClient {
    pub fn new(url: String) -> Self {
        Self { url }
    }
}

impl Client for ReqwestPikeClient {
    /// Submits a list of batches
    ///
    /// # Arguments
    ///
    /// * `wait` - wait time in seconds
    /// * `batch_list` - The `BatchList` to be submitted
    /// * `service_id` - optional - the service ID to post batches to if running splinter
    fn post_batches(
        &self,
        wait: u64,
        batch_list: &BatchList,
        service_id: Option<&str>,
    ) -> Result<(), ClientError> {
        post_batches(&self.url, wait, batch_list, service_id)
    }
}

impl PikeClient for ReqwestPikeClient {
    /// Fetches an agent based on its identifier
    ///
    /// # Arguments
    ///
    /// * `id` - the agent identifier, also known as public key
    /// * `service_id` - optional - the service ID to fetch the agent from
    fn get_agent(&self, id: String, service_id: Option<&str>) -> Result<PikeAgent, ClientError> {
        let dto =
            fetch_entity::<PikeAgentDto>(&self.url, format!("{}/{}", AGENT_ROUTE, id), service_id)?;
        Ok(PikeAgent::from(&dto))
    }

    /// Fetches agents
    ///
    /// # Arguments
    ///
    /// * `service_id` - optional - the service ID to fetch the agents from
    fn list_agents(&self, service_id: Option<&str>) -> Result<Vec<PikeAgent>, ClientError> {
        let dto_vec = fetch_entities_list::<PikeAgentDto>(
            &self.url,
            AGENT_ROUTE.to_string(),
            service_id,
            None,
        )?;
        Ok(dto_vec.iter().map(PikeAgent::from).collect())
    }

    /// Fetches an organization
    ///
    /// # Arguments
    ///
    /// * `id` - the Organization ID
    /// * `service_id` - optional - the service ID to fetch the organization from
    fn get_organization(
        &self,
        id: String,
        service_id: Option<&str>,
    ) -> Result<PikeOrganization, ClientError> {
        let dto = fetch_entity::<PikeOrganizationDto>(
            &self.url,
            format!("{}/{}", ORGANIZATION_ROUTE, id),
            service_id,
        )?;
        Ok(PikeOrganization::from(&dto))
    }

    /// Fetches all organizations
    ///
    /// # Arguments
    ///
    /// * `service_id` - optional - the service ID to fetch the organizations from
    fn list_organizations(
        &self,
        service_id: Option<&str>,
    ) -> Result<Vec<PikeOrganization>, ClientError> {
        let dto_vec = fetch_entities_list::<PikeOrganizationDto>(
            &self.url,
            ORGANIZATION_ROUTE.to_string(),
            service_id,
            None,
        )?;
        Ok(dto_vec.iter().map(PikeOrganization::from).collect())
    }

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
    ) -> Result<PikeRole, ClientError> {
        let dto = fetch_entity::<PikeRoleDto>(
            &self.url,
            format!("{}/{}/{}", ROLE_ROUTE, org_id, name),
            service_id,
        )?;
        Ok(PikeRole::from(&dto))
    }

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
    ) -> Result<Vec<PikeRole>, ClientError> {
        let dto_vec = fetch_entities_list::<PikeRoleDto>(
            &self.url,
            format!("{}/{}", ROLE_ROUTE, org_id),
            service_id,
            None,
        )?;
        Ok(dto_vec.iter().map(PikeRole::from).collect())
    }
}
