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

use super::errors::BuilderError;
use crate::protocol::schema::state::{PropertyDefinition, PropertyValue};
use crate::protos::track_and_trace_state;
use crate::protos::{
    FromBytes, FromNative, FromProto, IntoBytes, IntoNative, IntoProto, ProtoConversionError,
};
use protobuf::Message;
use protobuf::RepeatedField;

#[derive(Debug, Clone, PartialEq)]
pub struct Reporter {
    public_key: String,
    authorized: bool,
    index: u32,
}

impl Reporter {
    pub fn public_key(&self) -> &str {
        &self.public_key
    }
    pub fn authorized(&self) -> &bool {
        &self.authorized
    }
    pub fn index(&self) -> &u32 {
        &self.index
    }
    pub fn into_builder(self) -> ReporterBuilder {
        ReporterBuilder::new()
            .with_public_key(self.public_key)
            .with_authorized(self.authorized)
            .with_index(self.index)
    }
}

#[derive(Default, Clone)]
pub struct ReporterBuilder {
    public_key: Option<String>,
    authorized: Option<bool>,
    index: Option<u32>,
}

impl ReporterBuilder {
    pub fn new() -> Self {
        ReporterBuilder::default()
    }
    pub fn with_public_key(mut self, value: String) -> Self {
        self.public_key = Some(value);
        self
    }
    pub fn with_authorized(mut self, value: bool) -> Self {
        self.authorized = Some(value);
        self
    }
    pub fn with_index(mut self, value: u32) -> Self {
        self.index = Some(value);
        self
    }
    pub fn build(self) -> Result<Reporter, BuilderError> {
        let public_key = self
            .public_key
            .ok_or_else(|| BuilderError::MissingField("public_key".into()))?;
        let authorized = self
            .authorized
            .ok_or_else(|| BuilderError::MissingField("authorized".into()))?;
        let index = self
            .index
            .ok_or_else(|| BuilderError::MissingField("index".into()))?;
        Ok(Reporter {
            public_key,
            authorized,
            index,
        })
    }
}

impl FromProto<track_and_trace_state::Property_Reporter> for Reporter {
    fn from_proto(
        proto: track_and_trace_state::Property_Reporter,
    ) -> Result<Self, ProtoConversionError> {
        Ok(Reporter {
            public_key: proto.get_public_key().to_string(),
            authorized: proto.get_authorized(),
            index: proto.get_index(),
        })
    }
}

impl FromNative<Reporter> for track_and_trace_state::Property_Reporter {
    fn from_native(native: Reporter) -> Result<Self, ProtoConversionError> {
        let mut proto = track_and_trace_state::Property_Reporter::new();
        proto.set_public_key(native.public_key().to_string());
        proto.set_authorized(*native.authorized());
        proto.set_index(*native.index());

        Ok(proto)
    }
}

impl FromBytes<Reporter> for Reporter {
    fn from_bytes(bytes: &[u8]) -> Result<Reporter, ProtoConversionError> {
        let proto: track_and_trace_state::Property_Reporter = protobuf::parse_from_bytes(bytes)
            .map_err(|_| {
                ProtoConversionError::SerializationError("Unable to get Reporter from bytes".into())
            })?;
        proto.into_native()
    }
}
impl IntoBytes for Reporter {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError("Unable to get Reporter from bytes".into())
        })?;
        Ok(bytes)
    }
}
impl IntoProto<track_and_trace_state::Property_Reporter> for Reporter {}
impl IntoNative<Reporter> for track_and_trace_state::Property_Reporter {}

#[derive(Debug, Clone, PartialEq)]
pub struct Property {
    name: String,
    record_id: String,
    property_definition: PropertyDefinition,
    reporters: Vec<Reporter>,
    current_page: u32,
    wrapped: bool,
}

impl Property {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn record_id(&self) -> &str {
        &self.record_id
    }
    pub fn property_definition(&self) -> &PropertyDefinition {
        &self.property_definition
    }
    pub fn reporters(&self) -> &[Reporter] {
        &self.reporters
    }
    pub fn current_page(&self) -> &u32 {
        &self.current_page
    }
    pub fn wrapped(&self) -> &bool {
        &self.wrapped
    }

    pub fn into_builder(self) -> PropertyBuilder {
        PropertyBuilder::new()
            .with_name(self.name)
            .with_record_id(self.record_id)
            .with_property_definition(self.property_definition)
            .with_reporters(self.reporters)
            .with_current_page(self.current_page)
            .with_wrapped(self.wrapped)
    }
}

#[derive(Default, Debug)]
pub struct PropertyBuilder {
    name: Option<String>,
    record_id: Option<String>,
    property_definition: Option<PropertyDefinition>,
    reporters: Option<Vec<Reporter>>,
    current_page: Option<u32>,
    wrapped: Option<bool>,
}

impl PropertyBuilder {
    pub fn new() -> Self {
        PropertyBuilder::default()
    }
    pub fn with_name(mut self, value: String) -> Self {
        self.name = Some(value);
        self
    }
    pub fn with_record_id(mut self, value: String) -> Self {
        self.record_id = Some(value);
        self
    }
    pub fn with_property_definition(mut self, value: PropertyDefinition) -> Self {
        self.property_definition = Some(value);
        self
    }
    pub fn with_reporters(mut self, value: Vec<Reporter>) -> Self {
        self.reporters = Some(value);
        self
    }
    pub fn with_current_page(mut self, value: u32) -> Self {
        self.current_page = Some(value);
        self
    }
    pub fn with_wrapped(mut self, value: bool) -> Self {
        self.wrapped = Some(value);
        self
    }
    pub fn build(self) -> Result<Property, BuilderError> {
        let name = self
            .name
            .ok_or_else(|| BuilderError::MissingField("name".into()))?;
        let record_id = self
            .record_id
            .ok_or_else(|| BuilderError::MissingField("record_id".into()))?;
        let property_definition = self
            .property_definition
            .ok_or_else(|| BuilderError::MissingField("property_definition".into()))?;
        let reporters = self
            .reporters
            .ok_or_else(|| BuilderError::MissingField("reporters".into()))?;
        let current_page = self
            .current_page
            .ok_or_else(|| BuilderError::MissingField("current_page".into()))?;
        let wrapped = self
            .wrapped
            .ok_or_else(|| BuilderError::MissingField("wrapped".into()))?;
        Ok(Property {
            name,
            record_id,
            property_definition,
            reporters,
            current_page,
            wrapped,
        })
    }
}

impl FromProto<track_and_trace_state::Property> for Property {
    fn from_proto(proto: track_and_trace_state::Property) -> Result<Self, ProtoConversionError> {
        Ok(Property {
            name: proto.get_name().to_string(),
            record_id: proto.get_record_id().to_string(),
            property_definition: PropertyDefinition::from_proto(
                proto.get_property_definition().clone(),
            )?,
            reporters: proto
                .get_reporters()
                .to_vec()
                .into_iter()
                .map(Reporter::from_proto)
                .collect::<Result<Vec<Reporter>, ProtoConversionError>>()?,
            current_page: proto.get_current_page(),
            wrapped: proto.get_wrapped(),
        })
    }
}

impl FromNative<Property> for track_and_trace_state::Property {
    fn from_native(native: Property) -> Result<Self, ProtoConversionError> {
        let mut proto = track_and_trace_state::Property::new();
        proto.set_name(native.name().to_string());
        proto.set_record_id(native.record_id().to_string());
        proto.set_property_definition(native.property_definition().clone().into_proto()?);
        proto.set_reporters(RepeatedField::from_vec(
            native.reporters()
            .to_vec()
            .into_iter()
            .map(Reporter::into_proto)
            .collect::<Result<Vec<track_and_trace_state::Property_Reporter>, ProtoConversionError>>()?));
        proto.set_current_page(*native.current_page());
        proto.set_wrapped(*native.wrapped());

        Ok(proto)
    }
}

