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

use protobuf::Message;
use protobuf::RepeatedField;

use crate::protos;
use crate::protos::{
    FromBytes, FromNative, FromProto, IntoBytes, IntoNative, IntoProto, ProtoConversionError,
};

use super::BuilderError;

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum PikeAction {
    CreateAgent(CreateAgentAction),
    UpdateAgent(UpdateAgentAction),
    CreateOrganization(CreateOrganizationAction),
    UpdateOrganization(UpdateOrganizationAction),
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct CreateAgentAction {
    org_id: String,
    public_key: String,
    active: bool,
    #[serde(default)]
    roles: Vec<String>,
    #[serde(default)]
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

impl FromProto<protos::pike_payload::CreateAgentAction> for CreateAgentAction {
    fn from_proto(
        create_agent: protos::pike_payload::CreateAgentAction,
    ) -> Result<Self, ProtoConversionError> {
        Ok(CreateAgentAction {
            org_id: create_agent.get_org_id().to_string(),
            public_key: create_agent.get_public_key().to_string(),
            active: create_agent.get_active(),
            roles: create_agent.get_roles().to_vec(),
            metadata: create_agent
                .get_metadata()
                .to_vec()
                .into_iter()
                .map(KeyValueEntry::from_proto)
                .collect::<Result<Vec<KeyValueEntry>, ProtoConversionError>>()?,
        })
    }
}

impl FromNative<CreateAgentAction> for protos::pike_payload::CreateAgentAction {
    fn from_native(create_agent: CreateAgentAction) -> Result<Self, ProtoConversionError> {
        let mut proto_create_agent = protos::pike_payload::CreateAgentAction::new();

        proto_create_agent.set_org_id(create_agent.org_id().to_string());
        proto_create_agent.set_public_key(create_agent.public_key().to_string());
        proto_create_agent.set_active(*create_agent.active());
        proto_create_agent.set_org_id(create_agent.org_id().to_string());
        proto_create_agent.set_roles(RepeatedField::from_vec(create_agent.roles().to_vec()));
        proto_create_agent.set_metadata(RepeatedField::from_vec(
            create_agent
                .metadata()
                .to_vec()
                .into_iter()
                .map(KeyValueEntry::into_proto)
                .collect::<Result<Vec<protos::pike_state::KeyValueEntry>, ProtoConversionError>>(
                )?,
        ));

        Ok(proto_create_agent)
    }
}

impl FromBytes<CreateAgentAction> for CreateAgentAction {
    fn from_bytes(bytes: &[u8]) -> Result<CreateAgentAction, ProtoConversionError> {
        let proto: protos::pike_payload::CreateAgentAction = Message::parse_from_bytes(bytes)
            .map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get CreateAgentAction from bytes".to_string(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for CreateAgentAction {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from CreateAgentAction".to_string(),
            )
        })?;
        Ok(bytes)
    }
}

impl IntoProto<protos::pike_payload::CreateAgentAction> for CreateAgentAction {}
impl IntoNative<CreateAgentAction> for protos::pike_payload::CreateAgentAction {}

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

impl FromProto<protos::pike_payload::UpdateAgentAction> for UpdateAgentAction {
    fn from_proto(
        update_agent: protos::pike_payload::UpdateAgentAction,
    ) -> Result<Self, ProtoConversionError> {
        Ok(UpdateAgentAction {
            org_id: update_agent.get_org_id().to_string(),
            public_key: update_agent.get_public_key().to_string(),
            active: update_agent.get_active(),
            roles: update_agent.get_roles().to_vec(),
            metadata: update_agent
                .get_metadata()
                .to_vec()
                .into_iter()
                .map(KeyValueEntry::from_proto)
                .collect::<Result<Vec<KeyValueEntry>, ProtoConversionError>>()?,
        })
    }
}

impl FromNative<UpdateAgentAction> for protos::pike_payload::UpdateAgentAction {
    fn from_native(update_agent: UpdateAgentAction) -> Result<Self, ProtoConversionError> {
        let mut proto_update_agent = protos::pike_payload::UpdateAgentAction::new();

        proto_update_agent.set_org_id(update_agent.org_id().to_string());
        proto_update_agent.set_public_key(update_agent.public_key().to_string());
        proto_update_agent.set_active(*update_agent.active());
        proto_update_agent.set_org_id(update_agent.org_id().to_string());
        proto_update_agent.set_roles(RepeatedField::from_vec(update_agent.roles().to_vec()));
        proto_update_agent.set_metadata(RepeatedField::from_vec(
            update_agent
                .metadata()
                .to_vec()
                .into_iter()
                .map(KeyValueEntry::into_proto)
                .collect::<Result<Vec<protos::pike_state::KeyValueEntry>, ProtoConversionError>>(
                )?,
        ));

        Ok(proto_update_agent)
    }
}

