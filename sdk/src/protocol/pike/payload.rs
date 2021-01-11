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

use crate::protocol::pike::state::{AlternateID, KeyValueEntry};
use crate::protos;
use crate::protos::{
    FromBytes, FromNative, FromProto, IntoBytes, IntoNative, IntoProto, ProtoConversionError,
};

/// Native implementation for PikePayload_Action
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    CreateAgent,
    UpdateAgent,
    DeleteAgent,
    CreateOrganization,
    UpdateOrganization,
    DeleteOrganization,
    CreateRole,
    UpdateRole,
    DeleteRole,
}

impl FromProto<protos::pike_payload::PikePayload_Action> for Action {
    fn from_proto(
        actions: protos::pike_payload::PikePayload_Action,
    ) -> Result<Self, ProtoConversionError> {
        match actions {
            protos::pike_payload::PikePayload_Action::CREATE_AGENT => Ok(Action::CreateAgent),
            protos::pike_payload::PikePayload_Action::UPDATE_AGENT => Ok(Action::UpdateAgent),
            protos::pike_payload::PikePayload_Action::DELETE_AGENT => Ok(Action::DeleteAgent),
            protos::pike_payload::PikePayload_Action::CREATE_ORGANIZATION => {
                Ok(Action::CreateOrganization)
            }
            protos::pike_payload::PikePayload_Action::UPDATE_ORGANIZATION => {
                Ok(Action::UpdateOrganization)
            }
            protos::pike_payload::PikePayload_Action::DELETE_ORGANIZATION => {
                Ok(Action::DeleteOrganization)
            }
            protos::pike_payload::PikePayload_Action::CREATE_ROLE => Ok(Action::CreateRole),
            protos::pike_payload::PikePayload_Action::UPDATE_ROLE => Ok(Action::UpdateRole),
            protos::pike_payload::PikePayload_Action::DELETE_ROLE => Ok(Action::DeleteRole),
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
            Action::DeleteAgent => Ok(protos::pike_payload::PikePayload_Action::DELETE_AGENT),
            Action::CreateOrganization => {
                Ok(protos::pike_payload::PikePayload_Action::CREATE_ORGANIZATION)
            }
            Action::UpdateOrganization => {
                Ok(protos::pike_payload::PikePayload_Action::UPDATE_ORGANIZATION)
            }
            Action::DeleteOrganization => {
                Ok(protos::pike_payload::PikePayload_Action::DELETE_ORGANIZATION)
            }
            Action::CreateRole => Ok(protos::pike_payload::PikePayload_Action::CREATE_ROLE),
            Action::UpdateRole => Ok(protos::pike_payload::PikePayload_Action::UPDATE_ROLE),
            Action::DeleteRole => Ok(protos::pike_payload::PikePayload_Action::DELETE_ROLE),
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

    fn cause(&self) -> Option<&dyn StdError> {
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

    fn cause(&self) -> Option<&dyn StdError> {
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

/// Native implementation for DeleteAgentAction
#[derive(Debug, Default, Clone, PartialEq)]
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

impl FromProto<protos::pike_payload::DeleteAgentAction> for DeleteAgentAction {
    fn from_proto(
        delete_agent: protos::pike_payload::DeleteAgentAction,
    ) -> Result<Self, ProtoConversionError> {
        Ok(DeleteAgentAction {
            org_id: delete_agent.get_org_id().to_string(),
            public_key: delete_agent.get_public_key().to_string(),
        })
    }
}

impl FromNative<DeleteAgentAction> for protos::pike_payload::DeleteAgentAction {
    fn from_native(delete_agent: DeleteAgentAction) -> Result<Self, ProtoConversionError> {
        let mut proto_delete_agent = protos::pike_payload::DeleteAgentAction::new();

        proto_delete_agent.set_org_id(delete_agent.org_id().to_string());
        proto_delete_agent.set_public_key(delete_agent.public_key().to_string());

        Ok(proto_delete_agent)
    }
}

impl FromBytes<DeleteAgentAction> for DeleteAgentAction {
    fn from_bytes(bytes: &[u8]) -> Result<DeleteAgentAction, ProtoConversionError> {
        let proto: protos::pike_payload::DeleteAgentAction = Message::parse_from_bytes(bytes)
            .map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get DeleteAgentAction from bytes".to_string(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for DeleteAgentAction {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from DeleteAgentAction".to_string(),
            )
        })?;
        Ok(bytes)
    }
}

impl IntoProto<protos::pike_payload::DeleteAgentAction> for DeleteAgentAction {}
impl IntoNative<DeleteAgentAction> for protos::pike_payload::DeleteAgentAction {}

/// Native implementation for CreateOrganizationAction
#[derive(Debug, Default, Clone, PartialEq)]
pub struct CreateOrganizationAction {
    org_id: String,
    name: String,
    locations: Vec<String>,
    alternate_ids: Vec<AlternateID>,
    metadata: Vec<KeyValueEntry>,
}

impl CreateOrganizationAction {
    pub fn org_id(&self) -> &str {
        &self.org_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn locations(&self) -> &[String] {
        &self.locations
    }

    pub fn alternate_ids(&self) -> &[AlternateID] {
        &self.alternate_ids
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
            locations: create_org.get_locations().to_vec(),
            alternate_ids: create_org
                .get_alternate_ids()
                .to_vec()
                .into_iter()
                .map(AlternateID::from_proto)
                .collect::<Result<Vec<AlternateID>, ProtoConversionError>>()?,
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
        proto_create_org.set_locations(RepeatedField::from_vec(create_org.locations().to_vec()));
        proto_create_org.set_alternate_ids(RepeatedField::from_vec(
            create_org
                .alternate_ids()
                .to_vec()
                .into_iter()
                .map(AlternateID::into_proto)
                .collect::<Result<Vec<protos::pike_state::AlternateID>, ProtoConversionError>>()?,
        ));
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

    fn cause(&self) -> Option<&dyn StdError> {
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
    pub locations: Vec<String>,
    pub alternate_ids: Vec<AlternateID>,
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

    pub fn with_locations(mut self, locations: Vec<String>) -> CreateOrganizationActionBuilder {
        self.locations = locations;
        self
    }

    pub fn with_alternate_ids(
        mut self,
        alternate_ids: Vec<AlternateID>,
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

    pub fn build(self) -> Result<CreateOrganizationAction, CreateOrganizationActionBuildError> {
        let org_id = self.org_id.ok_or_else(|| {
            CreateOrganizationActionBuildError::MissingField(
                "'org_id' field is required".to_string(),
            )
        })?;

        let name = self.name.ok_or_else(|| {
            CreateOrganizationActionBuildError::MissingField("'name' field is required".to_string())
        })?;

        let locations = self.locations;

        let alternate_ids = self.alternate_ids;

        let metadata = self.metadata;

        Ok(CreateOrganizationAction {
            org_id,
            name,
            locations,
            alternate_ids,
            metadata,
        })
    }
}

/// Native implementation for UpdateOrganizationAction
#[derive(Debug, Default, Clone, PartialEq)]
pub struct UpdateOrganizationAction {
    org_id: String,
    name: String,
    locations: Vec<String>,
    alternate_ids: Vec<AlternateID>,
    metadata: Vec<KeyValueEntry>,
}

impl UpdateOrganizationAction {
    pub fn org_id(&self) -> &str {
        &self.org_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn locations(&self) -> &[String] {
        &self.locations
    }

    pub fn alternate_ids(&self) -> &[AlternateID] {
        &self.alternate_ids
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
            locations: create_org.get_locations().to_vec(),
            alternate_ids: create_org
                .get_alternate_ids()
                .to_vec()
                .into_iter()
                .map(AlternateID::from_proto)
                .collect::<Result<Vec<AlternateID>, ProtoConversionError>>()?,
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
        proto_update_org.set_locations(RepeatedField::from_vec(update_org.locations().to_vec()));
        proto_update_org.set_alternate_ids(RepeatedField::from_vec(
            update_org
                .alternate_ids()
                .to_vec()
                .into_iter()
                .map(AlternateID::into_proto)
                .collect::<Result<Vec<protos::pike_state::AlternateID>, ProtoConversionError>>()?,
        ));
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

    fn cause(&self) -> Option<&dyn StdError> {
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
    pub locations: Vec<String>,
    pub alternate_ids: Vec<AlternateID>,
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

    pub fn with_locations(mut self, locations: Vec<String>) -> UpdateOrganizationActionBuilder {
        self.locations = locations;
        self
    }

    pub fn with_alternate_ids(
        mut self,
        alternate_ids: Vec<AlternateID>,
    ) -> UpdateOrganizationActionBuilder {
        self.alternate_ids = alternate_ids;
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

        let locations = self.locations;

        let alternate_ids = self.alternate_ids;

        let metadata = self.metadata;

        Ok(UpdateOrganizationAction {
            org_id,
            name,
            locations,
            alternate_ids,
            metadata,
        })
    }
}

/// Native implementation for DeleteOrganizationAction
#[derive(Debug, Default, Clone, PartialEq)]
pub struct DeleteOrganizationAction {
    id: String,
}

impl DeleteOrganizationAction {
    pub fn id(&self) -> &str {
        &self.id
    }
}

impl FromProto<protos::pike_payload::DeleteOrganizationAction> for DeleteOrganizationAction {
    fn from_proto(
        delete_organization: protos::pike_payload::DeleteOrganizationAction,
    ) -> Result<Self, ProtoConversionError> {
        Ok(DeleteOrganizationAction {
            id: delete_organization.get_id().to_string(),
        })
    }
}

impl FromNative<DeleteOrganizationAction> for protos::pike_payload::DeleteOrganizationAction {
    fn from_native(
        delete_organization: DeleteOrganizationAction,
    ) -> Result<Self, ProtoConversionError> {
        let mut proto_delete_organization = protos::pike_payload::DeleteOrganizationAction::new();

        proto_delete_organization.set_id(delete_organization.id().to_string());

        Ok(proto_delete_organization)
    }
}

impl FromBytes<DeleteOrganizationAction> for DeleteOrganizationAction {
    fn from_bytes(bytes: &[u8]) -> Result<DeleteOrganizationAction, ProtoConversionError> {
        let proto: protos::pike_payload::DeleteOrganizationAction =
            Message::parse_from_bytes(bytes).map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get DeleteOrganizationAction from bytes".to_string(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for DeleteOrganizationAction {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from DeleteOrganizationAction".to_string(),
            )
        })?;
        Ok(bytes)
    }
}

impl IntoProto<protos::pike_payload::DeleteOrganizationAction> for DeleteOrganizationAction {}
impl IntoNative<DeleteOrganizationAction> for protos::pike_payload::DeleteOrganizationAction {}

#[derive(Debug)]
pub enum DeleteOrganizationActionBuildError {
    MissingField(String),
}

impl StdError for DeleteOrganizationActionBuildError {
    fn description(&self) -> &str {
        match *self {
            DeleteOrganizationActionBuildError::MissingField(ref msg) => msg,
        }
    }

    fn cause(&self) -> Option<&dyn StdError> {
        match *self {
            DeleteOrganizationActionBuildError::MissingField(_) => None,
        }
    }
}

impl std::fmt::Display for DeleteOrganizationActionBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            DeleteOrganizationActionBuildError::MissingField(ref s) => {
                write!(f, "MissingField: {}", s)
            }
        }
    }
}

/// Builder used to create a DeleteOrganizationAction
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

    pub fn build(self) -> Result<DeleteOrganizationAction, DeleteOrganizationActionBuildError> {
        let id = self.id.ok_or_else(|| {
            DeleteOrganizationActionBuildError::MissingField("'id' field is required".to_string())
        })?;

        Ok(DeleteOrganizationAction { id })
    }
}

/// Native implementation for CreateRoleAction
#[derive(Debug, Default, Clone, PartialEq)]
pub struct CreateRoleAction {
    org_id: String,
    name: String,
    description: String,
    permissions: Vec<String>,
    allowed_organizations: Vec<String>,
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

impl FromProto<protos::pike_payload::CreateRoleAction> for CreateRoleAction {
    fn from_proto(
        create_role: protos::pike_payload::CreateRoleAction,
    ) -> Result<Self, ProtoConversionError> {
        Ok(CreateRoleAction {
            org_id: create_role.get_org_id().to_string(),
            name: create_role.get_name().to_string(),
            description: create_role.get_description().to_string(),
            permissions: create_role.get_permissions().to_vec(),
            allowed_organizations: create_role.get_allowed_organizations().to_vec(),
            inherit_from: create_role.get_inherit_from().to_vec(),
            active: create_role.get_active(),
        })
    }
}

impl FromNative<CreateRoleAction> for protos::pike_payload::CreateRoleAction {
    fn from_native(create_role: CreateRoleAction) -> Result<Self, ProtoConversionError> {
        let mut proto_create_role = protos::pike_payload::CreateRoleAction::new();

        proto_create_role.set_org_id(create_role.org_id().to_string());
        proto_create_role.set_name(create_role.name().to_string());
        proto_create_role.set_description(create_role.description().to_string());
        proto_create_role
            .set_permissions(RepeatedField::from_vec(create_role.permissions().to_vec()));
        proto_create_role.set_allowed_organizations(RepeatedField::from_vec(
            create_role.allowed_organizations().to_vec(),
        ));
        proto_create_role
            .set_inherit_from(RepeatedField::from_vec(create_role.inherit_from().to_vec()));
        proto_create_role.set_active(*create_role.active());

        Ok(proto_create_role)
    }
}

impl FromBytes<CreateRoleAction> for CreateRoleAction {
    fn from_bytes(bytes: &[u8]) -> Result<CreateRoleAction, ProtoConversionError> {
        let proto: protos::pike_payload::CreateRoleAction = Message::parse_from_bytes(bytes)
            .map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get CreateRoleAction from bytes".to_string(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for CreateRoleAction {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from CreateRoleAction".to_string(),
            )
        })?;
        Ok(bytes)
    }
}

