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

use crate::client::pike::{
    AlternateId as ClientAlternateId, InheritFrom as ClientInheritFrom,
    OrganizationMetadata as ClientOrganizationMetadata, PikeAgent as ClientPikeAgent,
    PikeOrganization as ClientPikeOrganization, PikeRole as ClientPikeRole,
};

use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct AlternateId {
    pub id_type: String,
    pub id: String,
}

impl From<&AlternateId> for ClientAlternateId {
    fn from(d: &AlternateId) -> Self {
        Self {
            id_type: d.id_type.to_string(),
            id: d.id.to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct OrganizationMetadata {
    pub key: String,
    pub value: String,
    pub service_id: Option<String>,
}

impl From<&OrganizationMetadata> for ClientOrganizationMetadata {
    fn from(d: &OrganizationMetadata) -> Self {
        Self {
            key: d.key.to_string(),
            value: d.value.to_string(),
            service_id: d.service_id.as_ref().map(String::from),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct PikeOrganization {
    pub org_id: String,
    pub name: String,
    pub locations: Vec<String>,
    pub alternate_ids: Vec<AlternateId>,
    pub metadata: Vec<OrganizationMetadata>,
    pub service_id: Option<String>,
}

impl From<&PikeOrganization> for ClientPikeOrganization {
    fn from(d: &PikeOrganization) -> Self {
        Self {
            org_id: d.org_id.to_string(),
            name: d.name.to_string(),
            locations: d.locations.iter().map(String::from).collect(),
            alternate_ids: d
                .alternate_ids
                .iter()
                .map(ClientAlternateId::from)
                .collect(),
            metadata: d
                .metadata
                .iter()
                .map(ClientOrganizationMetadata::from)
                .collect(),
            service_id: d.service_id.as_ref().map(String::from),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct PikeAgent {
    pub public_key: String,
    pub org_id: String,
    pub active: bool,
    pub roles: Vec<String>,
    pub service_id: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl From<&PikeAgent> for ClientPikeAgent {
    fn from(d: &PikeAgent) -> Self {
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
pub struct PikeRole {
    pub org_id: String,
    pub name: String,
    pub description: String,
    pub active: bool,
    pub permissions: Vec<String>,
    pub inherit_from: Vec<String>,
    pub allowed_organizations: Vec<String>,
}

impl From<&PikeRole> for ClientPikeRole {
    fn from(d: &PikeRole) -> Self {
        Self {
            org_id: d.org_id.to_string(),
            name: d.name.to_string(),
            description: d.description.to_string(),
            active: d.active,
            permissions: d.permissions.iter().map(String::from).collect(),
            inherit_from: d
                .inherit_from
                .iter()
                .map(|i| ClientInheritFrom::from((i, &d.org_id)))
                .collect(),
            allowed_organizations: d.allowed_organizations.iter().map(String::from).collect(),
        }
    }
}

impl From<(&String, &String)> for ClientInheritFrom {
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
