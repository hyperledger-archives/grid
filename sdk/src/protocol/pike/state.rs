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

use crate::protos;
use crate::protos::{
    FromBytes, FromNative, FromProto, IntoBytes, IntoNative, IntoProto, ProtoConversionError,
};

/// Native implementation for KeyValueEntry
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
            protobuf::parse_from_bytes(bytes).map_err(|_| {
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

    fn cause(&self) -> Option<&StdError> {
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

/// Builder used to create a KeyValueEntry
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

/// Native implementation of Agent
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
        agent_proto.set_active(agent.active().clone());
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
        let proto: protos::pike_state::Agent = protobuf::parse_from_bytes(bytes).map_err(|_| {
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

    fn cause(&self) -> Option<&StdError> {
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

/// Builder used to create a Agent
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

/// Native implementation of AgentList
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
            protobuf::parse_from_bytes(bytes).map_err(|_| {
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

    fn cause(&self) -> Option<&StdError> {
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

/// Builder used to create a AgentList
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

/// Native implementation for Organization
#[derive(Debug, Clone, PartialEq)]
pub struct Organization {
    org_id: String,
    name: String,
    address: String,
    metadata: Vec<KeyValueEntry>,
}

impl Organization {
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

impl FromProto<protos::pike_state::Organization> for Organization {
    fn from_proto(org: protos::pike_state::Organization) -> Result<Self, ProtoConversionError> {
        Ok(Organization {
            org_id: org.get_org_id().to_string(),
            name: org.get_name().to_string(),
            address: org.get_address().to_string(),
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
        org_proto.set_address(org.address().to_string());
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
            protobuf::parse_from_bytes(bytes).map_err(|_| {
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

    fn cause(&self) -> Option<&StdError> {
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

/// Builder used to create a Organization
#[derive(Default, Clone)]
pub struct OrganizationBuilder {
    pub org_id: Option<String>,
    pub name: Option<String>,
    pub address: Option<String>,
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

    pub fn with_address(mut self, name: String) -> OrganizationBuilder {
        self.address = Some(name);
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

        let address = self.address.ok_or_else(|| {
            OrganizationBuildError::MissingField("'address' field is required".to_string())
        })?;

        let metadata = self.metadata;

        Ok(Organization {
            org_id,
            name,
            address,
            metadata,
        })
    }
}

/// Native implementation of OrganizationList
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
        let proto: protos::pike_state::OrganizationList = protobuf::parse_from_bytes(bytes)
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

    fn cause(&self) -> Option<&StdError> {
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

/// Builder used to create a Agent
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
    // check that a KeyValueEntry is built correctly
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
    // check that a KeyValueEntry can be converted to bytes and back
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
    // check that a Agent is built correctly
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
    // check that a Agent can be converted to bytes and back
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
    // check that a AgentList is built correctly
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
    // check that a AgentList can be converted to bytes and back
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
    // check that a Organization is built correctly
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
            .with_address("address".to_string())
            .with_metadata(vec![key_value.clone()])
            .build()
            .unwrap();

        assert_eq!(organization.org_id(), "organization");
        assert_eq!(organization.name(), "name");
        assert_eq!(organization.address(), "address");
        assert_eq!(organization.metadata(), [key_value]);
    }

    #[test]
    // check that a Organization can be converted to bytes and back
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
            .with_address("address".to_string())
            .with_metadata(vec![key_value.clone()])
            .build()
            .unwrap();

        let bytes = original.clone().into_bytes().unwrap();
        let org = Organization::from_bytes(&bytes).unwrap();
        assert_eq!(org, original);
    }

    #[test]
    // check that a OrganizationList is built correctly
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
            .with_address("address".to_string())
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
    // check that a OrganizationList can be converted to bytes and back
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
            .with_address("address".to_string())
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
