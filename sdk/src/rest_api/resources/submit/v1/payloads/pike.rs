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
    DeleteAgent(DeleteAgentAction),
    CreateOrganization(CreateOrganizationAction),
    UpdateOrganization(UpdateOrganizationAction),
    DeleteOrganization(DeleteOrganizationAction),
    CreateRole(CreateRoleAction),
    UpdateRole(UpdateRoleAction),
    DeleteRole(DeleteRoleAction),
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
    alternate_ids: Vec<AlternateID>,
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
            id: create_org.get_id().to_string(),
            name: create_org.get_name().to_string(),
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

        proto_create_org.set_id(create_org.id().to_string());
        proto_create_org.set_name(create_org.name().to_string());
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

#[derive(Default, Clone)]
pub struct CreateOrganizationActionBuilder {
    id: Option<String>,
    name: Option<String>,
    alternate_ids: Vec<AlternateID>,
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
    alternate_ids: Vec<AlternateID>,
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

    pub fn alternate_ids(&self) -> &[AlternateID] {
        &self.alternate_ids
    }

    pub fn locations(&self) -> &[String] {
        &self.locations
    }

    pub fn metadata(&self) -> &[KeyValueEntry] {
        &self.metadata
    }
}

impl FromProto<protos::pike_payload::UpdateOrganizationAction> for UpdateOrganizationAction {
    fn from_proto(
        update_org: protos::pike_payload::UpdateOrganizationAction,
    ) -> Result<Self, ProtoConversionError> {
        Ok(UpdateOrganizationAction {
            id: update_org.get_id().to_string(),
            name: update_org.get_name().to_string(),
            alternate_ids: update_org
                .get_alternate_ids()
                .to_vec()
                .into_iter()
                .map(AlternateID::from_proto)
                .collect::<Result<Vec<AlternateID>, ProtoConversionError>>()?,
            locations: update_org.get_locations().to_vec(),
            metadata: update_org
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

        proto_update_org.set_id(update_org.id().to_string());
        proto_update_org.set_name(update_org.name().to_string());
        proto_update_org.set_alternate_ids(RepeatedField::from_vec(
            update_org
                .alternate_ids()
                .to_vec()
                .into_iter()
                .map(AlternateID::into_proto)
                .collect::<Result<Vec<protos::pike_state::AlternateID>, ProtoConversionError>>()?,
        ));
        proto_update_org.set_locations(RepeatedField::from_vec(update_org.locations().to_vec()));
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
    id: Option<String>,
    name: Option<String>,
    alternate_ids: Vec<AlternateID>,
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
        alternate_ids: Vec<AlternateID>,
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
pub struct AlternateID {
    id_type: String,
    id: String,
}

impl AlternateID {
    pub fn id_type(&self) -> &str {
        &self.id_type
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

impl FromProto<protos::pike_state::AlternateID> for AlternateID {
    fn from_proto(
        alternate_id: protos::pike_state::AlternateID,
    ) -> Result<Self, ProtoConversionError> {
        Ok(AlternateID {
            id_type: alternate_id.get_id_type().to_string(),
            id: alternate_id.get_id().to_string(),
        })
    }
}

impl FromNative<AlternateID> for protos::pike_state::AlternateID {
    fn from_native(alternate_id: AlternateID) -> Result<Self, ProtoConversionError> {
        let mut alternate_id_proto = protos::pike_state::AlternateID::new();

        alternate_id_proto.set_id_type(alternate_id.id_type().to_string());
        alternate_id_proto.set_id(alternate_id.id().to_string());

        Ok(alternate_id_proto)
    }
}

impl FromBytes<AlternateID> for AlternateID {
    fn from_bytes(bytes: &[u8]) -> Result<AlternateID, ProtoConversionError> {
        let proto: protos::pike_state::AlternateID =
            Message::parse_from_bytes(bytes).map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get Alternate from bytes".to_string(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for AlternateID {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from AlternateID".to_string(),
            )
        })?;
        Ok(bytes)
    }
}

impl IntoProto<protos::pike_state::AlternateID> for AlternateID {}
impl IntoNative<AlternateID> for protos::pike_state::AlternateID {}

#[derive(Default, Clone)]
pub struct AlternateIDBuilder {
    id_type: Option<String>,
    id: Option<String>,
}

impl AlternateIDBuilder {
    pub fn new() -> Self {
        AlternateIDBuilder::default()
    }

    pub fn with_id_type(mut self, id_type: String) -> AlternateIDBuilder {
        self.id_type = Some(id_type);
        self
    }

    pub fn with_id(mut self, id: String) -> AlternateIDBuilder {
        self.id = Some(id);
        self
    }

    pub fn build(self) -> Result<AlternateID, BuilderError> {
        let id_type = self
            .id_type
            .ok_or_else(|| BuilderError::MissingField("'id_type' field is required".to_string()))?;

        let id = self
            .id
            .ok_or_else(|| BuilderError::MissingField("'id' field is required".to_string()))?;

        Ok(AlternateID { id_type, id })
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
            protos::pike_payload::PikePayload_Action::DELETE_AGENT => PikeAction::DeleteAgent(
                DeleteAgentAction::from_proto(payload.get_delete_agent().clone())?,
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
            protos::pike_payload::PikePayload_Action::DELETE_ORGANIZATION => {
                PikeAction::DeleteOrganization(DeleteOrganizationAction::from_proto(
                    payload.get_delete_organization().clone(),
                )?)
            }
            protos::pike_payload::PikePayload_Action::CREATE_ROLE => PikeAction::CreateRole(
                CreateRoleAction::from_proto(payload.get_create_role().clone())?,
            ),
            protos::pike_payload::PikePayload_Action::UPDATE_ROLE => PikeAction::UpdateRole(
                UpdateRoleAction::from_proto(payload.get_update_role().clone())?,
            ),
            protos::pike_payload::PikePayload_Action::DELETE_ROLE => PikeAction::DeleteRole(
                DeleteRoleAction::from_proto(payload.get_delete_role().clone())?,
            ),
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
            PikeAction::DeleteAgent(payload) => {
                proto.set_action(protos::pike_payload::PikePayload_Action::DELETE_AGENT);
                proto.set_delete_agent(payload.clone().into_proto()?);
            }
            PikeAction::CreateOrganization(payload) => {
                proto.set_action(protos::pike_payload::PikePayload_Action::CREATE_ORGANIZATION);
                proto.set_create_organization(payload.clone().into_proto()?);
            }
            PikeAction::UpdateOrganization(payload) => {
                proto.set_action(protos::pike_payload::PikePayload_Action::UPDATE_ORGANIZATION);
                proto.set_update_organization(payload.clone().into_proto()?);
            }
            PikeAction::DeleteOrganization(payload) => {
                proto.set_action(protos::pike_payload::PikePayload_Action::DELETE_ORGANIZATION);
                proto.set_delete_organization(payload.clone().into_proto()?);
            }
            PikeAction::CreateRole(payload) => {
                proto.set_action(protos::pike_payload::PikePayload_Action::CREATE_ROLE);
                proto.set_create_role(payload.clone().into_proto()?);
            }
            PikeAction::UpdateRole(payload) => {
                proto.set_action(protos::pike_payload::PikePayload_Action::UPDATE_ROLE);
                proto.set_update_role(payload.clone().into_proto()?);
            }
            PikeAction::DeleteRole(payload) => {
                proto.set_action(protos::pike_payload::PikePayload_Action::DELETE_ROLE);
                proto.set_delete_role(payload.clone().into_proto()?);
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
