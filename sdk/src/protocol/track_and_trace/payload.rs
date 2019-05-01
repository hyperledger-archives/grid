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

use std::default::Default;

use super::errors::BuilderError;
use crate::protocol::{schema::state::PropertyValue, track_and_trace::state::Role};
use crate::protos;
use crate::protos::{
    track_and_trace_payload, track_and_trace_payload::TrackAndTracePayload_Action,
};
use crate::protos::{
    FromBytes, FromNative, FromProto, IntoBytes, IntoNative, IntoProto, ProtoConversionError,
};

#[derive(Debug, Clone, PartialEq)]
pub struct CreateRecordAction {
    record_id: String,
    schema: String,
    properties: Vec<PropertyValue>,
}

impl CreateRecordAction {
    pub fn record_id(&self) -> &str {
        &self.record_id
    }
    pub fn schema(&self) -> &str {
        &self.schema
    }
    pub fn properties(&self) -> &[PropertyValue] {
        &self.properties
    }
}

#[derive(Default, Debug)]
pub struct CreateRecordActionBuilder {
    record_id: Option<String>,
    schema: Option<String>,
    properties: Option<Vec<PropertyValue>>,
}

impl CreateRecordActionBuilder {
    pub fn new() -> Self {
        CreateRecordActionBuilder::default()
    }
    pub fn with_record_id(mut self, value: String) -> Self {
        self.record_id = Some(value);
        self
    }
    pub fn with_schema(mut self, value: String) -> Self {
        self.schema = Some(value);
        self
    }
    pub fn with_properties(mut self, value: Vec<PropertyValue>) -> Self {
        self.properties = Some(value);
        self
    }
    pub fn build(self) -> Result<CreateRecordAction, BuilderError> {
        let record_id = self
            .record_id
            .ok_or_else(|| BuilderError::MissingField("record_id".into()))?;
        let schema = self
            .schema
            .ok_or_else(|| BuilderError::MissingField("schema".into()))?;
        let properties = self
            .properties
            .ok_or_else(|| BuilderError::MissingField("properties".into()))?;
        Ok(CreateRecordAction {
            record_id,
            schema,
            properties,
        })
    }
}

impl FromProto<track_and_trace_payload::CreateRecordAction> for CreateRecordAction {
    fn from_proto(
        proto: track_and_trace_payload::CreateRecordAction,
    ) -> Result<Self, ProtoConversionError> {
        Ok(CreateRecordAction {
            record_id: proto.get_record_id().to_string(),
            schema: proto.get_schema().to_string(),
            properties: proto
                .get_properties()
                .to_vec()
                .into_iter()
                .map(PropertyValue::from_proto)
                .collect::<Result<Vec<PropertyValue>, ProtoConversionError>>()?,
        })
    }
}

impl FromNative<CreateRecordAction> for track_and_trace_payload::CreateRecordAction {
    fn from_native(create_record_action: CreateRecordAction) -> Result<Self, ProtoConversionError> {
        let mut proto = track_and_trace_payload::CreateRecordAction::new();
        proto.set_record_id(create_record_action.record_id().to_string());
        proto.set_schema(create_record_action.schema().to_string());
        proto.set_properties(RepeatedField::from_vec(
            create_record_action
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

impl FromBytes<CreateRecordAction> for CreateRecordAction {
    fn from_bytes(bytes: &[u8]) -> Result<CreateRecordAction, ProtoConversionError> {
        let proto: track_and_trace_payload::CreateRecordAction = protobuf::parse_from_bytes(bytes)
            .map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get CreateRecordAction from bytes".into(),
                )
            })?;
        proto.into_native()
    }
}
impl IntoBytes for CreateRecordAction {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get CreateRecordAction from bytes".into(),
            )
        })?;
        Ok(bytes)
    }
}
impl IntoProto<track_and_trace_payload::CreateRecordAction> for CreateRecordAction {}
impl IntoNative<CreateRecordAction> for track_and_trace_payload::CreateRecordAction {}

#[derive(Debug, Clone, PartialEq)]
pub struct FinalizeRecordAction {
    record_id: String,
}

impl FinalizeRecordAction {
    pub fn record_id(&self) -> &str {
        &self.record_id
    }
}

