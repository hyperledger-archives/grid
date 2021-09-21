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

//! Database representations used to implement a diesel backend for the `PikeStore`.
//! These structs differ slightly from their associated native representation to conform to
//! the requirements for storing data with a diesel backend.

use chrono::NaiveDateTime;

use super::{Agent, AlternateId, Organization, OrganizationMetadata, Role};
use crate::commits::MAX_COMMIT_NUM;
use crate::pike::addressing::{
    compute_agent_address, compute_organization_address, compute_role_address,
};
use crate::pike::store::diesel::schema::*;

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "pike_agent"]
pub struct NewAgentModel {
    pub state_address: String,
    pub public_key: String,
    pub org_id: String,
    pub active: bool,
    pub metadata: Vec<u8>,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

/// Database model representation of a Pike `Agent`
#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "pike_agent"]
pub struct AgentModel {
    ///  This is the record id for the slowly-changing-dimensions table.
    pub id: i64,
    pub state_address: String,
    pub public_key: String,
    pub org_id: String,
    pub active: bool,
    pub metadata: Vec<u8>,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
    pub last_updated: Option<NaiveDateTime>,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "pike_role"]
pub struct NewRoleModel {
    pub state_address: String,
    pub org_id: String,
    pub name: String,
    pub description: String,
    pub active: bool,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

/// Database model representation of a Pike `Role`
#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "pike_role"]
pub struct RoleModel {
    ///  This is the record id for the slowly-changing-dimensions table.
    pub id: i64,
    pub state_address: String,
    pub org_id: String,
    pub name: String,
    pub description: String,
    pub active: bool,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
    pub last_updated: Option<NaiveDateTime>,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "pike_agent_role_assoc"]
pub struct NewRoleAssociationModel {
    pub agent_public_key: String,
    pub org_id: String,
    pub role_name: String,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

/// Database model representation of a Pike `Role` associated with an `Organization` and `Agent`
#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "pike_agent_role_assoc"]
pub struct RoleAssociationModel {
    ///  This is the record id for the slowly-changing-dimensions table.
    pub id: i64,
    pub agent_public_key: String,
    pub org_id: String,
    pub role_name: String,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "pike_role_state_address_assoc"]
pub struct NewRoleStateAddressAssociationModel {
    pub state_address: String,
    pub org_id: String,
    pub name: String,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

/// Database model representation of a Pike `Role` associated with a state address
#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "pike_role_state_address_assoc"]
pub struct RoleStateAddressAssociationModel {
    ///  This is the record id for the slowly-changing-dimensions table.
    pub id: i64,
    pub state_address: String,
    pub org_id: String,
    pub name: String,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "pike_permissions"]
pub struct NewPermissionModel {
    pub role_name: String,
    pub org_id: String,
    pub name: String,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

/// Database model representation of a Pike `Permission`
#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "pike_permissions"]
pub struct PermissionModel {
    ///  This is the record id for the slowly-changing-dimensions table.
    pub id: i64,
    pub role_name: String,
    pub org_id: String,
    pub name: String,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "pike_inherit_from"]
pub struct NewInheritFromModel {
    pub role_name: String,
    pub org_id: String,
    pub inherit_from_role_name: String,
    pub inherit_from_org_id: String,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

/// Database model representation of the Pike `roles` that a `role` inherits attributes from
#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "pike_inherit_from"]
pub struct InheritFromModel {
    ///  This is the record id for the slowly-changing-dimensions table.
    pub id: i64,
    pub role_name: String,
    pub org_id: String,
    pub inherit_from_role_name: String,
    pub inherit_from_org_id: String,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "pike_allowed_orgs"]
pub struct NewAllowedOrgModel {
    pub role_name: String,
    pub org_id: String,
    pub allowed_org_id: String,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

/// Database model representation of Pike `organizations` allowed to use the specified `Role`,
/// besides the defining `organization`
#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "pike_allowed_orgs"]
pub struct AllowedOrgModel {
    ///  This is the record id for the slowly-changing-dimensions table.
    pub id: i64,
    pub role_name: String,
    pub org_id: String,
    pub allowed_org_id: String,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "pike_organization"]
pub struct NewOrganizationModel {
    pub state_address: String,
    pub org_id: String,
    pub name: String,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

/// Database model representation of a Pike `organization`
#[derive(Queryable, PartialEq, Identifiable, Debug)]
#[table_name = "pike_organization"]
pub struct OrganizationModel {
    ///  This is the record id for the slowly-changing-dimensions table.
    pub id: i64,
    pub state_address: String,
    pub org_id: String,
    pub name: String,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
    pub last_updated: Option<NaiveDateTime>,
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

/// Database model representation of a Pike `organization`'s `metadata`
#[derive(Queryable, PartialEq, Identifiable, Debug)]
#[table_name = "pike_organization_metadata"]
pub struct OrganizationMetadataModel {
    ///  This is the record id for the slowly-changing-dimensions table.
    pub id: i64,
    pub org_id: String,
    pub key: String,
    pub value: Vec<u8>,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "pike_organization_alternate_id"]
pub struct NewAlternateIdModel {
    pub org_id: String,
    pub alternate_id_type: String,
    pub alternate_id: String,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

/// Database model representation of a Pike `alternate_id`
#[derive(Queryable, PartialEq, Identifiable, Debug)]
#[table_name = "pike_organization_alternate_id"]
pub struct AlternateIdModel {
    ///  This is the record id for the slowly-changing-dimensions table.
    pub id: i64,
    pub org_id: String,
    pub alternate_id_type: String,
    pub alternate_id: String,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

pub fn make_alternate_id_models(org: &Organization) -> Vec<NewAlternateIdModel> {
    let mut models = Vec::new();
    for entry in &org.alternate_ids {
        let model = NewAlternateIdModel {
            org_id: org.org_id.to_string(),
            alternate_id_type: entry.alternate_id_type.to_string(),
            alternate_id: entry.alternate_id.to_string(),
            start_commit_num: org.start_commit_num,
            end_commit_num: org.end_commit_num,
            service_id: org.service_id.clone(),
        };

        models.push(model);
    }

    models
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "pike_organization_location_assoc"]
pub struct NewLocationAssociationModel {
    pub org_id: String,
    pub location_id: String,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

/// Database model representation of a Pike `organization`'s associated `location`
#[derive(Queryable, PartialEq, Identifiable, Debug)]
#[table_name = "pike_organization_location_assoc"]
pub struct LocationAssociationModel {
    ///  This is the record id for the slowly-changing-dimensions table.
    pub id: i64,
    pub org_id: String,
    pub location_id: String,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

impl From<&NewRoleModel> for NewRoleStateAddressAssociationModel {
    fn from(role: &NewRoleModel) -> Self {
        Self {
            state_address: compute_role_address(&role.name, &role.org_id),
            org_id: role.org_id.to_string(),
            name: role.name.to_string(),
            start_commit_num: role.start_commit_num,
            end_commit_num: MAX_COMMIT_NUM,
            service_id: role.service_id.clone(),
        }
    }
}

impl
    From<(
        RoleModel,
        Vec<InheritFromModel>,
        Vec<PermissionModel>,
        Vec<AllowedOrgModel>,
    )> for Role
{
    fn from(
        (role, inherit_from, permissions, allowed_orgs): (
            RoleModel,
            Vec<InheritFromModel>,
            Vec<PermissionModel>,
            Vec<AllowedOrgModel>,
        ),
    ) -> Self {
        let role_org_id = role.org_id;
        Self {
            org_id: role_org_id.to_string(),
            name: role.name,
            description: role.description,
            active: role.active,
            permissions: permissions.iter().map(|p| p.name.to_string()).collect(),
            allowed_organizations: allowed_orgs
                .iter()
                .map(|o| o.allowed_org_id.to_string())
                .collect(),
            inherit_from: inherit_from
                .iter()
                .map(|i| {
                    if i.inherit_from_org_id == role_org_id {
                        i.inherit_from_role_name.to_string()
                    } else {
                        format!("{}.{}", i.inherit_from_org_id, i.inherit_from_role_name)
                    }
                })
                .collect(),
            start_commit_num: role.start_commit_num,
            end_commit_num: role.end_commit_num,
            service_id: role.service_id,
            last_updated: role.last_updated.map(|d| d.timestamp()),
        }
    }
}

impl From<Role> for NewRoleModel {
    fn from(role: Role) -> Self {
        Self {
            state_address: compute_role_address(&role.name, &role.org_id),
            org_id: role.org_id,
            name: role.name,
            description: role.description,
            active: role.active,
            start_commit_num: role.start_commit_num,
            end_commit_num: MAX_COMMIT_NUM,
            service_id: role.service_id,
        }
    }
}

pub fn make_inherit_from_models(role: &Role) -> Vec<NewInheritFromModel> {
    let mut models = Vec::new();

    for i in &role.inherit_from {
        let mut ifoid = role.org_id.to_string();
        if i.contains('.') {
            let inherit_from: Vec<&str> = i.split('.').collect();
            ifoid = inherit_from[0].to_string();
        }

        let model = NewInheritFromModel {
            role_name: role.name.to_string(),
            org_id: role.org_id.to_string(),
            inherit_from_role_name: i.to_string(),
            inherit_from_org_id: ifoid,
            start_commit_num: role.start_commit_num,
            end_commit_num: role.end_commit_num,
            service_id: role.service_id.clone(),
        };

        models.push(model);
    }

    models
}

pub fn make_location_association_models(org: &Organization) -> Vec<NewLocationAssociationModel> {
    let mut models = Vec::new();

    for l in &org.locations {
        let model = NewLocationAssociationModel {
            org_id: org.org_id.to_string(),
            location_id: l.to_string(),
            start_commit_num: org.start_commit_num,
            end_commit_num: org.end_commit_num,
            service_id: org.service_id.clone(),
        };

        models.push(model);
    }

    models
}

pub fn make_permissions_models(role: &Role) -> Vec<NewPermissionModel> {
    let mut models = Vec::new();

    for p in &role.permissions {
        let model = NewPermissionModel {
            role_name: role.name.to_string(),
            org_id: role.org_id.to_string(),
            name: p.to_string(),
            start_commit_num: role.start_commit_num,
            end_commit_num: role.end_commit_num,
            service_id: role.service_id.clone(),
        };

        models.push(model);
    }

    models
}

pub fn make_allowed_orgs_models(role: &Role) -> Vec<NewAllowedOrgModel> {
    let mut models = Vec::new();

    for a in &role.allowed_organizations {
        let model = NewAllowedOrgModel {
            role_name: role.name.to_string(),
            org_id: role.org_id.to_string(),
            allowed_org_id: a.to_string(),
            start_commit_num: role.start_commit_num,
            end_commit_num: role.end_commit_num,
            service_id: role.service_id.clone(),
        };

        models.push(model)
    }

    models
}

impl From<(AgentModel, Vec<RoleAssociationModel>)> for Agent {
    fn from((agent_model, role_models): (AgentModel, Vec<RoleAssociationModel>)) -> Self {
        let agent_model_org_id = agent_model.org_id;
        Self {
            public_key: agent_model.public_key,
            org_id: agent_model_org_id.to_string(),
            active: agent_model.active,
            metadata: agent_model.metadata,
            roles: role_models
                .iter()
                .map(|role| {
                    if role.org_id == agent_model_org_id {
                        role.role_name.to_string()
                    } else {
                        format!("{}.{}", role.org_id, role.role_name)
                    }
                })
                .collect(),
            start_commit_num: agent_model.start_commit_num,
            end_commit_num: agent_model.end_commit_num,
            service_id: agent_model.service_id,
            last_updated: agent_model.last_updated.map(|d| d.timestamp()),
        }
    }
}

impl From<Agent> for NewAgentModel {
    fn from(agent: Agent) -> Self {
        Self {
            state_address: compute_agent_address(&agent.public_key),
            public_key: agent.public_key,
            org_id: agent.org_id,
            active: agent.active,
            metadata: agent.metadata,
            start_commit_num: agent.start_commit_num,
            end_commit_num: MAX_COMMIT_NUM,
            service_id: agent.service_id,
        }
    }
}

pub fn make_role_association_models(agent: &Agent) -> Vec<NewRoleAssociationModel> {
    let mut role_assocs = Vec::new();

    for role in &agent.roles {
        role_assocs.push(NewRoleAssociationModel {
            agent_public_key: agent.public_key.to_string(),
            org_id: agent.org_id.to_string(),
            role_name: role.to_string(),
            start_commit_num: agent.start_commit_num,
            end_commit_num: agent.end_commit_num,
            service_id: agent.service_id.clone(),
        })
    }

    role_assocs
}

impl
    From<(
        OrganizationModel,
        Vec<OrganizationMetadataModel>,
        Vec<AlternateIdModel>,
    )> for Organization
{
    fn from(
        (org, metadata, alternate_ids): (
            OrganizationModel,
            Vec<OrganizationMetadataModel>,
            Vec<AlternateIdModel>,
        ),
    ) -> Self {
        Self {
            org_id: org.org_id,
            name: org.name,
            locations: Vec::new(),
            alternate_ids: alternate_ids.iter().map(AlternateId::from).collect(),
            metadata: metadata.iter().map(OrganizationMetadata::from).collect(),
            start_commit_num: org.start_commit_num,
            end_commit_num: org.end_commit_num,
            service_id: org.service_id,
            last_updated: org.last_updated.map(|d| d.timestamp()),
        }
    }
}

impl
    From<(
        NewOrganizationModel,
        Vec<OrganizationMetadataModel>,
        Vec<AlternateIdModel>,
    )> for Organization
{
    fn from(
        (org, metadata, alternate_ids): (
            NewOrganizationModel,
            Vec<OrganizationMetadataModel>,
            Vec<AlternateIdModel>,
        ),
    ) -> Self {
        Self {
            org_id: org.org_id,
            name: org.name,
            locations: Vec::new(),
            alternate_ids: alternate_ids.iter().map(AlternateId::from).collect(),
            metadata: metadata.iter().map(OrganizationMetadata::from).collect(),
            start_commit_num: org.start_commit_num,
            end_commit_num: org.end_commit_num,
            service_id: org.service_id,
            last_updated: None,
        }
    }
}

impl
    From<(
        OrganizationModel,
        Vec<OrganizationMetadataModel>,
        Vec<AlternateIdModel>,
        Vec<LocationAssociationModel>,
    )> for Organization
{
    fn from(
        (org, metadata, alternate_ids, locations): (
            OrganizationModel,
            Vec<OrganizationMetadataModel>,
            Vec<AlternateIdModel>,
            Vec<LocationAssociationModel>,
        ),
    ) -> Self {
        Self {
            org_id: org.org_id,
            name: org.name,
            locations: locations
                .iter()
                .map(|l| l.location_id.to_string())
                .collect(),
            alternate_ids: alternate_ids.iter().map(AlternateId::from).collect(),
            metadata: metadata.iter().map(OrganizationMetadata::from).collect(),
            start_commit_num: org.start_commit_num,
            end_commit_num: org.end_commit_num,
            service_id: org.service_id,
            last_updated: org.last_updated.map(|d| d.timestamp()),
        }
    }
}

impl From<Organization> for NewOrganizationModel {
    fn from(org: Organization) -> Self {
        Self {
            state_address: compute_organization_address(&org.org_id),
            org_id: org.org_id,
            name: org.name,
            start_commit_num: org.start_commit_num,
            end_commit_num: org.end_commit_num,
            service_id: org.service_id,
        }
    }
}

impl From<&AlternateIdModel> for AlternateId {
    fn from(id: &AlternateIdModel) -> Self {
        Self {
            org_id: id.org_id.to_string(),
            alternate_id_type: id.alternate_id_type.to_string(),
            alternate_id: id.alternate_id.to_string(),
            start_commit_num: id.start_commit_num,
            end_commit_num: id.end_commit_num,
            service_id: id.service_id.clone(),
        }
    }
}

impl From<&AlternateId> for NewAlternateIdModel {
    fn from(alt_id: &AlternateId) -> Self {
        Self {
            org_id: alt_id.org_id.to_string(),
            alternate_id_type: alt_id.alternate_id_type.to_string(),
            alternate_id: alt_id.alternate_id.to_string(),
            start_commit_num: alt_id.start_commit_num,
            end_commit_num: alt_id.end_commit_num,
            service_id: alt_id.service_id.clone(),
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