impl IntoProto<protos::pike_payload::CreateRoleAction> for CreateRoleAction {}
impl IntoNative<CreateRoleAction> for protos::pike_payload::CreateRoleAction {}

#[derive(Debug)]
pub enum CreateRoleActionBuildError {
    MissingField(String),
}

impl StdError for CreateRoleActionBuildError {
    fn description(&self) -> &str {
        match *self {
            CreateRoleActionBuildError::MissingField(ref msg) => msg,
        }
    }

    fn cause(&self) -> Option<&dyn StdError> {
        match *self {
            CreateRoleActionBuildError::MissingField(_) => None,
        }
    }
}

impl std::fmt::Display for CreateRoleActionBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            CreateRoleActionBuildError::MissingField(ref s) => {
                write!(f, "MissingField: {}", s)
            }
        }
    }
}

/// Builder used to create a CreateRoleAction
#[derive(Default, Clone)]
pub struct CreateRoleActionBuilder {
    pub org_id: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub permissions: Vec<String>,
    pub allowed_organizations: Vec<String>,
    pub inherit_from: Vec<String>,
    pub active: Option<bool>,
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
        self.active = Some(active);
        self
    }

    pub fn build(self) -> Result<CreateRoleAction, CreateRoleActionBuildError> {
        let org_id = self.org_id.ok_or_else(|| {
            CreateRoleActionBuildError::MissingField("'org_id' field is required".to_string())
        })?;

        let name = self.name.ok_or_else(|| {
            CreateRoleActionBuildError::MissingField("'name' field is required".to_string())
        })?;

        let description = self.description.ok_or_else(|| {
            CreateRoleActionBuildError::MissingField("'description' field is required".to_string())
        })?;

        let permissions = self.permissions;

        let allowed_organizations = self.allowed_organizations;

        let inherit_from = self.inherit_from;

        let active = self.active.ok_or_else(|| {
            CreateRoleActionBuildError::MissingField("'active' field is required".to_string())
        })?;

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

/// Native implementation for UpdateRoleAction
#[derive(Debug, Default, Clone, PartialEq)]
pub struct UpdateRoleAction {
    org_id: String,
    name: String,
    description: String,
    permissions: Vec<String>,
    allowed_organizations: Vec<String>,
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

impl FromProto<protos::pike_payload::UpdateRoleAction> for UpdateRoleAction {
    fn from_proto(
        update_role: protos::pike_payload::UpdateRoleAction,
    ) -> Result<Self, ProtoConversionError> {
        Ok(UpdateRoleAction {
            org_id: update_role.get_org_id().to_string(),
            name: update_role.get_name().to_string(),
            description: update_role.get_description().to_string(),
            permissions: update_role.get_permissions().to_vec(),
            allowed_organizations: update_role.get_allowed_organizations().to_vec(),
            inherit_from: update_role.get_inherit_from().to_vec(),
            active: update_role.get_active(),
        })
    }
}

impl FromNative<UpdateRoleAction> for protos::pike_payload::UpdateRoleAction {
    fn from_native(update_role: UpdateRoleAction) -> Result<Self, ProtoConversionError> {
        let mut proto_update_role = protos::pike_payload::UpdateRoleAction::new();

        proto_update_role.set_org_id(update_role.org_id().to_string());
        proto_update_role.set_name(update_role.name().to_string());
        proto_update_role.set_description(update_role.description().to_string());
        proto_update_role
            .set_permissions(RepeatedField::from_vec(update_role.permissions().to_vec()));
        proto_update_role.set_allowed_organizations(RepeatedField::from_vec(
            update_role.allowed_organizations().to_vec(),
        ));
        proto_update_role
            .set_inherit_from(RepeatedField::from_vec(update_role.inherit_from().to_vec()));
        proto_update_role.set_active(*update_role.active());

        Ok(proto_update_role)
    }
}

impl FromBytes<UpdateRoleAction> for UpdateRoleAction {
    fn from_bytes(bytes: &[u8]) -> Result<UpdateRoleAction, ProtoConversionError> {
        let proto: protos::pike_payload::UpdateRoleAction = Message::parse_from_bytes(bytes)
            .map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get UpdateRoleAction from bytes".to_string(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for UpdateRoleAction {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from UpdateRoleAction".to_string(),
            )
        })?;
        Ok(bytes)
    }
}