#[derive(Default, Debug)]
pub struct FinalizeRecordActionBuilder {
    record_id: Option<String>,
}

impl FinalizeRecordActionBuilder {
    pub fn new() -> Self {
        FinalizeRecordActionBuilder::default()
    }
    pub fn with_record_id(mut self, value: String) -> Self {
        self.record_id = Some(value);
        self
    }
    pub fn build(self) -> Result<FinalizeRecordAction, BuilderError> {
        let record_id = self
            .record_id
            .ok_or_else(|| BuilderError::MissingField("record_id".into()))?;
        Ok(FinalizeRecordAction { record_id })
    }
}

impl FromProto<track_and_trace_payload::FinalizeRecordAction> for FinalizeRecordAction {
    fn from_proto(
        proto: track_and_trace_payload::FinalizeRecordAction,
    ) -> Result<Self, ProtoConversionError> {
        Ok(FinalizeRecordAction {
            record_id: proto.get_record_id().to_string(),
        })
    }
}

impl FromNative<FinalizeRecordAction> for track_and_trace_payload::FinalizeRecordAction {
    fn from_native(native: FinalizeRecordAction) -> Result<Self, ProtoConversionError> {
        let mut proto = track_and_trace_payload::FinalizeRecordAction::new();
        proto.set_record_id(native.record_id().to_string());
        Ok(proto)
    }
}

impl FromBytes<FinalizeRecordAction> for FinalizeRecordAction {
    fn from_bytes(bytes: &[u8]) -> Result<FinalizeRecordAction, ProtoConversionError> {
        let proto: track_and_trace_payload::FinalizeRecordAction =
            protobuf::parse_from_bytes(bytes).map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get CreateFinalizeAction from bytes".into(),
                )
            })?;
        proto.into_native()
    }
}
impl IntoBytes for FinalizeRecordAction {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get CreateFinalizeAction from bytes".into(),
            )
        })?;
        Ok(bytes)
    }
}
impl IntoProto<track_and_trace_payload::FinalizeRecordAction> for FinalizeRecordAction {}
impl IntoNative<FinalizeRecordAction> for track_and_trace_payload::FinalizeRecordAction {}

#[derive(Debug, Clone, PartialEq)]
pub struct UpdatePropertiesAction {
    record_id: String,
    properties: Vec<PropertyValue>,
}

impl UpdatePropertiesAction {
    pub fn record_id(&self) -> &str {
        &self.record_id
    }
    pub fn properties(&self) -> &[PropertyValue] {
        &self.properties
    }
}

#[derive(Default, Debug)]
pub struct UpdatePropertiesActionBuilder {
    record_id: Option<String>,
    properties: Option<Vec<PropertyValue>>,
}

impl UpdatePropertiesActionBuilder {
    pub fn new() -> Self {
        UpdatePropertiesActionBuilder::default()
    }
    pub fn with_record_id(mut self, value: String) -> Self {
        self.record_id = Some(value);
        self
    }
    pub fn with_properties(mut self, value: Vec<PropertyValue>) -> Self {
        self.properties = Some(value);
        self
    }
    pub fn build(self) -> Result<UpdatePropertiesAction, BuilderError> {
        let record_id = self
            .record_id
            .ok_or_else(|| BuilderError::MissingField("record_id".into()))?;
        let properties = self
            .properties
            .ok_or_else(|| BuilderError::MissingField("properties".into()))?;
        Ok(UpdatePropertiesAction {
            record_id,
            properties,
        })
    }
}

impl FromProto<track_and_trace_payload::UpdatePropertiesAction> for UpdatePropertiesAction {
    fn from_proto(
        proto: track_and_trace_payload::UpdatePropertiesAction,
    ) -> Result<Self, ProtoConversionError> {
        Ok(UpdatePropertiesAction {
            record_id: proto.get_record_id().to_string(),
            properties: proto
                .get_properties()
                .to_vec()
                .into_iter()
                .map(PropertyValue::from_proto)
                .collect::<Result<Vec<PropertyValue>, ProtoConversionError>>()?,
        })
    }
}

