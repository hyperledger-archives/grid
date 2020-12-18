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

use std::error::Error as StdError;

use crate::protos;
use crate::protos::{
    location_payload, location_payload::LocationPayload_Action, product_payload,
    product_payload::ProductPayload_Action,
};
use crate::protos::{
    FromBytes, FromNative, FromProto, IntoBytes, IntoNative, IntoProto, ProtoConversionError,
};

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct SubmitBatchRequest {
    #[serde(default)]
    pub circuit_id: Option<String>,
    #[serde(default)]
    pub service_id: Option<String>,
    pub batches: Vec<Batch>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Batch {
    pub transactions: Vec<Transaction>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Transaction {
    pub family_name: String,
    pub version: String,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub inputs: Vec<String>,
    #[serde(default)]
    pub outputs: Vec<String>,
    pub payload: Payload,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Payload {
    Pike(PikePayload),
    Product(ProductPayload),
    Location(LocationPayload),
    Schema(SchemaPayload),
}

impl IntoBytes for Payload {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        match self {
            Payload::Pike(payload) => payload.into_bytes(),
            Payload::Product(payload) => payload.into_bytes(),
            Payload::Location(payload) => payload.into_bytes(),
            Payload::Schema(payload) => payload.into_bytes(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SubmitBatchResponse {
    id: String,
    message: String,
}

impl SubmitBatchResponse {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            message: format!("Batch {} submitted successfully", id),
        }
    }
}

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
    pub org_id: Option<String>,
    pub public_key: Option<String>,
    pub active: Option<bool>,
    pub roles: Vec<String>,
    pub metadata: Vec<KeyValueEntry>,
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
    pub org_id: Option<String>,
    pub name: Option<String>,
    pub address: Option<String>,
    pub metadata: Vec<KeyValueEntry>,
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
    pub org_id: Option<String>,
    pub name: Option<String>,
    pub address: Option<String>,
    pub metadata: Vec<KeyValueEntry>,
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
    pub key: Option<String>,
    pub value: Option<String>,
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

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum SchemaAction {
    SchemaCreate(SchemaCreateAction),
    SchemaUpdate(SchemaUpdateAction),
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct SchemaPayload {
    action: SchemaAction,
}

impl SchemaPayload {
    pub fn new(action: SchemaAction) -> Self {
        Self { action }
    }

    pub fn action(&self) -> &SchemaAction {
        &self.action
    }
}

impl FromProto<protos::schema_payload::SchemaPayload> for SchemaPayload {
    fn from_proto(
        payload: protos::schema_payload::SchemaPayload,
    ) -> Result<Self, ProtoConversionError> {
        let action = match payload.get_action() {
            protos::schema_payload::SchemaPayload_Action::SCHEMA_CREATE => {
                SchemaAction::SchemaCreate(SchemaCreateAction::from_proto(
                    payload.get_schema_create().clone(),
                )?)
            }
            protos::schema_payload::SchemaPayload_Action::SCHEMA_UPDATE => {
                SchemaAction::SchemaUpdate(SchemaUpdateAction::from_proto(
                    payload.get_schema_update().clone(),
                )?)
            }
            protos::schema_payload::SchemaPayload_Action::UNSET_ACTION => {
                return Err(ProtoConversionError::InvalidTypeError(
                    "Cannot convert SchemaPayload_Action with type unset.".to_string(),
                ));
            }
        };
        Ok(SchemaPayload { action })
    }
}

impl FromNative<SchemaPayload> for protos::schema_payload::SchemaPayload {
    fn from_native(payload: SchemaPayload) -> Result<Self, ProtoConversionError> {
        let mut proto_payload = protos::schema_payload::SchemaPayload::new();
        match payload.action() {
            SchemaAction::SchemaCreate(payload) => {
                proto_payload
                    .set_action(protos::schema_payload::SchemaPayload_Action::SCHEMA_CREATE);
                proto_payload.set_schema_create(payload.clone().into_proto()?);
            }
            SchemaAction::SchemaUpdate(payload) => {
                proto_payload
                    .set_action(protos::schema_payload::SchemaPayload_Action::SCHEMA_UPDATE);
                proto_payload.set_schema_update(payload.clone().into_proto()?);
            }
        }
        Ok(proto_payload)
    }
}

impl FromBytes<SchemaPayload> for SchemaPayload {
    fn from_bytes(bytes: &[u8]) -> Result<SchemaPayload, ProtoConversionError> {
        let proto: protos::schema_payload::SchemaPayload = Message::parse_from_bytes(bytes)
            .map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get SchemaPayload from bytes".to_string(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for SchemaPayload {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from SchemaPayload".to_string(),
            )
        })?;
        Ok(bytes)
    }
}

impl IntoProto<protos::schema_payload::SchemaPayload> for SchemaPayload {}
impl IntoNative<SchemaPayload> for protos::schema_payload::SchemaPayload {}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct SchemaCreateAction {
    schema_name: String,
    description: String,
    properties: Vec<PropertyDefinition>,
}

impl SchemaCreateAction {
    pub fn schema_name(&self) -> &str {
        &self.schema_name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn properties(&self) -> &[PropertyDefinition] {
        &self.properties
    }
}

impl FromProto<protos::schema_payload::SchemaCreateAction> for SchemaCreateAction {
    fn from_proto(
        schema_create: protos::schema_payload::SchemaCreateAction,
    ) -> Result<Self, ProtoConversionError> {
        Ok(SchemaCreateAction {
            schema_name: schema_create.get_schema_name().to_string(),
            description: schema_create.get_description().to_string(),
            properties: schema_create
                .get_properties()
                .to_vec()
                .into_iter()
                .map(PropertyDefinition::from_proto)
                .collect::<Result<Vec<PropertyDefinition>, ProtoConversionError>>()?,
        })
    }
}

impl FromNative<SchemaCreateAction> for protos::schema_payload::SchemaCreateAction {
    fn from_native(schema_create: SchemaCreateAction) -> Result<Self, ProtoConversionError> {
        let mut proto_schema_create = protos::schema_payload::SchemaCreateAction::new();

        proto_schema_create.set_schema_name(schema_create.schema_name().to_string());
        proto_schema_create.set_description(schema_create.description().to_string());
        proto_schema_create.set_properties(
            RepeatedField::from_vec(
            schema_create.properties().to_vec().into_iter()
            .map(PropertyDefinition::into_proto)
            .collect::<Result<Vec<protos::schema_state::PropertyDefinition>, ProtoConversionError>>()?,));

        Ok(proto_schema_create)
    }
}

impl FromBytes<SchemaCreateAction> for SchemaCreateAction {
    fn from_bytes(bytes: &[u8]) -> Result<SchemaCreateAction, ProtoConversionError> {
        let proto: protos::schema_payload::SchemaCreateAction = Message::parse_from_bytes(bytes)
            .map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get SchemaCreateAction from bytes".to_string(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for SchemaCreateAction {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from SchemaCreateAction".to_string(),
            )
        })?;
        Ok(bytes)
    }
}

impl IntoProto<protos::schema_payload::SchemaCreateAction> for SchemaCreateAction {}
impl IntoNative<SchemaCreateAction> for protos::schema_payload::SchemaCreateAction {}

#[derive(Default, Clone)]
pub struct SchemaCreateBuilder {
    schema_name: Option<String>,
    description: Option<String>,
    properties: Vec<PropertyDefinition>,
}

impl SchemaCreateBuilder {
    pub fn new() -> Self {
        SchemaCreateBuilder::default()
    }

    pub fn with_schema_name(mut self, schema_name: String) -> SchemaCreateBuilder {
        self.schema_name = Some(schema_name);
        self
    }

    pub fn with_description(mut self, description: String) -> SchemaCreateBuilder {
        self.description = Some(description);
        self
    }

    pub fn with_properties(mut self, properties: Vec<PropertyDefinition>) -> SchemaCreateBuilder {
        self.properties = properties;
        self
    }