impl FromBytes<UpdateAgentAction> for UpdateAgentAction {
    fn from_bytes(bytes: &[u8]) -> Result<UpdateAgentAction, ProtoConversionError> {
        let proto: protos::pike_payload::UpdateAgentAction = Message::parse_from_bytes(bytes)
            .map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get UpdateAgentAction from bytes".to_string(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for UpdateAgentAction {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from UpdateAgentAction".to_string(),
            )
        })?;
        Ok(bytes)
    }
}

impl IntoProto<protos::pike_payload::UpdateAgentAction> for UpdateAgentAction {}
impl IntoNative<UpdateAgentAction> for protos::pike_payload::UpdateAgentAction {}

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

#[derive(Debug, Default, Deserialize, Clone, PartialEq)]
pub struct CreateOrganizationAction {
    org_id: String,
    name: String,
    address: String,
    #[serde(default)]
    metadata: Vec<KeyValueEntry>,
}

impl CreateOrganizationAction {
    pub fn org_id(&self) -> &str {
        &self.org_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn metadata(&self) -> &[KeyValueEntry] {
        &self.metadata
    }
}

impl FromProto<protos::pike_payload::CreateOrganizationAction> for CreateOrganizationAction {
    fn from_proto(
        create_org: protos::pike_payload::CreateOrganizationAction,
    ) -> Result<Self, ProtoConversionError> {
        Ok(CreateOrganizationAction {
            org_id: create_org.get_id().to_string(),
            name: create_org.get_name().to_string(),
            address: create_org.get_address().to_string(),
            metadata: create_org
                .get_metadata()
                .to_vec()
                .into_iter()
                .map(KeyValueEntry::from_proto)
                .collect::<Result<Vec<KeyValueEntry>, ProtoConversionError>>()?,
        })
    }
}

impl FromNative<CreateOrganizationAction> for protos::pike_payload::CreateOrganizationAction {
    fn from_native(create_org: CreateOrganizationAction) -> Result<Self, ProtoConversionError> {
        let mut proto_create_org = protos::pike_payload::CreateOrganizationAction::new();

        proto_create_org.set_id(create_org.org_id().to_string());
        proto_create_org.set_name(create_org.name().to_string());
        proto_create_org.set_address(create_org.address().to_string());
        proto_create_org.set_metadata(RepeatedField::from_vec(
            create_org
                .metadata()
                .to_vec()
                .into_iter()
                .map(KeyValueEntry::into_proto)
                .collect::<Result<Vec<protos::pike_state::KeyValueEntry>, ProtoConversionError>>(
                )?,
        ));

        Ok(proto_create_org)
    }
}

impl FromBytes<CreateOrganizationAction> for CreateOrganizationAction {
    fn from_bytes(bytes: &[u8]) -> Result<CreateOrganizationAction, ProtoConversionError> {
        let proto: protos::pike_payload::CreateOrganizationAction =
            Message::parse_from_bytes(bytes).map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get CreateOrganizationAction from bytes".to_string(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for CreateOrganizationAction {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from CreateOrganizationAction".to_string(),
            )
        })?;
        Ok(bytes)
    }
}

impl IntoProto<protos::pike_payload::CreateOrganizationAction> for CreateOrganizationAction {}
impl IntoNative<CreateOrganizationAction> for protos::pike_payload::CreateOrganizationAction {}

#[derive(Default, Clone)]
pub struct CreateOrganizationActionBuilder {
    org_id: Option<String>,
    name: Option<String>,
    address: Option<String>,
    metadata: Vec<KeyValueEntry>,
}

impl CreateOrganizationActionBuilder {
    pub fn new() -> Self {
        CreateOrganizationActionBuilder::default()
    }

    pub fn with_org_id(mut self, org_id: String) -> CreateOrganizationActionBuilder {
        self.org_id = Some(org_id);
        self
    }

    pub fn with_name(mut self, name: String) -> CreateOrganizationActionBuilder {
        self.name = Some(name);
        self
    }