impl IntoProto<protos::pike_payload::UpdateRoleAction> for UpdateRoleAction {}
impl IntoNative<UpdateRoleAction> for protos::pike_payload::UpdateRoleAction {}

#[derive(Debug)]
pub enum UpdateRoleActionBuildError {
    MissingField(String),
}

impl StdError for UpdateRoleActionBuildError {
    fn description(&self) -> &str {
        match *self {
            UpdateRoleActionBuildError::MissingField(ref msg) => msg,
        }
    }

    fn cause(&self) -> Option<&dyn StdError> {
        match *self {
            UpdateRoleActionBuildError::MissingField(_) => None,
        }
    }
}

impl std::fmt::Display for UpdateRoleActionBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            UpdateRoleActionBuildError::MissingField(ref s) => {
                write!(f, "MissingField: {}", s)
            }
        }
    }
}

/// Builder used to create a UpdateRoleAction
#[derive(Default, Clone)]
pub struct UpdateRoleActionBuilder {
    pub org_id: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub permissions: Vec<String>,
    pub allowed_organizations: Vec<String>,
    pub inherit_from: Vec<String>,
    pub active: Option<bool>,
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
        self.active = Some(active);
        self
    }

    pub fn build(self) -> Result<UpdateRoleAction, UpdateRoleActionBuildError> {
        let org_id = self.org_id.ok_or_else(|| {
            UpdateRoleActionBuildError::MissingField("'org_id' field is required".to_string())
        })?;

        let name = self.name.ok_or_else(|| {
            UpdateRoleActionBuildError::MissingField("'name' field is required".to_string())
        })?;

        let description = self.description.ok_or_else(|| {
            UpdateRoleActionBuildError::MissingField("'description' field is required".to_string())
        })?;

        let permissions = self.permissions;

        let allowed_organizations = self.allowed_organizations;

        let inherit_from = self.inherit_from;

        let active = self.active.ok_or_else(|| {
            UpdateRoleActionBuildError::MissingField("'active' field is required".to_string())
        })?;

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

/// Native implementation for DeleteRoleAction
#[derive(Debug, Default, Clone, PartialEq)]
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

impl FromProto<protos::pike_payload::DeleteRoleAction> for DeleteRoleAction {
    fn from_proto(
        delete_role: protos::pike_payload::DeleteRoleAction,
    ) -> Result<Self, ProtoConversionError> {
        Ok(DeleteRoleAction {
            org_id: delete_role.get_org_id().to_string(),
            name: delete_role.get_name().to_string(),
        })
    }
}

impl FromNative<DeleteRoleAction> for protos::pike_payload::DeleteRoleAction {
    fn from_native(delete_role: DeleteRoleAction) -> Result<Self, ProtoConversionError> {
        let mut proto_delete_role = protos::pike_payload::DeleteRoleAction::new();

        proto_delete_role.set_org_id(delete_role.org_id().to_string());
        proto_delete_role.set_name(delete_role.name().to_string());

        Ok(proto_delete_role)
    }
}

impl FromBytes<DeleteRoleAction> for DeleteRoleAction {
    fn from_bytes(bytes: &[u8]) -> Result<DeleteRoleAction, ProtoConversionError> {
        let proto: protos::pike_payload::DeleteRoleAction = Message::parse_from_bytes(bytes)
            .map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get DeleteRoleAction from bytes".to_string(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for DeleteRoleAction {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from DeleteRoleAction".to_string(),
            )
        })?;
        Ok(bytes)
    }
}

