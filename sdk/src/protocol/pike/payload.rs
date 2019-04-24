// Copyright 2019 Cargill Incorporated
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

use crate::protocol::pike::state::KeyValueEntry;
use crate::protos;
use crate::protos::{
    FromBytes, FromNative, FromProto, IntoBytes, IntoNative, IntoProto, ProtoConversionError,
};

/// Native implementation for PikePayload_Action
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    CreateAgent,
    UpdateAgent,
    CreateOrganization,
    UpdateOrganization,
}

impl FromProto<protos::pike_payload::PikePayload_Action> for Action {
    fn from_proto(
        actions: protos::pike_payload::PikePayload_Action,
    ) -> Result<Self, ProtoConversionError> {
        match actions {
            protos::pike_payload::PikePayload_Action::CREATE_AGENT => Ok(Action::CreateAgent),
            protos::pike_payload::PikePayload_Action::UPDATE_AGENT => Ok(Action::UpdateAgent),
            protos::pike_payload::PikePayload_Action::CREATE_ORGANIZATION => {
                Ok(Action::CreateOrganization)
            }
            protos::pike_payload::PikePayload_Action::UPDATE_ORGANIZATION => {
                Ok(Action::UpdateOrganization)
            }
            protos::pike_payload::PikePayload_Action::ACTION_UNSET => {
                Err(ProtoConversionError::InvalidTypeError(
                    "Cannot convert PikePayload_Action with type unset.".to_string(),
                ))
            }
        }
    }
}

impl FromNative<Action> for protos::pike_payload::PikePayload_Action {
    fn from_native(action: Action) -> Result<Self, ProtoConversionError> {
        match action {
            Action::CreateAgent => Ok(protos::pike_payload::PikePayload_Action::CREATE_AGENT),
            Action::UpdateAgent => Ok(protos::pike_payload::PikePayload_Action::UPDATE_AGENT),
            Action::CreateOrganization => {
                Ok(protos::pike_payload::PikePayload_Action::CREATE_ORGANIZATION)
            }
            Action::UpdateOrganization => {
                Ok(protos::pike_payload::PikePayload_Action::UPDATE_ORGANIZATION)
            }
        }
    }
}

impl IntoProto<protos::pike_payload::PikePayload_Action> for Action {}
impl IntoNative<Action> for protos::pike_payload::PikePayload_Action {}

/// Native implementation for CreateAgentAction
#[derive(Debug, Default, Clone, PartialEq)]
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
        proto_create_agent.set_active(create_agent.active().clone());
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
        let proto: protos::pike_payload::CreateAgentAction = protobuf::parse_from_bytes(bytes)
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

#[derive(Debug)]
pub enum CreateAgentActionBuildError {
    MissingField(String),
}

impl StdError for CreateAgentActionBuildError {
    fn description(&self) -> &str {
        match *self {
            CreateAgentActionBuildError::MissingField(ref msg) => msg,
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            CreateAgentActionBuildError::MissingField(_) => None,
        }
    }
}

impl std::fmt::Display for CreateAgentActionBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            CreateAgentActionBuildError::MissingField(ref s) => write!(f, "MissingField: {}", s),
        }
    }
}

