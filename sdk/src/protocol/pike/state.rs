// Copyright 2019-2021 Cargill Incorporated
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

//! Protocol structs for Pike state

use protobuf::Message;
use protobuf::RepeatedField;

use std::error::Error as StdError;

use crate::protos;
use crate::protos::{
    FromBytes, FromNative, FromProto, IntoBytes, IntoNative, IntoProto, ProtoConversionError,
};

/// Native representation of a `KeyValueEntry`
///
/// A `KeyValueEntry` organizes additional data, `metadata`, for state objects. Any data,
/// represented as a `String`, may be stored within a `KeyValueEntry`.
#[derive(Debug, Clone, PartialEq)]
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

/// Returned if any required fields in a `KeyValueEntry` are not present when being
/// converted from the corresponding builder
#[derive(Debug)]
pub enum KeyValueEntryBuildError {
    MissingField(String),
}

impl StdError for KeyValueEntryBuildError {
    fn description(&self) -> &str {
        match *self {
            KeyValueEntryBuildError::MissingField(ref msg) => msg,
        }
    }

    fn cause(&self) -> Option<&dyn StdError> {
        match *self {
            KeyValueEntryBuildError::MissingField(_) => None,
        }
    }
}

impl std::fmt::Display for KeyValueEntryBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            KeyValueEntryBuildError::MissingField(ref s) => write!(f, "MissingField: {}", s),
        }
    }
}

/// Builder used to create a `KeyValueEntry`
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

    pub fn build(self) -> Result<KeyValueEntry, KeyValueEntryBuildError> {
        let key = self.key.ok_or_else(|| {
            KeyValueEntryBuildError::MissingField("'key' field is required".to_string())
        })?;

        let value = self.value.ok_or_else(|| {
            KeyValueEntryBuildError::MissingField("'value' field is required".to_string())
        })?;

        Ok(KeyValueEntry { key, value })
    }
}

/// Native representation of a `Role`
///
/// A `Role` defines the permissions that are able to be assigned to an `Organization`'s `Agent`.
/// This allows agents to perform actions defined within their role.
#[derive(Debug, Clone, PartialEq)]
pub struct Role {
    org_id: String,
    name: String,
    description: String,
    active: bool,
    permissions: Vec<String>,
    allowed_organizations: Vec<String>,
    inherit_from: Vec<String>,
}

impl Role {
    pub fn org_id(&self) -> &str {
        &self.org_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn active(&self) -> &bool {
        &self.active
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
}

impl FromProto<protos::pike_state::Role> for Role {
    fn from_proto(role: protos::pike_state::Role) -> Result<Self, ProtoConversionError> {
        Ok(Role {
            org_id: role.get_org_id().to_string(),
            name: role.get_name().to_string(),
            description: role.get_description().to_string(),
            active: role.get_active(),
            permissions: role.get_permissions().to_vec(),
            allowed_organizations: role.get_allowed_organizations().to_vec(),
            inherit_from: role.get_inherit_from().to_vec(),
        })
    }
}

impl FromNative<Role> for protos::pike_state::Role {
    fn from_native(role: Role) -> Result<Self, ProtoConversionError> {
        let mut role_proto = protos::pike_state::Role::new();

        role_proto.set_org_id(role.org_id().to_string());
        role_proto.set_name(role.name().to_string());
        role_proto.set_description(role.description().to_string());
        role_proto.set_active(*role.active());
        role_proto.set_permissions(RepeatedField::from_vec(role.permissions().to_vec()));
        role_proto.set_allowed_organizations(RepeatedField::from_vec(
            role.allowed_organizations().to_vec(),
        ));
        role_proto.set_inherit_from(RepeatedField::from_vec(role.inherit_from().to_vec()));

        Ok(role_proto)
    }
}

impl FromBytes<Role> for Role {
    fn from_bytes(bytes: &[u8]) -> Result<Role, ProtoConversionError> {
        let proto: protos::pike_state::Role = Message::parse_from_bytes(bytes).map_err(|_| {
            ProtoConversionError::SerializationError("Unable to get Role from bytes".to_string())
        })?;
        proto.into_native()
    }
}

impl IntoBytes for Role {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError("Unable to get bytes from Role".to_string())
        })?;
        Ok(bytes)
    }
}

impl IntoProto<protos::pike_state::Role> for Role {}
impl IntoNative<Role> for protos::pike_state::Role {}

/// Returned if any required fields in a `Role` are not present when being
/// converted from the corresponding builder
#[derive(Debug)]
pub enum RoleBuildError {
    MissingField(String),
}

impl StdError for RoleBuildError {
    fn description(&self) -> &str {
        match *self {
            RoleBuildError::MissingField(ref msg) => msg,
        }
    }

    fn cause(&self) -> Option<&dyn StdError> {
        match *self {
            RoleBuildError::MissingField(_) => None,
        }
    }
}

impl std::fmt::Display for RoleBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            RoleBuildError::MissingField(ref s) => write!(f, "MissingField: {}", s),
        }
    }
}

/// Builder used to create a `Role`
#[derive(Default, Clone)]
pub struct RoleBuilder {
    org_id: Option<String>,
    name: Option<String>,
    description: Option<String>,
    active: bool,
    permissions: Vec<String>,
    allowed_organizations: Vec<String>,
    inherit_from: Vec<String>,
}

impl RoleBuilder {
    pub fn new() -> Self {
        RoleBuilder::default()
    }

    pub fn with_org_id(mut self, org_id: String) -> RoleBuilder {
        self.org_id = Some(org_id);
        self
    }

    pub fn with_name(mut self, name: String) -> RoleBuilder {
        self.name = Some(name);
        self
    }

    pub fn with_description(mut self, description: String) -> RoleBuilder {
        self.description = Some(description);
        self
    }