impl FromBytes<Property> for Property {
    fn from_bytes(bytes: &[u8]) -> Result<Property, ProtoConversionError> {
        let proto: track_and_trace_state::Property =
            protobuf::parse_from_bytes(bytes).map_err(|_| {
                ProtoConversionError::SerializationError("Unable to get Property from bytes".into())
            })?;
        proto.into_native()
    }
}
impl IntoBytes for Property {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError("Unable to get Property from bytes".into())
        })?;
        Ok(bytes)
    }
}
impl IntoProto<track_and_trace_state::Property> for Property {}
impl IntoNative<Property> for track_and_trace_state::Property {}

#[derive(Debug, Clone, PartialEq)]
pub struct PropertyList {
    properties: Vec<Property>,
}

impl PropertyList {
    pub fn properties(&self) -> &[Property] {
        &self.properties
    }

    pub fn into_builder(self) -> PropertyListBuilder {
        PropertyListBuilder::new().with_properties(self.properties)
    }
}

#[derive(Default, Clone)]
pub struct PropertyListBuilder {
    properties: Option<Vec<Property>>,
}

impl PropertyListBuilder {
    pub fn new() -> Self {
        PropertyListBuilder::default()
    }
    pub fn with_properties(mut self, value: Vec<Property>) -> Self {
        self.properties = Some(value);
        self
    }
    pub fn build(self) -> Result<PropertyList, BuilderError> {
        let properties = self
            .properties
            .ok_or_else(|| BuilderError::MissingField("properties".into()))?;
        Ok(PropertyList { properties })
    }
}

impl FromProto<track_and_trace_state::PropertyList> for PropertyList {
    fn from_proto(
        proto: track_and_trace_state::PropertyList,
    ) -> Result<Self, ProtoConversionError> {
        Ok(PropertyList {
            properties: proto
                .get_entries()
                .to_vec()
                .into_iter()
                .map(Property::from_proto)
                .collect::<Result<Vec<Property>, ProtoConversionError>>()?,
        })
    }
}

impl FromNative<PropertyList> for track_and_trace_state::PropertyList {
    fn from_native(native: PropertyList) -> Result<Self, ProtoConversionError> {
        let mut proto = track_and_trace_state::PropertyList::new();
        proto.set_entries(RepeatedField::from_vec(
            native
                .properties()
                .to_vec()
                .into_iter()
                .map(Property::into_proto)
                .collect::<Result<Vec<track_and_trace_state::Property>, ProtoConversionError>>()?,
        ));

        Ok(proto)
    }
}

impl FromBytes<PropertyList> for PropertyList {
    fn from_bytes(bytes: &[u8]) -> Result<PropertyList, ProtoConversionError> {
        let proto: track_and_trace_state::PropertyList = protobuf::parse_from_bytes(bytes)
            .map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get PropertyList from Bytes".into(),
                )
            })?;
        proto.into_native()
    }
}
impl IntoBytes for PropertyList {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError("Unable to get PropertyList from Bytes".into())
        })?;
        Ok(bytes)
    }
}
impl IntoProto<track_and_trace_state::PropertyList> for PropertyList {}
impl IntoNative<PropertyList> for track_and_trace_state::PropertyList {}

#[derive(Debug, Clone, PartialEq)]
pub struct ReportedValue {
    reporter_index: u32,
    timestamp: u64,
    value: PropertyValue,
}

impl ReportedValue {
    pub fn reporter_index(&self) -> &u32 {
        &self.reporter_index
    }
    pub fn timestamp(&self) -> &u64 {
        &self.timestamp
    }
    pub fn value(&self) -> &PropertyValue {
        &self.value
    }
    pub fn into_builder(self) -> ReportedValueBuilder {
        ReportedValueBuilder::new()
            .with_reporter_index(self.reporter_index)
            .with_timestamp(self.timestamp)
            .with_value(self.value)
    }
}

#[derive(Default, Debug)]
pub struct ReportedValueBuilder {
    reporter_index: Option<u32>,
    timestamp: Option<u64>,
    value: Option<PropertyValue>,
}

impl ReportedValueBuilder {
    pub fn new() -> Self {
        ReportedValueBuilder::default()
    }
    pub fn with_reporter_index(mut self, value: u32) -> Self {
        self.reporter_index = Some(value);
        self
    }
    pub fn with_timestamp(mut self, value: u64) -> Self {
        self.timestamp = Some(value);
        self
    }
    pub fn with_value(mut self, value: PropertyValue) -> Self {
        self.value = Some(value);
        self
    }
    pub fn build(self) -> Result<ReportedValue, BuilderError> {
        let reporter_index = self
            .reporter_index
            .ok_or_else(|| BuilderError::MissingField("reporter_index".into()))?;
        let timestamp = self
            .timestamp
            .ok_or_else(|| BuilderError::MissingField("timestamp".into()))?;
        let value = self
            .value
            .ok_or_else(|| BuilderError::MissingField("value".into()))?;
        Ok(ReportedValue {
            reporter_index,
            timestamp,
            value,
        })
    }
}

impl FromProto<track_and_trace_state::PropertyPage_ReportedValue> for ReportedValue {
    fn from_proto(
        proto: track_and_trace_state::PropertyPage_ReportedValue,
    ) -> Result<Self, ProtoConversionError> {
        Ok(ReportedValue {
            reporter_index: proto.get_reporter_index(),
            timestamp: proto.get_timestamp(),
            value: PropertyValue::from_proto(proto.get_value().clone())?,
        })
    }
}

impl FromNative<ReportedValue> for track_and_trace_state::PropertyPage_ReportedValue {
    fn from_native(native: ReportedValue) -> Result<Self, ProtoConversionError> {
        let mut proto = track_and_trace_state::PropertyPage_ReportedValue::new();
        proto.set_reporter_index(*native.reporter_index());
        proto.set_timestamp(*native.timestamp());
        proto.set_value(native.value().clone().into_proto()?);

        Ok(proto)
    }
}

impl FromBytes<ReportedValue> for ReportedValue {
    fn from_bytes(bytes: &[u8]) -> Result<ReportedValue, ProtoConversionError> {
        let proto: track_and_trace_state::PropertyPage_ReportedValue =
            protobuf::parse_from_bytes(bytes).map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get ReportedValue from bytes".into(),
                )
            })?;
        proto.into_native()
    }
}
impl IntoBytes for ReportedValue {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get ReportedValue from bytes".into(),
            )
        })?;
        Ok(bytes)
    }
}
impl IntoProto<track_and_trace_state::PropertyPage_ReportedValue> for ReportedValue {}
impl IntoNative<ReportedValue> for track_and_trace_state::PropertyPage_ReportedValue {}

#[derive(Debug, Clone, PartialEq)]
pub struct PropertyPage {
    name: String,
    record_id: String,
    reported_values: Vec<ReportedValue>,
}

impl PropertyPage {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn record_id(&self) -> &str {
        &self.record_id
    }
    pub fn reported_values(&self) -> &[ReportedValue] {
        &self.reported_values
    }

    pub fn into_builder(self) -> PropertyPageBuilder {
        PropertyPageBuilder::new()
            .with_name(self.name)
            .with_record_id(self.record_id)
            .with_reported_values(self.reported_values)
    }
}

#[derive(Default, Debug)]
pub struct PropertyPageBuilder {
    name: Option<String>,
    record_id: Option<String>,
    reported_values: Option<Vec<ReportedValue>>,
}

impl PropertyPageBuilder {
    pub fn new() -> Self {
        PropertyPageBuilder::default()
    }
    pub fn with_name(mut self, value: String) -> Self {
        self.name = Some(value);
        self
    }
    pub fn with_record_id(mut self, value: String) -> Self {
        self.record_id = Some(value);
        self
    }
    pub fn with_reported_values(mut self, value: Vec<ReportedValue>) -> Self {
        self.reported_values = Some(value);
        self
    }
    pub fn build(self) -> Result<PropertyPage, BuilderError> {
        let name = self
            .name
            .ok_or_else(|| BuilderError::MissingField("name".into()))?;
        let record_id = self
            .record_id
            .ok_or_else(|| BuilderError::MissingField("record_id".into()))?;
        let reported_values = self
            .reported_values
            .ok_or_else(|| BuilderError::MissingField("reported_values".into()))?;
        Ok(PropertyPage {
            name,
            record_id,
            reported_values,
        })
    }
}

