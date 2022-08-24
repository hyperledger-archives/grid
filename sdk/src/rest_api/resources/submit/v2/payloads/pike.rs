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

use std::convert::TryFrom;
use std::time::{SystemTime, UNIX_EPOCH};

use cylinder::Signer;
use serde::{self, Deserialize};
use serde_json::Value;
use transact::protocol::{sabre::ExecuteContractActionBuilder, transaction::Transaction};

use crate::pike::addressing::GRID_PIKE_NAMESPACE;
use crate::protocol::pike::{payload as payload_protocol, state as state_protocol};
use crate::protos::IntoBytes;
use crate::rest_api::resources::{
    error::ErrorResponse, submit::v2::error::BuilderError, submit::v2::payloads::TransactionPayload,
};

pub(super) const GRID_PIKE_FAMILY_NAME: &str = "grid_pike";
pub(super) const GRID_PIKE_FAMILY_VERSION: &str = "2";

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum PikeAction {
    CreateAgent(CreateAgentAction),
    UpdateAgent(UpdateAgentAction),
    DeleteAgent(DeleteAgentAction),
    CreateOrganization(CreateOrganizationAction),
    UpdateOrganization(UpdateOrganizationAction),
    DeleteOrganization(DeleteOrganizationAction),
    CreateRole(CreateRoleAction),
    UpdateRole(UpdateRoleAction),
    DeleteRole(DeleteRoleAction),
}

impl PikeAction {
    pub fn into_inner(self) -> Box<dyn TransactionPayload> {
        match self {
            PikeAction::CreateAgent(inner) => Box::new(inner),
            PikeAction::UpdateAgent(inner) => Box::new(inner),
            PikeAction::DeleteAgent(inner) => Box::new(inner),
            PikeAction::CreateOrganization(inner) => Box::new(inner),
            PikeAction::UpdateOrganization(inner) => Box::new(inner),
            PikeAction::DeleteOrganization(inner) => Box::new(inner),
            PikeAction::CreateRole(inner) => Box::new(inner),
            PikeAction::UpdateRole(inner) => Box::new(inner),
            PikeAction::DeleteRole(inner) => Box::new(inner),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(try_from = "CreateAgentActionBuilder")]
pub struct CreateAgentAction {
    org_id: String,
    public_key: String,
    active: bool,
    roles: Vec<String>,
    metadata: Vec<KeyValueEntry>,
}

impl CreateAgentAction {
    pub fn org_id(&self) -> &str {
        &self.org_id
    }

    pub fn public_key(&self) -> &str {
        &self.public_key
    }

    pub fn active(&self) -> &bool {
        &self.active
    }

    pub fn roles(&self) -> &[String] {
        &self.roles
    }

    pub fn metadata(&self) -> &[KeyValueEntry] {
        &self.metadata
    }
}

impl TryFrom<&CreateAgentAction> for payload_protocol::CreateAgentAction {
    type Error = ErrorResponse;

    fn try_from(action: &CreateAgentAction) -> Result<Self, Self::Error> {
        let metadata = action
            .metadata()
            .iter()
            .map(state_protocol::KeyValueEntry::try_from)
            .collect::<Result<Vec<_>, ErrorResponse>>()?;
        payload_protocol::CreateAgentActionBuilder::default()
            .with_org_id(action.org_id().to_string())
            .with_public_key(action.public_key().to_string())
            .with_active(*action.active())
            .with_roles(action.roles().to_vec())
            .with_metadata(metadata)
            .build()
            .map_err(|err| {
                ErrorResponse::new(
                    400,
                    &format!("Unable to build protocol CreateAgentAction: {err}"),
                )
            })
    }
}

impl TransactionPayload for CreateAgentAction {
    fn build_transaction(&self, signer: Box<dyn Signer>) -> Result<Transaction, ErrorResponse> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .map_err(|err| ErrorResponse::internal_error(Box::new(err)))?;
        let action = payload_protocol::CreateAgentAction::try_from(self)?;
        let payload_bytes = payload_protocol::PikePayloadBuilder::default()
            .with_action(payload_protocol::Action::CreateAgent(action))
            .with_timestamp(timestamp)
            .build()
            .map_err(|err| {
                ErrorResponse::new(
                    400,
                    &format!("Failed to build protocol Pike payload: {err}"),
                )
            })?
            .into_bytes()
            .map_err(|err| ErrorResponse::internal_error(Box::new(err)))?;
        // Turn contract-specific action into Sabre `ExecuteContractActionBuilder`
        let sabre_payload_builder = ExecuteContractActionBuilder::new()
            .with_name(GRID_PIKE_FAMILY_NAME.to_string())
            .with_version(GRID_PIKE_FAMILY_VERSION.to_string())
            .with_inputs(vec![GRID_PIKE_NAMESPACE.to_string()])
            .with_outputs(vec![GRID_PIKE_NAMESPACE.to_string()])
            .with_payload(payload_bytes)
            .into_payload_builder()
            .map_err(|err| ErrorResponse::internal_error(Box::new(err)))?;
        // Turn the Sabre `ExecuteContractActionBuilder` into a `Transaction`
        sabre_payload_builder
            .into_transaction_builder()
            .map_err(|err| ErrorResponse::internal_error(Box::new(err)))?
            .build(&*signer)
            .map_err(|err| ErrorResponse::internal_error(Box::new(err)))
    }
}

#[derive(Default, Clone, Deserialize)]
pub struct CreateAgentActionBuilder {
    pub org_id: Option<String>,
    pub public_key: Option<String>,
    pub active: Option<bool>,
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub metadata: Vec<KeyValueEntry>,
}

impl CreateAgentActionBuilder {
    pub fn new() -> Self {
        CreateAgentActionBuilder::default()
    }

    pub fn with_org_id(mut self, org_id: String) -> CreateAgentActionBuilder {
        self.org_id = Some(org_id);
        self
    }