    pub fn with_active(mut self, active: bool) -> RoleBuilder {
        self.active = active;
        self
    }

    pub fn with_permissions(mut self, permissions: Vec<String>) -> RoleBuilder {
        self.permissions = permissions;
        self
    }

    pub fn with_allowed_organizations(mut self, allowed_organizations: Vec<String>) -> RoleBuilder {
        self.allowed_organizations = allowed_organizations;
        self
    }

    pub fn with_inherit_from(mut self, inherit_from: Vec<String>) -> RoleBuilder {
        self.inherit_from = inherit_from;
        self
    }

    pub fn build(self) -> Result<Role, RoleBuildError> {
        let org_id = self.org_id.ok_or_else(|| {
            RoleBuildError::MissingField("'org_id' field is required".to_string())
        })?;

        let name = self
            .name
            .ok_or_else(|| RoleBuildError::MissingField("'name' field is required".to_string()))?;

        let description = self.description.unwrap_or_else(|| "".to_string());

        let active = self.active;

        let permissions = self.permissions;
        let allowed_organizations = self.allowed_organizations;
        let inherit_from = self.inherit_from;

        Ok(Role {
            org_id,
            name,
            description,
            active,
            permissions,
            allowed_organizations,
            inherit_from,
        })
    }
}

/// Native representation of a list of `Role`s
#[derive(Debug, Clone, PartialEq)]
pub struct RoleList {
    roles: Vec<Role>,
}

impl RoleList {
    pub fn roles(&self) -> &[Role] {
        &self.roles
    }
}

impl FromProto<protos::pike_state::RoleList> for RoleList {
    fn from_proto(role_list: protos::pike_state::RoleList) -> Result<Self, ProtoConversionError> {
        Ok(RoleList {
            roles: role_list
                .get_roles()
                .to_vec()
                .into_iter()
                .map(Role::from_proto)
                .collect::<Result<Vec<Role>, ProtoConversionError>>()?,
        })
    }
}

impl FromNative<RoleList> for protos::pike_state::RoleList {
    fn from_native(role_list: RoleList) -> Result<Self, ProtoConversionError> {
        let mut role_list_proto = protos::pike_state::RoleList::new();

        role_list_proto.set_roles(RepeatedField::from_vec(
            role_list
                .roles()
                .to_vec()
                .into_iter()
                .map(Role::into_proto)
                .collect::<Result<Vec<protos::pike_state::Role>, ProtoConversionError>>()?,
        ));

        Ok(role_list_proto)
    }
}

impl FromBytes<RoleList> for RoleList {
    fn from_bytes(bytes: &[u8]) -> Result<RoleList, ProtoConversionError> {
        let proto: protos::pike_state::RoleList =
            Message::parse_from_bytes(bytes).map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get RoleList from bytes".to_string(),
                )
            })?;

        proto.into_native()
    }
}

impl IntoBytes for RoleList {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from RoleList".to_string(),
            )
        })?;
        Ok(bytes)
    }
}

impl IntoProto<protos::pike_state::RoleList> for RoleList {}
impl IntoNative<RoleList> for protos::pike_state::RoleList {}

/// Returned if any required fields in a `RoleList` are not present when being
/// converted from the corresponding builder
#[derive(Debug)]
pub enum RoleListBuildError {
    MissingField(String),
}

impl StdError for RoleListBuildError {
    fn description(&self) -> &str {
        match *self {
            RoleListBuildError::MissingField(ref msg) => msg,
        }
    }

    fn cause(&self) -> Option<&dyn StdError> {
        match *self {
            RoleListBuildError::MissingField(_) => None,
        }
    }
}

impl std::fmt::Display for RoleListBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            RoleListBuildError::MissingField(ref s) => write!(f, "MissingField: {}", s),
        }
    }
}

/// Builder used to create a `RoleList`
#[derive(Default, Clone)]
pub struct RoleListBuilder {
    pub roles: Vec<Role>,
}

impl RoleListBuilder {
    pub fn new() -> Self {
        RoleListBuilder::default()
    }

    pub fn with_roles(mut self, roles: Vec<Role>) -> RoleListBuilder {
        self.roles = roles;
        self
    }

    pub fn build(self) -> Result<RoleList, RoleListBuildError> {
        let roles = {
            if self.roles.is_empty() {
                return Err(RoleListBuildError::MissingField(
                    "'roles' cannot be empty".to_string(),
                ));
            } else {
                self.roles
            }
        };

        Ok(RoleList { roles })
    }
}

/// Native representation of the state object `AlternateIdIndexEntry`
///
/// The `AlternateIdIndexEntry` serves as an index to fetch an `Organization` from an externally
/// known ID and ensures the alternate ID is unique to the owning organization.
#[derive(Debug, Clone, PartialEq)]
pub struct AlternateIdIndexEntry {
    id_type: String,
    id: String,
    grid_identity_id: String,
}

impl AlternateIdIndexEntry {
    pub fn id_type(&self) -> &str {
        &self.id_type
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn grid_identity_id(&self) -> &str {
        &self.grid_identity_id
    }
}

impl FromProto<protos::pike_state::AlternateIdIndexEntry> for AlternateIdIndexEntry {
    fn from_proto(
        id: protos::pike_state::AlternateIdIndexEntry,
    ) -> Result<Self, ProtoConversionError> {
        Ok(AlternateIdIndexEntry {
            id_type: id.get_id_type().to_string(),
            id: id.get_id().to_string(),
            grid_identity_id: id.get_grid_identity_id().to_string(),
        })
    }
}

impl FromNative<AlternateIdIndexEntry> for protos::pike_state::AlternateIdIndexEntry {
    fn from_native(id: AlternateIdIndexEntry) -> Result<Self, ProtoConversionError> {
        let mut alt_id_proto = protos::pike_state::AlternateIdIndexEntry::new();

        alt_id_proto.set_id_type(id.id_type().to_string());
        alt_id_proto.set_id(id.id().to_string());
        alt_id_proto.set_grid_identity_id(id.grid_identity_id().to_string());

        Ok(alt_id_proto)
    }
}

impl FromBytes<AlternateIdIndexEntry> for AlternateIdIndexEntry {
    fn from_bytes(bytes: &[u8]) -> Result<AlternateIdIndexEntry, ProtoConversionError> {
        let proto: protos::pike_state::AlternateIdIndexEntry = Message::parse_from_bytes(bytes)
            .map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get AlternateIdIndexEntry from bytes".to_string(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for AlternateIdIndexEntry {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from AlternateIdIndexEntry".to_string(),
            )
        })?;
        Ok(bytes)
    }
}