impl IntoProto<protos::pike_payload::DeleteRoleAction> for DeleteRoleAction {}
impl IntoNative<DeleteRoleAction> for protos::pike_payload::DeleteRoleAction {}

#[derive(Debug)]
pub enum DeleteRoleActionBuildError {
    MissingField(String),
}

impl StdError for DeleteRoleActionBuildError {
    fn description(&self) -> &str {
        match *self {
            DeleteRoleActionBuildError::MissingField(ref msg) => msg,
        }
    }

    fn cause(&self) -> Option<&dyn StdError> {
        match *self {
            DeleteRoleActionBuildError::MissingField(_) => None,
        }
    }
}

impl std::fmt::Display for DeleteRoleActionBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            DeleteRoleActionBuildError::MissingField(ref s) => {
                write!(f, "MissingField: {}", s)
            }
        }
    }
}

/// Builder used to create a DeleteRoleAction
#[derive(Default, Clone)]
pub struct DeleteRoleActionBuilder {
    pub org_id: Option<String>,
    pub name: Option<String>,
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

    pub fn build(self) -> Result<DeleteRoleAction, DeleteRoleActionBuildError> {
        let org_id = self.org_id.ok_or_else(|| {
            DeleteRoleActionBuildError::MissingField("'org_id' field is required".to_string())
        })?;

        let name = self.name.ok_or_else(|| {
            DeleteRoleActionBuildError::MissingField("'name' field is required".to_string())
        })?;

        Ok(DeleteRoleAction { org_id, name })
    }
}