impl FromNative<UpdatePropertiesAction> for track_and_trace_payload::UpdatePropertiesAction {
    fn from_native(native: UpdatePropertiesAction) -> Result<Self, ProtoConversionError> {
        let mut proto = track_and_trace_payload::UpdatePropertiesAction::new();
        proto.set_record_id(native.record_id().to_string());
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

impl FromBytes<UpdatePropertiesAction> for UpdatePropertiesAction {
    fn from_bytes(bytes: &[u8]) -> Result<UpdatePropertiesAction, ProtoConversionError> {
        let proto: track_and_trace_payload::UpdatePropertiesAction =
            protobuf::parse_from_bytes(bytes).map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get UpdatePropertiesAction from bytes".into(),
                )
            })?;
        proto.into_native()
    }
}
impl IntoBytes for UpdatePropertiesAction {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get UpdatePropertiesAction from bytes".into(),
            )
        })?;
        Ok(bytes)
    }
}
impl IntoProto<track_and_trace_payload::UpdatePropertiesAction> for UpdatePropertiesAction {}
impl IntoNative<UpdatePropertiesAction> for track_and_trace_payload::UpdatePropertiesAction {}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateProposalAction {
    record_id: String,
    receiving_agent: String,
    role: Role,
    properties: Vec<String>,
}

impl CreateProposalAction {
    pub fn record_id(&self) -> &str {
        &self.record_id
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
}

#[derive(Default, Debug)]
pub struct CreateProposalActionBuilder {
    record_id: Option<String>,
    receiving_agent: Option<String>,
    role: Option<Role>,
    properties: Option<Vec<String>>,
}

impl CreateProposalActionBuilder {
    pub fn new() -> Self {
        CreateProposalActionBuilder::default()
    }
    pub fn with_record_id(mut self, value: String) -> Self {
        self.record_id = Some(value);
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
    pub fn build(self) -> Result<CreateProposalAction, BuilderError> {
        let record_id = self
            .record_id
            .ok_or_else(|| BuilderError::MissingField("record_id".into()))?;
        let receiving_agent = self
            .receiving_agent
            .ok_or_else(|| BuilderError::MissingField("receiving_agent".into()))?;
        let role = self
            .role
            .ok_or_else(|| BuilderError::MissingField("role".into()))?;
        let properties = self
            .properties
            .ok_or_else(|| BuilderError::MissingField("properties".into()))?;
        Ok(CreateProposalAction {
            record_id,
            receiving_agent,
            role,
            properties,
        })
    }
}

impl FromProto<track_and_trace_payload::CreateProposalAction> for CreateProposalAction {
    fn from_proto(
        proto: track_and_trace_payload::CreateProposalAction,
    ) -> Result<Self, ProtoConversionError> {
        Ok(CreateProposalAction {
            record_id: proto.get_record_id().to_string(),
            receiving_agent: proto.get_receiving_agent().to_string(),
            role: Role::from_proto(proto.get_role())?,
            properties: proto
                .get_properties()
                .to_vec()
                .into_iter()
                .map(String::from)
                .collect(),
        })
    }
}

impl FromNative<CreateProposalAction> for track_and_trace_payload::CreateProposalAction {
    fn from_native(native: CreateProposalAction) -> Result<Self, ProtoConversionError> {
        let mut proto = track_and_trace_payload::CreateProposalAction::new();
        proto.set_record_id(native.record_id().to_string());
        proto.set_receiving_agent(native.receiving_agent().to_string());
        proto.set_role(native.role().clone().into_proto()?);
        proto.set_properties(RepeatedField::from_vec(native.properties().to_vec()));

        Ok(proto)
    }
}

impl FromBytes<CreateProposalAction> for CreateProposalAction {
    fn from_bytes(bytes: &[u8]) -> Result<CreateProposalAction, ProtoConversionError> {
        let proto: track_and_trace_payload::CreateProposalAction =
            protobuf::parse_from_bytes(bytes).map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get CreateProposalAction from bytes".into(),
                )
            })?;
        proto.into_native()
    }
}
impl IntoBytes for CreateProposalAction {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get CreateProposalAction from bytes".into(),
            )
        })?;
        Ok(bytes)
    }
}
impl IntoProto<track_and_trace_payload::CreateProposalAction> for CreateProposalAction {}
impl IntoNative<CreateProposalAction> for track_and_trace_payload::CreateProposalAction {}