impl FromProto<track_and_trace_state::PropertyPage> for PropertyPage {
    fn from_proto(
        proto: track_and_trace_state::PropertyPage,
    ) -> Result<Self, ProtoConversionError> {
        Ok(PropertyPage {
            name: proto.get_name().to_string(),
            record_id: proto.get_record_id().to_string(),
            reported_values: proto
                .get_reported_values()
                .to_vec()
                .into_iter()
                .map(ReportedValue::from_proto)
                .collect::<Result<Vec<ReportedValue>, ProtoConversionError>>()?,
        })
    }
}

impl FromNative<PropertyPage> for track_and_trace_state::PropertyPage {
    fn from_native(native: PropertyPage) -> Result<Self, ProtoConversionError> {
        let mut proto = track_and_trace_state::PropertyPage::new();
        proto.set_name(native.name().to_string());
        proto.set_record_id(native.record_id().to_string());
        proto.set_reported_values(RepeatedField::from_vec(
            native
                .reported_values()
                .to_vec()
                .into_iter()
                .map(ReportedValue::into_proto)
                .collect::<Result<
                    Vec<track_and_trace_state::PropertyPage_ReportedValue>,
                    ProtoConversionError,
                >>()?,
        ));

        Ok(proto)
    }
}

impl FromBytes<PropertyPage> for PropertyPage {
    fn from_bytes(bytes: &[u8]) -> Result<PropertyPage, ProtoConversionError> {
        let proto: track_and_trace_state::PropertyPage = protobuf::parse_from_bytes(bytes)
            .map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Undable to get PropertyPage from bytes".into(),
                )
            })?;
        proto.into_native()
    }
}
impl IntoBytes for PropertyPage {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Undable to get PropertyPage from bytes".into(),
            )
        })?;
        Ok(bytes)
    }
}
impl IntoProto<track_and_trace_state::PropertyPage> for PropertyPage {}
impl IntoNative<PropertyPage> for track_and_trace_state::PropertyPage {}

#[derive(Debug, Clone, PartialEq)]
pub struct PropertyPageList {
    property_pages: Vec<PropertyPage>,
}

impl PropertyPageList {
    pub fn property_pages(&self) -> &[PropertyPage] {
        &self.property_pages
    }

    pub fn into_builder(self) -> PropertyPageListBuilder {
        PropertyPageListBuilder::new().with_property_pages(self.property_pages)
    }
}

#[derive(Default, Debug)]
pub struct PropertyPageListBuilder {
    property_pages: Option<Vec<PropertyPage>>,
}

impl PropertyPageListBuilder {
    pub fn new() -> Self {
        PropertyPageListBuilder::default()
    }
    pub fn with_property_pages(mut self, value: Vec<PropertyPage>) -> Self {
        self.property_pages = Some(value);
        self
    }
    pub fn build(self) -> Result<PropertyPageList, BuilderError> {
        let property_pages = self
            .property_pages
            .ok_or_else(|| BuilderError::MissingField("property_pages".into()))?;
        Ok(PropertyPageList { property_pages })
    }
}

impl FromProto<track_and_trace_state::PropertyPageList> for PropertyPageList {
    fn from_proto(
        proto: track_and_trace_state::PropertyPageList,
    ) -> Result<Self, ProtoConversionError> {
        Ok(PropertyPageList {
            property_pages: proto
                .get_entries()
                .to_vec()
                .into_iter()
                .map(PropertyPage::from_proto)
                .collect::<Result<Vec<PropertyPage>, ProtoConversionError>>()?,
        })
    }
}

impl FromNative<PropertyPageList> for track_and_trace_state::PropertyPageList {
    fn from_native(native: PropertyPageList) -> Result<Self, ProtoConversionError> {
        let mut proto = track_and_trace_state::PropertyPageList::new();
        proto.set_entries(RepeatedField::from_vec(
            native
                .property_pages()
                .to_vec()
                .into_iter()
                .map(PropertyPage::into_proto)
                .collect::<Result<Vec<track_and_trace_state::PropertyPage>, ProtoConversionError>>(
                )?,
        ));

        Ok(proto)
    }
}

impl FromBytes<PropertyPageList> for PropertyPageList {
    fn from_bytes(bytes: &[u8]) -> Result<PropertyPageList, ProtoConversionError> {
        let proto: track_and_trace_state::PropertyPageList = protobuf::parse_from_bytes(bytes)
            .map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get PropertyPageList from Bytes".into(),
                )
            })?;
        proto.into_native()
    }
}
impl IntoBytes for PropertyPageList {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get PropertyPageList from Bytes".into(),
            )
        })?;
        Ok(bytes)
    }
}
impl IntoProto<track_and_trace_state::PropertyPageList> for PropertyPageList {}
impl IntoNative<PropertyPageList> for track_and_trace_state::PropertyPageList {}

#[derive(Debug, Clone, PartialEq)]
pub enum Role {
    Owner,
    Custodian,
    Reporter,
}

impl Default for Role {
    fn default() -> Role {
        Role::Owner
    }
}

impl FromProto<track_and_trace_state::Proposal_Role> for Role {
    fn from_proto(
        roles: track_and_trace_state::Proposal_Role,
    ) -> Result<Self, ProtoConversionError> {
        match roles {
            track_and_trace_state::Proposal_Role::OWNER => Ok(Role::Owner),
            track_and_trace_state::Proposal_Role::CUSTODIAN => Ok(Role::Custodian),
            track_and_trace_state::Proposal_Role::REPORTER => Ok(Role::Reporter),
        }
    }
}

impl FromNative<Role> for track_and_trace_state::Proposal_Role {
    fn from_native(roles: Role) -> Result<Self, ProtoConversionError> {
        match roles {
            Role::Owner => Ok(track_and_trace_state::Proposal_Role::OWNER),
            Role::Custodian => Ok(track_and_trace_state::Proposal_Role::CUSTODIAN),
            Role::Reporter => Ok(track_and_trace_state::Proposal_Role::REPORTER),
        }
    }
}

impl IntoProto<track_and_trace_state::Proposal_Role> for Role {}
impl IntoNative<Role> for track_and_trace_state::Proposal_Role {}

#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    Open,
    Accepted,
    Rejected,
    Canceled,
}

impl Default for Status {
    fn default() -> Status {
        Status::Open
    }
}

impl FromProto<track_and_trace_state::Proposal_Status> for Status {
    fn from_proto(
        statuses: track_and_trace_state::Proposal_Status,
    ) -> Result<Self, ProtoConversionError> {
        match statuses {
            track_and_trace_state::Proposal_Status::OPEN => Ok(Status::Open),
            track_and_trace_state::Proposal_Status::ACCEPTED => Ok(Status::Accepted),
            track_and_trace_state::Proposal_Status::REJECTED => Ok(Status::Rejected),
            track_and_trace_state::Proposal_Status::CANCELED => Ok(Status::Canceled),
        }
    }
}

impl FromNative<Status> for track_and_trace_state::Proposal_Status {
    fn from_native(statuses: Status) -> Result<Self, ProtoConversionError> {
        match statuses {
            Status::Open => Ok(track_and_trace_state::Proposal_Status::OPEN),
            Status::Accepted => Ok(track_and_trace_state::Proposal_Status::ACCEPTED),
            Status::Rejected => Ok(track_and_trace_state::Proposal_Status::REJECTED),
            Status::Canceled => Ok(track_and_trace_state::Proposal_Status::CANCELED),
        }
    }
}

impl IntoProto<track_and_trace_state::Proposal_Status> for Status {}
impl IntoNative<Status> for track_and_trace_state::Proposal_Status {}

#[derive(Debug, Clone, PartialEq)]
pub struct Proposal {
    record_id: String,
    timestamp: u64,
    issuing_agent: String,
    receiving_agent: String,
    role: Role,
    properties: Vec<String>,
    status: Status,
    terms: String,
}

