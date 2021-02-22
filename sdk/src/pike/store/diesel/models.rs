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

use super::{Agent, Organization, OrganizationMetadata, Role};
use crate::commits::MAX_COMMIT_NUM;
use crate::pike::store::diesel::schema::*;

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "pike_agent"]
pub struct NewAgentModel {
    pub public_key: String,
    pub org_id: String,
    pub active: bool,
    pub metadata: Vec<u8>,

    pub start_commit_num: i64,
    pub end_commit_num: i64,

    pub service_id: Option<String>,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "pike_agent"]
pub struct AgentModel {
    ///  This is the record id for the slowly-changing-dimensions table.
    pub id: i64,
    pub public_key: String,
    pub org_id: String,
    pub active: bool,
    pub metadata: Vec<u8>,

    pub start_commit_num: i64,
    pub end_commit_num: i64,

    pub service_id: Option<String>,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "pike_role"]
pub struct NewRoleModel {
    pub public_key: String,
    pub role_name: String,

    pub start_commit_num: i64,
    pub end_commit_num: i64,

    pub service_id: Option<String>,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "pike_role"]
pub struct RoleModel {
    pub id: i64,
    pub public_key: String,
    pub role_name: String,

    pub start_commit_num: i64,
    pub end_commit_num: i64,

    pub service_id: Option<String>,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "pike_organization"]
pub struct NewOrganizationModel {
    pub org_id: String,
    pub name: String,
    pub address: String,

    pub start_commit_num: i64,
    pub end_commit_num: i64,

    pub service_id: Option<String>,
}

#[derive(Queryable, PartialEq, Identifiable, Debug)]
#[table_name = "pike_organization"]
pub struct OrganizationModel {
    ///  This is the record id for the slowly-changing-dimensions table.
    pub id: i64,
    pub org_id: String,
    pub name: String,
    pub address: String,

    pub start_commit_num: i64,
    pub end_commit_num: i64,

    pub service_id: Option<String>,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "pike_organization_metadata"]
pub struct NewOrganizationMetadataModel {
    pub org_id: String,
    pub key: String,
    pub value: Vec<u8>,

    pub start_commit_num: i64,
    pub end_commit_num: i64,

    pub service_id: Option<String>,
}

#[derive(Queryable, PartialEq, Identifiable, Debug)]
#[table_name = "pike_organization_metadata"]
pub struct OrganizationMetadataModel {
    pub id: i64,
    pub org_id: String,
    pub key: String,
    pub value: Vec<u8>,

    pub start_commit_num: i64,
    pub end_commit_num: i64,

    pub service_id: Option<String>,
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

impl From<(OrganizationModel, Vec<OrganizationMetadata>)> for Organization {
    fn from((org, metadata): (OrganizationModel, Vec<OrganizationMetadata>)) -> Self {
        Self {
            org_id: org.org_id,
            name: org.name,
            address: org.address,
            metadata,
            start_commit_num: org.start_commit_num,
            end_commit_num: org.end_commit_num,
            service_id: org.service_id,
        }
    }
}

impl From<(NewOrganizationModel, Vec<OrganizationMetadata>)> for Organization {
    fn from((org, metadata): (NewOrganizationModel, Vec<OrganizationMetadata>)) -> Self {
        Self {
            org_id: org.org_id,
            name: org.name,
            address: org.address,
            metadata,
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
            start_commit_num: self.start_commit_num,
            end_commit_num: self.end_commit_num,
            service_id: self.service_id,
        }
    }
}

impl From<NewOrganizationMetadataModel> for OrganizationMetadata {
    fn from(metadata: NewOrganizationMetadataModel) -> Self {
        Self {
            key: metadata.key,
            value: String::from_utf8(metadata.value).unwrap(),
            start_commit_num: metadata.start_commit_num,
            end_commit_num: metadata.end_commit_num,
            service_id: metadata.service_id,
        }
    }
}

impl From<&OrganizationMetadataModel> for OrganizationMetadata {
    fn from(metadata: &OrganizationMetadataModel) -> Self {
        Self {
            key: metadata.key.to_string(),
            value: String::from_utf8(metadata.value.clone()).unwrap(),
            start_commit_num: metadata.start_commit_num,
            end_commit_num: metadata.end_commit_num,
            service_id: metadata.service_id.clone(),
        }
    }
}

pub fn make_org_metadata_models(org: &Organization) -> Vec<NewOrganizationMetadataModel> {
    let mut metadata = Vec::new();

    for data in &org.metadata {
        metadata.push(NewOrganizationMetadataModel {
            org_id: org.org_id.to_string(),
            key: data.key.to_string(),
            value: data.value.as_bytes().to_vec(),
            start_commit_num: data.start_commit_num,
            end_commit_num: data.end_commit_num,
            service_id: data.service_id.clone(),
        })
    }

    metadata
}