    pub fn with_public_key(mut self, public_key: String) -> CreateAgentActionBuilder {
        self.public_key = Some(public_key);
        self
    }

    pub fn with_active(mut self, active: bool) -> CreateAgentActionBuilder {
        self.active = Some(active);
        self
    }

    pub fn with_roles(mut self, roles: Vec<String>) -> CreateAgentActionBuilder {
        self.roles = roles;
        self
    }

    pub fn with_metadata(mut self, metadata: Vec<KeyValueEntry>) -> CreateAgentActionBuilder {
        self.metadata = metadata;
        self
    }

    pub fn build(self) -> Result<CreateAgentAction, BuilderError> {
        let org_id = self
            .org_id
            .ok_or_else(|| BuilderError::MissingField("'org_id' field is required".to_string()))?;

        let public_key = self.public_key.ok_or_else(|| {
            BuilderError::MissingField("'public_key' field is required".to_string())
        })?;

        let active = self.active.unwrap_or_default();
        let roles = self.roles;
        let metadata = self.metadata;

        Ok(CreateAgentAction {
            org_id,
            public_key,
            active,
            roles,
            metadata,
        })
    }
}

impl TryFrom<CreateAgentActionBuilder> for CreateAgentAction {
    type Error = ErrorResponse;

    fn try_from(builder: CreateAgentActionBuilder) -> Result<Self, Self::Error> {
        builder.build().map_err(|err| {
            ErrorResponse::new(400, &format!("Unable to build `CreateAgentAction`: {err}"))
        })
    }
}

#[derive(Debug, Default, Deserialize, Clone, PartialEq, Eq)]
pub struct UpdateAgentAction {
    org_id: String,
    public_key: String,
    active: bool,
    #[serde(default)]
    roles: Vec<String>,
    #[serde(default)]
    metadata: Vec<KeyValueEntry>,
}

impl UpdateAgentAction {
    pub fn org_id(&self) -> &str {
        &self.org_id
    }

    pub fn public_key(&self) -> &str {
        &self.public_key
    }

    pub fn active(&self) -> &bool {
        &self.active
    }

    pub fn roles(&self) -> &[String] {
        &self.roles
    }

    pub fn metadata(&self) -> &[KeyValueEntry] {
        &self.metadata
    }
}

impl TransactionPayload for UpdateAgentAction {
    fn build_transaction(&self, _signer: Box<dyn Signer>) -> Result<Transaction, ErrorResponse> {
        unimplemented!();
    }
}

#[derive(Default, Clone)]
pub struct UpdateAgentActionBuilder {
    org_id: Option<String>,
    public_key: Option<String>,
    active: Option<bool>,
    roles: Vec<String>,
    metadata: Vec<KeyValueEntry>,
}

impl UpdateAgentActionBuilder {
    pub fn new() -> Self {
        UpdateAgentActionBuilder::default()
    }

    pub fn with_org_id(mut self, org_id: String) -> UpdateAgentActionBuilder {
        self.org_id = Some(org_id);
        self
    }

    pub fn with_public_key(mut self, public_key: String) -> UpdateAgentActionBuilder {
        self.public_key = Some(public_key);
        self
    }

    pub fn with_active(mut self, active: bool) -> UpdateAgentActionBuilder {
        self.active = Some(active);
        self
    }

    pub fn with_roles(mut self, roles: Vec<String>) -> UpdateAgentActionBuilder {
        self.roles = roles;
        self
    }

    pub fn with_metadata(mut self, metadata: Vec<KeyValueEntry>) -> UpdateAgentActionBuilder {
        self.metadata = metadata;
        self
    }