impl Proposal {
    pub fn record_id(&self) -> &str {
        &self.record_id
    }
    pub fn timestamp(&self) -> &u64 {
        &self.timestamp
    }
    pub fn issuing_agent(&self) -> &str {
        &self.issuing_agent
    }
    pub fn receiving_agent(&self) -> &str {
        &self.receiving_agent
    }
    pub fn role(&self) -> &Role {
        &self.role
    }
    pub fn properties(&self) -> &[String] {
        &self.properties
    }
    pub fn status(&self) -> &Status {
        &self.status
    }
    pub fn terms(&self) -> &str {
        &self.terms
    }
    pub fn into_builder(self) -> ProposalBuilder {
        ProposalBuilder::new()
            .with_record_id(self.record_id)
            .with_timestamp(self.timestamp)
            .with_issuing_agent(self.issuing_agent)
            .with_receiving_agent(self.receiving_agent)
            .with_role(self.role)
            .with_properties(self.properties)
            .with_status(self.status)
            .with_terms(self.terms)
    }
}

#[derive(Default, Debug)]
pub struct ProposalBuilder {
    record_id: Option<String>,
    timestamp: Option<u64>,
    issuing_agent: Option<String>,
    receiving_agent: Option<String>,
    role: Option<Role>,
    properties: Option<Vec<String>>,
    status: Option<Status>,
    terms: Option<String>,
}

impl ProposalBuilder {
    pub fn new() -> Self {
        ProposalBuilder::default()
    }
    pub fn with_record_id(mut self, value: String) -> Self {
        self.record_id = Some(value);
        self
    }
    pub fn with_timestamp(mut self, value: u64) -> Self {
        self.timestamp = Some(value);
        self
    }
    pub fn with_issuing_agent(mut self, value: String) -> Self {
        self.issuing_agent = Some(value);
        self
    }
    pub fn with_receiving_agent(mut self, value: String) -> Self {
        self.receiving_agent = Some(value);
        self
    }
    pub fn with_role(mut self, value: Role) -> Self {
        self.role = Some(value);
        self
    }
    pub fn with_properties(mut self, value: Vec<String>) -> Self {
        self.properties = Some(value);
        self
    }
    pub fn with_status(mut self, value: Status) -> Self {
        self.status = Some(value);
        self
    }
    pub fn with_terms(mut self, value: String) -> Self {
        self.terms = Some(value);
        self
    }
    pub fn build(self) -> Result<Proposal, BuilderError> {
        let record_id = self
            .record_id
            .ok_or_else(|| BuilderError::MissingField("record_id".into()))?;
        let timestamp = self
            .timestamp
            .ok_or_else(|| BuilderError::MissingField("timestamp".into()))?;
        let issuing_agent = self
            .issuing_agent
            .ok_or_else(|| BuilderError::MissingField("issuing_agent".into()))?;
        let receiving_agent = self
            .receiving_agent
            .ok_or_else(|| BuilderError::MissingField("receiving_agent".into()))?;
        let role = self
            .role
            .ok_or_else(|| BuilderError::MissingField("role".into()))?;
        let properties = self
            .properties
            .ok_or_else(|| BuilderError::MissingField("properties".into()))?;
        let status = self
            .status
            .ok_or_else(|| BuilderError::MissingField("status".into()))?;
        let terms = self
            .terms
            .ok_or_else(|| BuilderError::MissingField("terms".into()))?;
        Ok(Proposal {
            record_id,
            timestamp,
            issuing_agent,
            receiving_agent,
            role,
            properties,
            status,
            terms,
        })
    }
}

impl FromProto<track_and_trace_state::Proposal> for Proposal {
    fn from_proto(proto: track_and_trace_state::Proposal) -> Result<Self, ProtoConversionError> {
        Ok(Proposal {
            record_id: proto.get_record_id().to_string(),
            timestamp: proto.get_timestamp(),
            issuing_agent: proto.get_issuing_agent().to_string(),
            receiving_agent: proto.get_receiving_agent().to_string(),
            role: Role::from_proto(proto.get_role())?,
            properties: proto
                .get_properties()
                .to_vec()
                .into_iter()
                .map(String::from)
                .collect(),
            status: Status::from_proto(proto.get_status())?,
            terms: proto.get_terms().to_string(),
        })
    }
}

impl FromNative<Proposal> for track_and_trace_state::Proposal {
    fn from_native(native: Proposal) -> Result<Self, ProtoConversionError> {
        let mut proto = track_and_trace_state::Proposal::new();

        proto.set_record_id(native.record_id().to_string());
        proto.set_timestamp(*native.timestamp());
        proto.set_issuing_agent(native.issuing_agent().to_string());
        proto.set_receiving_agent(native.receiving_agent().to_string());
        proto.set_role(native.role().clone().into_proto()?);
        proto.set_properties(RepeatedField::from_vec(native.properties().to_vec()));
        proto.set_status(native.status().clone().into_proto()?);
        proto.set_terms(native.terms().to_string());

        Ok(proto)
    }
}

impl FromBytes<Proposal> for Proposal {
    fn from_bytes(bytes: &[u8]) -> Result<Proposal, ProtoConversionError> {
        let proto: track_and_trace_state::Proposal =
            protobuf::parse_from_bytes(bytes).map_err(|_| {
                ProtoConversionError::SerializationError("Unable to get Proposal from bytes".into())
            })?;
        proto.into_native()
    }
}

impl IntoBytes for Proposal {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError("Unable to get Proposal from bytes".into())
        })?;
        Ok(bytes)
    }
}
impl IntoProto<track_and_trace_state::Proposal> for Proposal {}
impl IntoNative<Proposal> for track_and_trace_state::Proposal {}

#[derive(Debug, Clone, PartialEq)]
pub struct ProposalList {
    proposals: Vec<Proposal>,
}

impl ProposalList {
    pub fn proposals(&self) -> &[Proposal] {
        &self.proposals
    }

    pub fn into_builder(self) -> ProposalListBuilder {
        ProposalListBuilder::new().with_proposals(self.proposals)
    }
}

#[derive(Default, Debug)]
pub struct ProposalListBuilder {
    proposals: Option<Vec<Proposal>>,
}

impl ProposalListBuilder {
    pub fn new() -> Self {
        ProposalListBuilder::default()
    }
    pub fn with_proposals(mut self, value: Vec<Proposal>) -> Self {
        self.proposals = Some(value);
        self
    }
    pub fn build(self) -> Result<ProposalList, BuilderError> {
        let proposals = self
            .proposals
            .ok_or_else(|| BuilderError::MissingField("proposals".into()))?;
        Ok(ProposalList { proposals })
    }
}

impl FromProto<track_and_trace_state::ProposalList> for ProposalList {
    fn from_proto(
        proto: track_and_trace_state::ProposalList,
    ) -> Result<Self, ProtoConversionError> {
        Ok(ProposalList {
            proposals: proto
                .get_entries()
                .to_vec()
                .into_iter()
                .map(Proposal::from_proto)
                .collect::<Result<Vec<Proposal>, ProtoConversionError>>()?,
        })
    }
}

impl FromNative<ProposalList> for track_and_trace_state::ProposalList {
    fn from_native(native: ProposalList) -> Result<Self, ProtoConversionError> {
        let mut proto = track_and_trace_state::ProposalList::new();
        proto.set_entries(RepeatedField::from_vec(
            native
                .proposals()
                .to_vec()
                .into_iter()
                .map(Proposal::into_proto)
                .collect::<Result<Vec<track_and_trace_state::Proposal>, ProtoConversionError>>()?,
        ));

        Ok(proto)
    }
}

impl FromBytes<ProposalList> for ProposalList {
    fn from_bytes(bytes: &[u8]) -> Result<ProposalList, ProtoConversionError> {
        let proto: track_and_trace_state::ProposalList = protobuf::parse_from_bytes(bytes)
            .map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get ProposalList from bytes".into(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for ProposalList {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError("Unable to get ProposalList from bytes".into())
        })?;
        Ok(bytes)
    }
}