    pub fn build(self) -> Result<SchemaCreateAction, BuilderError> {
        let schema_name = self.schema_name.ok_or_else(|| {
            BuilderError::MissingField("'schema_name' field is required".to_string())
        })?;

        let description = self.description.unwrap_or_default();

        let properties = {
            if !self.properties.is_empty() {
                self.properties
            } else {
                return Err(BuilderError::MissingField(
                    "'properties' field is required".to_string(),
                ));
            }
        };

        Ok(SchemaCreateAction {
            schema_name,
            description,
            properties,
        })
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct SchemaUpdateAction {
    schema_name: String,
    properties: Vec<PropertyDefinition>,
}

impl SchemaUpdateAction {
    pub fn schema_name(&self) -> &str {
        &self.schema_name
    }

    pub fn properties(&self) -> &[PropertyDefinition] {
        &self.properties
    }
}

impl FromProto<protos::schema_payload::SchemaUpdateAction> for SchemaUpdateAction {
    fn from_proto(
        schema_update: protos::schema_payload::SchemaUpdateAction,
    ) -> Result<Self, ProtoConversionError> {
        Ok(SchemaUpdateAction {
            schema_name: schema_update.get_schema_name().to_string(),
            properties: schema_update
                .get_properties()
                .to_vec()
                .into_iter()
                .map(PropertyDefinition::from_proto)
                .collect::<Result<Vec<PropertyDefinition>, ProtoConversionError>>()?,
        })
    }
}

impl FromNative<SchemaUpdateAction> for protos::schema_payload::SchemaUpdateAction {
    fn from_native(schema_update: SchemaUpdateAction) -> Result<Self, ProtoConversionError> {
        let mut proto_schema_update = protos::schema_payload::SchemaUpdateAction::new();

        proto_schema_update.set_schema_name(schema_update.schema_name().to_string());
        proto_schema_update.set_properties(
            RepeatedField::from_vec(
            schema_update.properties().to_vec().into_iter()
            .map(PropertyDefinition::into_proto)
            .collect::<Result<Vec<protos::schema_state::PropertyDefinition>, ProtoConversionError>>()?,));

        Ok(proto_schema_update)
    }
}

impl FromBytes<SchemaUpdateAction> for SchemaUpdateAction {
    fn from_bytes(bytes: &[u8]) -> Result<SchemaUpdateAction, ProtoConversionError> {
        let proto: protos::schema_payload::SchemaUpdateAction = Message::parse_from_bytes(bytes)
            .map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get SchemaUpdateAction from bytes".to_string(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for SchemaUpdateAction {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from SchemaUpdateAction".to_string(),
            )
        })?;
        Ok(bytes)
    }
}

impl IntoProto<protos::schema_payload::SchemaUpdateAction> for SchemaUpdateAction {}
impl IntoNative<SchemaUpdateAction> for protos::schema_payload::SchemaUpdateAction {}

#[derive(Default, Clone)]
pub struct SchemaUpdateBuilder {
    schema_name: Option<String>,
    description: Option<String>,
    properties: Vec<PropertyDefinition>,
}

impl SchemaUpdateBuilder {
    pub fn new() -> Self {
        SchemaUpdateBuilder::default()
    }

    pub fn with_schema_name(mut self, schema_name: String) -> SchemaUpdateBuilder {
        self.schema_name = Some(schema_name);
        self
    }

    pub fn with_properties(mut self, properties: Vec<PropertyDefinition>) -> SchemaUpdateBuilder {
        self.properties = properties;
        self
    }

    pub fn build(self) -> Result<SchemaUpdateAction, BuilderError> {
        let schema_name = self
            .schema_name
            .ok_or_else(|| BuilderError::MissingField("'schema field is required".to_string()))?;

        let properties = {
            if !self.properties.is_empty() {
                self.properties
            } else {
                return Err(BuilderError::MissingField(
                    "'properties' field is required".to_string(),
                ));
            }
        };

        Ok(SchemaUpdateAction {
            schema_name,
            properties,
        })
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub enum ProductNamespace {
    GS1,
}

impl Default for ProductNamespace {
    fn default() -> Self {
        ProductNamespace::GS1
    }
}

impl FromProto<protos::product_state::Product_ProductNamespace> for ProductNamespace {
    fn from_proto(
        product_namespace: protos::product_state::Product_ProductNamespace,
    ) -> Result<Self, ProtoConversionError> {
        match product_namespace {
            protos::product_state::Product_ProductNamespace::GS1 => Ok(ProductNamespace::GS1),
            protos::product_state::Product_ProductNamespace::UNSET_TYPE => {
                Err(ProtoConversionError::InvalidTypeError(
                    "Cannot convert Product_ProductNamespace with type UNSET_TYPE".to_string(),
                ))
            }
        }
    }
}

impl FromNative<ProductNamespace> for protos::product_state::Product_ProductNamespace {
    fn from_native(product_namespace: ProductNamespace) -> Result<Self, ProtoConversionError> {
        match product_namespace {
            ProductNamespace::GS1 => Ok(protos::product_state::Product_ProductNamespace::GS1),
        }
    }
}

impl IntoProto<protos::product_state::Product_ProductNamespace> for ProductNamespace {}
impl IntoNative<ProductNamespace> for protos::product_state::Product_ProductNamespace {}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum ProductAction {
    ProductCreate(ProductCreateAction),
    ProductUpdate(ProductUpdateAction),
    ProductDelete(ProductDeleteAction),
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct ProductPayload {
    action: ProductAction,
    timestamp: u64,
}

impl ProductPayload {
    pub fn new(timestamp: u64, action: ProductAction) -> Self {
        Self { timestamp, action }
    }

    pub fn action(&self) -> &ProductAction {
        &self.action
    }
    pub fn timestamp(&self) -> &u64 {
        &self.timestamp
    }
}

impl FromProto<protos::product_payload::ProductPayload> for ProductPayload {
    fn from_proto(
        payload: protos::product_payload::ProductPayload,
    ) -> Result<Self, ProtoConversionError> {
        let action = match payload.get_action() {
            ProductPayload_Action::PRODUCT_CREATE => ProductAction::ProductCreate(
                ProductCreateAction::from_proto(payload.get_product_create().clone())?,
            ),
            ProductPayload_Action::PRODUCT_UPDATE => ProductAction::ProductUpdate(
                ProductUpdateAction::from_proto(payload.get_product_update().clone())?,
            ),
            ProductPayload_Action::PRODUCT_DELETE => ProductAction::ProductDelete(
                ProductDeleteAction::from_proto(payload.get_product_delete().clone())?,
            ),
            ProductPayload_Action::UNSET_ACTION => {
                return Err(ProtoConversionError::InvalidTypeError(
                    "Cannot convert ProductPayload_Action with type unset".to_string(),
                ));
            }
        };
        Ok(ProductPayload {
            action,
            timestamp: payload.get_timestamp(),
        })
    }
}

impl FromNative<ProductPayload> for protos::product_payload::ProductPayload {
    fn from_native(native: ProductPayload) -> Result<Self, ProtoConversionError> {
        let mut proto = product_payload::ProductPayload::new();

        proto.set_timestamp(*native.timestamp());

        match native.action() {
            ProductAction::ProductCreate(payload) => {
                proto.set_action(ProductPayload_Action::PRODUCT_CREATE);
                proto.set_product_create(payload.clone().into_proto()?);
            }
            ProductAction::ProductUpdate(payload) => {
                proto.set_action(ProductPayload_Action::PRODUCT_UPDATE);
                proto.set_product_update(payload.clone().into_proto()?);
            }
            ProductAction::ProductDelete(payload) => {
                proto.set_action(ProductPayload_Action::PRODUCT_DELETE);
                proto.set_product_delete(payload.clone().into_proto()?);
            }
        }

        Ok(proto)
    }
}

impl FromBytes<ProductPayload> for ProductPayload {
    fn from_bytes(bytes: &[u8]) -> Result<ProductPayload, ProtoConversionError> {
        let proto: product_payload::ProductPayload =
            Message::parse_from_bytes(bytes).map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get ProductPayload from bytes".into(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for ProductPayload {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get ProductPayload from bytes".into(),
            )
        })?;
        Ok(bytes)
    }
}