#[derive(Debug, Clone, PartialEq)]
pub enum Response {
    Accept,
    Reject,
    Cancel,
}

impl Default for Response {
    fn default() -> Response {
        Response::Accept
    }
}

impl FromProto<track_and_trace_payload::AnswerProposalAction_Response> for Response {
    fn from_proto(
        responses: track_and_trace_payload::AnswerProposalAction_Response,
    ) -> Result<Self, ProtoConversionError> {
        match responses {
            track_and_trace_payload::AnswerProposalAction_Response::ACCEPT => Ok(Response::Accept),
            track_and_trace_payload::AnswerProposalAction_Response::REJECT => Ok(Response::Reject),
            track_and_trace_payload::AnswerProposalAction_Response::CANCEL => Ok(Response::Cancel),
        }
    }
}

impl FromNative<Response> for track_and_trace_payload::AnswerProposalAction_Response {
    fn from_native(responses: Response) -> Result<Self, ProtoConversionError> {
        match responses {
            Response::Accept => Ok(track_and_trace_payload::AnswerProposalAction_Response::ACCEPT),
            Response::Reject => Ok(track_and_trace_payload::AnswerProposalAction_Response::REJECT),
            Response::Cancel => Ok(track_and_trace_payload::AnswerProposalAction_Response::CANCEL),
        }
    }
}

impl IntoProto<track_and_trace_payload::AnswerProposalAction_Response> for Response {}
impl IntoNative<Response> for track_and_trace_payload::AnswerProposalAction_Response {}

#[derive(Debug, Clone, PartialEq)]
pub struct AnswerProposalAction {
    record_id: String,
    receiving_agent: String,
    role: Role,
    response: Response,
}

impl AnswerProposalAction {
    pub fn record_id(&self) -> &str {
        &self.record_id
    }
    pub fn receiving_agent(&self) -> &str {
        &self.receiving_agent
    }
    pub fn role(&self) -> &Role {
        &self.role
    }
    pub fn response(&self) -> &Response {
        &self.response
    }
}

#[derive(Default, Debug)]
pub struct AnswerProposalActionBuilder {
    record_id: Option<String>,
    receiving_agent: Option<String>,
    role: Option<Role>,
    response: Option<Response>,
}

impl AnswerProposalActionBuilder {
    pub fn new() -> Self {
        AnswerProposalActionBuilder::default()
    }
    pub fn with_record_id(mut self, value: String) -> Self {
        self.record_id = Some(value);
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
    pub fn with_response(mut self, value: Response) -> Self {
        self.response = Some(value);
        self
    }
    pub fn build(self) -> Result<AnswerProposalAction, BuilderError> {
        let record_id = self
            .record_id
            .ok_or_else(|| BuilderError::MissingField("record_id".into()))?;
        let receiving_agent = self
            .receiving_agent
            .ok_or_else(|| BuilderError::MissingField("receiving_agent".into()))?;
        let role = self
            .role
            .ok_or_else(|| BuilderError::MissingField("role".into()))?;
        let response = self
            .response
            .ok_or_else(|| BuilderError::MissingField("response".into()))?;
        Ok(AnswerProposalAction {
            record_id,
            receiving_agent,
            role,
            response,
        })
    }
}

impl FromProto<track_and_trace_payload::AnswerProposalAction> for AnswerProposalAction {
    fn from_proto(
        proto: track_and_trace_payload::AnswerProposalAction,
    ) -> Result<Self, ProtoConversionError> {
        Ok(AnswerProposalAction {
            record_id: proto.get_record_id().to_string(),
            receiving_agent: proto.get_receiving_agent().to_string(),
            role: Role::from_proto(proto.get_role())?,
            response: Response::from_proto(proto.get_response())?,
        })
    }
}

impl FromNative<AnswerProposalAction> for track_and_trace_payload::AnswerProposalAction {
    fn from_native(native: AnswerProposalAction) -> Result<Self, ProtoConversionError> {
        let mut proto = track_and_trace_payload::AnswerProposalAction::new();
        proto.set_record_id(native.record_id().to_string());
        proto.set_receiving_agent(native.receiving_agent().to_string());
        proto.set_role(native.role().clone().into_proto()?);
        proto.set_response(native.response().clone().into_proto()?);

        Ok(proto)
    }
}

impl FromBytes<AnswerProposalAction> for AnswerProposalAction {
    fn from_bytes(bytes: &[u8]) -> Result<AnswerProposalAction, ProtoConversionError> {
        let proto: track_and_trace_payload::AnswerProposalAction =
            protobuf::parse_from_bytes(bytes).map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get AnswerProposalAction from bytes".into(),
                )
            })?;
        proto.into_native()
    }
}
impl IntoBytes for AnswerProposalAction {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get AnswerProposalAction from bytes".into(),
            )
        })?;
        Ok(bytes)
    }
}
impl IntoProto<track_and_trace_payload::AnswerProposalAction> for AnswerProposalAction {}
impl IntoNative<AnswerProposalAction> for track_and_trace_payload::AnswerProposalAction {}