impl IntoProto<track_and_trace_state::ProposalList> for ProposalList {}
impl IntoNative<ProposalList> for track_and_trace_state::ProposalList {}

#[derive(Debug, Clone, PartialEq)]
pub struct AssociatedAgent {
    agent_id: String,
    timestamp: u64,
}

impl AssociatedAgent {
    pub fn agent_id(&self) -> &str {
        &self.agent_id
    }
    pub fn timestamp(&self) -> &u64 {
        &self.timestamp
    }

    pub fn into_builder(self) -> AssociatedAgentBuilder {
        AssociatedAgentBuilder::new()
            .with_agent_id(self.agent_id)
            .with_timestamp(self.timestamp)
    }
}

#[derive(Default, Debug)]
pub struct AssociatedAgentBuilder {
    agent_id: Option<String>,
    timestamp: Option<u64>,
}

impl AssociatedAgentBuilder {
    pub fn new() -> Self {
        AssociatedAgentBuilder::default()
    }
    pub fn with_agent_id(mut self, value: String) -> Self {
        self.agent_id = Some(value);
        self
    }
    pub fn with_timestamp(mut self, value: u64) -> Self {
        self.timestamp = Some(value);
        self
    }
    pub fn build(self) -> Result<AssociatedAgent, BuilderError> {
        let agent_id = self
            .agent_id
            .ok_or_else(|| BuilderError::MissingField("agent_id".into()))?;
        let timestamp = self
            .timestamp
            .ok_or_else(|| BuilderError::MissingField("timestamp".into()))?;
        Ok(AssociatedAgent {
            agent_id,
            timestamp,
        })
    }
}

impl FromProto<track_and_trace_state::Record_AssociatedAgent> for AssociatedAgent {
    fn from_proto(
        proto: track_and_trace_state::Record_AssociatedAgent,
    ) -> Result<Self, ProtoConversionError> {
        Ok(AssociatedAgent {
            agent_id: proto.get_agent_id().to_string(),
            timestamp: proto.get_timestamp(),
        })
    }
}

impl FromNative<AssociatedAgent> for track_and_trace_state::Record_AssociatedAgent {
    fn from_native(native: AssociatedAgent) -> Result<Self, ProtoConversionError> {
        let mut proto = track_and_trace_state::Record_AssociatedAgent::new();

        proto.set_agent_id(native.agent_id().to_string());
        proto.set_timestamp(*native.timestamp());

        Ok(proto)
    }
}

impl FromBytes<AssociatedAgent> for AssociatedAgent {
    fn from_bytes(bytes: &[u8]) -> Result<AssociatedAgent, ProtoConversionError> {
        let proto: track_and_trace_state::Record_AssociatedAgent =
            protobuf::parse_from_bytes(bytes).map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get AssociatedAgent from bytes".into(),
                )
            })?;
        proto.into_native()
    }
}
impl IntoBytes for AssociatedAgent {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get AssociatedAgent from bytes".into(),
            )
        })?;
        Ok(bytes)
    }
}
impl IntoProto<track_and_trace_state::Record_AssociatedAgent> for AssociatedAgent {}
impl IntoNative<AssociatedAgent> for track_and_trace_state::Record_AssociatedAgent {}

#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    record_id: String,
    schema: String,
    owners: Vec<AssociatedAgent>,
    custodians: Vec<AssociatedAgent>,
    field_final: bool,
}

impl Record {
    pub fn record_id(&self) -> &str {
        &self.record_id
    }
    pub fn schema(&self) -> &str {
        &self.schema
    }
    pub fn owners(&self) -> &[AssociatedAgent] {
        &self.owners
    }
    pub fn custodians(&self) -> &[AssociatedAgent] {
        &self.custodians
    }
    pub fn field_final(&self) -> &bool {
        &self.field_final
    }
    pub fn into_builder(self) -> RecordBuilder {
        RecordBuilder::new()
            .with_record_id(self.record_id)
            .with_schema(self.schema)
            .with_owners(self.owners)
            .with_custodians(self.custodians)
            .with_field_final(self.field_final)
    }
}

#[derive(Default, Debug)]
pub struct RecordBuilder {
    record_id: Option<String>,
    schema: Option<String>,
    owners: Option<Vec<AssociatedAgent>>,
    custodians: Option<Vec<AssociatedAgent>>,
    field_final: Option<bool>,
}

impl RecordBuilder {
    pub fn new() -> Self {
        RecordBuilder::default()
    }
    pub fn with_record_id(mut self, value: String) -> Self {
        self.record_id = Some(value);
        self
    }
    pub fn with_schema(mut self, value: String) -> Self {
        self.schema = Some(value);
        self
    }
    pub fn with_owners(mut self, value: Vec<AssociatedAgent>) -> Self {
        self.owners = Some(value);
        self
    }
    pub fn with_custodians(mut self, value: Vec<AssociatedAgent>) -> Self {
        self.custodians = Some(value);
        self
    }
    pub fn with_field_final(mut self, value: bool) -> Self {
        self.field_final = Some(value);
        self
    }
    pub fn build(self) -> Result<Record, BuilderError> {
        let record_id = self
            .record_id
            .ok_or_else(|| BuilderError::MissingField("record_id".into()))?;
        let schema = self
            .schema
            .ok_or_else(|| BuilderError::MissingField("schema".into()))?;
        let owners = self
            .owners
            .ok_or_else(|| BuilderError::MissingField("owners".into()))?;
        let custodians = self
            .custodians
            .ok_or_else(|| BuilderError::MissingField("custodians".into()))?;
        let field_final = self
            .field_final
            .ok_or_else(|| BuilderError::MissingField("field_final".into()))?;
        Ok(Record {
            record_id,
            schema,
            owners,
            custodians,
            field_final,
        })
    }
}

impl FromProto<track_and_trace_state::Record> for Record {
    fn from_proto(proto: track_and_trace_state::Record) -> Result<Self, ProtoConversionError> {
        Ok(Record {
            record_id: proto.get_record_id().to_string(),
            schema: proto.get_schema().to_string(),
            owners: proto
                .get_owners()
                .to_vec()
                .into_iter()
                .map(AssociatedAgent::from_proto)
                .collect::<Result<Vec<AssociatedAgent>, ProtoConversionError>>()?,
            custodians: proto
                .get_custodians()
                .to_vec()
                .into_iter()
                .map(AssociatedAgent::from_proto)
                .collect::<Result<Vec<AssociatedAgent>, ProtoConversionError>>()?,
            field_final: proto.get_field_final(),
        })
    }
}

impl FromNative<Record> for track_and_trace_state::Record {
    fn from_native(native: Record) -> Result<Self, ProtoConversionError> {
        let mut proto = track_and_trace_state::Record::new();
        proto.set_record_id(native.record_id().to_string());
        proto.set_schema(native.schema().to_string());
        proto.set_owners(
            RepeatedField::from_vec(
                native
                    .owners()
                    .to_vec()
                    .into_iter()
                    .map(AssociatedAgent::into_proto)
                    .collect::<Result<
                        Vec<track_and_trace_state::Record_AssociatedAgent>,
                        ProtoConversionError,
                    >>()?,
            ),
        );
        proto.set_custodians(
            RepeatedField::from_vec(
                native
                    .custodians()
                    .to_vec()
                    .into_iter()
                    .map(AssociatedAgent::into_proto)
                    .collect::<Result<
                        Vec<track_and_trace_state::Record_AssociatedAgent>,
                        ProtoConversionError,
                    >>()?,
            ),
        );
        proto.set_field_final(*native.field_final());

        Ok(proto)
    }
}

impl FromBytes<Record> for Record {
    fn from_bytes(bytes: &[u8]) -> Result<Record, ProtoConversionError> {
        let proto: track_and_trace_state::Record =
            protobuf::parse_from_bytes(bytes).map_err(|_| {
                ProtoConversionError::SerializationError("Unable to get Record from bytes".into())
            })?;
        proto.into_native()
    }
}
impl IntoBytes for Record {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError("Unable to get Record from bytes".into())
        })?;
        Ok(bytes)
    }
}
impl IntoProto<track_and_trace_state::Record> for Record {}
impl IntoNative<Record> for track_and_trace_state::Record {}