impl IntoProto<protos::product_payload::ProductPayload> for ProductPayload {}
impl IntoNative<ProductPayload> for protos::product_payload::ProductPayload {}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct ProductCreateAction {
    product_namespace: ProductNamespace,
    product_id: String,
    owner: String,
    properties: Vec<PropertyValue>,
}

impl ProductCreateAction {
    pub fn product_namespace(&self) -> &ProductNamespace {
        &self.product_namespace
    }

    pub fn product_id(&self) -> &str {
        &self.product_id
    }

    pub fn owner(&self) -> &str {
        &self.owner
    }

    pub fn properties(&self) -> &[PropertyValue] {
        &self.properties
    }
}

impl FromProto<product_payload::ProductCreateAction> for ProductCreateAction {
    fn from_proto(
        proto: product_payload::ProductCreateAction,
    ) -> Result<Self, ProtoConversionError> {
        Ok(ProductCreateAction {
            product_namespace: ProductNamespace::from_proto(proto.get_product_namespace())?,
            product_id: proto.get_product_id().to_string(),
            owner: proto.get_owner().to_string(),
            properties: proto
                .get_properties()
                .to_vec()
                .into_iter()
                .map(PropertyValue::from_proto)
                .collect::<Result<Vec<PropertyValue>, ProtoConversionError>>()?,
        })
    }
}

impl FromNative<ProductCreateAction> for product_payload::ProductCreateAction {
    fn from_native(native: ProductCreateAction) -> Result<Self, ProtoConversionError> {
        let mut proto = protos::product_payload::ProductCreateAction::new();
        proto.set_product_namespace(native.product_namespace().clone().into_proto()?);
        proto.set_product_id(native.product_id().to_string());
        proto.set_owner(native.owner().to_string());
        proto.set_properties(RepeatedField::from_vec(
            native
                .properties()
                .to_vec()
                .into_iter()
                .map(PropertyValue::into_proto)
                .collect::<Result<Vec<protos::schema_state::PropertyValue>, ProtoConversionError>>(
                )?,
        ));
        Ok(proto)
    }
}

impl FromBytes<ProductCreateAction> for ProductCreateAction {
    fn from_bytes(bytes: &[u8]) -> Result<ProductCreateAction, ProtoConversionError> {
        let proto: protos::product_payload::ProductCreateAction = Message::parse_from_bytes(bytes)
            .map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get ProductCreateAction from bytes".to_string(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for ProductCreateAction {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from ProductCreateAction".to_string(),
            )
        })?;
        Ok(bytes)
    }
}

impl IntoProto<protos::product_payload::ProductCreateAction> for ProductCreateAction {}
impl IntoNative<ProductCreateAction> for protos::product_payload::ProductCreateAction {}

#[derive(Default, Debug)]
pub struct ProductCreateActionBuilder {
    product_namespace: Option<ProductNamespace>,
    product_id: Option<String>,
    owner: Option<String>,
    properties: Option<Vec<PropertyValue>>,
}

impl ProductCreateActionBuilder {
    pub fn new() -> Self {
        ProductCreateActionBuilder::default()
    }
    pub fn with_product_namespace(mut self, value: ProductNamespace) -> Self {
        self.product_namespace = Some(value);
        self
    }
    pub fn with_product_id(mut self, value: String) -> Self {
        self.product_id = Some(value);
        self
    }
    pub fn with_owner(mut self, value: String) -> Self {
        self.owner = Some(value);
        self
    }
    pub fn with_properties(mut self, value: Vec<PropertyValue>) -> Self {
        self.properties = Some(value);
        self
    }
    pub fn build(self) -> Result<ProductCreateAction, BuilderError> {
        let product_namespace = self.product_namespace.ok_or_else(|| {
            BuilderError::MissingField("'product_namespace' field is required".to_string())
        })?;
        let product_id = self
            .product_id
            .ok_or_else(|| BuilderError::MissingField("'product_id' field is required".into()))?;
        let owner = self
            .owner
            .ok_or_else(|| BuilderError::MissingField("'owner' field is required".into()))?;
        let properties = self
            .properties
            .ok_or_else(|| BuilderError::MissingField("'properties' field is required".into()))?;
        Ok(ProductCreateAction {
            product_namespace,
            product_id,
            owner,
            properties,
        })
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct ProductUpdateAction {
    product_namespace: ProductNamespace,
    product_id: String,
    properties: Vec<PropertyValue>,
}

impl ProductUpdateAction {
    pub fn product_namespace(&self) -> &ProductNamespace {
        &self.product_namespace
    }

    pub fn product_id(&self) -> &str {
        &self.product_id
    }

    pub fn properties(&self) -> &[PropertyValue] {
        &self.properties
    }
}

impl FromProto<protos::product_payload::ProductUpdateAction> for ProductUpdateAction {
    fn from_proto(
        proto: protos::product_payload::ProductUpdateAction,
    ) -> Result<Self, ProtoConversionError> {
        Ok(ProductUpdateAction {
            product_namespace: ProductNamespace::from_proto(proto.get_product_namespace())?,
            product_id: proto.get_product_id().to_string(),
            properties: proto
                .get_properties()
                .to_vec()
                .into_iter()
                .map(PropertyValue::from_proto)
                .collect::<Result<Vec<PropertyValue>, ProtoConversionError>>()?,
        })
    }
}

impl FromNative<ProductUpdateAction> for protos::product_payload::ProductUpdateAction {
    fn from_native(native: ProductUpdateAction) -> Result<Self, ProtoConversionError> {
        let mut proto = protos::product_payload::ProductUpdateAction::new();
        proto.set_product_namespace(native.product_namespace().clone().into_proto()?);
        proto.set_product_id(native.product_id().to_string());
        proto.set_properties(RepeatedField::from_vec(
            native
                .properties()
                .to_vec()
                .into_iter()
                .map(PropertyValue::into_proto)
                .collect::<Result<Vec<protos::schema_state::PropertyValue>, ProtoConversionError>>(
                )?,
        ));

        Ok(proto)
    }
}

impl FromBytes<ProductUpdateAction> for ProductUpdateAction {
    fn from_bytes(bytes: &[u8]) -> Result<ProductUpdateAction, ProtoConversionError> {
        let proto: protos::product_payload::ProductUpdateAction = Message::parse_from_bytes(bytes)
            .map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get ProductUpdateAction from bytes".to_string(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for ProductUpdateAction {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from ProductUpdateAction".to_string(),
            )
        })?;
        Ok(bytes)
    }
}

impl IntoProto<protos::product_payload::ProductUpdateAction> for ProductUpdateAction {}
impl IntoNative<ProductUpdateAction> for protos::product_payload::ProductUpdateAction {}

#[derive(Default, Clone)]
pub struct ProductUpdateActionBuilder {
    product_namespace: Option<ProductNamespace>,
    product_id: Option<String>,
    properties: Vec<PropertyValue>,
}

impl ProductUpdateActionBuilder {
    pub fn new() -> Self {
        ProductUpdateActionBuilder::default()
    }

    pub fn with_product_namespace(mut self, product_namespace: ProductNamespace) -> Self {
        self.product_namespace = Some(product_namespace);
        self
    }

    pub fn with_product_id(mut self, product_id: String) -> Self {
        self.product_id = Some(product_id);
        self
    }

    pub fn with_properties(mut self, properties: Vec<PropertyValue>) -> Self {
        self.properties = properties;
        self
    }

