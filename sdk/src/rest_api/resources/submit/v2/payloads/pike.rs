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
use transact::protocol::{sabre::ExecuteContractActionBuilder, transaction::Transaction};

use crate::pike::addressing::GRID_PIKE_NAMESPACE;
use crate::protocol::pike::{payload as payload_protocol, state as state_protocol};
use crate::protos::IntoBytes;
use crate::rest_api::resources::{
    error::ErrorResponse, submit::v2::error::BuilderError, submit::v2::payloads::TransactionPayload,
};

pub(super) const GRID_PIKE_FAMILY_NAME: &str = "grid_pike";
pub(super) const GRID_PIKE_FAMILY_VERSION: &str = "2";

#[derive(Clone, Debug, PartialEq, Deserialize)]
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

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
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

#[derive(Default, Clone)]
pub struct CreateAgentActionBuilder {
    pub org_id: Option<String>,
    pub public_key: Option<String>,
    pub active: Option<bool>,
    pub roles: Vec<String>,
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

#[derive(Debug, Default, Deserialize, Clone, PartialEq)]
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

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
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

#[derive(Debug, Default, Deserialize, Clone, PartialEq)]
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

#[derive(Debug, Default, Deserialize, Clone, PartialEq)]
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

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
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

#[derive(Debug, Default, Deserialize, Clone, PartialEq)]
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

#[derive(Debug, Default, Deserialize, Clone, PartialEq)]
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

#[derive(Debug, Default, Deserialize, Clone, PartialEq)]
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

#[derive(Clone, Debug, Deserialize, PartialEq)]
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

#[derive(Clone, Debug, Deserialize, PartialEq)]
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

#[derive(Clone, Debug, PartialEq, Deserialize)]
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