#[derive(Debug, Clone, PartialEq)]
pub struct RevokeReporterAction {
    record_id: String,
    reporter_id: String,
    properties: Vec<String>,
}

impl RevokeReporterAction {
    pub fn record_id(&self) -> &str {
        &self.record_id
    }
    pub fn reporter_id(&self) -> &str {
        &self.reporter_id
    }
    pub fn properties(&self) -> &[String] {
        &self.properties
    }
}

#[derive(Default, Debug)]
pub struct RevokeReporterActionBuilder {
    record_id: Option<String>,
    reporter_id: Option<String>,
    properties: Option<Vec<String>>,
}

impl RevokeReporterActionBuilder {
    pub fn new() -> Self {
        RevokeReporterActionBuilder::default()
    }
    pub fn with_record_id(mut self, value: String) -> Self {
        self.record_id = Some(value);
        self
    }
    pub fn with_reporter_id(mut self, value: String) -> Self {
        self.reporter_id = Some(value);
        self
    }
    pub fn with_properties(mut self, value: Vec<String>) -> Self {
        self.properties = Some(value);
        self
    }
    pub fn build(self) -> Result<RevokeReporterAction, BuilderError> {
        let record_id = self
            .record_id
            .ok_or_else(|| BuilderError::MissingField("record_id".into()))?;
        let reporter_id = self
            .reporter_id
            .ok_or_else(|| BuilderError::MissingField("reporter_id".into()))?;
        let properties = self
            .properties
            .ok_or_else(|| BuilderError::MissingField("properties".into()))?;
        Ok(RevokeReporterAction {
            record_id,
            reporter_id,
            properties,
        })
    }
}

impl FromProto<track_and_trace_payload::RevokeReporterAction> for RevokeReporterAction {
    fn from_proto(
        proto: track_and_trace_payload::RevokeReporterAction,
    ) -> Result<Self, ProtoConversionError> {
        Ok(RevokeReporterAction {
            record_id: proto.get_record_id().to_string(),
            reporter_id: proto.get_reporter_id().to_string(),
            properties: proto
                .get_properties()
                .to_vec()
                .into_iter()
                .map(String::from)
                .collect(),
        })
    }
}

impl FromNative<RevokeReporterAction> for track_and_trace_payload::RevokeReporterAction {
    fn from_native(native: RevokeReporterAction) -> Result<Self, ProtoConversionError> {
        let mut proto = track_and_trace_payload::RevokeReporterAction::new();
        proto.set_record_id(native.record_id().to_string());
        proto.set_reporter_id(native.reporter_id().to_string());
        proto.set_properties(RepeatedField::from_vec(native.properties().to_vec()));

        Ok(proto)
    }
}