    pub fn build(self) -> Result<ProductUpdateAction, BuilderError> {
        let product_namespace = self.product_namespace.ok_or_else(|| {
            BuilderError::MissingField("'product_namespace' field is required".to_string())
        })?;

        let product_id = self.product_id.ok_or_else(|| {
            BuilderError::MissingField("'product_id' field is required".to_string())
        })?;

        let properties = {
            if !self.properties.is_empty() {
                self.properties
            } else {
                return Err(BuilderError::MissingField(
                    "'properties' field is required".to_string(),
                ));
            }
        };

        Ok(ProductUpdateAction {
            product_namespace,
            product_id,
            properties,
        })
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct ProductDeleteAction {
    product_namespace: ProductNamespace,
    product_id: String,
}

impl ProductDeleteAction {
    pub fn product_namespace(&self) -> &ProductNamespace {
        &self.product_namespace
    }

    pub fn product_id(&self) -> &str {
        &self.product_id
    }
}

impl FromProto<protos::product_payload::ProductDeleteAction> for ProductDeleteAction {
    fn from_proto(
        proto: protos::product_payload::ProductDeleteAction,
    ) -> Result<Self, ProtoConversionError> {
        Ok(ProductDeleteAction {
            product_namespace: ProductNamespace::from_proto(proto.get_product_namespace())?,
            product_id: proto.get_product_id().to_string(),
        })
    }
}

impl FromNative<ProductDeleteAction> for protos::product_payload::ProductDeleteAction {
    fn from_native(native: ProductDeleteAction) -> Result<Self, ProtoConversionError> {
        let mut proto = protos::product_payload::ProductDeleteAction::new();
        proto.set_product_namespace(native.product_namespace().clone().into_proto()?);
        proto.set_product_id(native.product_id().to_string());
        Ok(proto)
    }
}

impl FromBytes<ProductDeleteAction> for ProductDeleteAction {
    fn from_bytes(bytes: &[u8]) -> Result<ProductDeleteAction, ProtoConversionError> {
        let proto: protos::product_payload::ProductDeleteAction = Message::parse_from_bytes(bytes)
            .map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get ProductDeleteAction from bytes".to_string(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for ProductDeleteAction {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from ProductDeleteAction".to_string(),
            )
        })?;
        Ok(bytes)
    }
}

impl IntoProto<protos::product_payload::ProductDeleteAction> for ProductDeleteAction {}
impl IntoNative<ProductDeleteAction> for protos::product_payload::ProductDeleteAction {}

#[derive(Default, Clone)]
pub struct ProductDeleteActionBuilder {
    product_namespace: Option<ProductNamespace>,
    product_id: Option<String>,
}

impl ProductDeleteActionBuilder {
    pub fn new() -> Self {
        ProductDeleteActionBuilder::default()
    }

    pub fn with_product_namespace(mut self, product_namespace: ProductNamespace) -> Self {
        self.product_namespace = Some(product_namespace);
        self
    }

    pub fn with_product_id(mut self, product_id: String) -> Self {
        self.product_id = Some(product_id);
        self
    }

    pub fn build(self) -> Result<ProductDeleteAction, BuilderError> {
        let product_namespace = self.product_namespace.ok_or_else(|| {
            BuilderError::MissingField("'product_namespace' field is required".to_string())
        })?;

        let product_id = self.product_id.ok_or_else(|| {
            BuilderError::MissingField("'product_id' field is required".to_string())
        })?;

        Ok(ProductDeleteAction {
            product_namespace,
            product_id,
        })
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
pub enum LocationNamespace {
    GS1,
}

impl Default for LocationNamespace {
    fn default() -> Self {
        LocationNamespace::GS1
    }
}

impl FromProto<protos::location_payload::LocationNamespace> for LocationNamespace {
    fn from_proto(
        namespace: protos::location_payload::LocationNamespace,
    ) -> Result<Self, ProtoConversionError> {
        match namespace {
            protos::location_payload::LocationNamespace::GS1 => Ok(LocationNamespace::GS1),
            protos::location_payload::LocationNamespace::UNSET_TYPE => {
                Err(ProtoConversionError::InvalidTypeError(
                    "Cannot convert Location_LocationType with type UNSET_TYPE".to_string(),
                ))
            }
        }
    }
}

impl FromNative<LocationNamespace> for protos::location_payload::LocationNamespace {
    fn from_native(namespace: LocationNamespace) -> Result<Self, ProtoConversionError> {
        match namespace {
            LocationNamespace::GS1 => Ok(protos::location_payload::LocationNamespace::GS1),
        }
    }
}

impl IntoProto<protos::location_payload::LocationNamespace> for LocationNamespace {}
impl IntoNative<LocationNamespace> for protos::location_payload::LocationNamespace {}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum LocationAction {
    LocationCreate(LocationCreateAction),
    LocationUpdate(LocationUpdateAction),
    LocationDelete(LocationDeleteAction),
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct LocationPayload {
    action: LocationAction,
    timestamp: u64,
}

impl LocationPayload {
    pub fn action(&self) -> &LocationAction {
        &self.action
    }

    pub fn timestamp(&self) -> &u64 {
        &self.timestamp
    }
}

impl FromProto<protos::location_payload::LocationPayload> for LocationPayload {
    fn from_proto(
        payload: protos::location_payload::LocationPayload,
    ) -> Result<Self, ProtoConversionError> {
        let action = match payload.get_action() {
            LocationPayload_Action::LOCATION_CREATE => LocationAction::LocationCreate(
                LocationCreateAction::from_proto(payload.get_location_create().clone())?,
            ),
            LocationPayload_Action::LOCATION_UPDATE => LocationAction::LocationUpdate(
                LocationUpdateAction::from_proto(payload.get_location_update().clone())?,
            ),
            LocationPayload_Action::LOCATION_DELETE => LocationAction::LocationDelete(
                LocationDeleteAction::from_proto(payload.get_location_delete().clone())?,
            ),
            LocationPayload_Action::UNSET_ACTION => {
                return Err(ProtoConversionError::InvalidTypeError(
                    "Cannot convert LocationPayload_Action with type unset".to_string(),
                ));
            }
        };
        Ok(LocationPayload {
            action,
            timestamp: payload.get_timestamp(),
        })
    }
}

impl FromNative<LocationPayload> for protos::location_payload::LocationPayload {
    fn from_native(native: LocationPayload) -> Result<Self, ProtoConversionError> {
        let mut proto = location_payload::LocationPayload::new();

        proto.set_timestamp(*native.timestamp());

        match native.action() {
            LocationAction::LocationCreate(payload) => {
                proto.set_action(LocationPayload_Action::LOCATION_CREATE);
                proto.set_location_create(payload.clone().into_proto()?);
            }
            LocationAction::LocationUpdate(payload) => {
                proto.set_action(LocationPayload_Action::LOCATION_UPDATE);
                proto.set_location_update(payload.clone().into_proto()?);
            }
            LocationAction::LocationDelete(payload) => {
                proto.set_action(LocationPayload_Action::LOCATION_DELETE);
                proto.set_location_delete(payload.clone().into_proto()?);
            }
        }

        Ok(proto)
    }
}

impl FromBytes<LocationPayload> for LocationPayload {
    fn from_bytes(bytes: &[u8]) -> Result<LocationPayload, ProtoConversionError> {
        let proto: location_payload::LocationPayload =
            Message::parse_from_bytes(bytes).map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get LocationPayload from bytes".into(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for LocationPayload {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get LocationPayload from bytes".into(),
            )
        })?;
        Ok(bytes)
    }
}

impl IntoProto<protos::location_payload::LocationPayload> for LocationPayload {}
impl IntoNative<LocationPayload> for protos::location_payload::LocationPayload {}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct LocationCreateAction {
    namespace: LocationNamespace,
    location_id: String,
    owner: String,
    properties: Vec<PropertyValue>,
}

impl LocationCreateAction {
    pub fn namespace(&self) -> &LocationNamespace {
        &self.namespace
    }

    pub fn location_id(&self) -> &str {
        &self.location_id
    }