impl IntoProto<protos::pike_state::AlternateIdIndexEntry> for AlternateIdIndexEntry {}
impl IntoNative<AlternateIdIndexEntry> for protos::pike_state::AlternateIdIndexEntry {}

/// Returned if any required fields in an `AlternateIdIndexEntry` are not present when being
/// converted from the corresponding builder
#[derive(Debug)]
pub enum AlternateIdIndexEntryBuildError {
    MissingField(String),
}

impl StdError for AlternateIdIndexEntryBuildError {
    fn description(&self) -> &str {
        match *self {
            AlternateIdIndexEntryBuildError::MissingField(ref msg) => msg,
        }
    }

    fn cause(&self) -> Option<&dyn StdError> {
        match *self {
            AlternateIdIndexEntryBuildError::MissingField(_) => None,
        }
    }
}

impl std::fmt::Display for AlternateIdIndexEntryBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            AlternateIdIndexEntryBuildError::MissingField(ref s) => {
                write!(f, "MissingField: {}", s)
            }
        }
    }
}

/// Builder used to create an `AlternateIdIndexEntry`
#[derive(Default, Clone)]
pub struct AlternateIdIndexEntryBuilder {
    pub id_type: Option<String>,
    pub id: Option<String>,
    pub grid_identity_id: Option<String>,
}

impl AlternateIdIndexEntryBuilder {
    pub fn new() -> Self {
        AlternateIdIndexEntryBuilder::default()
    }

    pub fn with_id_type(mut self, id_type: String) -> AlternateIdIndexEntryBuilder {
        self.id_type = Some(id_type);
        self
    }

    pub fn with_id(mut self, id: String) -> AlternateIdIndexEntryBuilder {
        self.id = Some(id);
        self
    }

    pub fn with_grid_identity_id(
        mut self,
        grid_identity_id: String,
    ) -> AlternateIdIndexEntryBuilder {
        self.grid_identity_id = Some(grid_identity_id);
        self
    }

    pub fn build(self) -> Result<AlternateIdIndexEntry, AlternateIdIndexEntryBuildError> {
        let id_type = self.id_type.ok_or_else(|| {
            AlternateIdIndexEntryBuildError::MissingField("'id_type' field is required".to_string())
        })?;

        let id = self.id.ok_or_else(|| {
            AlternateIdIndexEntryBuildError::MissingField("'id' field is required".to_string())
        })?;

        let grid_identity_id = self.grid_identity_id.ok_or_else(|| {
            AlternateIdIndexEntryBuildError::MissingField(
                "'grid_identity_id' field is required".to_string(),
            )
        })?;

        Ok(AlternateIdIndexEntry {
            id_type,
            id,
            grid_identity_id,
        })
    }
}

/// Native representation of a list of `AlternateIdIndexEntry`s
#[derive(Debug, Clone, PartialEq)]
pub struct AlternateIdIndexEntryList {
    entries: Vec<AlternateIdIndexEntry>,
}

impl AlternateIdIndexEntryList {
    pub fn entries(&self) -> &[AlternateIdIndexEntry] {
        &self.entries
    }
}

impl FromProto<protos::pike_state::AlternateIdIndexEntryList> for AlternateIdIndexEntryList {
    fn from_proto(
        entry_list: protos::pike_state::AlternateIdIndexEntryList,
    ) -> Result<Self, ProtoConversionError> {
        Ok(AlternateIdIndexEntryList {
            entries: entry_list
                .get_entries()
                .to_vec()
                .into_iter()
                .map(AlternateIdIndexEntry::from_proto)
                .collect::<Result<Vec<AlternateIdIndexEntry>, ProtoConversionError>>()?,
        })
    }
}

impl FromNative<AlternateIdIndexEntryList> for protos::pike_state::AlternateIdIndexEntryList {
    fn from_native(entry_list: AlternateIdIndexEntryList) -> Result<Self, ProtoConversionError> {
        let mut entry_list_proto = protos::pike_state::AlternateIdIndexEntryList::new();

        entry_list_proto.set_entries(RepeatedField::from_vec(
            entry_list
                .entries()
                .to_vec()
                .into_iter()
                .map(AlternateIdIndexEntry::into_proto)
                .collect::<Result<Vec<protos::pike_state::AlternateIdIndexEntry>, ProtoConversionError>>()?,
        ));

        Ok(entry_list_proto)
    }
}

impl FromBytes<AlternateIdIndexEntryList> for AlternateIdIndexEntryList {
    fn from_bytes(bytes: &[u8]) -> Result<AlternateIdIndexEntryList, ProtoConversionError> {
        let proto: protos::pike_state::AlternateIdIndexEntryList = Message::parse_from_bytes(bytes)
            .map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get AlternateIdIndexEntryList from bytes".to_string(),
                )
            })?;

        proto.into_native()
    }
}

impl IntoBytes for AlternateIdIndexEntryList {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from AlternateIdIndexEntryList".to_string(),
            )
        })?;
        Ok(bytes)
    }
}