/// Native implementation for PikePayload
#[derive(Debug, Clone, PartialEq)]
pub struct PikePayload {
    action: Action,
    create_agent: CreateAgentAction,
    update_agent: UpdateAgentAction,
    delete_agent: DeleteAgentAction,
    create_organization: CreateOrganizationAction,
    update_organization: UpdateOrganizationAction,
    delete_organization: DeleteOrganizationAction,
    create_role: CreateRoleAction,
    update_role: UpdateRoleAction,
    delete_role: DeleteRoleAction,
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

    pub fn delete_agent(&self) -> &DeleteAgentAction {
        &self.delete_agent
    }

    pub fn create_organization(&self) -> &CreateOrganizationAction {
        &self.create_organization
    }

    pub fn update_organization(&self) -> &UpdateOrganizationAction {
        &self.update_organization
    }

    pub fn delete_organization(&self) -> &DeleteOrganizationAction {
        &self.delete_organization
    }

    pub fn create_role(&self) -> &CreateRoleAction {
        &self.create_role
    }

    pub fn update_role(&self) -> &UpdateRoleAction {
        &self.update_role
    }

    pub fn delete_role(&self) -> &DeleteRoleAction {
        &self.delete_role
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
            delete_agent: DeleteAgentAction::from_proto(payload.get_delete_agent().clone())?,
            create_organization: CreateOrganizationAction::from_proto(
                payload.get_create_organization().clone(),
            )?,
            update_organization: UpdateOrganizationAction::from_proto(
                payload.get_update_organization().clone(),
            )?,
            delete_organization: DeleteOrganizationAction::from_proto(
                payload.get_delete_organization().clone(),
            )?,
            create_role: CreateRoleAction::from_proto(payload.get_create_role().clone())?,
            update_role: UpdateRoleAction::from_proto(payload.get_update_role().clone())?,
            delete_role: DeleteRoleAction::from_proto(payload.get_delete_role().clone())?,
        })
    }
}