#[derive(Debug, Clone, PartialEq)]
pub struct RecordList {
    records: Vec<Record>,
}

impl RecordList {
    pub fn records(&self) -> &[Record] {
        &self.records
    }

    pub fn into_builder(self) -> RecordListBuilder {
        RecordListBuilder::new().with_records(self.records)
    }
}

#[derive(Default, Debug)]
pub struct RecordListBuilder {
    records: Option<Vec<Record>>,
}

impl RecordListBuilder {
    pub fn new() -> Self {
        RecordListBuilder::default()
    }
    pub fn with_records(mut self, value: Vec<Record>) -> Self {
        self.records = Some(value);
        self
    }
    pub fn build(self) -> Result<RecordList, BuilderError> {
        let records = self
            .records
            .ok_or_else(|| BuilderError::MissingField("records".into()))?;
        Ok(RecordList { records })
    }
}

impl FromProto<track_and_trace_state::RecordList> for RecordList {
    fn from_proto(proto: track_and_trace_state::RecordList) -> Result<Self, ProtoConversionError> {
        Ok(RecordList {
            records: proto
                .get_entries()
                .to_vec()
                .into_iter()
                .map(Record::from_proto)
                .collect::<Result<Vec<Record>, ProtoConversionError>>()?,
        })
    }
}

impl FromNative<RecordList> for track_and_trace_state::RecordList {
    fn from_native(native: RecordList) -> Result<Self, ProtoConversionError> {
        let mut proto = track_and_trace_state::RecordList::new();
        proto.set_entries(RepeatedField::from_vec(
            native
                .records()
                .to_vec()
                .into_iter()
                .map(Record::into_proto)
                .collect::<Result<Vec<track_and_trace_state::Record>, ProtoConversionError>>()?,
        ));

        Ok(proto)
    }
}

impl FromBytes<RecordList> for RecordList {
    fn from_bytes(bytes: &[u8]) -> Result<RecordList, ProtoConversionError> {
        let proto: track_and_trace_state::RecordList =
            protobuf::parse_from_bytes(bytes).map_err(|_| {
                ProtoConversionError::SerializationError("Unable to get Record from bytes".into())
            })?;
        proto.into_native()
    }
}

impl IntoBytes for RecordList {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError("Unable to get Record from bytes".into())
        })?;
        Ok(bytes)
    }
}
impl IntoProto<track_and_trace_state::RecordList> for RecordList {}
impl IntoNative<RecordList> for track_and_trace_state::RecordList {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::schema::state::{
        DataType, PropertyDefinitionBuilder, PropertyValueBuilder,
    };
    use std::fmt::Debug;

    fn test_from_bytes<T: FromBytes<T> + Clone + PartialEq + IntoBytes + Debug, F>(
        under_test: T,
        from_bytes: F,
    ) where
        F: Fn(&[u8]) -> Result<T, ProtoConversionError>,
    {
        let bytes = under_test.clone().into_bytes().unwrap();
        let created_from_bytes = from_bytes(&bytes).unwrap();
        assert_eq!(under_test, created_from_bytes);
    }

    #[test]
    fn test_reporter_builder() {
        let builder = ReporterBuilder::new();
        let reporter = builder
            .with_public_key("1234".to_string())
            .with_authorized(true)
            .with_index(0)
            .build()
            .unwrap();

        assert_eq!(reporter.public_key(), "1234");
        assert_eq!(*reporter.authorized(), true);
        assert_eq!(*reporter.index(), 0);
    }

    #[test]
    fn test_reporter_into_builder() {
        let reporter = ReporterBuilder::new()
            .with_public_key("1234".to_string())
            .with_authorized(true)
            .with_index(0)
            .build()
            .unwrap();

        let builder = reporter.into_builder();

        assert_eq!(builder.public_key, Some("1234".to_string()));
        assert_eq!(builder.authorized, Some(true));
        assert_eq!(builder.index, Some(0));
    }

    #[test]
    fn test_reporter_bytes() {
        let builder = ReporterBuilder::new();
        let original = builder
            .with_public_key("1234".to_string())
            .with_authorized(true)
            .with_index(0)
            .build()
            .unwrap();

        test_from_bytes(original, Reporter::from_bytes);
    }

    #[test]
    fn test_property_builder() {
        let property_definition = PropertyDefinitionBuilder::new()
            .with_name("i dunno".into())
            .with_data_type(DataType::String)
            .with_required(true)
            .with_description("test".into())
            .build()
            .unwrap();

        let reporter = ReporterBuilder::new()
            .with_public_key("1234".to_string())
            .with_authorized(true)
            .with_index(0)
            .build()
            .unwrap();

        let property = PropertyBuilder::new()
            .with_name("taco".into())
            .with_record_id("taco1234".into())
            .with_property_definition(property_definition.clone())
            .with_reporters(vec![reporter.clone()])
            .with_current_page(0)
            .with_wrapped(true)
            .build()
            .unwrap();

        assert_eq!(property.name(), "taco");
        assert_eq!(property.record_id(), "taco1234");
        assert_eq!(*property.property_definition(), property_definition);
        assert!(property.reporters().iter().any(|x| *x == reporter));
        assert_eq!(*property.current_page(), 0);
        assert_eq!(*property.wrapped(), true);
    }

    #[test]
    fn test_property_into_builder() {
        let property_definition = PropertyDefinitionBuilder::new()
            .with_name("i dunno".into())
            .with_data_type(DataType::String)
            .with_required(true)
            .with_description("test".into())
            .build()
            .unwrap();

        let reporter = ReporterBuilder::new()
            .with_public_key("1234".to_string())
            .with_authorized(true)
            .with_index(0)
            .build()
            .unwrap();

        let property = PropertyBuilder::new()
            .with_name("taco".into())
            .with_record_id("taco1234".into())
            .with_property_definition(property_definition.clone())
            .with_reporters(vec![reporter.clone()])
            .with_current_page(0)
            .with_wrapped(true)
            .build()
            .unwrap();

        let builder = property.into_builder();

        assert_eq!(builder.name, Some("taco".to_string()));
        assert_eq!(builder.record_id, Some("taco1234".to_string()));
        assert_eq!(builder.property_definition, Some(property_definition));
        assert_eq!(builder.reporters, Some(vec![reporter]));
        assert_eq!(builder.current_page, Some(0));
        assert_eq!(builder.wrapped, Some(true));
    }

    #[test]
    fn test_property_bytes() {
        let property_definition = PropertyDefinitionBuilder::new()
            .with_name("i dunno".into())
            .with_data_type(DataType::String)
            .with_required(true)
            .with_description("test".into())
            .build()
            .unwrap();

        let reporter = ReporterBuilder::new()
            .with_public_key("1234".to_string())
            .with_authorized(true)
            .with_index(0)
            .build()
            .unwrap();

        let property = PropertyBuilder::new()
            .with_name("taco".into())
            .with_record_id("taco1234".into())
            .with_property_definition(property_definition.clone())
            .with_reporters(vec![reporter.clone()])
            .with_current_page(0)
            .with_wrapped(true)
            .build()
            .unwrap();

        test_from_bytes(property, Property::from_bytes);
    }

    #[test]
    fn test_property_list_builder() {
        let property_definition = PropertyDefinitionBuilder::new()
            .with_name("i dunno".into())
            .with_data_type(DataType::String)
            .with_required(true)
            .with_description("test".into())
            .build()
            .unwrap();

        let reporter = ReporterBuilder::new()
            .with_public_key("1234".to_string())
            .with_authorized(true)
            .with_index(0)
            .build()
            .unwrap();

        let property = PropertyBuilder::new()
            .with_name("taco".into())
            .with_record_id("taco1234".into())
            .with_property_definition(property_definition.clone())
            .with_reporters(vec![reporter.clone()])
            .with_current_page(0)
            .with_wrapped(true)
            .build()
            .unwrap();

        let property_list = PropertyListBuilder::new()
            .with_properties(vec![property.clone()])
            .build()
            .unwrap();

        assert!(property_list.properties().iter().any(|x| *x == property));
    }