impl FromBytes<RevokeReporterAction> for RevokeReporterAction {
    fn from_bytes(bytes: &[u8]) -> Result<RevokeReporterAction, ProtoConversionError> {
        let proto: track_and_trace_payload::RevokeReporterAction =
            protobuf::parse_from_bytes(bytes).map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get RevokeReporterAction from bytes".into(),
                )
            })?;
        proto.into_native()
    }
}
impl IntoBytes for RevokeReporterAction {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get RevokeReporterAction from bytes".into(),
            )
        })?;
        Ok(bytes)
    }
}
impl IntoProto<track_and_trace_payload::RevokeReporterAction> for RevokeReporterAction {}
impl IntoNative<RevokeReporterAction> for track_and_trace_payload::RevokeReporterAction {}

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    CreateRecord(CreateRecordAction),
    FinalizeRecord(FinalizeRecordAction),
    UpdateProperties(UpdatePropertiesAction),
    CreateProposal(CreateProposalAction),
    AnswerProposal(AnswerProposalAction),
    RevokeReporter(RevokeReporterAction),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrackAndTracePayload {
    action: Action,
    timestamp: u64,
}

impl TrackAndTracePayload {
    pub fn action(&self) -> &Action {
        &self.action
    }
    pub fn timestamp(&self) -> &u64 {
        &self.timestamp
    }
}

#[derive(Default, Debug)]
pub struct TrackAndTracePayloadBuilder {
    action: Option<Action>,
    timestamp: Option<u64>,
}

impl TrackAndTracePayloadBuilder {
    pub fn new() -> Self {
        TrackAndTracePayloadBuilder::default()
    }
    pub fn with_action(mut self, value: Action) -> Self {
        self.action = Some(value);
        self
    }
    pub fn with_timestamp(mut self, value: u64) -> Self {
        self.timestamp = Some(value);
        self
    }
    pub fn build(self) -> Result<TrackAndTracePayload, BuilderError> {
        let action = self
            .action
            .ok_or_else(|| BuilderError::MissingField("action".into()))?;
        let timestamp = self
            .timestamp
            .ok_or_else(|| BuilderError::MissingField("timestamp".into()))?;
        Ok(TrackAndTracePayload { action, timestamp })
    }
}

impl FromProto<track_and_trace_payload::TrackAndTracePayload> for TrackAndTracePayload {
    fn from_proto(
        proto: track_and_trace_payload::TrackAndTracePayload,
    ) -> Result<Self, ProtoConversionError> {
        let action = match proto.get_action() {
            TrackAndTracePayload_Action::CREATE_RECORD => Action::CreateRecord(
                CreateRecordAction::from_proto(proto.get_create_record().clone())?,
            ),
            TrackAndTracePayload_Action::FINALIZE_RECORD => Action::FinalizeRecord(
                FinalizeRecordAction::from_proto(proto.get_finalize_record().clone())?,
            ),
            TrackAndTracePayload_Action::UPDATE_PROPERTIES => Action::UpdateProperties(
                UpdatePropertiesAction::from_proto(proto.get_update_properties().clone())?,
            ),
            TrackAndTracePayload_Action::CREATE_PROPOSAL => Action::CreateProposal(
                CreateProposalAction::from_proto(proto.get_create_proposal().clone())?,
            ),
            TrackAndTracePayload_Action::ANSWER_PROPOSAL => Action::AnswerProposal(
                AnswerProposalAction::from_proto(proto.get_answer_proposal().clone())?,
            ),
            TrackAndTracePayload_Action::REVOKE_REPORTER => Action::RevokeReporter(
                RevokeReporterAction::from_proto(proto.get_revoke_reporter().clone())?,
            ),
            TrackAndTracePayload_Action::UNSET_ACTION => {
                return Err(ProtoConversionError::InvalidTypeError(
                    "Cannot convert TrackAndTracePayload_Action with type unset.".to_string(),
                ));
            }
        };

        Ok(TrackAndTracePayload {
            action,
            timestamp: proto.get_timestamp(),
        })
    }
}