    pub fn owner(&self) -> &str {
        &self.owner
    }

    pub fn properties(&self) -> &[PropertyValue] {
        &self.properties
    }
}

impl FromProto<location_payload::LocationCreateAction> for LocationCreateAction {
    fn from_proto(
        proto: location_payload::LocationCreateAction,
    ) -> Result<Self, ProtoConversionError> {
        Ok(LocationCreateAction {
            namespace: LocationNamespace::from_proto(proto.get_namespace())?,
            location_id: proto.get_location_id().to_string(),
            owner: proto.get_owner().to_string(),
            properties: proto
                .get_properties()
                .to_vec()
                .into_iter()
                .map(PropertyValue::from_proto)
                .collect::<Result<Vec<PropertyValue>, ProtoConversionError>>()?,
        })
    }
}

impl FromNative<LocationCreateAction> for location_payload::LocationCreateAction {
    fn from_native(native: LocationCreateAction) -> Result<Self, ProtoConversionError> {
        let mut proto = protos::location_payload::LocationCreateAction::new();
        proto.set_namespace(native.namespace().clone().into_proto()?);
        proto.set_location_id(native.location_id().to_string());
        proto.set_owner(native.owner().to_string());
        proto.set_properties(RepeatedField::from_vec(
            native
                .properties()
                .to_vec()
                .into_iter()
                .map(PropertyValue::into_proto)
                .collect::<Result<Vec<protos::schema_state::PropertyValue>, ProtoConversionError>>(
                )?,
        ));
        Ok(proto)
    }
}

impl FromBytes<LocationCreateAction> for LocationCreateAction {
    fn from_bytes(bytes: &[u8]) -> Result<LocationCreateAction, ProtoConversionError> {
        let proto: protos::location_payload::LocationCreateAction =
            Message::parse_from_bytes(bytes).map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get LocationCreateAction from bytes".to_string(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for LocationCreateAction {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from LocationCreateAction".to_string(),
            )
        })?;
        Ok(bytes)
    }
}

impl IntoProto<protos::location_payload::LocationCreateAction> for LocationCreateAction {}
impl IntoNative<LocationCreateAction> for protos::location_payload::LocationCreateAction {}

#[derive(Default, Debug)]
pub struct LocationCreateActionBuilder {
    namespace: Option<LocationNamespace>,
    location_id: Option<String>,
    owner: Option<String>,
    properties: Option<Vec<PropertyValue>>,
}

impl LocationCreateActionBuilder {
    pub fn new() -> Self {
        LocationCreateActionBuilder::default()
    }
    pub fn with_namespace(mut self, value: LocationNamespace) -> Self {
        self.namespace = Some(value);
        self
    }
    pub fn with_location_id(mut self, value: String) -> Self {
        self.location_id = Some(value);
        self
    }
    pub fn with_owner(mut self, value: String) -> Self {
        self.owner = Some(value);
        self
    }
    pub fn with_properties(mut self, value: Vec<PropertyValue>) -> Self {
        self.properties = Some(value);
        self
    }
    pub fn build(self) -> Result<LocationCreateAction, BuilderError> {
        let namespace = self.namespace.ok_or_else(|| {
            BuilderError::MissingField("'namespace' field is required".to_string())
        })?;
        let location_id = self
            .location_id
            .ok_or_else(|| BuilderError::MissingField("'location_id' field is required".into()))?;
        let owner = self
            .owner
            .ok_or_else(|| BuilderError::MissingField("'owner' field is required".into()))?;
        let properties = self
            .properties
            .ok_or_else(|| BuilderError::MissingField("'properties' field is required".into()))?;
        Ok(LocationCreateAction {
            namespace,
            location_id,
            owner,
            properties,
        })
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct LocationUpdateAction {
    namespace: LocationNamespace,
    location_id: String,
    properties: Vec<PropertyValue>,
}

impl LocationUpdateAction {
    pub fn namespace(&self) -> &LocationNamespace {
        &self.namespace
    }

    pub fn location_id(&self) -> &str {
        &self.location_id
    }

    pub fn properties(&self) -> &[PropertyValue] {
        &self.properties
    }
}

impl FromProto<protos::location_payload::LocationUpdateAction> for LocationUpdateAction {
    fn from_proto(
        proto: protos::location_payload::LocationUpdateAction,
    ) -> Result<Self, ProtoConversionError> {
        Ok(LocationUpdateAction {
            namespace: LocationNamespace::from_proto(proto.get_namespace())?,
            location_id: proto.get_location_id().to_string(),
            properties: proto
                .get_properties()
                .to_vec()
                .into_iter()
                .map(PropertyValue::from_proto)
                .collect::<Result<Vec<PropertyValue>, ProtoConversionError>>()?,
        })
    }
}

impl FromNative<LocationUpdateAction> for protos::location_payload::LocationUpdateAction {
    fn from_native(native: LocationUpdateAction) -> Result<Self, ProtoConversionError> {
        let mut proto = protos::location_payload::LocationUpdateAction::new();
        proto.set_namespace(native.namespace().clone().into_proto()?);
        proto.set_location_id(native.location_id().to_string());
        proto.set_properties(RepeatedField::from_vec(
            native
                .properties()
                .to_vec()
                .into_iter()
                .map(PropertyValue::into_proto)
                .collect::<Result<Vec<protos::schema_state::PropertyValue>, ProtoConversionError>>(
                )?,
        ));

        Ok(proto)
    }
}

impl FromBytes<LocationUpdateAction> for LocationUpdateAction {
    fn from_bytes(bytes: &[u8]) -> Result<LocationUpdateAction, ProtoConversionError> {
        let proto: protos::location_payload::LocationUpdateAction =
            Message::parse_from_bytes(bytes).map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get LocationUpdateAction from bytes".to_string(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for LocationUpdateAction {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from LocationUpdateAction".to_string(),
            )
        })?;
        Ok(bytes)
    }
}

impl IntoProto<protos::location_payload::LocationUpdateAction> for LocationUpdateAction {}
impl IntoNative<LocationUpdateAction> for protos::location_payload::LocationUpdateAction {}

#[derive(Default, Clone)]
pub struct LocationUpdateActionBuilder {
    namespace: Option<LocationNamespace>,
    location_id: Option<String>,
    properties: Vec<PropertyValue>,
}

impl LocationUpdateActionBuilder {
    pub fn new() -> Self {
        LocationUpdateActionBuilder::default()
    }

    pub fn with_namespace(mut self, namespace: LocationNamespace) -> Self {
        self.namespace = Some(namespace);
        self
    }

    pub fn with_location_id(mut self, location_id: String) -> Self {
        self.location_id = Some(location_id);
        self
    }

    pub fn with_properties(mut self, properties: Vec<PropertyValue>) -> Self {
        self.properties = properties;
        self
    }

    pub fn build(self) -> Result<LocationUpdateAction, BuilderError> {
        let namespace = self.namespace.ok_or_else(|| {
            BuilderError::MissingField("'namespace' field is required".to_string())
        })?;

        let location_id = self.location_id.ok_or_else(|| {
            BuilderError::MissingField("'location_id' field is required".to_string())
        })?;

        let properties = {
            if !self.properties.is_empty() {
                self.properties
            } else {
                return Err(BuilderError::MissingField(
                    "'properties' field is required".to_string(),
                ));
            }
        };

        Ok(LocationUpdateAction {
            namespace,
            location_id,
            properties,
        })
    }
}
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct LocationDeleteAction {
    namespace: LocationNamespace,
    location_id: String,
}

/// Native implementation for LocationDeleteAction
impl LocationDeleteAction {
    pub fn namespace(&self) -> &LocationNamespace {
        &self.namespace
    }

    pub fn location_id(&self) -> &str {
        &self.location_id
    }
}

