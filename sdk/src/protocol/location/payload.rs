// Copyright 2020 Cargill Incorporated
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

use super::errors::BuilderError;

use crate::protocol::schema::state::PropertyValue;
use crate::protos;
use crate::protos::{location_payload, location_payload::LocationPayload_Action};
use crate::protos::{
    FromBytes, FromNative, FromProto, IntoBytes, IntoNative, IntoProto, ProtoConversionError,
};

#[derive(Copy, Clone, Debug, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    LocationCreate(LocationCreateAction),
    LocationUpdate(LocationUpdateAction),
    LocationDelete(LocationDeleteAction),
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocationPayload {
    action: Action,
    timestamp: u64,
}

impl LocationPayload {
    pub fn action(&self) -> &Action {
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
            LocationPayload_Action::LOCATION_CREATE => Action::LocationCreate(
                LocationCreateAction::from_proto(payload.get_location_create().clone())?,
            ),
            LocationPayload_Action::LOCATION_UPDATE => Action::LocationUpdate(
                LocationUpdateAction::from_proto(payload.get_location_update().clone())?,
            ),
            LocationPayload_Action::LOCATION_DELETE => Action::LocationDelete(
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
            Action::LocationCreate(payload) => {
                proto.set_action(LocationPayload_Action::LOCATION_CREATE);
                proto.set_location_create(payload.clone().into_proto()?);
            }
            Action::LocationUpdate(payload) => {
                proto.set_action(LocationPayload_Action::LOCATION_UPDATE);
                proto.set_location_update(payload.clone().into_proto()?);
            }
            Action::LocationDelete(payload) => {
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
            protobuf::parse_from_bytes(bytes).map_err(|_| {
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

#[derive(Debug)]
pub enum LocationPayloadBuildError {
    MissingField(String),
}

impl StdError for LocationPayloadBuildError {
    fn description(&self) -> &str {
        match *self {
            LocationPayloadBuildError::MissingField(ref msg) => msg,
        }
    }

    fn cause(&self) -> Option<&dyn StdError> {
        match *self {
            LocationPayloadBuildError::MissingField(_) => None,
        }
    }
}

impl std::fmt::Display for LocationPayloadBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            LocationPayloadBuildError::MissingField(ref s) => write!(f, "missing field \"{}\"", s),
        }
    }
}

#[derive(Default, Clone)]
pub struct LocationPayloadBuilder {
    action: Option<Action>,
    timestamp: Option<u64>,
}

impl LocationPayloadBuilder {
    pub fn new() -> Self {
        LocationPayloadBuilder::default()
    }
    pub fn with_action(mut self, action: Action) -> Self {
        self.action = Some(action);
        self
    }
    pub fn with_timestamp(mut self, value: u64) -> Self {
        self.timestamp = Some(value);
        self
    }
    pub fn build(self) -> Result<LocationPayload, BuilderError> {
        let action = self
            .action
            .ok_or_else(|| BuilderError::MissingField("'action' field is required".into()))?;
        let timestamp = self
            .timestamp
            .ok_or_else(|| BuilderError::MissingField("'timestamp' field is required".into()))?;
        Ok(LocationPayload { action, timestamp })
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
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
            protobuf::parse_from_bytes(bytes).map_err(|_| {
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

#[derive(Debug, Default, Clone, PartialEq)]
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
            protobuf::parse_from_bytes(bytes).map_err(|_| {
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

/// Builder used to create a LocationUpdateAction
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

#[derive(Debug, Default, Clone, PartialEq)]
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
            protobuf::parse_from_bytes(bytes).map_err(|_| {
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

/// Builder used to create a LocationDeleteAction
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