impl FromNative<TrackAndTracePayload> for track_and_trace_payload::TrackAndTracePayload {
    fn from_native(native: TrackAndTracePayload) -> Result<Self, ProtoConversionError> {
        let mut proto = track_and_trace_payload::TrackAndTracePayload::new();

        proto.set_timestamp(*native.timestamp());

        match native.action() {
            Action::CreateRecord(payload) => {
                proto.set_action(TrackAndTracePayload_Action::CREATE_RECORD);
                proto.set_create_record(payload.clone().into_proto()?);
            }
            Action::FinalizeRecord(payload) => {
                proto.set_action(TrackAndTracePayload_Action::FINALIZE_RECORD);
                proto.set_finalize_record(payload.clone().into_proto()?);
            }
            Action::UpdateProperties(payload) => {
                proto.set_action(TrackAndTracePayload_Action::UPDATE_PROPERTIES);
                proto.set_update_properties(payload.clone().into_proto()?);
            }
            Action::CreateProposal(payload) => {
                proto.set_action(TrackAndTracePayload_Action::CREATE_PROPOSAL);
                proto.set_create_proposal(payload.clone().into_proto()?);
            }
            Action::AnswerProposal(payload) => {
                proto.set_action(TrackAndTracePayload_Action::ANSWER_PROPOSAL);
                proto.set_answer_proposal(payload.clone().into_proto()?);
            }
            Action::RevokeReporter(payload) => {
                proto.set_action(TrackAndTracePayload_Action::REVOKE_REPORTER);
                proto.set_revoke_reporter(payload.clone().into_proto()?);
            }
        }

        Ok(proto)
    }
}

impl FromBytes<TrackAndTracePayload> for TrackAndTracePayload {
    fn from_bytes(bytes: &[u8]) -> Result<TrackAndTracePayload, ProtoConversionError> {
        let proto: track_and_trace_payload::TrackAndTracePayload =
            protobuf::parse_from_bytes(bytes).map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get TrackAndTracePaylaod from bytes".into(),
                )
            })?;
        proto.into_native()
    }
}
impl IntoBytes for TrackAndTracePayload {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get TrackAndTracePaylaod from bytes".into(),
            )
        })?;
        Ok(bytes)
    }
}
impl IntoProto<track_and_trace_payload::TrackAndTracePayload> for TrackAndTracePayload {}
impl IntoNative<TrackAndTracePayload> for track_and_trace_payload::TrackAndTracePayload {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::schema::state::{DataType, PropertyValueBuilder};
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
    fn test_create_record_builder() {
        let property_value = PropertyValueBuilder::new()
            .with_name("egg".into())
            .with_data_type(DataType::Number)
            .with_number_value(42)
            .build()
            .unwrap();

        let action = CreateRecordActionBuilder::new()
            .with_record_id("32".into())
            .with_schema("schema".into())
            .with_properties(vec![property_value.clone()])
            .build()
            .unwrap();

        assert_eq!(action.record_id(), "32");
        assert_eq!(action.schema(), "schema");
        assert!(action.properties().iter().any(|x| *x == property_value));
    }

    #[test]
    fn test_create_record_bytes() {
        let property_value = PropertyValueBuilder::new()
            .with_name("egg".into())
            .with_data_type(DataType::Number)
            .with_number_value(42)
            .build()
            .unwrap();

        let action = CreateRecordActionBuilder::new()
            .with_record_id("32".into())
            .with_schema("schema".into())
            .with_properties(vec![property_value.clone()])
            .build()
            .unwrap();

        test_from_bytes(action, CreateRecordAction::from_bytes);
    }

    #[test]
    fn test_finalize_record_action_builder() {
        let action = FinalizeRecordActionBuilder::new()
            .with_record_id("32".into())
            .build()
            .unwrap();

        assert_eq!(action.record_id(), "32");
    }

    #[test]
    fn test_finalize_record_action_bytes() {
        let action = FinalizeRecordActionBuilder::new()
            .with_record_id("32".into())
            .build()
            .unwrap();

        test_from_bytes(action, FinalizeRecordAction::from_bytes);
    }

    #[test]
    fn test_update_properties_action() {
        let property_value = PropertyValueBuilder::new()
            .with_name("egg".into())
            .with_data_type(DataType::Number)
            .with_number_value(42)
            .build()
            .unwrap();

        let action = UpdatePropertiesActionBuilder::new()
            .with_record_id("32".into())
            .with_properties(vec![property_value.clone()])
            .build()
            .unwrap();

        assert_eq!(action.record_id(), "32");
        assert!(action.properties().iter().any(|x| *x == property_value));
    }