impl FromProto<protos::location_payload::LocationDeleteAction> for LocationDeleteAction {
    fn from_proto(
        proto: protos::location_payload::LocationDeleteAction,
    ) -> Result<Self, ProtoConversionError> {
        Ok(LocationDeleteAction {
            namespace: LocationNamespace::from_proto(proto.get_namespace())?,
            location_id: proto.get_location_id().to_string(),
        })
    }
}

impl FromNative<LocationDeleteAction> for protos::location_payload::LocationDeleteAction {
    fn from_native(native: LocationDeleteAction) -> Result<Self, ProtoConversionError> {
        let mut proto = protos::location_payload::LocationDeleteAction::new();
        proto.set_namespace(native.namespace().clone().into_proto()?);
        proto.set_location_id(native.location_id().to_string());
        Ok(proto)
    }
}

impl FromBytes<LocationDeleteAction> for LocationDeleteAction {
    fn from_bytes(bytes: &[u8]) -> Result<LocationDeleteAction, ProtoConversionError> {
        let proto: protos::location_payload::LocationDeleteAction =
            Message::parse_from_bytes(bytes).map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get LocationDeleteAction from bytes".to_string(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for LocationDeleteAction {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from LocationDeleteAction".to_string(),
            )
        })?;
        Ok(bytes)
    }
}

impl IntoProto<protos::location_payload::LocationDeleteAction> for LocationDeleteAction {}
impl IntoNative<LocationDeleteAction> for protos::location_payload::LocationDeleteAction {}

#[derive(Default, Clone)]
pub struct LocationDeleteActionBuilder {
    namespace: Option<LocationNamespace>,
    location_id: Option<String>,
}

impl LocationDeleteActionBuilder {
    pub fn new() -> Self {
        LocationDeleteActionBuilder::default()
    }

    pub fn with_namespace(mut self, namespace: LocationNamespace) -> Self {
        self.namespace = Some(namespace);
        self
    }

    pub fn with_location_id(mut self, location_id: String) -> Self {
        self.location_id = Some(location_id);
        self
    }

    pub fn build(self) -> Result<LocationDeleteAction, BuilderError> {
        let namespace = self.namespace.ok_or_else(|| {
            BuilderError::MissingField("'namespace' field is required".to_string())
        })?;

        let location_id = self.location_id.ok_or_else(|| {
            BuilderError::MissingField("'location_id' field is required".to_string())
        })?;

        Ok(LocationDeleteAction {
            namespace,
            location_id,
        })
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct PropertyDefinition {
    name: String,
    data_type: DataType,
    required: bool,
    description: String,
    number_exponent: i32,
    enum_options: Vec<String>,
    struct_properties: Vec<PropertyDefinition>,
}

impl PropertyDefinition {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn data_type(&self) -> &DataType {
        &self.data_type
    }

    pub fn required(&self) -> &bool {
        &self.required
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn number_exponent(&self) -> &i32 {
        &self.number_exponent
    }

    pub fn enum_options(&self) -> &[String] {
        &self.enum_options
    }

    pub fn struct_properties(&self) -> &[PropertyDefinition] {
        &self.struct_properties
    }
}

impl FromProto<protos::schema_state::PropertyDefinition> for PropertyDefinition {
    fn from_proto(
        property_definition: protos::schema_state::PropertyDefinition,
    ) -> Result<Self, ProtoConversionError> {
        Ok(PropertyDefinition {
            name: property_definition.get_name().to_string(),
            data_type: DataType::from_proto(property_definition.get_data_type())?,
            required: property_definition.get_required(),
            description: property_definition.get_description().to_string(),
            number_exponent: property_definition.get_number_exponent(),
            enum_options: property_definition.get_enum_options().to_vec(),
            struct_properties: property_definition
                .get_struct_properties()
                .to_vec()
                .into_iter()
                .map(PropertyDefinition::from_proto)
                .collect::<Result<Vec<PropertyDefinition>, ProtoConversionError>>()?,
        })
    }
}

impl FromNative<PropertyDefinition> for protos::schema_state::PropertyDefinition {
    fn from_native(property_definition: PropertyDefinition) -> Result<Self, ProtoConversionError> {
        let mut proto_property_definition = protos::schema_state::PropertyDefinition::new();
        proto_property_definition.set_name(property_definition.name().to_string());
        proto_property_definition
            .set_data_type(property_definition.data_type().clone().into_proto()?);
        proto_property_definition.set_required(*property_definition.required());
        proto_property_definition.set_description(property_definition.description().to_string());
        proto_property_definition.set_number_exponent(*property_definition.number_exponent());
        proto_property_definition.set_enum_options(RepeatedField::from_vec(
            property_definition.enum_options().to_vec(),
        ));
        proto_property_definition.set_struct_properties(
            RepeatedField::from_vec(
            property_definition.struct_properties().to_vec().into_iter()
            .map(PropertyDefinition::into_proto)
            .collect::<Result<Vec<protos::schema_state::PropertyDefinition>, ProtoConversionError>>()?,));
        Ok(proto_property_definition)
    }
}

impl FromBytes<PropertyDefinition> for PropertyDefinition {
    fn from_bytes(bytes: &[u8]) -> Result<PropertyDefinition, ProtoConversionError> {
        let proto: protos::schema_state::PropertyDefinition = Message::parse_from_bytes(bytes)
            .map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get PropertyDefinition from bytes".to_string(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for PropertyDefinition {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from PropertyDefinition".to_string(),
            )
        })?;
        Ok(bytes)
    }
}

impl IntoProto<protos::schema_state::PropertyDefinition> for PropertyDefinition {}
impl IntoNative<PropertyDefinition> for protos::schema_state::PropertyDefinition {}

#[derive(Debug)]
pub enum PropertyDefinitionBuildError {
    MissingField(String),
    EmptyVec(String),
}

impl StdError for PropertyDefinitionBuildError {
    fn description(&self) -> &str {
        match *self {
            PropertyDefinitionBuildError::MissingField(ref msg) => msg,
            PropertyDefinitionBuildError::EmptyVec(ref msg) => msg,
        }
    }

    fn cause(&self) -> Option<&dyn StdError> {
        match *self {
            PropertyDefinitionBuildError::MissingField(_) => None,
            PropertyDefinitionBuildError::EmptyVec(_) => None,
        }
    }
}

impl std::fmt::Display for PropertyDefinitionBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            PropertyDefinitionBuildError::MissingField(ref s) => write!(f, "MissingField: {}", s),
            PropertyDefinitionBuildError::EmptyVec(ref s) => write!(f, "EmptyVec: {}", s),
        }
    }
}

/// Builder used to create a PropertyDefinition
#[derive(Default, Clone, PartialEq)]
pub struct PropertyDefinitionBuilder {
    pub name: Option<String>,
    pub data_type: Option<DataType>,
    pub required: Option<bool>,
    pub description: Option<String>,
    pub number_exponent: Option<i32>,
    pub enum_options: Vec<String>,
    pub struct_properties: Vec<PropertyDefinition>,
}

impl PropertyDefinitionBuilder {
    pub fn new() -> Self {
        PropertyDefinitionBuilder::default()
    }

    pub fn with_name(mut self, name: String) -> PropertyDefinitionBuilder {
        self.name = Some(name);
        self
    }

    pub fn with_data_type(mut self, data_type: DataType) -> PropertyDefinitionBuilder {
        self.data_type = Some(data_type);
        self
    }

    pub fn with_required(mut self, required: bool) -> PropertyDefinitionBuilder {
        self.required = Some(required);
        self
    }

    pub fn with_description(mut self, description: String) -> PropertyDefinitionBuilder {
        self.description = Some(description);
        self
    }

    pub fn with_number_exponent(mut self, number_exponent: i32) -> PropertyDefinitionBuilder {
        self.number_exponent = Some(number_exponent);
        self
    }

    pub fn with_enum_options(mut self, enum_options: Vec<String>) -> PropertyDefinitionBuilder {
        self.enum_options = enum_options;
        self
    }