    #[test]
    fn test_property_list_into_builder() {
        let property_definition = PropertyDefinitionBuilder::new()
            .with_name("i dunno".into())
            .with_data_type(DataType::String)
            .with_required(true)
            .with_description("test".into())
            .build()
            .unwrap();

        let reporter = ReporterBuilder::new()
            .with_public_key("1234".to_string())
            .with_authorized(true)
            .with_index(0)
            .build()
            .unwrap();

        let property = PropertyBuilder::new()
            .with_name("taco".into())
            .with_record_id("taco1234".into())
            .with_property_definition(property_definition.clone())
            .with_reporters(vec![reporter.clone()])
            .with_current_page(0)
            .with_wrapped(true)
            .build()
            .unwrap();

        let property_list = PropertyListBuilder::new()
            .with_properties(vec![property.clone()])
            .build()
            .unwrap();

        let builder = property_list.into_builder();

        assert_eq!(builder.properties, Some(vec![property]));
    }

    #[test]
    fn test_property_list_bytes() {
        let property_definition = PropertyDefinitionBuilder::new()
            .with_name("i dunno".into())
            .with_data_type(DataType::String)
            .with_required(true)
            .with_description("test".into())
            .build()
            .unwrap();

        let reporter = ReporterBuilder::new()
            .with_public_key("1234".to_string())
            .with_authorized(true)
            .with_index(0)
            .build()
            .unwrap();

        let property = PropertyBuilder::new()
            .with_name("taco".into())
            .with_record_id("taco1234".into())
            .with_property_definition(property_definition.clone())
            .with_reporters(vec![reporter.clone()])
            .with_current_page(0)
            .with_wrapped(true)
            .build()
            .unwrap();

        let property_list = PropertyListBuilder::new()
            .with_properties(vec![property])
            .build()
            .unwrap();

        test_from_bytes(property_list, PropertyList::from_bytes);
    }

    #[test]
    fn test_property_page() {
        let property_value = PropertyValueBuilder::new()
            .with_name("egg".into())
            .with_data_type(DataType::Number)
            .with_number_value(42)
            .build()
            .unwrap();

        let reported_value = ReportedValueBuilder::new()
            .with_reporter_index(0)
            .with_timestamp(214)
            .with_value(property_value)
            .build()
            .unwrap();

        let property_page = PropertyPageBuilder::new()
            .with_name("egg".into())
            .with_record_id("egg1234".into())
            .with_reported_values(vec![reported_value.clone()])
            .build()
            .unwrap();

        assert_eq!(property_page.name(), "egg");
        assert_eq!(property_page.record_id(), "egg1234");
        assert!(property_page
            .reported_values()
            .iter()
            .any(|x| *x == reported_value));
    }

    #[test]
    fn test_property_page_into_builder() {
        let property_value = PropertyValueBuilder::new()
            .with_name("egg".into())
            .with_data_type(DataType::Number)
            .with_number_value(42)
            .build()
            .unwrap();

        let reported_value = ReportedValueBuilder::new()
            .with_reporter_index(0)
            .with_timestamp(214)
            .with_value(property_value)
            .build()
            .unwrap();

        let property_page = PropertyPageBuilder::new()
            .with_name("egg".into())
            .with_record_id("egg1234".into())
            .with_reported_values(vec![reported_value.clone()])
            .build()
            .unwrap();

        let builder = property_page.into_builder();

        assert_eq!(builder.name, Some("egg".to_string()));
        assert_eq!(builder.record_id, Some("egg1234".to_string()));
        assert_eq!(builder.reported_values, Some(vec![reported_value]));
    }

    #[test]
    fn test_property_page_bytes() {
        let property_value = PropertyValueBuilder::new()
            .with_name("egg".into())
            .with_data_type(DataType::Number)
            .with_number_value(42)
            .build()
            .unwrap();

        let reported_value = ReportedValueBuilder::new()
            .with_reporter_index(0)
            .with_timestamp(214)
            .with_value(property_value)
            .build()
            .unwrap();

        let property_page = PropertyPageBuilder::new()
            .with_name("egg".into())
            .with_record_id("egg1234".into())
            .with_reported_values(vec![reported_value.clone()])
            .build()
            .unwrap();

        test_from_bytes(property_page, PropertyPage::from_bytes);
    }

    #[test]
    fn test_property_page_list() {
        let property_value = PropertyValueBuilder::new()
            .with_name("egg".into())
            .with_data_type(DataType::Number)
            .with_number_value(42)
            .build()
            .unwrap();

        let reported_value = ReportedValueBuilder::new()
            .with_reporter_index(0)
            .with_timestamp(214)
            .with_value(property_value)
            .build()
            .unwrap();

        let property_page = PropertyPageBuilder::new()
            .with_name("egg".into())
            .with_record_id("egg1234".into())
            .with_reported_values(vec![reported_value.clone()])
            .build()
            .unwrap();

        let property_page_list = PropertyPageListBuilder::new()
            .with_property_pages(vec![property_page.clone()])
            .build()
            .unwrap();

        assert!(property_page_list
            .property_pages()
            .iter()
            .any(|x| *x == property_page))
    }

    #[test]
    fn test_property_page_list_into_builder() {
        let property_value = PropertyValueBuilder::new()
            .with_name("egg".into())
            .with_data_type(DataType::Number)
            .with_number_value(42)
            .build()
            .unwrap();

        let reported_value = ReportedValueBuilder::new()
            .with_reporter_index(0)
            .with_timestamp(214)
            .with_value(property_value)
            .build()
            .unwrap();

        let property_page = PropertyPageBuilder::new()
            .with_name("egg".into())
            .with_record_id("egg1234".into())
            .with_reported_values(vec![reported_value.clone()])
            .build()
            .unwrap();

        let property_page_list = PropertyPageListBuilder::new()
            .with_property_pages(vec![property_page.clone()])
            .build()
            .unwrap();

        let builder = property_page_list.into_builder();

        assert_eq!(builder.property_pages, Some(vec![property_page]))
    }

    #[test]
    fn test_property_page_list_bytes() {
        let property_value = PropertyValueBuilder::new()
            .with_name("egg".into())
            .with_data_type(DataType::Number)
            .with_number_value(42)
            .build()
            .unwrap();

        let reported_value = ReportedValueBuilder::new()
            .with_reporter_index(0)
            .with_timestamp(214)
            .with_value(property_value)
            .build()
            .unwrap();

        let property_page = PropertyPageBuilder::new()
            .with_name("egg".into())
            .with_record_id("egg1234".into())
            .with_reported_values(vec![reported_value.clone()])
            .build()
            .unwrap();

        let property_page_list = PropertyPageListBuilder::new()
            .with_property_pages(vec![property_page.clone()])
            .build()
            .unwrap();

        test_from_bytes(property_page_list, PropertyPageList::from_bytes);
    }

    #[test]
    fn test_proposal_builder() {
        let proposal = ProposalBuilder::new()
            .with_record_id("egg1234".into())
            .with_timestamp(214)
            .with_issuing_agent("james".into())
            .with_receiving_agent("joe".into())
            .with_role(Role::Owner)
            .with_properties(vec!["wet".into()])
            .with_status(Status::Open)
            .with_terms("a term".into())
            .build()
            .unwrap();

        assert_eq!(proposal.record_id(), "egg1234");
        assert_eq!(*proposal.timestamp(), 214);
        assert_eq!(proposal.issuing_agent(), "james");
        assert_eq!(proposal.receiving_agent(), "joe");
        assert_eq!(*proposal.role(), Role::Owner);
        assert!(proposal.properties().iter().any(|x| x == "wet"));
        assert_eq!(*proposal.status(), Status::Open);
        assert_eq!(proposal.terms(), "a term");
    }