    pub fn build(self) -> Result<UpdateAgentAction, BuilderError> {
        let org_id = self
            .org_id
            .ok_or_else(|| BuilderError::MissingField("'org_id' field is required".to_string()))?;

        let public_key = self.public_key.ok_or_else(|| {
            BuilderError::MissingField("'public_key' field is required".to_string())
        })?;

        let active = self.active.unwrap_or_default();
        let roles = self.roles;
        let metadata = self.metadata;

        Ok(UpdateAgentAction {
            org_id,
            public_key,
            active,
            roles,
            metadata,
        })
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
pub struct DeleteAgentAction {
    org_id: String,
    public_key: String,
}

impl DeleteAgentAction {
    pub fn org_id(&self) -> &str {
        &self.org_id
    }

    pub fn public_key(&self) -> &str {
        &self.public_key
    }
}

impl TransactionPayload for DeleteAgentAction {
    fn build_transaction(&self, _signer: Box<dyn Signer>) -> Result<Transaction, ErrorResponse> {
        unimplemented!();
    }
}

#[derive(Default, Clone)]
pub struct DeleteAgentActionBuilder {
    pub org_id: Option<String>,
    pub public_key: Option<String>,
}

impl DeleteAgentActionBuilder {
    pub fn new() -> Self {
        DeleteAgentActionBuilder::default()
    }

    pub fn with_org_id(mut self, org_id: String) -> DeleteAgentActionBuilder {
        self.org_id = Some(org_id);
        self
    }

    pub fn with_public_key(mut self, public_key: String) -> DeleteAgentActionBuilder {
        self.public_key = Some(public_key);
        self
    }

    pub fn build(self) -> Result<DeleteAgentAction, BuilderError> {
        let org_id = self
            .org_id
            .ok_or_else(|| BuilderError::MissingField("'org_id' field is required".to_string()))?;

        let public_key = self.public_key.ok_or_else(|| {
            BuilderError::MissingField("'public_key' field is required".to_string())
        })?;

        Ok(DeleteAgentAction { org_id, public_key })
    }
}

#[derive(Debug, Default, Deserialize, Clone, PartialEq, Eq)]
pub struct CreateOrganizationAction {
    id: String,
    name: String,
    #[serde(default)]
    alternate_ids: Vec<AlternateId>,
    #[serde(default)]
    metadata: Vec<KeyValueEntry>,
}

impl CreateOrganizationAction {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn alternate_ids(&self) -> &[AlternateId] {
        &self.alternate_ids
    }

    pub fn metadata(&self) -> &[KeyValueEntry] {
        &self.metadata
    }
}

impl TransactionPayload for CreateOrganizationAction {
    fn build_transaction(&self, _signer: Box<dyn Signer>) -> Result<Transaction, ErrorResponse> {
        unimplemented!();
    }
}

#[derive(Default, Clone)]
pub struct CreateOrganizationActionBuilder {
    id: Option<String>,
    name: Option<String>,
    alternate_ids: Vec<AlternateId>,
    metadata: Vec<KeyValueEntry>,
}

impl CreateOrganizationActionBuilder {
    pub fn new() -> Self {
        CreateOrganizationActionBuilder::default()
    }

    pub fn with_id(mut self, id: String) -> CreateOrganizationActionBuilder {
        self.id = Some(id);
        self
    }

    pub fn with_name(mut self, name: String) -> CreateOrganizationActionBuilder {
        self.name = Some(name);
        self
    }

    pub fn with_alternate_ids(
        mut self,
        alternate_ids: Vec<AlternateId>,
    ) -> CreateOrganizationActionBuilder {
        self.alternate_ids = alternate_ids;
        self
    }

    pub fn with_metadata(
        mut self,
        metadata: Vec<KeyValueEntry>,
    ) -> CreateOrganizationActionBuilder {
        self.metadata = metadata;
        self
    }

    pub fn build(self) -> Result<CreateOrganizationAction, BuilderError> {
        let id = self
            .id
            .ok_or_else(|| BuilderError::MissingField("'id' field is required".to_string()))?;

        let name = self
            .name
            .ok_or_else(|| BuilderError::MissingField("'name' field is required".to_string()))?;

        let alternate_ids = self.alternate_ids;

        let metadata = self.metadata;

        Ok(CreateOrganizationAction {
            id,
            name,
            alternate_ids,
            metadata,
        })
    }
}

#[derive(Debug, Default, Deserialize, Clone, PartialEq, Eq)]
pub struct UpdateOrganizationAction {
    id: String,
    name: String,
    #[serde(default)]
    alternate_ids: Vec<AlternateId>,
    #[serde(default)]
    locations: Vec<String>,
    #[serde(default)]
    metadata: Vec<KeyValueEntry>,
}

impl UpdateOrganizationAction {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn alternate_ids(&self) -> &[AlternateId] {
        &self.alternate_ids
    }

    pub fn locations(&self) -> &[String] {
        &self.locations
    }

    pub fn metadata(&self) -> &[KeyValueEntry] {
        &self.metadata
    }
}

impl TransactionPayload for UpdateOrganizationAction {
    fn build_transaction(&self, _signer: Box<dyn Signer>) -> Result<Transaction, ErrorResponse> {
        unimplemented!();
    }
}

#[derive(Default, Clone)]
pub struct UpdateOrganizationActionBuilder {
    id: Option<String>,
    name: Option<String>,
    alternate_ids: Vec<AlternateId>,
    locations: Vec<String>,
    metadata: Vec<KeyValueEntry>,
}

impl UpdateOrganizationActionBuilder {
    pub fn new() -> Self {
        UpdateOrganizationActionBuilder::default()
    }

    pub fn with_id(mut self, id: String) -> UpdateOrganizationActionBuilder {
        self.id = Some(id);
        self
    }

    pub fn with_name(mut self, name: String) -> UpdateOrganizationActionBuilder {
        self.name = Some(name);
        self
    }

    pub fn alternate_ids(
        mut self,
        alternate_ids: Vec<AlternateId>,
    ) -> UpdateOrganizationActionBuilder {
        self.alternate_ids = alternate_ids;
        self
    }

    pub fn locations(mut self, locations: Vec<String>) -> UpdateOrganizationActionBuilder {
        self.locations = locations;
        self
    }

    pub fn with_metadata(
        mut self,
        metadata: Vec<KeyValueEntry>,
    ) -> UpdateOrganizationActionBuilder {
        self.metadata = metadata;
        self
    }

    pub fn build(self) -> Result<UpdateOrganizationAction, BuilderError> {
        let id = self
            .id
            .ok_or_else(|| BuilderError::MissingField("'id' field is required".to_string()))?;

        let name = self.name.unwrap_or_default();

        let locations = self.locations;

        let alternate_ids = self.alternate_ids;

        let metadata = self.metadata;

        Ok(UpdateOrganizationAction {
            id,
            name,
            alternate_ids,
            locations,
            metadata,
        })
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
pub struct DeleteOrganizationAction {
    id: String,
}

impl DeleteOrganizationAction {
    pub fn id(&self) -> &str {
        &self.id
    }
}

impl TransactionPayload for DeleteOrganizationAction {
    fn build_transaction(&self, _signer: Box<dyn Signer>) -> Result<Transaction, ErrorResponse> {
        unimplemented!();
    }
}

#[derive(Default, Clone)]
pub struct DeleteOrganizationActionBuilder {
    pub id: Option<String>,
}

impl DeleteOrganizationActionBuilder {
    pub fn new() -> Self {
        DeleteOrganizationActionBuilder::default()
    }

    pub fn with_id(mut self, id: String) -> DeleteOrganizationActionBuilder {
        self.id = Some(id);
        self
    }

    pub fn build(self) -> Result<DeleteOrganizationAction, BuilderError> {
        let id = self
            .id
            .ok_or_else(|| BuilderError::MissingField("'id' field is required".to_string()))?;

        Ok(DeleteOrganizationAction { id })
    }
}

#[derive(Debug, Default, Deserialize, Clone, PartialEq, Eq)]
pub struct CreateRoleAction {
    org_id: String,
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    permissions: Vec<String>,
    #[serde(default)]
    allowed_organizations: Vec<String>,
    #[serde(default)]
    inherit_from: Vec<String>,
    active: bool,
}

impl CreateRoleAction {
    pub fn org_id(&self) -> &str {
        &self.org_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn permissions(&self) -> &[String] {
        &self.permissions
    }

    pub fn allowed_organizations(&self) -> &[String] {
        &self.allowed_organizations
    }

    pub fn inherit_from(&self) -> &[String] {
        &self.inherit_from
    }

    pub fn active(&self) -> &bool {
        &self.active
    }
}

impl TransactionPayload for CreateRoleAction {
    fn build_transaction(&self, _signer: Box<dyn Signer>) -> Result<Transaction, ErrorResponse> {
        unimplemented!();
    }
}

#[derive(Default, Clone)]
pub struct CreateRoleActionBuilder {
    org_id: Option<String>,
    name: Option<String>,
    description: Option<String>,
    permissions: Vec<String>,
    allowed_organizations: Vec<String>,
    inherit_from: Vec<String>,
    active: bool,
}

impl CreateRoleActionBuilder {
    pub fn new() -> Self {
        CreateRoleActionBuilder::default()
    }

    pub fn with_org_id(mut self, org_id: String) -> CreateRoleActionBuilder {
        self.org_id = Some(org_id);
        self
    }

    pub fn with_name(mut self, name: String) -> CreateRoleActionBuilder {
        self.name = Some(name);
        self
    }

    pub fn with_description(mut self, description: String) -> CreateRoleActionBuilder {
        self.description = Some(description);
        self
    }

    pub fn with_permissions(mut self, permissions: Vec<String>) -> CreateRoleActionBuilder {
        self.permissions = permissions;
        self
    }

    pub fn with_allowed_organizations(
        mut self,
        allowed_organizations: Vec<String>,
    ) -> CreateRoleActionBuilder {
        self.allowed_organizations = allowed_organizations;
        self
    }

    pub fn with_inherit_from(mut self, inherit_from: Vec<String>) -> CreateRoleActionBuilder {
        self.inherit_from = inherit_from;
        self
    }

    pub fn with_active(mut self, active: bool) -> CreateRoleActionBuilder {
        self.active = active;
        self
    }

    pub fn build(self) -> Result<CreateRoleAction, BuilderError> {
        let org_id = self
            .org_id
            .ok_or_else(|| BuilderError::MissingField("'org_id' field is required".to_string()))?;

        let name = self
            .name
            .ok_or_else(|| BuilderError::MissingField("'name' field is required".to_string()))?;

        let description = self.description.ok_or_else(|| {
            BuilderError::MissingField("'description' field is required".to_string())
        })?;

        let permissions = self.permissions;

        let allowed_organizations = self.allowed_organizations;

        let inherit_from = self.inherit_from;

        let active = self.active;

        Ok(CreateRoleAction {
            org_id,
            name,
            description,
            permissions,
            allowed_organizations,
            inherit_from,
            active,
        })
    }
}

#[derive(Debug, Default, Deserialize, Clone, PartialEq, Eq)]
pub struct UpdateRoleAction {
    org_id: String,
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    permissions: Vec<String>,
    #[serde(default)]
    allowed_organizations: Vec<String>,
    #[serde(default)]
    inherit_from: Vec<String>,
    active: bool,
}

impl UpdateRoleAction {
    pub fn org_id(&self) -> &str {
        &self.org_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn permissions(&self) -> &[String] {
        &self.permissions
    }

    pub fn allowed_organizations(&self) -> &[String] {
        &self.allowed_organizations
    }

    pub fn inherit_from(&self) -> &[String] {
        &self.inherit_from
    }

    pub fn active(&self) -> &bool {
        &self.active
    }
}

impl TransactionPayload for UpdateRoleAction {
    fn build_transaction(&self, _signer: Box<dyn Signer>) -> Result<Transaction, ErrorResponse> {
        unimplemented!();
    }
}

#[derive(Default, Clone)]
pub struct UpdateRoleActionBuilder {
    org_id: Option<String>,
    name: Option<String>,
    description: Option<String>,
    permissions: Vec<String>,
    allowed_organizations: Vec<String>,
    inherit_from: Vec<String>,
    active: bool,
}

impl UpdateRoleActionBuilder {
    pub fn new() -> Self {
        UpdateRoleActionBuilder::default()
    }

    pub fn with_org_id(mut self, org_id: String) -> UpdateRoleActionBuilder {
        self.org_id = Some(org_id);
        self
    }

    pub fn with_name(mut self, name: String) -> UpdateRoleActionBuilder {
        self.name = Some(name);
        self
    }

    pub fn with_description(mut self, description: String) -> UpdateRoleActionBuilder {
        self.description = Some(description);
        self
    }

    pub fn with_permissions(mut self, permissions: Vec<String>) -> UpdateRoleActionBuilder {
        self.permissions = permissions;
        self
    }

    pub fn with_allowed_organizations(
        mut self,
        allowed_organizations: Vec<String>,
    ) -> UpdateRoleActionBuilder {
        self.allowed_organizations = allowed_organizations;
        self
    }

    pub fn with_inherit_from(mut self, inherit_from: Vec<String>) -> UpdateRoleActionBuilder {
        self.inherit_from = inherit_from;
        self
    }

    pub fn with_active(mut self, active: bool) -> UpdateRoleActionBuilder {
        self.active = active;
        self
    }

    pub fn build(self) -> Result<UpdateRoleAction, BuilderError> {
        let org_id = self
            .org_id
            .ok_or_else(|| BuilderError::MissingField("'org_id' field is required".to_string()))?;

        let name = self
            .name
            .ok_or_else(|| BuilderError::MissingField("'name' field is required".to_string()))?;

        let description = self.description.ok_or_else(|| {
            BuilderError::MissingField("'description' field is required".to_string())
        })?;

        let permissions = self.permissions;

        let allowed_organizations = self.allowed_organizations;

        let inherit_from = self.inherit_from;

        let active = self.active;

        Ok(UpdateRoleAction {
            org_id,
            name,
            description,
            permissions,
            allowed_organizations,
            inherit_from,
            active,
        })
    }
}

#[derive(Debug, Default, Deserialize, Clone, PartialEq, Eq)]
pub struct DeleteRoleAction {
    org_id: String,
    name: String,
}

impl DeleteRoleAction {
    pub fn org_id(&self) -> &str {
        &self.org_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl TransactionPayload for DeleteRoleAction {
    fn build_transaction(&self, _signer: Box<dyn Signer>) -> Result<Transaction, ErrorResponse> {
        unimplemented!();
    }
}

#[derive(Default, Clone)]
pub struct DeleteRoleActionBuilder {
    org_id: Option<String>,
    name: Option<String>,
}

impl DeleteRoleActionBuilder {
    pub fn new() -> Self {
        DeleteRoleActionBuilder::default()
    }

    pub fn with_org_id(mut self, org_id: String) -> DeleteRoleActionBuilder {
        self.org_id = Some(org_id);
        self
    }

    pub fn with_name(mut self, name: String) -> DeleteRoleActionBuilder {
        self.name = Some(name);
        self
    }

    pub fn build(self) -> Result<DeleteRoleAction, BuilderError> {
        let org_id = self
            .org_id
            .ok_or_else(|| BuilderError::MissingField("'org_id' field is required".to_string()))?;

        let name = self
            .name
            .ok_or_else(|| BuilderError::MissingField("'name' field is required".to_string()))?;

        Ok(DeleteRoleAction { org_id, name })
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct AlternateId {
    id_type: String,
    id: String,
}

impl AlternateId {
    pub fn id_type(&self) -> &str {
        &self.id_type
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

#[derive(Default, Clone)]
pub struct AlternateIdBuilder {
    id_type: Option<String>,
    id: Option<String>,
}

impl AlternateIdBuilder {
    pub fn new() -> Self {
        AlternateIdBuilder::default()
    }

    pub fn with_id_type(mut self, id_type: String) -> AlternateIdBuilder {
        self.id_type = Some(id_type);
        self
    }

    pub fn with_id(mut self, id: String) -> AlternateIdBuilder {
        self.id = Some(id);
        self
    }

    pub fn build(self) -> Result<AlternateId, BuilderError> {
        let id_type = self
            .id_type
            .ok_or_else(|| BuilderError::MissingField("'id_type' field is required".to_string()))?;

        let id = self
            .id
            .ok_or_else(|| BuilderError::MissingField("'id' field is required".to_string()))?;

        Ok(AlternateId { id_type, id })
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct KeyValueEntry {
    key: String,
    value: String,
}

impl KeyValueEntry {
    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

impl TryFrom<&KeyValueEntry> for state_protocol::KeyValueEntry {
    type Error = ErrorResponse;

    fn try_from(entry: &KeyValueEntry) -> Result<Self, Self::Error> {
        state_protocol::KeyValueEntryBuilder::default()
            .with_key(entry.key().to_string())
            .with_value(entry.value().to_string())
            .build()
            .map_err(|err| {
                ErrorResponse::new(
                    400,
                    &format!("Unable to build protocol KeyValueEntry: {err}"),
                )
            })
    }
}

#[derive(Default, Clone)]
pub struct KeyValueEntryBuilder {
    key: Option<String>,
    value: Option<String>,
}

impl KeyValueEntryBuilder {
    pub fn new() -> Self {
        KeyValueEntryBuilder::default()
    }

    pub fn with_key(mut self, key: String) -> KeyValueEntryBuilder {
        self.key = Some(key);
        self
    }

    pub fn with_value(mut self, value: String) -> KeyValueEntryBuilder {
        self.value = Some(value);
        self
    }

    pub fn build(self) -> Result<KeyValueEntry, BuilderError> {
        let key = self
            .key
            .ok_or_else(|| BuilderError::MissingField("'key' field is required".to_string()))?;

        let value = self
            .value
            .ok_or_else(|| BuilderError::MissingField("'value' field is required".to_string()))?;

        Ok(KeyValueEntry { key, value })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(try_from = "DeserializablePikePayload")]
pub struct PikePayload {
    #[serde(flatten)]
    action: PikeAction,
    timestamp: u64,
}

impl PikePayload {
    pub fn new(action: PikeAction, timestamp: u64) -> Self {
        Self { action, timestamp }
    }

    pub fn action(&self) -> &PikeAction {
        &self.action
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn into_transaction_payload(self) -> Box<dyn TransactionPayload> {
        self.action.into_inner()
    }
}

// Interim struct to assist deserializing into `PikePayload`
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
struct DeserializablePikePayload {
    #[serde(flatten)]
    action: Value,
    target: String,
}

// Conversion helper function to correctly identify the type of action submitted in a `PikePayload`
impl TryFrom<DeserializablePikePayload> for PikePayload {
    type Error = ErrorResponse;

    fn try_from(d: DeserializablePikePayload) -> Result<Self, Self::Error> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .map_err(|err| ErrorResponse::internal_error(Box::new(err)))?;
        // Make the characters of the string matchable
        let lower_target = d.target.to_lowercase();
        let mut target_parts = lower_target.split_whitespace();
        // Retrieve the method of the `target` url to determine the type of payload to build
        let method = target_parts.next().ok_or_else(|| {
            ErrorResponse::new(400, "Invalid `target`, must provide request method")
        })?;
        // Retrieve the beginning of the `target` path
        let mut target_url_iter = target_parts
            .next()
            .ok_or_else(|| ErrorResponse::new(400, "Invalid `target`, must provide request URI"))?
            .trim_start_matches('/')
            .split('/');
        let action: PikeAction = match (method, target_url_iter.next()) {
            ("post", Some("agent")) => {
                PikeAction::CreateAgent(serde_json::from_value::<CreateAgentAction>(d.action)?)
            }
            ("put", Some("agent")) => {
                PikeAction::UpdateAgent(serde_json::from_value::<UpdateAgentAction>(d.action)?)
            }
            ("delete", Some("agent")) => {
                PikeAction::DeleteAgent(serde_json::from_value::<DeleteAgentAction>(d.action)?)
            }
            ("post", Some("organization")) => PikeAction::CreateOrganization(
                serde_json::from_value::<CreateOrganizationAction>(d.action)?,
            ),
            ("put", Some("organization")) => PikeAction::UpdateOrganization(
                serde_json::from_value::<UpdateOrganizationAction>(d.action)?,
            ),
            ("delete", Some("organization")) => PikeAction::DeleteOrganization(
                serde_json::from_value::<DeleteOrganizationAction>(d.action)?,
            ),
            ("post", Some("role")) => {
                PikeAction::CreateRole(serde_json::from_value::<CreateRoleAction>(d.action)?)
            }
            ("put", Some("role")) => {
                PikeAction::UpdateRole(serde_json::from_value::<UpdateRoleAction>(d.action)?)
            }
            ("delete", Some("role")) => {
                PikeAction::DeleteRole(serde_json::from_value::<DeleteRoleAction>(d.action)?)
            }
            _ => {
                return Err(ErrorResponse::new(
                    400,
                    "Unable to deserialize action, invalid `target`",
                ))
            }
        };
        Ok(PikePayload { action, timestamp })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::rest_api::resources::submit::v2::payloads::Payload;
    use serde_json;

    const ORG: &str = "myorg";
    const ROLE: &str = "test_role";
    const PUBLIC_KEY: &str = "PUBLIC_KEY";

    // Example JSON `PikeAction`s

    const JSON_CREATE_AGENT_ACTION: &str =
        "{\"org_id\": \"myorg\", \"public_key\": \"PUBLIC_KEY\", \"active\": true}";
    const BAD_JSON_CREATE_AGENT_ACTION: &str = "{\"org_id\": \"myorg\", \"active\": true}";
    const JSON_UPDATE_AGENT_ACTION: &str =
        "{ \"org_id\": \"myorg\", \"public_key\": \"PUBLIC_KEY\", \"active\": true}";
    const JSON_DELETE_AGENT_ACTION: &str =
        "{ \"org_id\": \"myorg\", \"public_key\": \"PUBLIC_KEY\" }";

    const JSON_CREATE_ORGANIZATION_ACTION: &str = "{ \"id\": \"myorg\", \"name\": \"myorg\" }";
    const JSON_UPDATE_ORGANIZATION_ACTION: &str = "{ \"id\": \"myorg\", \"name\": \"myorg\" }";
    const JSON_DELETE_ORGANIZATION_ACTION: &str = "{ \"id\": \"myorg\" }";

    const JSON_CREATE_ROLE_ACTION: &str =
        "{ \"org_id\": \"myorg\", \"name\": \"test_role\", \"description\": \"test_role\", \
        \"permissions\": [\"schema::can_create_schema\"], \"active\": true }";
    const JSON_UPDATE_ROLE_ACTION: &str =
        "{ \"org_id\": \"myorg\", \"name\": \"test_role\", \"description\": \"test_role\", \
        \"permissions\": [\"schema::can_create_schema\"], \"active\": false }";
    const JSON_DELETE_ROLE_ACTION: &str = "{ \"org_id\": \"myorg\", \"name\": \"test_role\" }";

    // Example JSON `PikePayload`s

    const JSON_CREATE_AGENT_PAYLOAD: &str =
        "{ \"org_id\": \"myorg\", \"public_key\": \"PUBLIC_KEY\", \"active\": true, \
        \"target\": \"POST /agent\" }";
    const JSON_UPDATE_AGENT_PAYLOAD: &str =
        "{ \"org_id\": \"myorg\", \"public_key\": \"PUBLIC_KEY\", \"active\": true, \
        \"target\": \"PUT /agent/PUBLIC_KEY\"}";
    const JSON_DELETE_AGENT_PAYLOAD: &str =
        "{ \"org_id\": \"myorg\", \"public_key\": \"PUBLIC_KEY\", \
        \"target\": \"DELETE /agent/PUBLIC_KEY\" }";

    const JSON_CREATE_ORGANIZATION_PAYLOAD: &str =
        "{ \"id\": \"myorg\", \"name\": \"myorg\", \"target\": \"POST /organization\" }";
    const JSON_UPDATE_ORGANIZATION_PAYLOAD: &str =
        "{ \"id\": \"myorg\", \"name\": \"myorg\", \"target\": \"PUT /organization/myorg\" }";
    const JSON_DELETE_ORGANIZATION_PAYLOAD: &str =
        "{ \"id\": \"myorg\", \"target\": \"DELETE /organization/myorg\" }";

    const JSON_CREATE_ROLE_PAYLOAD: &str =
        "{ \"org_id\": \"myorg\", \"name\": \"test_role\", \"description\": \"test_role\", \
        \"permissions\": [\"schema::can_create_schema\"], \"active\": true, \
        \"target\": \"POST /role\" }";
    const JSON_UPDATE_ROLE_PAYLOAD: &str =
        "{ \"org_id\": \"myorg\", \"name\": \"test_role\", \"description\": \"test_role\", \
        \"permissions\": [\"schema::can_create_schema\"], \"active\": false, \
        \"target\": \"PUT /role/test_role\" }";
    const JSON_DELETE_ROLE_PAYLOAD: &str = "{ \"org_id\": \"myorg\", \"name\": \"test_role\", \
        \"target\": \"DELETE /role/test_role\" }";

    #[test]
    /// Validate a `CreateAgentAction` may be deserialized from the action itself, and from a
    /// `PikePayload`
    fn test_deserialize_json_create_agent() {
        let example_action = CreateAgentAction {
            org_id: ORG.to_string(),
            public_key: PUBLIC_KEY.to_string(),
            active: true,
            roles: vec![],
            metadata: vec![],
        };
        // Attempt to deserialize the JSON `create agent` action
        let de_action: CreateAgentAction = serde_json::from_str(JSON_CREATE_AGENT_ACTION)
            .expect("Unable to parse 'create agent' action");
        // Assert the deserialized action is as we expect
        assert_eq!(example_action, de_action);
        // Attempt to deserialize a bad JSON `create agent` action
        let failed_de_action =
            serde_json::from_str::<CreateAgentAction>(BAD_JSON_CREATE_AGENT_ACTION);
        // Assert the malformed JSON returned an error, specifying the issue
        match failed_de_action {
            Ok(_) => panic!("Deserialization should have failed for `CreateAgentAction`"),
            Err(test_err) => {
                // Build the expected error
                let builder_err =
                    BuilderError::MissingField("'public_key' field is required".to_string());
                let err_resp: ErrorResponse = ErrorResponse::new(
                    400,
                    &format!("Unable to build `CreateAgentAction`: {builder_err}"),
                );
                assert_eq!(format!("{test_err}"), format!("{err_resp}"));
            }
        }
        //Attempt to deserialize the JSON `create agent` payload
        let de_payload: PikePayload = serde_json::from_str(JSON_CREATE_AGENT_PAYLOAD)
            .expect("Unable to parse 'create agent' payload");
        assert_ne!(de_payload.timestamp(), 0);
        if let PikeAction::CreateAgent(action) = de_payload.action() {
            assert_eq!(&example_action, action);
        } else {
            panic!("`PikePayload` action should be `CreateAgentAction` type");
        }
    }

    #[test]
    /// Validate a `UpdateAgentAction` may be deserialized from the action itself, and from a
    /// `PikePayload`
    fn test_deserialize_json_update_agent() {
        let example_action = UpdateAgentAction {
            org_id: ORG.to_string(),
            public_key: PUBLIC_KEY.to_string(),
            active: true,
            roles: vec![],
            metadata: vec![],
        };
        // Attempt to deserialize the JSON `update agent` action
        let de_action: UpdateAgentAction = serde_json::from_str(JSON_UPDATE_AGENT_ACTION)
            .expect("Unable to parse 'update agent' action");
        // Assert the deserialized action is as we expect
        assert_eq!(example_action, de_action);
        //Attempt to deserialize the JSON `update agent` payload
        let de_payload: PikePayload = serde_json::from_str(JSON_UPDATE_AGENT_PAYLOAD)
            .expect("Unable to parse 'update agent' payload");
        assert_ne!(de_payload.timestamp(), 0);
        if let PikeAction::UpdateAgent(action) = de_payload.action() {
            assert_eq!(&example_action, action);
        } else {
            panic!("`PikePayload` action should be `UpdateAgentAction` type");
        }
    }

    #[test]
    /// Validate a `DeleteAgentAction` may be deserialized from the action itself, and from a
    /// `PikePayload`
    fn test_deserialize_json_delete_agent() {
        let example_action = DeleteAgentAction {
            org_id: ORG.to_string(),
            public_key: PUBLIC_KEY.to_string(),
        };
        // Attempt to deserialize the JSON `delete agent` action
        let de_action: DeleteAgentAction = serde_json::from_str(JSON_DELETE_AGENT_ACTION)
            .expect("Unable to parse 'delete agent' action");
        // Assert the deserialized action is as we expect
        assert_eq!(example_action, de_action);
        //Attempt to deserialize the JSON `delete agent` payload
        let de_payload: PikePayload = serde_json::from_str(JSON_DELETE_AGENT_PAYLOAD)
            .expect("Unable to parse 'delete agent' payload");
        assert_ne!(de_payload.timestamp(), 0);
        if let PikeAction::DeleteAgent(action) = de_payload.action() {
            assert_eq!(&example_action, action);
        } else {
            panic!("`PikePayload` action should be `DeleteAgentAction` type");
        }
    }

    #[test]
    /// Validate a `CreateOrganizationAction` may be deserialized from the action itself, and from
    /// a `PikePayload`
    fn test_deserialize_json_create_organization() {
        let example_action = CreateOrganizationAction {
            id: ORG.to_string(),
            name: ORG.to_string(),
            alternate_ids: vec![],
            metadata: vec![],
        };
        // Attempt to deserialize the JSON `create organization` action
        let de_action: CreateOrganizationAction =
            serde_json::from_str(JSON_CREATE_ORGANIZATION_ACTION)
                .expect("Unable to parse 'create organization' action");
        // Assert the deserialized action is as we expect
        assert_eq!(example_action, de_action);
        //Attempt to deserialize the JSON `create organization` payload
        let de_payload: PikePayload = serde_json::from_str(JSON_CREATE_ORGANIZATION_PAYLOAD)
            .expect("Unable to parse 'create organization' payload");
        assert_ne!(de_payload.timestamp(), 0);
        if let PikeAction::CreateOrganization(action) = de_payload.action() {
            assert_eq!(&example_action, action);
        } else {
            panic!("`PikePayload` action should be `CreateOrganizationAction` type");
        }
    }

    #[test]
    /// Validate a `UpdateOrganizationAction` may be deserialized from the action itself, and from
    /// a `PikePayload`
    fn test_deserialize_json_update_organization() {
        let example_action = UpdateOrganizationAction {
            id: ORG.to_string(),
            name: ORG.to_string(),
            alternate_ids: vec![],
            locations: vec![],
            metadata: vec![],
        };
        // Attempt to deserialize the JSON `update organization` action
        let de_action: UpdateOrganizationAction =
            serde_json::from_str(JSON_UPDATE_ORGANIZATION_ACTION)
                .expect("Unable to parse 'create organization' action");
        // Assert the deserialized action is as we expect
        assert_eq!(example_action, de_action);
        //Attempt to deserialize the JSON `update organization` payload
        let de_payload: PikePayload = serde_json::from_str(JSON_UPDATE_ORGANIZATION_PAYLOAD)
            .expect("Unable to parse 'create organization' payload");
        assert_ne!(de_payload.timestamp(), 0);
        if let PikeAction::UpdateOrganization(action) = de_payload.action() {
            assert_eq!(&example_action, action);
        } else {
            panic!("`PikePayload` action should be `UpdateOrganizationAction` type");
        }
    }

    #[test]
    /// Validate a `DeleteOrganizationAction` may be deserialized from the action itself, and from
    /// a `PikePayload`
    fn test_deserialize_json_delete_organization() {
        let example_action = DeleteOrganizationAction {
            id: ORG.to_string(),
        };
        // Attempt to deserialize the JSON `delete organization` action
        let de_action: DeleteOrganizationAction =
            serde_json::from_str(JSON_DELETE_ORGANIZATION_ACTION)
                .expect("Unable to parse 'delete organization' action");
        // Assert the deserialized action is as we expect
        assert_eq!(example_action, de_action);
        //Attempt to deserialize the JSON `delete organization` payload
        let de_payload: PikePayload = serde_json::from_str(JSON_DELETE_ORGANIZATION_PAYLOAD)
            .expect("Unable to parse 'delete organization' payload");
        assert_ne!(de_payload.timestamp(), 0);
        if let PikeAction::DeleteOrganization(action) = de_payload.action() {
            assert_eq!(&example_action, action);
        } else {
            panic!("`PikePayload` action should be `DeleteOrganizationAction` type");
        }
    }

    #[test]
    /// Validate a `CreateRoleAction` may be deserialized from the action itself, and from
    /// a `PikePayload`
    fn test_deserialize_json_create_role() {
        let example_action = CreateRoleAction {
            org_id: ORG.to_string(),
            name: ROLE.to_string(),
            description: ROLE.to_string(),
            permissions: vec!["schema::can_create_schema".to_string()],
            allowed_organizations: vec![],
            inherit_from: vec![],
            active: true,
        };
        // Attempt to deserialize the JSON `create role` action
        let de_action: CreateRoleAction = serde_json::from_str(JSON_CREATE_ROLE_ACTION)
            .expect("Unable to parse 'create role' action");
        // Assert the deserialized action is as we expect
        assert_eq!(example_action, de_action);
        //Attempt to deserialize the JSON `create role` payload
        let de_payload: PikePayload = serde_json::from_str(JSON_CREATE_ROLE_PAYLOAD)
            .expect("Unable to parse 'create role' payload");
        assert_ne!(de_payload.timestamp(), 0);
        if let PikeAction::CreateRole(action) = de_payload.action() {
            assert_eq!(&example_action, action);
        } else {
            panic!("`PikePayload` action should be `CreateRoleAction` type");
        }
    }

    #[test]
    /// Validate a `UpdateRoleAction` may be deserialized from the action itself, and from
    /// a `PikePayload`
    fn test_deserialize_json_update_role() {
        let example_action = UpdateRoleAction {
            org_id: ORG.to_string(),
            name: ROLE.to_string(),
            description: ROLE.to_string(),
            permissions: vec!["schema::can_create_schema".to_string()],
            allowed_organizations: vec![],
            inherit_from: vec![],
            active: false,
        };
        // Attempt to deserialize the JSON `update role` action
        let de_action: UpdateRoleAction = serde_json::from_str(JSON_UPDATE_ROLE_ACTION)
            .expect("Unable to parse 'update role' action");
        // Assert the deserialized action is as we expect
        assert_eq!(example_action, de_action);
        //Attempt to deserialize the JSON `update role` payload
        let de_payload: PikePayload = serde_json::from_str(JSON_UPDATE_ROLE_PAYLOAD)
            .expect("Unable to parse 'update role' payload");
        assert_ne!(de_payload.timestamp(), 0);
        if let PikeAction::UpdateRole(action) = de_payload.action() {
            assert_eq!(&example_action, action);
        } else {
            panic!("`PikePayload` action should be `UpdateRoleAction` type");
        }
    }

    #[test]
    /// Validate a `DeleteRoleAction` may be deserialized from the action itself, and from
    /// a `PikePayload`
    fn test_deserialize_json_delete_role() {
        let example_action = DeleteRoleAction {
            org_id: ORG.to_string(),
            name: ROLE.to_string(),
        };
        // Attempt to deserialize the JSON `delete role` action
        let de_action: DeleteRoleAction = serde_json::from_str(JSON_DELETE_ROLE_ACTION)
            .expect("Unable to parse 'delete role' action");
        // Assert the deserialized action is as we expect
        assert_eq!(example_action, de_action);
        //Attempt to deserialize the JSON `delete role` payload
        let de_payload: Payload = serde_json::from_str(JSON_DELETE_ROLE_PAYLOAD)
            .expect("Unable to parse 'delete role' payload");
        match de_payload {
            Payload::Pike(payload) => {
                assert_ne!(payload.timestamp(), 0);
                if let PikeAction::DeleteRole(action) = payload.action() {
                    assert_eq!(&example_action, action);
                } else {
                    panic!("`PikePayload` action should be `DeleteRoleAction` type");
                }
            }
            _ => panic!("Unable to deserialize Payload"),
        }
    }
}