impl IntoProto<protos::pike_state::AlternateIdIndexEntryList> for AlternateIdIndexEntryList {}
impl IntoNative<AlternateIdIndexEntryList> for protos::pike_state::AlternateIdIndexEntryList {}

/// Returned if any required fields in an `AlternateIdIndexEntryList` are not present when being
/// converted from the corresponding builder
#[derive(Debug)]
pub enum AlternateIdIndexEntryListBuildError {
    MissingField(String),
}

impl StdError for AlternateIdIndexEntryListBuildError {
    fn description(&self) -> &str {
        match *self {
            AlternateIdIndexEntryListBuildError::MissingField(ref msg) => msg,
        }
    }

    fn cause(&self) -> Option<&dyn StdError> {
        match *self {
            AlternateIdIndexEntryListBuildError::MissingField(_) => None,
        }
    }
}

impl std::fmt::Display for AlternateIdIndexEntryListBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            AlternateIdIndexEntryListBuildError::MissingField(ref s) => {
                write!(f, "MissingField: {}", s)
            }
        }
    }
}

/// Builder used to create an `AlternateIdIndexEntryList`
#[derive(Default, Clone)]
pub struct AlternateIdIndexEntryListBuilder {
    pub entries: Vec<AlternateIdIndexEntry>,
}

impl AlternateIdIndexEntryListBuilder {
    pub fn new() -> Self {
        AlternateIdIndexEntryListBuilder::default()
    }

    pub fn with_entries(
        mut self,
        entries: Vec<AlternateIdIndexEntry>,
    ) -> AlternateIdIndexEntryListBuilder {
        self.entries = entries;
        self
    }

    pub fn build(self) -> Result<AlternateIdIndexEntryList, AlternateIdIndexEntryListBuildError> {
        let entries = {
            if self.entries.is_empty() {
                return Err(AlternateIdIndexEntryListBuildError::MissingField(
                    "'entries' cannot be empty".to_string(),
                ));
            } else {
                self.entries
            }
        };

        Ok(AlternateIdIndexEntryList { entries })
    }
}

/// Native representation of the state object `AlternateId`
///
/// An `AlternateId` is a separate identifier from the `Organization`'s unique identifier, `org_id`.
/// This enables certain smart contracts to identify an `Organization` within its own context.
#[derive(Debug, Clone, PartialEq)]
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

impl FromProto<protos::pike_state::AlternateId> for AlternateId {
    fn from_proto(id: protos::pike_state::AlternateId) -> Result<Self, ProtoConversionError> {
        Ok(AlternateId {
            id_type: id.get_id_type().to_string(),
            id: id.get_id().to_string(),
        })
    }
}

impl FromNative<AlternateId> for protos::pike_state::AlternateId {
    fn from_native(id: AlternateId) -> Result<Self, ProtoConversionError> {
        let mut alt_id_proto = protos::pike_state::AlternateId::new();

        alt_id_proto.set_id_type(id.id_type().to_string());
        alt_id_proto.set_id(id.id().to_string());

        Ok(alt_id_proto)
    }
}

impl FromBytes<AlternateId> for AlternateId {
    fn from_bytes(bytes: &[u8]) -> Result<AlternateId, ProtoConversionError> {
        let proto: protos::pike_state::AlternateId =
            Message::parse_from_bytes(bytes).map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get AlternateId from bytes".to_string(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for AlternateId {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from AlternateId".to_string(),
            )
        })?;
        Ok(bytes)
    }
}

impl IntoProto<protos::pike_state::AlternateId> for AlternateId {}
impl IntoNative<AlternateId> for protos::pike_state::AlternateId {}

/// Returned if any required fields in an `AlternateId` are not present when being
/// converted from the corresponding builder
#[derive(Debug)]
pub enum AlternateIdBuildError {
    MissingField(String),
}

impl StdError for AlternateIdBuildError {
    fn description(&self) -> &str {
        match *self {
            AlternateIdBuildError::MissingField(ref msg) => msg,
        }
    }

    fn cause(&self) -> Option<&dyn StdError> {
        match *self {
            AlternateIdBuildError::MissingField(_) => None,
        }
    }
}

impl std::fmt::Display for AlternateIdBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            AlternateIdBuildError::MissingField(ref s) => write!(f, "MissingField: {}", s),
        }
    }
}

/// Builder used to create an `AlternateId`
#[derive(Default, Clone)]
pub struct AlternateIdBuilder {
    pub id_type: Option<String>,
    pub id: Option<String>,
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

    pub fn build(self) -> Result<AlternateId, AlternateIdBuildError> {
        let id_type = self.id_type.ok_or_else(|| {
            AlternateIdBuildError::MissingField("'id_type' field is required".to_string())
        })?;

        let id = self.id.ok_or_else(|| {
            AlternateIdBuildError::MissingField("'id' field is required".to_string())
        })?;

        Ok(AlternateId { id_type, id })
    }
}

/// Native representation of the state object `Agent`
///
/// An `Agent` is essentially a cryptographic public key which has a relationship, defined by the
/// agent's `Role`s, with an `Organization`.
#[derive(Debug, Clone, PartialEq)]
pub struct Agent {
    org_id: String,
    public_key: String,
    active: bool,
    roles: Vec<String>,
    metadata: Vec<KeyValueEntry>,
}