    pub fn with_address(mut self, address: String) -> CreateOrganizationActionBuilder {
        self.address = Some(address);
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
        let org_id = self
            .org_id
            .ok_or_else(|| BuilderError::MissingField("'org_id' field is required".to_string()))?;

        let name = self
            .name
            .ok_or_else(|| BuilderError::MissingField("'name' field is required".to_string()))?;

        let address = self
            .address
            .ok_or_else(|| BuilderError::MissingField("'address' field is required".to_string()))?;

        let metadata = self.metadata;

        Ok(CreateOrganizationAction {
            org_id,
            name,
            address,
            metadata,
        })
    }
}

#[derive(Debug, Default, Deserialize, Clone, PartialEq)]
pub struct UpdateOrganizationAction {
    org_id: String,
    name: String,
    address: String,
    #[serde(default)]
    metadata: Vec<KeyValueEntry>,
}

impl UpdateOrganizationAction {
    pub fn org_id(&self) -> &str {
        &self.org_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn metadata(&self) -> &[KeyValueEntry] {
        &self.metadata
    }
}

impl FromProto<protos::pike_payload::UpdateOrganizationAction> for UpdateOrganizationAction {
    fn from_proto(
        create_org: protos::pike_payload::UpdateOrganizationAction,
    ) -> Result<Self, ProtoConversionError> {
        Ok(UpdateOrganizationAction {
            org_id: create_org.get_id().to_string(),
            name: create_org.get_name().to_string(),
            address: create_org.get_address().to_string(),
            metadata: create_org
                .get_metadata()
                .to_vec()
                .into_iter()
                .map(KeyValueEntry::from_proto)
                .collect::<Result<Vec<KeyValueEntry>, ProtoConversionError>>()?,
        })
    }
}

impl FromNative<UpdateOrganizationAction> for protos::pike_payload::UpdateOrganizationAction {
    fn from_native(update_org: UpdateOrganizationAction) -> Result<Self, ProtoConversionError> {
        let mut proto_update_org = protos::pike_payload::UpdateOrganizationAction::new();

        proto_update_org.set_id(update_org.org_id().to_string());
        proto_update_org.set_name(update_org.name().to_string());
        proto_update_org.set_address(update_org.address().to_string());
        proto_update_org.set_metadata(RepeatedField::from_vec(
            update_org
                .metadata()
                .to_vec()
                .into_iter()
                .map(KeyValueEntry::into_proto)
                .collect::<Result<Vec<protos::pike_state::KeyValueEntry>, ProtoConversionError>>(
                )?,
        ));

        Ok(proto_update_org)
    }
}

impl FromBytes<UpdateOrganizationAction> for UpdateOrganizationAction {
    fn from_bytes(bytes: &[u8]) -> Result<UpdateOrganizationAction, ProtoConversionError> {
        let proto: protos::pike_payload::UpdateOrganizationAction =
            Message::parse_from_bytes(bytes).map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get UpdateOrganizationAction from bytes".to_string(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for UpdateOrganizationAction {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from UpdateOrganizationAction".to_string(),
            )
        })?;
        Ok(bytes)
    }
}

impl IntoProto<protos::pike_payload::UpdateOrganizationAction> for UpdateOrganizationAction {}
impl IntoNative<UpdateOrganizationAction> for protos::pike_payload::UpdateOrganizationAction {}

#[derive(Default, Clone)]
pub struct UpdateOrganizationActionBuilder {
    org_id: Option<String>,
    name: Option<String>,
    address: Option<String>,
    metadata: Vec<KeyValueEntry>,
}

impl UpdateOrganizationActionBuilder {
    pub fn new() -> Self {
        UpdateOrganizationActionBuilder::default()
    }

    pub fn with_org_id(mut self, org_id: String) -> UpdateOrganizationActionBuilder {
        self.org_id = Some(org_id);
        self
    }

    pub fn with_name(mut self, name: String) -> UpdateOrganizationActionBuilder {
        self.name = Some(name);
        self
    }