impl FromNative<PikePayload> for protos::pike_payload::PikePayload {
    fn from_native(payload: PikePayload) -> Result<Self, ProtoConversionError> {
        let mut proto_payload = protos::pike_payload::PikePayload::new();

        proto_payload.set_action(payload.action().clone().into_proto()?);
        proto_payload.set_create_agent(payload.create_agent().clone().into_proto()?);
        proto_payload.set_update_agent(payload.update_agent().clone().into_proto()?);
        proto_payload.set_delete_agent(payload.delete_agent().clone().into_proto()?);
        proto_payload.set_create_organization(payload.create_organization().clone().into_proto()?);
        proto_payload.set_update_organization(payload.update_organization().clone().into_proto()?);
        proto_payload.set_delete_organization(payload.delete_organization().clone().into_proto()?);
        proto_payload.set_create_role(payload.create_role().clone().into_proto()?);
        proto_payload.set_update_role(payload.update_role().clone().into_proto()?);
        proto_payload.set_delete_role(payload.delete_role().clone().into_proto()?);

        Ok(proto_payload)
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

    fn cause(&self) -> Option<&dyn StdError> {
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
    pub delete_agent: Option<DeleteAgentAction>,
    pub create_organization: Option<CreateOrganizationAction>,
    pub update_organization: Option<UpdateOrganizationAction>,
    pub delete_organization: Option<DeleteOrganizationAction>,
    pub create_role: Option<CreateRoleAction>,
    pub update_role: Option<UpdateRoleAction>,
    pub delete_role: Option<DeleteRoleAction>,
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

    pub fn with_delete_agent(mut self, delete_agent: DeleteAgentAction) -> PikePayloadBuilder {
        self.delete_agent = Some(delete_agent);
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

    pub fn with_delete_organization(
        mut self,
        delete_organization: DeleteOrganizationAction,
    ) -> PikePayloadBuilder {
        self.delete_organization = Some(delete_organization);
        self
    }

    pub fn with_create_role(mut self, create_role: CreateRoleAction) -> PikePayloadBuilder {
        self.create_role = Some(create_role);
        self
    }

    pub fn with_update_role(mut self, update_role: UpdateRoleAction) -> PikePayloadBuilder {
        self.update_role = Some(update_role);
        self
    }

    pub fn with_delete_role(mut self, delete_role: DeleteRoleAction) -> PikePayloadBuilder {
        self.delete_role = Some(delete_role);
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

        let delete_agent = {
            if action == Action::DeleteAgent {
                self.delete_agent.ok_or_else(|| {
                    PikePayloadBuildError::MissingField(
                        "'delete_agent' field is required".to_string(),
                    )
                })?
            } else {
                DeleteAgentAction::default()
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

        let delete_organization = {
            if action == Action::DeleteOrganization {
                self.delete_organization.ok_or_else(|| {
                    PikePayloadBuildError::MissingField(
                        "'delete_organization' field is required".to_string(),
                    )
                })?
            } else {
                DeleteOrganizationAction::default()
            }
        };

        let create_role = {
            if action == Action::CreateRole {
                self.create_role.ok_or_else(|| {
                    PikePayloadBuildError::MissingField(
                        "'create_role' field is required".to_string(),
                    )
                })?
            } else {
                CreateRoleAction::default()
            }
        };

        let update_role = {
            if action == Action::UpdateRole {
                self.update_role.ok_or_else(|| {
                    PikePayloadBuildError::MissingField(
                        "'update_role' field is required".to_string(),
                    )
                })?
            } else {
                UpdateRoleAction::default()
            }
        };

        let delete_role = {
            if action == Action::DeleteRole {
                self.delete_role.ok_or_else(|| {
                    PikePayloadBuildError::MissingField(
                        "'delete_role' field is required".to_string(),
                    )
                })?
            } else {
                DeleteRoleAction::default()
            }
        };

        Ok(PikePayload {
            action,
            create_agent,
            update_agent,
            delete_agent,
            create_organization,
            update_organization,
            delete_organization,
            create_role,
            update_role,
            delete_role,
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
            .with_locations(vec!["location".to_string()])
            .with_metadata(vec![key_value.clone()])
            .build()
            .unwrap();

        assert_eq!(create_organization.org_id(), "organization");
        assert_eq!(create_organization.name(), "name");
        assert_eq!(create_organization.locations(), ["location"]);
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
            .with_locations(vec!["location".to_string()])
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
            .with_locations(vec!["location".to_string()])
            .build()
            .unwrap();

        assert_eq!(update_organization.org_id(), "organization");
        assert_eq!(update_organization.name(), "name");
        assert_eq!(update_organization.locations(), ["location"]);
    }

    #[test]
    // check that a update_organization can be converted to bytes and back
    fn check_update_organization_bytes() {
        let builder = UpdateOrganizationActionBuilder::new();
        let original = builder
            .with_org_id("organization".to_string())
            .with_name("name".to_string())
            .with_locations(vec!["location".to_string()])
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
            .with_locations(vec!["location".to_string()])
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
            .with_locations(vec!["location".to_string()])
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
            .with_locations(vec!["location".to_string()])
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