impl Agent {
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

impl FromProto<protos::pike_state::Agent> for Agent {
    fn from_proto(agent: protos::pike_state::Agent) -> Result<Self, ProtoConversionError> {
        Ok(Agent {
            org_id: agent.get_org_id().to_string(),
            public_key: agent.get_public_key().to_string(),
            active: agent.get_active(),
            roles: agent.get_roles().to_vec(),
            metadata: agent
                .get_metadata()
                .to_vec()
                .into_iter()
                .map(KeyValueEntry::from_proto)
                .collect::<Result<Vec<KeyValueEntry>, ProtoConversionError>>()?,
        })
    }
}

impl FromNative<Agent> for protos::pike_state::Agent {
    fn from_native(agent: Agent) -> Result<Self, ProtoConversionError> {
        let mut agent_proto = protos::pike_state::Agent::new();

        agent_proto.set_org_id(agent.org_id().to_string());
        agent_proto.set_public_key(agent.public_key().to_string());
        agent_proto.set_active(*agent.active());
        agent_proto.set_org_id(agent.org_id().to_string());
        agent_proto.set_roles(RepeatedField::from_vec(agent.roles().to_vec()));
        agent_proto.set_metadata(RepeatedField::from_vec(
            agent
                .metadata()
                .to_vec()
                .into_iter()
                .map(KeyValueEntry::into_proto)
                .collect::<Result<Vec<protos::pike_state::KeyValueEntry>, ProtoConversionError>>(
                )?,
        ));

        Ok(agent_proto)
    }
}

impl FromBytes<Agent> for Agent {
    fn from_bytes(bytes: &[u8]) -> Result<Agent, ProtoConversionError> {
        let proto: protos::pike_state::Agent = Message::parse_from_bytes(bytes).map_err(|_| {
            ProtoConversionError::SerializationError("Unable to get Agent from bytes".to_string())
        })?;
        proto.into_native()
    }
}

impl IntoBytes for Agent {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError("Unable to get bytes from Agent".to_string())
        })?;
        Ok(bytes)
    }
}

impl IntoProto<protos::pike_state::Agent> for Agent {}
impl IntoNative<Agent> for protos::pike_state::Agent {}

/// Returned if any required fields in an `Agent` are not present when being
/// converted from the corresponding builder
#[derive(Debug)]
pub enum AgentBuildError {
    MissingField(String),
}

impl StdError for AgentBuildError {
    fn description(&self) -> &str {
        match *self {
            AgentBuildError::MissingField(ref msg) => msg,
        }
    }

    fn cause(&self) -> Option<&dyn StdError> {
        match *self {
            AgentBuildError::MissingField(_) => None,
        }
    }
}

impl std::fmt::Display for AgentBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            AgentBuildError::MissingField(ref s) => write!(f, "MissingField: {}", s),
        }
    }
}

/// Builder used to create an `Agent`
#[derive(Default, Clone)]
pub struct AgentBuilder {
    pub org_id: Option<String>,
    pub public_key: Option<String>,
    pub active: Option<bool>,
    pub roles: Vec<String>,
    pub metadata: Vec<KeyValueEntry>,
}

impl AgentBuilder {
    pub fn new() -> Self {
        AgentBuilder::default()
    }

    pub fn with_org_id(mut self, org_id: String) -> AgentBuilder {
        self.org_id = Some(org_id);
        self
    }

    pub fn with_public_key(mut self, public_key: String) -> AgentBuilder {
        self.public_key = Some(public_key);
        self
    }

    pub fn with_active(mut self, active: bool) -> AgentBuilder {
        self.active = Some(active);
        self
    }

    pub fn with_roles(mut self, roles: Vec<String>) -> AgentBuilder {
        self.roles = roles;
        self
    }

    pub fn with_metadata(mut self, metadata: Vec<KeyValueEntry>) -> AgentBuilder {
        self.metadata = metadata;
        self
    }

    pub fn build(self) -> Result<Agent, AgentBuildError> {
        let org_id = self.org_id.ok_or_else(|| {
            AgentBuildError::MissingField("'org_id' field is required".to_string())
        })?;

        let public_key = self.public_key.ok_or_else(|| {
            AgentBuildError::MissingField("'public_key' field is required".to_string())
        })?;

        let active = self.active.unwrap_or_default();
        let roles = self.roles;
        let metadata = self.metadata;

        Ok(Agent {
            org_id,
            public_key,
            active,
            roles,
            metadata,
        })
    }
}

/// Native representation of a list of `Agent`s
#[derive(Debug, Clone, PartialEq)]
pub struct AgentList {
    agents: Vec<Agent>,
}

impl AgentList {
    pub fn agents(&self) -> &[Agent] {
        &self.agents
    }
}

impl FromProto<protos::pike_state::AgentList> for AgentList {
    fn from_proto(agent_list: protos::pike_state::AgentList) -> Result<Self, ProtoConversionError> {
        Ok(AgentList {
            agents: agent_list
                .get_agents()
                .to_vec()
                .into_iter()
                .map(Agent::from_proto)
                .collect::<Result<Vec<Agent>, ProtoConversionError>>()?,
        })
    }
}

impl FromNative<AgentList> for protos::pike_state::AgentList {
    fn from_native(agent_list: AgentList) -> Result<Self, ProtoConversionError> {
        let mut agent_list_proto = protos::pike_state::AgentList::new();

        agent_list_proto.set_agents(RepeatedField::from_vec(
            agent_list
                .agents()
                .to_vec()
                .into_iter()
                .map(Agent::into_proto)
                .collect::<Result<Vec<protos::pike_state::Agent>, ProtoConversionError>>()?,
        ));

        Ok(agent_list_proto)
    }
}

impl FromBytes<AgentList> for AgentList {
    fn from_bytes(bytes: &[u8]) -> Result<AgentList, ProtoConversionError> {
        let proto: protos::pike_state::AgentList =
            Message::parse_from_bytes(bytes).map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get AgentList from bytes".to_string(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for AgentList {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from AgentList".to_string(),
            )
        })?;
        Ok(bytes)
    }
}

impl IntoProto<protos::pike_state::AgentList> for AgentList {}
impl IntoNative<AgentList> for protos::pike_state::AgentList {}

/// Returned if any required fields in an `AgentList` are not present when being
/// converted from the corresponding builder
#[derive(Debug)]
pub enum AgentListBuildError {
    MissingField(String),
}