    #[test]
    fn test_proposal_into_builder() {
        let proposal = ProposalBuilder::new()
            .with_record_id("egg1234".into())
            .with_timestamp(214)
            .with_issuing_agent("james".into())
            .with_receiving_agent("joe".into())
            .with_role(Role::Owner)
            .with_properties(vec!["wet".into()])
            .with_status(Status::Open)
            .with_terms("a term".into())
            .build()
            .unwrap();

        let builder = proposal.into_builder();

        assert_eq!(builder.record_id, Some("egg1234".to_string()));
        assert_eq!(builder.timestamp, Some(214));
        assert_eq!(builder.issuing_agent, Some("james".to_string()));
        assert_eq!(builder.receiving_agent, Some("joe".to_string()));
        assert_eq!(builder.role, Some(Role::Owner));
        assert_eq!(builder.properties, Some(vec!["wet".to_string()]));
        assert_eq!(builder.status, Some(Status::Open));
        assert_eq!(builder.terms, Some("a term".to_string()));
    }

    #[test]
    fn test_proposal_bytes() {
        let proposal = ProposalBuilder::new()
            .with_record_id("egg1234".into())
            .with_timestamp(214)
            .with_issuing_agent("james".into())
            .with_receiving_agent("joe".into())
            .with_role(Role::Owner)
            .with_properties(vec!["wet".into(), "gets everywhere".into()])
            .with_status(Status::Open)
            .with_terms("a term".into())
            .build()
            .unwrap();

        test_from_bytes(proposal, Proposal::from_bytes);
    }

    #[test]
    fn test_proposal_list() {
        let proposal = ProposalBuilder::new()
            .with_record_id("egg1234".into())
            .with_timestamp(214)
            .with_issuing_agent("james".into())
            .with_receiving_agent("joe".into())
            .with_role(Role::Owner)
            .with_properties(vec!["wet".into(), "gets everywhere".into()])
            .with_status(Status::Open)
            .with_terms("a term".into())
            .build()
            .unwrap();

        let proposal_list = ProposalListBuilder::new()
            .with_proposals(vec![proposal.clone()])
            .build()
            .unwrap();

        assert!(proposal_list.proposals().iter().any(|x| *x == proposal));
    }

    #[test]
    fn test_proposal_list_into_builder() {
        let proposal = ProposalBuilder::new()
            .with_record_id("egg1234".into())
            .with_timestamp(214)
            .with_issuing_agent("james".into())
            .with_receiving_agent("joe".into())
            .with_role(Role::Owner)
            .with_properties(vec!["wet".into(), "gets everywhere".into()])
            .with_status(Status::Open)
            .with_terms("a term".into())
            .build()
            .unwrap();

        let proposal_list = ProposalListBuilder::new()
            .with_proposals(vec![proposal.clone()])
            .build()
            .unwrap();

        let builder = proposal_list.into_builder();

        assert_eq!(builder.proposals, Some(vec![proposal]));
    }

    #[test]
    fn test_proposal_list_bytes() {
        let proposal = ProposalBuilder::new()
            .with_record_id("egg1234".into())
            .with_timestamp(214)
            .with_issuing_agent("james".into())
            .with_receiving_agent("joe".into())
            .with_role(Role::Owner)
            .with_properties(vec!["wet".into(), "gets everywhere".into()])
            .with_status(Status::Open)
            .with_terms("a term".into())
            .build()
            .unwrap();

        let proposal_list = ProposalListBuilder::new()
            .with_proposals(vec![proposal.clone()])
            .build()
            .unwrap();

        test_from_bytes(proposal_list, ProposalList::from_bytes);
    }

    #[test]
    fn test_record_builder() {
        let associated_agent = AssociatedAgentBuilder::new()
            .with_agent_id("agent1234".into())
            .with_timestamp(2132)
            .build()
            .unwrap();

        let record = RecordBuilder::new()
            .with_record_id("egg1234".into())
            .with_schema("egg".into())
            .with_owners(vec![associated_agent.clone()])
            .with_custodians(vec![associated_agent.clone()])
            .with_field_final(false)
            .build()
            .unwrap();

        assert_eq!(record.record_id(), "egg1234");
        assert_eq!(record.schema(), "egg");
        assert!(record.owners().iter().any(|x| *x == associated_agent));
        assert!(record.custodians().iter().any(|x| *x == associated_agent));
        assert_eq!(*record.field_final(), false);
    }

    #[test]
    fn test_record_into_builder() {
        let associated_agent = AssociatedAgentBuilder::new()
            .with_agent_id("agent1234".into())
            .with_timestamp(2132)
            .build()
            .unwrap();

        let record = RecordBuilder::new()
            .with_record_id("egg1234".into())
            .with_schema("egg".into())
            .with_owners(vec![associated_agent.clone()])
            .with_custodians(vec![associated_agent.clone()])
            .with_field_final(false)
            .build()
            .unwrap();

        let builder = record.into_builder();

        assert_eq!(builder.record_id, Some("egg1234".to_string()));
        assert_eq!(builder.schema, Some("egg".to_string()));
        assert_eq!(builder.owners, Some(vec![associated_agent.clone()]));
        assert_eq!(builder.custodians, Some(vec![associated_agent.clone()]));
        assert_eq!(builder.field_final, Some(false));
    }

    #[test]
    fn test_record_bytes() {
        let associated_agent = AssociatedAgentBuilder::new()
            .with_agent_id("agent1234".into())
            .with_timestamp(2132)
            .build()
            .unwrap();

        let record = RecordBuilder::new()
            .with_record_id("egg1234".into())
            .with_schema("egg".into())
            .with_owners(vec![associated_agent.clone()])
            .with_custodians(vec![associated_agent.clone()])
            .with_field_final(false)
            .build()
            .unwrap();

        test_from_bytes(record, Record::from_bytes);
    }

    #[test]
    fn test_record_list() {
        let associated_agent = AssociatedAgentBuilder::new()
            .with_agent_id("agent1234".into())
            .with_timestamp(2132)
            .build()
            .unwrap();

        let record = RecordBuilder::new()
            .with_record_id("egg1234".into())
            .with_schema("egg".into())
            .with_owners(vec![associated_agent.clone()])
            .with_custodians(vec![associated_agent.clone()])
            .with_field_final(false)
            .build()
            .unwrap();

        let record_list = RecordListBuilder::new()
            .with_records(vec![record.clone()])
            .build()
            .unwrap();

        assert!(record_list.records().iter().any(|x| *x == record));
    }

    #[test]
    fn test_record_list_into_builder() {
        let associated_agent = AssociatedAgentBuilder::new()
            .with_agent_id("agent1234".into())
            .with_timestamp(2132)
            .build()
            .unwrap();

        let record = RecordBuilder::new()
            .with_record_id("egg1234".into())
            .with_schema("egg".into())
            .with_owners(vec![associated_agent.clone()])
            .with_custodians(vec![associated_agent.clone()])
            .with_field_final(false)
            .build()
            .unwrap();

        let record_list = RecordListBuilder::new()
            .with_records(vec![record.clone()])
            .build()
            .unwrap();

        let builder = record_list.into_builder();

        assert_eq!(builder.records, Some(vec![record]));
    }

    #[test]
    fn test_record_list_bytes() {
        let associated_agent = AssociatedAgentBuilder::new()
            .with_agent_id("agent1234".into())
            .with_timestamp(2132)
            .build()
            .unwrap();

        let record = RecordBuilder::new()
            .with_record_id("egg1234".into())
            .with_schema("egg".into())
            .with_owners(vec![associated_agent.clone()])
            .with_custodians(vec![associated_agent.clone()])
            .with_field_final(false)
            .build()
            .unwrap();

        let record_list = RecordListBuilder::new()
            .with_records(vec![record.clone()])
            .build()
            .unwrap();

        test_from_bytes(record_list, RecordList::from_bytes);
    }

    #[test]
    fn test_associated_agent_into_builder() {
        let associated_agent = AssociatedAgentBuilder::new()
            .with_agent_id("agent1234".into())
            .with_timestamp(2132)
            .build()
            .unwrap();

        let builder = associated_agent.into_builder();

        assert_eq!(builder.agent_id, Some("agent1234".to_string()));
        assert_eq!(builder.timestamp, Some(2132));
    }
}