    pub fn with_address(mut self, address: String) -> UpdateOrganizationActionBuilder {
        self.address = Some(address);
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
        let org_id = self
            .org_id
            .ok_or_else(|| BuilderError::MissingField("'org_id' field is required".to_string()))?;

        let name = self.name.unwrap_or_default();

        let address = self.address.unwrap_or_default();

        let metadata = self.metadata;

        Ok(UpdateOrganizationAction {
            org_id,
            name,
            address,
            metadata,
        })
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

impl FromProto<protos::pike_state::KeyValueEntry> for KeyValueEntry {
    fn from_proto(
        key_value: protos::pike_state::KeyValueEntry,
    ) -> Result<Self, ProtoConversionError> {
        Ok(KeyValueEntry {
            key: key_value.get_key().to_string(),
            value: key_value.get_value().to_string(),
        })
    }
}

impl FromNative<KeyValueEntry> for protos::pike_state::KeyValueEntry {
    fn from_native(key_value: KeyValueEntry) -> Result<Self, ProtoConversionError> {
        let mut key_value_proto = protos::pike_state::KeyValueEntry::new();

        key_value_proto.set_key(key_value.key().to_string());
        key_value_proto.set_value(key_value.value().to_string());

        Ok(key_value_proto)
    }
}

impl FromBytes<KeyValueEntry> for KeyValueEntry {
    fn from_bytes(bytes: &[u8]) -> Result<KeyValueEntry, ProtoConversionError> {
        let proto: protos::pike_state::KeyValueEntry =
            Message::parse_from_bytes(bytes).map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get KeyValueEntry from bytes".to_string(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for KeyValueEntry {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from KeyValueEntry".to_string(),
            )
        })?;
        Ok(bytes)
    }
}

impl IntoProto<protos::pike_state::KeyValueEntry> for KeyValueEntry {}
impl IntoNative<KeyValueEntry> for protos::pike_state::KeyValueEntry {}

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

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct PikePayload {
    action: PikeAction,
}

impl PikePayload {
    pub fn new(action: PikeAction) -> Self {
        Self { action }
    }

    pub fn action(&self) -> &PikeAction {
        &self.action
    }
}

impl FromProto<protos::pike_payload::PikePayload> for PikePayload {
    fn from_proto(
        payload: protos::pike_payload::PikePayload,
    ) -> Result<Self, ProtoConversionError> {
        let action = match payload.get_action() {
            protos::pike_payload::PikePayload_Action::CREATE_AGENT => PikeAction::CreateAgent(
                CreateAgentAction::from_proto(payload.get_create_agent().clone())?,
            ),
            protos::pike_payload::PikePayload_Action::UPDATE_AGENT => PikeAction::UpdateAgent(
                UpdateAgentAction::from_proto(payload.get_update_agent().clone())?,
            ),
            protos::pike_payload::PikePayload_Action::CREATE_ORGANIZATION => {
                PikeAction::CreateOrganization(CreateOrganizationAction::from_proto(
                    payload.get_create_organization().clone(),
                )?)
            }
            protos::pike_payload::PikePayload_Action::UPDATE_ORGANIZATION => {
                PikeAction::UpdateOrganization(UpdateOrganizationAction::from_proto(
                    payload.get_update_organization().clone(),
                )?)
            }
            protos::pike_payload::PikePayload_Action::ACTION_UNSET => {
                return Err(ProtoConversionError::InvalidTypeError(
                    "Cannot convert PikePayload_Action with type unset.".to_string(),
                ));
            }
        };

        Ok(PikePayload { action })
    }
}

impl FromNative<PikePayload> for protos::pike_payload::PikePayload {
    fn from_native(native: PikePayload) -> Result<Self, ProtoConversionError> {
        let mut proto = protos::pike_payload::PikePayload::new();

        match native.action() {
            PikeAction::CreateAgent(payload) => {
                proto.set_action(protos::pike_payload::PikePayload_Action::CREATE_AGENT);
                proto.set_create_agent(payload.clone().into_proto()?);
            }
            PikeAction::UpdateAgent(payload) => {
                proto.set_action(protos::pike_payload::PikePayload_Action::UPDATE_AGENT);
                proto.set_update_agent(payload.clone().into_proto()?);
            }
            PikeAction::CreateOrganization(payload) => {
                proto.set_action(protos::pike_payload::PikePayload_Action::CREATE_ORGANIZATION);
                proto.set_create_organization(payload.clone().into_proto()?);
            }
            PikeAction::UpdateOrganization(payload) => {
                proto.set_action(protos::pike_payload::PikePayload_Action::UPDATE_ORGANIZATION);
                proto.set_update_organization(payload.clone().into_proto()?);
            }
        }

        Ok(proto)
    }
}

impl FromBytes<PikePayload> for PikePayload {
    fn from_bytes(bytes: &[u8]) -> Result<PikePayload, ProtoConversionError> {
        let proto: protos::pike_payload::PikePayload =
            Message::parse_from_bytes(bytes).map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get PikePayload from bytes".to_string(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for PikePayload {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from PikePayload".to_string(),
            )
        })?;
        Ok(bytes)
    }
}

impl IntoProto<protos::pike_payload::PikePayload> for PikePayload {}
impl IntoNative<PikePayload> for protos::pike_payload::PikePayload {}