impl StdError for AgentListBuildError {
    fn description(&self) -> &str {
        match *self {
            AgentListBuildError::MissingField(ref msg) => msg,
        }
    }

    fn cause(&self) -> Option<&dyn StdError> {
        match *self {
            AgentListBuildError::MissingField(_) => None,
        }
    }
}

impl std::fmt::Display for AgentListBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            AgentListBuildError::MissingField(ref s) => write!(f, "MissingField: {}", s),
        }
    }
}

/// Builder used to create an `AgentList`
#[derive(Default, Clone)]
pub struct AgentListBuilder {
    pub agents: Vec<Agent>,
}

impl AgentListBuilder {
    pub fn new() -> Self {
        AgentListBuilder::default()
    }

    pub fn with_agents(mut self, agents: Vec<Agent>) -> AgentListBuilder {
        self.agents = agents;
        self
    }

    pub fn build(self) -> Result<AgentList, AgentListBuildError> {
        let agents = {
            if self.agents.is_empty() {
                return Err(AgentListBuildError::MissingField(
                    "'agents' cannot be empty".to_string(),
                ));
            } else {
                self.agents
            }
        };

        Ok(AgentList { agents })
    }
}

/// Native representation of the state object `Organization`
///
/// `Organization`s provide a top-level identity to associate the lower-level objects to across
/// smart contracts.
#[derive(Debug, Clone, PartialEq)]
pub struct Organization {
    org_id: String,
    name: String,
    locations: Vec<String>,
    alternate_ids: Vec<AlternateId>,
    metadata: Vec<KeyValueEntry>,
}

impl Organization {
    pub fn org_id(&self) -> &str {
        &self.org_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn locations(&self) -> &[String] {
        &self.locations
    }

    pub fn alternate_ids(&self) -> &[AlternateId] {
        &self.alternate_ids
    }

    pub fn metadata(&self) -> &[KeyValueEntry] {
        &self.metadata
    }
}

impl FromProto<protos::pike_state::Organization> for Organization {
    fn from_proto(org: protos::pike_state::Organization) -> Result<Self, ProtoConversionError> {
        Ok(Organization {
            org_id: org.get_org_id().to_string(),
            name: org.get_name().to_string(),
            locations: org.get_locations().to_vec(),
            alternate_ids: org
                .get_alternate_ids()
                .to_vec()
                .into_iter()
                .map(AlternateId::from_proto)
                .collect::<Result<Vec<AlternateId>, ProtoConversionError>>()?,
            metadata: org
                .get_metadata()
                .to_vec()
                .into_iter()
                .map(KeyValueEntry::from_proto)
                .collect::<Result<Vec<KeyValueEntry>, ProtoConversionError>>()?,
        })
    }
}

impl FromNative<Organization> for protos::pike_state::Organization {
    fn from_native(org: Organization) -> Result<Self, ProtoConversionError> {
        let mut org_proto = protos::pike_state::Organization::new();

        org_proto.set_org_id(org.org_id().to_string());
        org_proto.set_name(org.name().to_string());
        org_proto.set_locations(RepeatedField::from_vec(org.locations().to_vec()));
        org_proto.set_alternate_ids(RepeatedField::from_vec(
            org.alternate_ids()
                .to_vec()
                .into_iter()
                .map(AlternateId::into_proto)
                .collect::<Result<Vec<protos::pike_state::AlternateId>, ProtoConversionError>>()?,
        ));
        org_proto.set_metadata(RepeatedField::from_vec(
            org.metadata()
                .to_vec()
                .into_iter()
                .map(KeyValueEntry::into_proto)
                .collect::<Result<Vec<protos::pike_state::KeyValueEntry>, ProtoConversionError>>(
                )?,
        ));

        Ok(org_proto)
    }
}

impl FromBytes<Organization> for Organization {
    fn from_bytes(bytes: &[u8]) -> Result<Organization, ProtoConversionError> {
        let proto: protos::pike_state::Organization =
            Message::parse_from_bytes(bytes).map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get Organization from bytes".to_string(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for Organization {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from Organization".to_string(),
            )
        })?;
        Ok(bytes)
    }
}

impl IntoProto<protos::pike_state::Organization> for Organization {}
impl IntoNative<Organization> for protos::pike_state::Organization {}

/// Returned if any required fields in an `Organization` are not present when being
/// converted from the corresponding builder
#[derive(Debug)]
pub enum OrganizationBuildError {
    MissingField(String),
}

impl StdError for OrganizationBuildError {
    fn description(&self) -> &str {
        match *self {
            OrganizationBuildError::MissingField(ref msg) => msg,
        }
    }

    fn cause(&self) -> Option<&dyn StdError> {
        match *self {
            OrganizationBuildError::MissingField(_) => None,
        }
    }
}

impl std::fmt::Display for OrganizationBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            OrganizationBuildError::MissingField(ref s) => write!(f, "MissingField: {}", s),
        }
    }
}

/// Builder used to create an `Organization`
#[derive(Default, Clone)]
pub struct OrganizationBuilder {
    pub org_id: Option<String>,
    pub name: Option<String>,
    pub locations: Vec<String>,
    pub alternate_ids: Vec<AlternateId>,
    pub metadata: Vec<KeyValueEntry>,
}

impl OrganizationBuilder {
    pub fn new() -> Self {
        OrganizationBuilder::default()
    }

    pub fn with_org_id(mut self, org_id: String) -> OrganizationBuilder {
        self.org_id = Some(org_id);
        self
    }

    pub fn with_name(mut self, name: String) -> OrganizationBuilder {
        self.name = Some(name);
        self
    }

    pub fn with_locations(mut self, locations: Vec<String>) -> OrganizationBuilder {
        self.locations = locations;
        self
    }

    pub fn with_alternate_ids(mut self, alternate_ids: Vec<AlternateId>) -> OrganizationBuilder {
        self.alternate_ids = alternate_ids;
        self
    }