    pub fn with_struct_properties(
        mut self,
        struct_properties: Vec<PropertyDefinition>,
    ) -> PropertyDefinitionBuilder {
        self.struct_properties = struct_properties;
        self
    }

    pub fn build(self) -> Result<PropertyDefinition, PropertyDefinitionBuildError> {
        let name = self.name.ok_or_else(|| {
            PropertyDefinitionBuildError::MissingField("'name' field is required".to_string())
        })?;

        let data_type = self.data_type.ok_or_else(|| {
            PropertyDefinitionBuildError::MissingField("'data_type' field is required".to_string())
        })?;

        let required = self.required.unwrap_or(false);
        let description = self.description.unwrap_or_default();

        let number_exponent = {
            if data_type == DataType::Number {
                self.number_exponent.ok_or_else(|| {
                    PropertyDefinitionBuildError::MissingField(
                        "'number_exponent' field is required".to_string(),
                    )
                })?
            } else {
                0
            }
        };

        let enum_options = {
            if data_type == DataType::Enum {
                if !self.enum_options.is_empty() {
                    self.enum_options
                } else {
                    return Err(PropertyDefinitionBuildError::EmptyVec(
                        "'enum_options' cannot be empty".to_string(),
                    ));
                }
            } else {
                self.enum_options
            }
        };

        let struct_properties = {
            if data_type == DataType::Struct {
                if !self.struct_properties.is_empty() {
                    self.struct_properties
                } else {
                    return Err(PropertyDefinitionBuildError::EmptyVec(
                        "'struct_properties' cannot be empty".to_string(),
                    ));
                }
            } else {
                self.struct_properties
            }
        };

        Ok(PropertyDefinition {
            name,
            data_type,
            required,
            description,
            number_exponent,
            enum_options,
            struct_properties,
        })
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct PropertyValue {
    name: String,
    data_type: DataType,
    bytes_value: Vec<u8>,
    boolean_value: bool,
    number_value: i64,
    string_value: String,
    enum_value: u32,
    struct_values: Vec<PropertyValue>,
    lat_long_value: LatLong,
}

impl PropertyValue {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn data_type(&self) -> &DataType {
        &self.data_type
    }

    pub fn bytes_value(&self) -> &[u8] {
        &self.bytes_value
    }

    pub fn boolean_value(&self) -> &bool {
        &self.boolean_value
    }

    pub fn number_value(&self) -> &i64 {
        &self.number_value
    }

    pub fn string_value(&self) -> &str {
        &self.string_value
    }

    pub fn enum_value(&self) -> &u32 {
        &self.enum_value
    }

    pub fn struct_values(&self) -> &[PropertyValue] {
        &self.struct_values
    }

    pub fn lat_long_value(&self) -> &LatLong {
        &self.lat_long_value
    }
}

impl FromProto<protos::schema_state::PropertyValue> for PropertyValue {
    fn from_proto(
        property_value: protos::schema_state::PropertyValue,
    ) -> Result<Self, ProtoConversionError> {
        Ok(PropertyValue {
            name: property_value.get_name().to_string(),
            data_type: DataType::from_proto(property_value.get_data_type())?,
            bytes_value: property_value.get_bytes_value().to_vec(),
            boolean_value: property_value.get_boolean_value(),
            number_value: property_value.get_number_value(),
            string_value: property_value.get_string_value().to_string(),
            enum_value: property_value.get_enum_value(),
            struct_values: property_value
                .get_struct_values()
                .to_vec()
                .into_iter()
                .map(PropertyValue::from_proto)
                .collect::<Result<Vec<PropertyValue>, ProtoConversionError>>()?,
            lat_long_value: property_value.get_lat_long_value().clone().into_native()?,
        })
    }
}

impl FromNative<PropertyValue> for protos::schema_state::PropertyValue {
    fn from_native(property_value: PropertyValue) -> Result<Self, ProtoConversionError> {
        let mut proto_property_value = protos::schema_state::PropertyValue::new();
        proto_property_value.set_name(property_value.name().to_string());
        proto_property_value.set_data_type(property_value.data_type().clone().into_proto()?);
        proto_property_value.set_bytes_value(property_value.bytes_value().to_vec());
        proto_property_value.set_boolean_value(*property_value.boolean_value());
        proto_property_value.set_number_value(*property_value.number_value());
        proto_property_value.set_string_value(property_value.string_value().to_string());
        proto_property_value.set_enum_value(*property_value.enum_value());
        proto_property_value.set_struct_values(RepeatedField::from_vec(
            property_value
                .struct_values()
                .to_vec()
                .into_iter()
                .map(PropertyValue::into_proto)
                .collect::<Result<Vec<protos::schema_state::PropertyValue>, ProtoConversionError>>(
                )?,
        ));
        proto_property_value
            .set_lat_long_value(property_value.lat_long_value().clone().into_proto()?);
        Ok(proto_property_value)
    }
}

impl FromBytes<PropertyValue> for PropertyValue {
    fn from_bytes(bytes: &[u8]) -> Result<PropertyValue, ProtoConversionError> {
        let proto: protos::schema_state::PropertyValue =
            Message::parse_from_bytes(bytes).map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get PropertyValue from bytes".to_string(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for PropertyValue {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from PropertyValue".to_string(),
            )
        })?;
        Ok(bytes)
    }
}

#[derive(Debug)]
pub enum PropertyValueBuildError {
    MissingField(String),
}

impl StdError for PropertyValueBuildError {
    fn description(&self) -> &str {
        match *self {
            PropertyValueBuildError::MissingField(ref msg) => msg,
        }
    }

    fn cause(&self) -> Option<&dyn StdError> {
        match *self {
            PropertyValueBuildError::MissingField(_) => None,
        }
    }
}

impl std::fmt::Display for PropertyValueBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            PropertyValueBuildError::MissingField(ref s) => write!(f, "MissingField: {}", s),
        }
    }
}

/// Builder used to create a PropertyValue
#[derive(Default, Clone)]
pub struct PropertyValueBuilder {
    pub name: Option<String>,
    pub data_type: Option<DataType>,
    pub bytes_value: Option<Vec<u8>>,
    pub boolean_value: Option<bool>,
    pub number_value: Option<i64>,
    pub string_value: Option<String>,
    pub enum_value: Option<u32>,
    pub struct_values: Vec<PropertyValue>,
    pub lat_long_value: Option<LatLong>,
}

impl PropertyValueBuilder {
    pub fn new() -> Self {
        PropertyValueBuilder::default()
    }

    pub fn with_name(mut self, name: String) -> PropertyValueBuilder {
        self.name = Some(name);
        self
    }

    pub fn with_data_type(mut self, data_type: DataType) -> PropertyValueBuilder {
        self.data_type = Some(data_type);
        self
    }

    pub fn with_bytes_value(mut self, bytes: Vec<u8>) -> PropertyValueBuilder {
        self.bytes_value = Some(bytes);
        self
    }

    pub fn with_boolean_value(mut self, boolean: bool) -> PropertyValueBuilder {
        self.boolean_value = Some(boolean);
        self
    }

    pub fn with_number_value(mut self, number: i64) -> PropertyValueBuilder {
        self.number_value = Some(number);
        self
    }

    pub fn with_enum_value(mut self, enum_value: u32) -> PropertyValueBuilder {
        self.enum_value = Some(enum_value);
        self
    }

    pub fn with_string_value(mut self, string: String) -> PropertyValueBuilder {
        self.string_value = Some(string);
        self
    }

    pub fn with_struct_values(mut self, struct_values: Vec<PropertyValue>) -> PropertyValueBuilder {
        self.struct_values = struct_values;
        self
    }

    pub fn with_lat_long_value(mut self, lat_long_value: LatLong) -> PropertyValueBuilder {
        self.lat_long_value = Some(lat_long_value);
        self
    }