    #[test]
    fn test_update_properties_action_bytes() {
        let property_value = PropertyValueBuilder::new()
            .with_name("egg".into())
            .with_data_type(DataType::Number)
            .with_number_value(42)
            .build()
            .unwrap();

        let action = UpdatePropertiesActionBuilder::new()
            .with_record_id("32".into())
            .with_properties(vec![property_value.clone()])
            .build()
            .unwrap();

        test_from_bytes(action, UpdatePropertiesAction::from_bytes);
    }

    #[test]
    fn test_create_proposal_action_builder() {
        let action = CreateProposalActionBuilder::new()
            .with_record_id("32".into())
            .with_receiving_agent("jim".into())
            .with_role(Role::Custodian)
            .with_properties(vec!["egg".into()])
            .build()
            .unwrap();

        assert_eq!(action.record_id(), "32");
        assert_eq!(action.receiving_agent(), "jim");
        assert_eq!(*action.role(), Role::Custodian);
        assert!(action.properties().iter().any(|x| x == "egg"));
    }

    #[test]
    fn test_create_proposal_action_bytes() {
        let action = CreateProposalActionBuilder::new()
            .with_record_id("32".into())
            .with_receiving_agent("jim".into())
            .with_role(Role::Custodian)
            .with_properties(vec!["egg".into()])
            .build()
            .unwrap();

        test_from_bytes(action, CreateProposalAction::from_bytes);
    }

    #[test]
    fn test_answer_proposal_action_builder() {
        let action = AnswerProposalActionBuilder::new()
            .with_record_id("32".into())
            .with_receiving_agent("jim".into())
            .with_role(Role::Custodian)
            .with_response(Response::Accept)
            .build()
            .unwrap();

        assert_eq!(action.record_id(), "32");
        assert_eq!(action.receiving_agent(), "jim");
        assert_eq!(*action.role(), Role::Custodian);
        assert_eq!(*action.response(), Response::Accept);
    }

    #[test]
    fn test_answer_proposal_action_bytes() {
        let action = AnswerProposalActionBuilder::new()
            .with_record_id("32".into())
            .with_receiving_agent("jim".into())
            .with_role(Role::Custodian)
            .with_response(Response::Accept)
            .build()
            .unwrap();

        test_from_bytes(action, AnswerProposalAction::from_bytes);
    }

    #[test]
    fn test_revoke_reporter_action_builder() {
        let action = RevokeReporterActionBuilder::new()
            .with_record_id("32".into())
            .with_reporter_id("jim".into())
            .with_properties(vec!["egg".into()])
            .build()
            .unwrap();

        assert_eq!(action.record_id(), "32");
        assert_eq!(action.reporter_id(), "jim");
        assert!(action.properties().iter().any(|x| x == "egg"));
    }

    #[test]
    fn test_revoke_reporter_action_bytes() {
        let action = RevokeReporterActionBuilder::new()
            .with_record_id("32".into())
            .with_reporter_id("jim".into())
            .with_properties(vec!["egg".into()])
            .build()
            .unwrap();

        test_from_bytes(action, RevokeReporterAction::from_bytes);
    }

    #[test]
    fn test_payload_builder() {
        let action = RevokeReporterActionBuilder::new()
            .with_record_id("32".into())
            .with_reporter_id("jim".into())
            .with_properties(vec!["egg".into()])
            .build()
            .unwrap();

        let payload = TrackAndTracePayloadBuilder::new()
            .with_action(Action::RevokeReporter(action.clone()))
            .with_timestamp(0)
            .build()
            .unwrap();

        assert_eq!(*payload.action(), Action::RevokeReporter(action));
        assert_eq!(*payload.timestamp(), 0);
    }

    #[test]
    fn test_payload_bytes() {
        let action = RevokeReporterActionBuilder::new()
            .with_record_id("32".into())
            .with_reporter_id("jim".into())
            .with_properties(vec!["egg".into()])
            .build()
            .unwrap();

        let payload = TrackAndTracePayloadBuilder::new()
            .with_action(Action::RevokeReporter(action.clone()))
            .with_timestamp(0)
            .build()
            .unwrap();

        test_from_bytes(payload, TrackAndTracePayload::from_bytes);
    }
}