/// Builder used to create a CreateAgentAction
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

    pub fn build(self) -> Result<CreateAgentAction, CreateAgentActionBuildError> {
        let org_id = self.org_id.ok_or_else(|| {
            CreateAgentActionBuildError::MissingField("'org_id' field is required".to_string())
        })?;

        let public_key = self.public_key.ok_or_else(|| {
            CreateAgentActionBuildError::MissingField("'public_key' field is required".to_string())
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

/// Native implementation for UpdateAgentAction
#[derive(Debug, Default, Clone, PartialEq)]
pub struct UpdateAgentAction {
    org_id: String,
    public_key: String,
    active: bool,
    roles: Vec<String>,
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
        proto_update_agent.set_active(update_agent.active().clone());
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
        let proto: protos::pike_payload::UpdateAgentAction = protobuf::parse_from_bytes(bytes)
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

#[derive(Debug)]
pub enum UpdateAgentActionBuildError {
    MissingField(String),
}

impl StdError for UpdateAgentActionBuildError {
    fn description(&self) -> &str {
        match *self {
            UpdateAgentActionBuildError::MissingField(ref msg) => msg,
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            UpdateAgentActionBuildError::MissingField(_) => None,
        }
    }
}

impl std::fmt::Display for UpdateAgentActionBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            UpdateAgentActionBuildError::MissingField(ref s) => write!(f, "MissingField: {}", s),
        }
    }
}

/// Builder used to create a UpdateAgentAction
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

    pub fn build(self) -> Result<UpdateAgentAction, UpdateAgentActionBuildError> {
        let org_id = self.org_id.ok_or_else(|| {
            UpdateAgentActionBuildError::MissingField("'org_id' field is required".to_string())
        })?;

        let public_key = self.public_key.ok_or_else(|| {
            UpdateAgentActionBuildError::MissingField("'public_key' field is required".to_string())
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

/// Native implementation for CreageOrganizationAction
#[derive(Debug, Default, Clone, PartialEq)]
pub struct CreateOrganizationAction {
    org_id: String,
    name: String,
    address: String,
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
            protobuf::parse_from_bytes(bytes).map_err(|_| {
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

#[derive(Debug)]
pub enum CreateOrganizationActionBuildError {
    MissingField(String),
}

impl StdError for CreateOrganizationActionBuildError {
    fn description(&self) -> &str {
        match *self {
            CreateOrganizationActionBuildError::MissingField(ref msg) => msg,
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            CreateOrganizationActionBuildError::MissingField(_) => None,
        }
    }
}

impl std::fmt::Display for CreateOrganizationActionBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            CreateOrganizationActionBuildError::MissingField(ref s) => {
                write!(f, "MissingField: {}", s)
            }
        }
    }
}

/// Builder used to create a CreateOrganizationAction
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

    pub fn build(self) -> Result<CreateOrganizationAction, CreateOrganizationActionBuildError> {
        let org_id = self.org_id.ok_or_else(|| {
            CreateOrganizationActionBuildError::MissingField(
                "'org_id' field is required".to_string(),
            )
        })?;

        let name = self.name.ok_or_else(|| {
            CreateOrganizationActionBuildError::MissingField("'name' field is required".to_string())
        })?;

        let address = self.address.ok_or_else(|| {
            CreateOrganizationActionBuildError::MissingField(
                "'address' field is required".to_string(),
            )
        })?;

        let metadata = self.metadata;

        Ok(CreateOrganizationAction {
            org_id,
            name,
            address,
            metadata,
        })
    }
}

/// Native implementation for UpdateOrganizationAction
#[derive(Debug, Default, Clone, PartialEq)]
pub struct UpdateOrganizationAction {
    org_id: String,
    name: String,
    address: String,
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
            protobuf::parse_from_bytes(bytes).map_err(|_| {
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

#[derive(Debug)]
pub enum UpdateOrganizationActionBuildError {
    MissingField(String),
}

impl StdError for UpdateOrganizationActionBuildError {
    fn description(&self) -> &str {
        match *self {
            UpdateOrganizationActionBuildError::MissingField(ref msg) => msg,
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            UpdateOrganizationActionBuildError::MissingField(_) => None,
        }
    }
}

impl std::fmt::Display for UpdateOrganizationActionBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            UpdateOrganizationActionBuildError::MissingField(ref s) => {
                write!(f, "MissingField: {}", s)
            }
        }
    }
}

/// Builder used to create a UpdateOrganizationAction
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

    pub fn build(self) -> Result<UpdateOrganizationAction, UpdateOrganizationActionBuildError> {
        let org_id = self.org_id.ok_or_else(|| {
            UpdateOrganizationActionBuildError::MissingField(
                "'org_id' field is required".to_string(),
            )
        })?;

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

/// Native implementation for PikePayload
#[derive(Debug, Clone, PartialEq)]
pub struct PikePayload {
    action: Action,
    create_agent: CreateAgentAction,
    update_agent: UpdateAgentAction,
    create_organization: CreateOrganizationAction,
    update_organization: UpdateOrganizationAction,
}

impl PikePayload {
    pub fn action(&self) -> &Action {
        &self.action
    }

    pub fn create_agent(&self) -> &CreateAgentAction {
        &self.create_agent
    }

    pub fn update_agent(&self) -> &UpdateAgentAction {
        &self.update_agent
    }

    pub fn create_organization(&self) -> &CreateOrganizationAction {
        &self.create_organization
    }

    pub fn update_organization(&self) -> &UpdateOrganizationAction {
        &self.update_organization
    }
}

impl FromProto<protos::pike_payload::PikePayload> for PikePayload {
    fn from_proto(
        payload: protos::pike_payload::PikePayload,
    ) -> Result<Self, ProtoConversionError> {
        Ok(PikePayload {
            action: Action::from_proto(payload.get_action())?,
            create_agent: CreateAgentAction::from_proto(payload.get_create_agent().clone())?,
            update_agent: UpdateAgentAction::from_proto(payload.get_update_agent().clone())?,
            create_organization: CreateOrganizationAction::from_proto(
                payload.get_create_organization().clone(),
            )?,
            update_organization: UpdateOrganizationAction::from_proto(
                payload.get_update_organization().clone(),
            )?,
        })
    }
}

impl FromNative<PikePayload> for protos::pike_payload::PikePayload {
    fn from_native(payload: PikePayload) -> Result<Self, ProtoConversionError> {
        let mut proto_payload = protos::pike_payload::PikePayload::new();

        proto_payload.set_action(payload.action().clone().into_proto()?);
        proto_payload.set_create_agent(payload.create_agent().clone().into_proto()?);
        proto_payload.set_update_agent(payload.update_agent().clone().into_proto()?);
        proto_payload.set_create_organization(payload.create_organization().clone().into_proto()?);
        proto_payload.set_update_organization(payload.update_organization().clone().into_proto()?);

        Ok(proto_payload)
    }
}

impl FromBytes<PikePayload> for PikePayload {
    fn from_bytes(bytes: &[u8]) -> Result<PikePayload, ProtoConversionError> {
        let proto: protos::pike_payload::PikePayload =
            protobuf::parse_from_bytes(bytes).map_err(|_| {
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

#[derive(Debug)]
pub enum PikePayloadBuildError {
    MissingField(String),
}

impl StdError for PikePayloadBuildError {
    fn description(&self) -> &str {
        match *self {
            PikePayloadBuildError::MissingField(ref msg) => msg,
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            PikePayloadBuildError::MissingField(_) => None,
        }
    }
}

impl std::fmt::Display for PikePayloadBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            PikePayloadBuildError::MissingField(ref s) => write!(f, "MissingField: {}", s),
        }
    }
}

/// Builder used to create a PikePayload
#[derive(Default, Clone)]
pub struct PikePayloadBuilder {
    pub action: Option<Action>,
    pub create_agent: Option<CreateAgentAction>,
    pub update_agent: Option<UpdateAgentAction>,
    pub create_organization: Option<CreateOrganizationAction>,
    pub update_organization: Option<UpdateOrganizationAction>,
}

impl PikePayloadBuilder {
    pub fn new() -> Self {
        PikePayloadBuilder::default()
    }

    pub fn with_action(mut self, action: Action) -> PikePayloadBuilder {
        self.action = Some(action);
        self
    }

    pub fn with_create_agent(mut self, create_agent: CreateAgentAction) -> PikePayloadBuilder {
        self.create_agent = Some(create_agent);
        self
    }

    pub fn with_update_agent(mut self, update_agent: UpdateAgentAction) -> PikePayloadBuilder {
        self.update_agent = Some(update_agent);
        self
    }

    pub fn with_create_organization(
        mut self,
        create_organization: CreateOrganizationAction,
    ) -> PikePayloadBuilder {
        self.create_organization = Some(create_organization);
        self
    }

    pub fn with_update_organization(
        mut self,
        update_organization: UpdateOrganizationAction,
    ) -> PikePayloadBuilder {
        self.update_organization = Some(update_organization);
        self
    }

    pub fn build(self) -> Result<PikePayload, PikePayloadBuildError> {
        let action = self.action.ok_or_else(|| {
            PikePayloadBuildError::MissingField("'action' field is required".to_string())
        })?;

        let create_agent = {
            if action == Action::CreateAgent {
                self.create_agent.ok_or_else(|| {
                    PikePayloadBuildError::MissingField(
                        "'create_agent' field is required".to_string(),
                    )
                })?
            } else {
                CreateAgentAction::default()
            }
        };

        let update_agent = {
            if action == Action::UpdateAgent {
                self.update_agent.ok_or_else(|| {
                    PikePayloadBuildError::MissingField(
                        "'update_agent' field is required".to_string(),
                    )
                })?
            } else {
                UpdateAgentAction::default()
            }
        };

        let create_organization = {
            if action == Action::CreateOrganization {
                self.create_organization.ok_or_else(|| {
                    PikePayloadBuildError::MissingField(
                        "'create_organization' field is required".to_string(),
                    )
                })?
            } else {
                CreateOrganizationAction::default()
            }
        };

        let update_organization = {
            if action == Action::UpdateOrganization {
                self.update_organization.ok_or_else(|| {
                    PikePayloadBuildError::MissingField(
                        "'update_organization' field is required".to_string(),
                    )
                })?
            } else {
                UpdateOrganizationAction::default()
            }
        };

        Ok(PikePayload {
            action,
            create_agent,
            update_agent,
            create_organization,
            update_organization,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::protocol::pike::state::KeyValueEntryBuilder;

    #[test]
    // check that a create_agent action is built correctly
    fn check_create_agent_action() {
        let builder = KeyValueEntryBuilder::new();
        let key_value = builder
            .with_key("Key".to_string())
            .with_value("Value".to_string())
            .build()
            .unwrap();

        let builder = CreateAgentActionBuilder::new();
        let create_agent = builder
            .with_org_id("organization".to_string())
            .with_public_key("public_key".to_string())
            .with_active(true)
            .with_roles(vec!["Role".to_string()])
            .with_metadata(vec![key_value.clone()])
            .build()
            .unwrap();

        assert_eq!(create_agent.org_id(), "organization");
        assert_eq!(create_agent.public_key(), "public_key");
        assert!(create_agent.active());
        assert_eq!(create_agent.roles(), ["Role".to_string()]);
        assert_eq!(create_agent.metadata(), [key_value]);
    }

    #[test]
    // check that a create_agent can be converted to bytes and back
    fn check_create_agent_bytes() {
        let builder = KeyValueEntryBuilder::new();
        let key_value = builder
            .with_key("Key".to_string())
            .with_value("Value".to_string())
            .build()
            .unwrap();

        let builder = CreateAgentActionBuilder::new();
        let original = builder
            .with_org_id("organization".to_string())
            .with_public_key("public_key".to_string())
            .with_active(true)
            .with_roles(vec!["Role".to_string()])
            .with_metadata(vec![key_value.clone()])
            .build()
            .unwrap();

        let bytes = original.clone().into_bytes().unwrap();
        let create_agent = CreateAgentAction::from_bytes(&bytes).unwrap();
        assert_eq!(create_agent, original);
    }

    #[test]
    // check that a update_agent action is built correctly
    fn check_update_agent_action() {
        let builder = KeyValueEntryBuilder::new();
        let key_value = builder
            .with_key("Key".to_string())
            .with_value("Value".to_string())
            .build()
            .unwrap();

        let builder = UpdateAgentActionBuilder::new();
        let update_agent = builder
            .with_org_id("organization".to_string())
            .with_public_key("public_key".to_string())
            .with_active(true)
            .with_roles(vec!["Role".to_string()])
            .with_metadata(vec![key_value.clone()])
            .build()
            .unwrap();

        assert_eq!(update_agent.org_id(), "organization");
        assert_eq!(update_agent.public_key(), "public_key");
        assert!(update_agent.active());
        assert_eq!(update_agent.roles(), ["Role".to_string()]);
        assert_eq!(update_agent.metadata(), [key_value]);
    }

    #[test]
    // check that a update_agent can be converted to bytes and back
    fn check_update_agent_bytes() {
        let builder = KeyValueEntryBuilder::new();
        let key_value = builder
            .with_key("Key".to_string())
            .with_value("Value".to_string())
            .build()
            .unwrap();

        let builder = UpdateAgentActionBuilder::new();
        let original = builder
            .with_org_id("organization".to_string())
            .with_public_key("public_key".to_string())
            .with_active(true)
            .with_roles(vec!["Role".to_string()])
            .with_metadata(vec![key_value.clone()])
            .build()
            .unwrap();

        let bytes = original.clone().into_bytes().unwrap();
        let update_agent = UpdateAgentAction::from_bytes(&bytes).unwrap();
        assert_eq!(update_agent, original);
    }

    #[test]
    // check that a create_organization is built correctly
    fn check_create_organization_builder() {
        let builder = KeyValueEntryBuilder::new();
        let key_value = builder
            .with_key("Key".to_string())
            .with_value("Value".to_string())
            .build()
            .unwrap();

        let builder = CreateOrganizationActionBuilder::new();
        let create_organization = builder
            .with_org_id("organization".to_string())
            .with_name("name".to_string())
            .with_address("address".to_string())
            .with_metadata(vec![key_value.clone()])
            .build()
            .unwrap();

        assert_eq!(create_organization.org_id(), "organization");
        assert_eq!(create_organization.name(), "name");
        assert_eq!(create_organization.address(), "address");
        assert_eq!(create_organization.metadata(), [key_value]);
    }

    #[test]
    // check that a create_organization can be converted to bytes and back
    fn check_create_organization_bytes() {
        let builder = KeyValueEntryBuilder::new();
        let key_value = builder
            .with_key("Key".to_string())
            .with_value("Value".to_string())
            .build()
            .unwrap();

        let builder = CreateOrganizationActionBuilder::new();
        let original = builder
            .with_org_id("organization".to_string())
            .with_name("name".to_string())
            .with_address("address".to_string())
            .with_metadata(vec![key_value.clone()])
            .build()
            .unwrap();

        let bytes = original.clone().into_bytes().unwrap();
        let org = CreateOrganizationAction::from_bytes(&bytes).unwrap();
        assert_eq!(org, original);
    }

    #[test]
    // check that a update_organization is built correctly
    fn check_update_organization_builder() {
        let builder = UpdateOrganizationActionBuilder::new();
        let update_organization = builder
            .with_org_id("organization".to_string())
            .with_name("name".to_string())
            .with_address("address".to_string())
            .build()
            .unwrap();

        assert_eq!(update_organization.org_id(), "organization");
        assert_eq!(update_organization.name(), "name");
        assert_eq!(update_organization.address(), "address");
    }

    #[test]
    // check that a update_organization can be converted to bytes and back
    fn check_update_organization_bytes() {
        let builder = UpdateOrganizationActionBuilder::new();
        let original = builder
            .with_org_id("organization".to_string())
            .with_name("name".to_string())
            .with_address("address".to_string())
            .build()
            .unwrap();

        let bytes = original.clone().into_bytes().unwrap();
        let org = UpdateOrganizationAction::from_bytes(&bytes).unwrap();
        assert_eq!(org, original);
    }

    #[test]
    // check that a pike payload with create_agent is built correctly
    fn check_pike_create_agent_payload() {
        let builder = KeyValueEntryBuilder::new();
        let key_value = builder
            .with_key("Key".to_string())
            .with_value("Value".to_string())
            .build()
            .unwrap();

        let builder = CreateAgentActionBuilder::new();
        let action = builder
            .with_org_id("organization".to_string())
            .with_public_key("public_key".to_string())
            .with_active(true)
            .with_roles(vec!["Role".to_string()])
            .with_metadata(vec![key_value.clone()])
            .build()
            .unwrap();

        let builder = PikePayloadBuilder::new();
        let payload = builder
            .with_action(Action::CreateAgent)
            .with_create_agent(action.clone())
            .build()
            .unwrap();

        assert_eq!(payload.action, Action::CreateAgent);
        assert_eq!(payload.create_agent, action);
        assert_eq!(payload.update_agent, UpdateAgentAction::default());
        assert_eq!(
            payload.create_organization,
            CreateOrganizationAction::default()
        );
        assert_eq!(
            payload.update_organization,
            UpdateOrganizationAction::default()
        );
    }

    #[test]
    // check that a pike payload with update_agent is built correctly
    fn check_pike_update_agent_payload() {
        let builder = KeyValueEntryBuilder::new();
        let key_value = builder
            .with_key("Key".to_string())
            .with_value("Value".to_string())
            .build()
            .unwrap();

        let builder = UpdateAgentActionBuilder::new();
        let action = builder
            .with_org_id("organization".to_string())
            .with_public_key("public_key".to_string())
            .with_active(true)
            .with_roles(vec!["Role".to_string()])
            .with_metadata(vec![key_value.clone()])
            .build()
            .unwrap();

        let builder = PikePayloadBuilder::new();
        let payload = builder
            .with_action(Action::UpdateAgent)
            .with_update_agent(action.clone())
            .build()
            .unwrap();

        assert_eq!(payload.action, Action::UpdateAgent);
        assert_eq!(payload.create_agent, CreateAgentAction::default());
        assert_eq!(payload.update_agent, action);
        assert_eq!(
            payload.create_organization,
            CreateOrganizationAction::default()
        );
        assert_eq!(
            payload.update_organization,
            UpdateOrganizationAction::default()
        );
    }

    #[test]
    // check that a pike payload with create_org is built correctly
    fn check_pike_create_organization_payload() {
        let builder = CreateOrganizationActionBuilder::new();
        let action = builder
            .with_org_id("organization".to_string())
            .with_name("name".to_string())
            .with_address("address".to_string())
            .build()
            .unwrap();

        let builder = PikePayloadBuilder::new();
        let payload = builder
            .with_action(Action::CreateOrganization)
            .with_create_organization(action.clone())
            .build()
            .unwrap();

        assert_eq!(payload.action, Action::CreateOrganization);
        assert_eq!(payload.create_agent, CreateAgentAction::default());
        assert_eq!(payload.update_agent, UpdateAgentAction::default());
        assert_eq!(payload.create_organization, action);
        assert_eq!(
            payload.update_organization,
            UpdateOrganizationAction::default()
        );
    }

    #[test]
    // check that a pike payload with update_org is built correctly
    fn check_pike_update_organiztion_payload() {
        let builder = UpdateOrganizationActionBuilder::new();
        let action = builder
            .with_org_id("organization".to_string())
            .with_name("name".to_string())
            .with_address("address".to_string())
            .build()
            .unwrap();

        let builder = PikePayloadBuilder::new();
        let payload = builder
            .with_action(Action::UpdateOrganization)
            .with_update_organization(action.clone())
            .build()
            .unwrap();

        assert_eq!(payload.action, Action::UpdateOrganization);
        assert_eq!(payload.create_agent, CreateAgentAction::default());
        assert_eq!(payload.update_agent, UpdateAgentAction::default());
        assert_eq!(
            payload.create_organization,
            CreateOrganizationAction::default()
        );
        assert_eq!(payload.update_organization, action);
    }

    #[test]
    // check that a pike payload can be converted to bytes and back
    fn check_pike_payload_bytes() {
        let builder = UpdateOrganizationActionBuilder::new();
        let action = builder
            .with_org_id("organization".to_string())
            .with_name("name".to_string())
            .with_address("address".to_string())
            .build()
            .unwrap();

        let builder = PikePayloadBuilder::new();
        let original = builder
            .with_action(Action::UpdateOrganization)
            .with_update_organization(action.clone())
            .build()
            .unwrap();

        let bytes = original.clone().into_bytes().unwrap();
        let payload = PikePayload::from_bytes(&bytes).unwrap();
        assert_eq!(payload, original);
    }
}