    pub fn with_metadata(mut self, metadata: Vec<KeyValueEntry>) -> OrganizationBuilder {
        self.metadata = metadata;
        self
    }

    pub fn build(self) -> Result<Organization, OrganizationBuildError> {
        let org_id = self.org_id.ok_or_else(|| {
            OrganizationBuildError::MissingField("'org_id' field is required".to_string())
        })?;

        let name = self.name.ok_or_else(|| {
            OrganizationBuildError::MissingField("'name' field is required".to_string())
        })?;

        let locations = self.locations;

        let alternate_ids = self.alternate_ids;

        let metadata = self.metadata;

        Ok(Organization {
            org_id,
            name,
            locations,
            alternate_ids,
            metadata,
        })
    }
}

/// Native representation of a list of `Organization`s
#[derive(Debug, Clone, PartialEq)]
pub struct OrganizationList {
    organizations: Vec<Organization>,
}

impl OrganizationList {
    pub fn organizations(&self) -> &[Organization] {
        &self.organizations
    }
}

impl FromProto<protos::pike_state::OrganizationList> for OrganizationList {
    fn from_proto(
        organization_list: protos::pike_state::OrganizationList,
    ) -> Result<Self, ProtoConversionError> {
        Ok(OrganizationList {
            organizations: organization_list
                .get_organizations()
                .to_vec()
                .into_iter()
                .map(Organization::from_proto)
                .collect::<Result<Vec<Organization>, ProtoConversionError>>()?,
        })
    }
}

impl FromNative<OrganizationList> for protos::pike_state::OrganizationList {
    fn from_native(org_list: OrganizationList) -> Result<Self, ProtoConversionError> {
        let mut org_list_proto = protos::pike_state::OrganizationList::new();

        org_list_proto.set_organizations(RepeatedField::from_vec(
            org_list
                .organizations()
                .to_vec()
                .into_iter()
                .map(Organization::into_proto)
                .collect::<Result<Vec<protos::pike_state::Organization>, ProtoConversionError>>()?,
        ));

        Ok(org_list_proto)
    }
}

impl FromBytes<OrganizationList> for OrganizationList {
    fn from_bytes(bytes: &[u8]) -> Result<OrganizationList, ProtoConversionError> {
        let proto: protos::pike_state::OrganizationList = Message::parse_from_bytes(bytes)
            .map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get OrganizationList from bytes".to_string(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for OrganizationList {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from OrganizationList".to_string(),
            )
        })?;
        Ok(bytes)
    }
}

impl IntoProto<protos::pike_state::OrganizationList> for OrganizationList {}
impl IntoNative<OrganizationList> for protos::pike_state::OrganizationList {}

/// Returned if any required fields in an `OrganizationList` are not present when being
/// converted from the corresponding builder
#[derive(Debug)]
pub enum OrganizationListBuildError {
    MissingField(String),
}

impl StdError for OrganizationListBuildError {
    fn description(&self) -> &str {
        match *self {
            OrganizationListBuildError::MissingField(ref msg) => msg,
        }
    }

    fn cause(&self) -> Option<&dyn StdError> {
        match *self {
            OrganizationListBuildError::MissingField(_) => None,
        }
    }
}

impl std::fmt::Display for OrganizationListBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            OrganizationListBuildError::MissingField(ref s) => write!(f, "MissingField: {}", s),
        }
    }
}

/// Builder used to create an `OrganizationList`
#[derive(Default, Clone)]
pub struct OrganizationListBuilder {
    pub organizations: Vec<Organization>,
}

impl OrganizationListBuilder {
    pub fn new() -> Self {
        OrganizationListBuilder::default()
    }

    pub fn with_organizations(
        mut self,
        organizations: Vec<Organization>,
    ) -> OrganizationListBuilder {
        self.organizations = organizations;
        self
    }