    pub fn build(self) -> Result<PropertyValue, PropertyValueBuildError> {
        let name = self.name.ok_or_else(|| {
            PropertyValueBuildError::MissingField("'name' field is required".to_string())
        })?;

        let data_type = self.data_type.ok_or_else(|| {
            PropertyValueBuildError::MissingField("'data_type' field is required".to_string())
        })?;

        let bytes_value = {
            if data_type == DataType::Bytes {
                self.bytes_value.ok_or_else(|| {
                    PropertyValueBuildError::MissingField(
                        "'bytes_value' field is required".to_string(),
                    )
                })?
            } else {
                vec![]
            }
        };

        let boolean_value = {
            if data_type == DataType::Boolean {
                self.boolean_value.ok_or_else(|| {
                    PropertyValueBuildError::MissingField(
                        "'boolean_value' field is required".to_string(),
                    )
                })?
            } else {
                false
            }
        };

        let number_value = {
            if data_type == DataType::Number {
                self.number_value.ok_or_else(|| {
                    PropertyValueBuildError::MissingField(
                        "'number_value' field is required".to_string(),
                    )
                })?
            } else {
                0
            }
        };

        let string_value = {
            if data_type == DataType::String {
                self.string_value.ok_or_else(|| {
                    PropertyValueBuildError::MissingField(
                        "'string_value' field is required".to_string(),
                    )
                })?
            } else {
                "".to_string()
            }
        };

        let enum_value = {
            if data_type == DataType::Enum {
                self.enum_value.ok_or_else(|| {
                    PropertyValueBuildError::MissingField(
                        "'enum_value' field is required".to_string(),
                    )
                })?
            } else {
                0
            }
        };

        let struct_values = {
            if data_type == DataType::Struct {
                if !self.struct_values.is_empty() {
                    self.struct_values
                } else {
                    return Err(PropertyValueBuildError::MissingField(
                        "'struct_values' cannot be empty".to_string(),
                    ));
                }
            } else {
                self.struct_values
            }
        };

        let lat_long_value = {
            if data_type == DataType::LatLong {
                self.lat_long_value.ok_or_else(|| {
                    PropertyValueBuildError::MissingField(
                        "'lat_long_value' field is required".to_string(),
                    )
                })?
            } else {
                LatLong {
                    latitude: 0,
                    longitude: 0,
                }
            }
        };

        Ok(PropertyValue {
            name,
            data_type,
            bytes_value,
            boolean_value,
            number_value,
            string_value,
            enum_value,
            struct_values,
            lat_long_value,
        })
    }
}

impl IntoProto<protos::schema_state::PropertyValue> for PropertyValue {}
impl IntoNative<PropertyValue> for protos::schema_state::PropertyValue {}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub enum DataType {
    Bytes,
    Boolean,
    Number,
    String,
    Enum,
    Struct,
    LatLong,
}

impl FromProto<protos::schema_state::PropertyDefinition_DataType> for DataType {
    fn from_proto(
        data_type: protos::schema_state::PropertyDefinition_DataType,
    ) -> Result<Self, ProtoConversionError> {
        match data_type {
            protos::schema_state::PropertyDefinition_DataType::BYTES => Ok(DataType::Bytes),
            protos::schema_state::PropertyDefinition_DataType::BOOLEAN => Ok(DataType::Boolean),
            protos::schema_state::PropertyDefinition_DataType::NUMBER => Ok(DataType::Number),
            protos::schema_state::PropertyDefinition_DataType::STRING => Ok(DataType::String),
            protos::schema_state::PropertyDefinition_DataType::ENUM => Ok(DataType::Enum),
            protos::schema_state::PropertyDefinition_DataType::STRUCT => Ok(DataType::Struct),
            protos::schema_state::PropertyDefinition_DataType::LAT_LONG => Ok(DataType::LatLong),
            protos::schema_state::PropertyDefinition_DataType::UNSET_DATA_TYPE => {
                Err(ProtoConversionError::InvalidTypeError(
                    "Cannot convert PropertyDefinition_DataType with type unset.".to_string(),
                ))
            }
        }
    }
}

impl FromNative<DataType> for protos::schema_state::PropertyDefinition_DataType {
    fn from_native(data_type: DataType) -> Result<Self, ProtoConversionError> {
        match data_type {
            DataType::Bytes => Ok(protos::schema_state::PropertyDefinition_DataType::BYTES),
            DataType::Boolean => Ok(protos::schema_state::PropertyDefinition_DataType::BOOLEAN),
            DataType::Number => Ok(protos::schema_state::PropertyDefinition_DataType::NUMBER),
            DataType::String => Ok(protos::schema_state::PropertyDefinition_DataType::STRING),
            DataType::Enum => Ok(protos::schema_state::PropertyDefinition_DataType::ENUM),
            DataType::Struct => Ok(protos::schema_state::PropertyDefinition_DataType::STRUCT),
            DataType::LatLong => Ok(protos::schema_state::PropertyDefinition_DataType::LAT_LONG),
        }
    }
}

impl IntoProto<protos::schema_state::PropertyDefinition_DataType> for DataType {}
impl IntoNative<DataType> for protos::schema_state::PropertyDefinition_DataType {}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct LatLong {
    latitude: i64,
    longitude: i64,
}

impl LatLong {
    pub fn latitude(&self) -> &i64 {
        &self.latitude
    }

    pub fn longitude(&self) -> &i64 {
        &self.longitude
    }
}

impl FromProto<protos::schema_state::LatLong> for LatLong {
    fn from_proto(lat_long: protos::schema_state::LatLong) -> Result<Self, ProtoConversionError> {
        Ok(LatLong {
            latitude: lat_long.get_latitude(),
            longitude: lat_long.get_longitude(),
        })
    }
}

impl FromNative<LatLong> for protos::schema_state::LatLong {
    fn from_native(lat_long: LatLong) -> Result<Self, ProtoConversionError> {
        let mut proto_lat_long = protos::schema_state::LatLong::new();
        proto_lat_long.set_latitude(*lat_long.latitude());
        proto_lat_long.set_longitude(*lat_long.longitude());
        Ok(proto_lat_long)
    }
}

impl IntoProto<protos::schema_state::LatLong> for LatLong {}
impl IntoNative<LatLong> for protos::schema_state::LatLong {}

#[derive(Debug)]
pub enum LatLongBuildError {
    InvalidLatitude(i64),
    InvalidLongitude(i64),
}

impl StdError for LatLongBuildError {}

impl std::fmt::Display for LatLongBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            LatLongBuildError::InvalidLatitude(ref s) => write!(
                f,
                "Invalid latitude - must in the range of \
                 -90000000 < lat < 90000000, but received: {}",
                s
            ),
            LatLongBuildError::InvalidLongitude(ref s) => write!(
                f,
                "Invalid longitude - must in the range of \
                 -180000000 < lat < 180000000, but received: {}",
                s
            ),
        }
    }
}

#[derive(Default, Clone, PartialEq)]
pub struct LatLongBuilder {
    pub latitude: i64,
    pub longitude: i64,
}

impl LatLongBuilder {
    pub fn new() -> Self {
        LatLongBuilder::default()
    }

    pub fn with_lat_long(mut self, latitude: i64, longitude: i64) -> LatLongBuilder {
        self.latitude = latitude;
        self.longitude = longitude;
        self
    }

    pub fn build(self) -> Result<LatLong, LatLongBuildError> {
        let latitude = self.latitude;
        let longitude = self.longitude;

        if latitude < -90_000_000 || latitude > 90_000_000 {
            Err(LatLongBuildError::InvalidLatitude(latitude))
        } else if longitude < -180_000_000 || longitude > 180_000_000 {
            Err(LatLongBuildError::InvalidLongitude(longitude))
        } else {
            Ok(LatLong {
                latitude,
                longitude,
            })
        }
    }
}

#[derive(Debug)]
pub enum BuilderError {
    MissingField(String),
}

impl std::fmt::Display for BuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            BuilderError::MissingField(ref s) => write!(f, "MissingField: {}", s),
        }
    }
}