    pub fn build(self) -> Result<OrganizationList, OrganizationListBuildError> {
        let organizations = {
            if self.organizations.is_empty() {
                return Err(OrganizationListBuildError::MissingField(
                    "'organization' cannot be empty".to_string(),
                ));
            } else {
                self.organizations
            }
        };

        Ok(OrganizationList { organizations })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Validate that a `KeyValueEntry` is built correctly
    fn check_key_value_entry_builder() {
        let builder = KeyValueEntryBuilder::new();
        let key_value = builder
            .with_key("Key".to_string())
            .with_value("Value".to_string())
            .build()
            .unwrap();

        assert_eq!(key_value.key(), "Key");
        assert_eq!(key_value.value(), "Value");
    }

    #[test]
    /// Validate that a `KeyValueEntry` may be correctly converted into bytes and back to its
    /// native representation
    fn check_key_value_entry_bytes() {
        let builder = KeyValueEntryBuilder::new();
        let original = builder
            .with_key("Key".to_string())
            .with_value("Value".to_string())
            .build()
            .unwrap();

        let bytes = original.clone().into_bytes().unwrap();
        let key_value = KeyValueEntry::from_bytes(&bytes).unwrap();
        assert_eq!(key_value, original);
    }

    #[test]
    /// Validate that an `Agent` is built correctly
    fn check_agent_builder() {
        let builder = KeyValueEntryBuilder::new();
        let key_value = builder
            .with_key("Key".to_string())
            .with_value("Value".to_string())
            .build()
            .unwrap();

        let builder = AgentBuilder::new();
        let agent = builder
            .with_org_id("organization".to_string())
            .with_public_key("public_key".to_string())
            .with_active(true)
            .with_roles(vec!["Role".to_string()])
            .with_metadata(vec![key_value.clone()])
            .build()
            .unwrap();

        assert_eq!(agent.org_id(), "organization");
        assert_eq!(agent.public_key(), "public_key");
        assert!(agent.active());
        assert_eq!(agent.roles(), ["Role".to_string()]);
        assert_eq!(agent.metadata(), [key_value]);
    }

    #[test]
    /// Validate that an `Agent` may be correctly converted into bytes and back to its native
    /// representation
    fn check_agent_bytes() {
        let builder = KeyValueEntryBuilder::new();
        let key_value = builder
            .with_key("Key".to_string())
            .with_value("Value".to_string())
            .build()
            .unwrap();

        let builder = AgentBuilder::new();
        let original = builder
            .with_org_id("organization".to_string())
            .with_public_key("public_key".to_string())
            .with_active(true)
            .with_roles(vec!["Role".to_string()])
            .with_metadata(vec![key_value.clone()])
            .build()
            .unwrap();

        let bytes = original.clone().into_bytes().unwrap();
        let agent = Agent::from_bytes(&bytes).unwrap();
        assert_eq!(agent, original);
    }

    #[test]
    /// Validate that an `AgentList` is built correctly
    fn check_agent_list_builder() {
        let builder = KeyValueEntryBuilder::new();
        let key_value = builder
            .with_key("Key".to_string())
            .with_value("Value".to_string())
            .build()
            .unwrap();

        let builder = AgentBuilder::new();
        let agent = builder
            .with_org_id("organization".to_string())
            .with_public_key("public_key".to_string())
            .with_active(true)
            .with_roles(vec!["Role".to_string()])
            .with_metadata(vec![key_value.clone()])
            .build()
            .unwrap();

        let builder = AgentListBuilder::new();
        let agent_list = builder.with_agents(vec![agent.clone()]).build().unwrap();

        assert_eq!(agent_list.agents(), [agent])
    }

    #[test]
    /// Validate that an `AgentList` may be converted into bytes and back to its native
    /// representation
    fn check_agent_list_bytes() {
        let builder = KeyValueEntryBuilder::new();
        let key_value = builder
            .with_key("Key".to_string())
            .with_value("Value".to_string())
            .build()
            .unwrap();

        let builder = AgentBuilder::new();
        let agent = builder
            .with_org_id("organization".to_string())
            .with_public_key("public_key".to_string())
            .with_active(true)
            .with_roles(vec!["Role".to_string()])
            .with_metadata(vec![key_value.clone()])
            .build()
            .unwrap();

        let builder = AgentListBuilder::new();
        let original = builder.with_agents(vec![agent.clone()]).build().unwrap();

        let bytes = original.clone().into_bytes().unwrap();
        let agent_list = AgentList::from_bytes(&bytes).unwrap();
        assert_eq!(agent_list, original);
    }

    #[test]
    /// Validate that an `Organization` is built correctly
    fn check_organization_builder() {
        let builder = KeyValueEntryBuilder::new();
        let key_value = builder
            .with_key("Key".to_string())
            .with_value("Value".to_string())
            .build()
            .unwrap();

        let builder = OrganizationBuilder::new();
        let organization = builder
            .with_org_id("organization".to_string())
            .with_name("name".to_string())
            .with_locations(vec!["location".to_string()])
            .with_metadata(vec![key_value.clone()])
            .build()
            .unwrap();

        assert_eq!(organization.org_id(), "organization");
        assert_eq!(organization.name(), "name");
        assert_eq!(organization.locations(), ["location"]);
        assert_eq!(organization.metadata(), [key_value]);
    }

    #[test]
    /// Validate that an `Organization` may be correctly converted into bytes and back to its
    /// native representation
    fn check_organization_bytes() {
        let builder = KeyValueEntryBuilder::new();
        let key_value = builder
            .with_key("Key".to_string())
            .with_value("Value".to_string())
            .build()
            .unwrap();

        let builder = OrganizationBuilder::new();
        let original = builder
            .with_org_id("organization".to_string())
            .with_name("name".to_string())
            .with_locations(vec!["locations".to_string()])
            .with_metadata(vec![key_value.clone()])
            .build()
            .unwrap();

        let bytes = original.clone().into_bytes().unwrap();
        let org = Organization::from_bytes(&bytes).unwrap();
        assert_eq!(org, original);
    }

    #[test]
    /// Validate that an `OrganizationList` is built correctly
    fn check_organization_lists_builder() {
        let builder = KeyValueEntryBuilder::new();
        let key_value = builder
            .with_key("Key".to_string())
            .with_value("Value".to_string())
            .build()
            .unwrap();

        let builder = OrganizationBuilder::new();
        let organization = builder
            .with_org_id("organization".to_string())
            .with_name("name".to_string())
            .with_locations(vec!["location".to_string()])
            .with_metadata(vec![key_value.clone()])
            .build()
            .unwrap();

        let builder = OrganizationListBuilder::new();
        let organization_list = builder
            .with_organizations(vec![organization.clone()])
            .build()
            .unwrap();

        assert_eq!(organization_list.organizations(), [organization])
    }

    #[test]
    /// Validate that an `OrganizationList` may be correctly converted into bytes and back to its
    /// native representation
    fn check_organization_list_bytes() {
        let builder = KeyValueEntryBuilder::new();
        let key_value = builder
            .with_key("Key".to_string())
            .with_value("Value".to_string())
            .build()
            .unwrap();

        let builder = OrganizationBuilder::new();
        let organization = builder
            .with_org_id("organization".to_string())
            .with_name("name".to_string())
            .with_locations(vec!["locations".to_string()])
            .with_metadata(vec![key_value.clone()])
            .build()
            .unwrap();

        let builder = OrganizationListBuilder::new();
        let original = builder
            .with_organizations(vec![organization.clone()])
            .build()
            .unwrap();

        let bytes = original.clone().into_bytes().unwrap();
        let org_list = OrganizationList::from_bytes(&bytes).unwrap();
        assert_eq!(org_list, original);
    }
}
