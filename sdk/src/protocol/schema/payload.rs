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

use crate::protocol::schema::state::PropertyDefinition;
use crate::protos;
use crate::protos::{
    FromBytes, FromNative, FromProto, IntoBytes, IntoNative, IntoProto, ProtoConversionError,
};

/// Native implementation for SchemaPayload_Action
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    SchemaCreate(SchemaCreateAction),
    SchemaUpdate(SchemaUpdateAction),
}

/// Native implementation for SchemaPayload
#[derive(Debug, Clone, PartialEq)]
pub struct SchemaPayload {
    action: Action,
}

impl SchemaPayload {
    pub fn action(&self) -> &Action {
        &self.action
    }
}

impl FromProto<protos::schema_payload::SchemaPayload> for SchemaPayload {
    fn from_proto(
        payload: protos::schema_payload::SchemaPayload,
    ) -> Result<Self, ProtoConversionError> {
        let action = match payload.get_action() {
            protos::schema_payload::SchemaPayload_Action::SCHEMA_CREATE => Action::SchemaCreate(
                SchemaCreateAction::from_proto(payload.get_schema_create().clone())?,
            ),
            protos::schema_payload::SchemaPayload_Action::SCHEMA_UPDATE => Action::SchemaUpdate(
                SchemaUpdateAction::from_proto(payload.get_schema_update().clone())?,
            ),
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
            Action::SchemaCreate(payload) => {
                proto_payload
                    .set_action(protos::schema_payload::SchemaPayload_Action::SCHEMA_CREATE);
                proto_payload.set_schema_create(payload.clone().into_proto()?);
            }
            Action::SchemaUpdate(payload) => {
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

#[derive(Debug)]
pub enum SchemaPayloadBuildError {
    MissingField(String),
}

impl StdError for SchemaPayloadBuildError {
    fn description(&self) -> &str {
        match *self {
            SchemaPayloadBuildError::MissingField(ref msg) => msg,
        }
    }

    fn cause(&self) -> Option<&dyn StdError> {
        match *self {
            SchemaPayloadBuildError::MissingField(_) => None,
        }
    }
}

impl std::fmt::Display for SchemaPayloadBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            SchemaPayloadBuildError::MissingField(ref s) => write!(f, "MissingField: {}", s),
        }
    }
}

/// Builder used to create a SchemaPayload
#[derive(Default, Clone)]
pub struct SchemaPayloadBuilder {
    action: Option<Action>,
}

impl SchemaPayloadBuilder {
    pub fn new() -> Self {
        SchemaPayloadBuilder::default()
    }

    pub fn with_action(mut self, action: Action) -> SchemaPayloadBuilder {
        self.action = Some(action);
        self
    }

    pub fn build(self) -> Result<SchemaPayload, SchemaPayloadBuildError> {
        let action = self.action.ok_or_else(|| {
            SchemaPayloadBuildError::MissingField("'action' field is required".to_string())
        })?;
        Ok(SchemaPayload { action })
    }
}

/// Native implementation for SchemaCreateAction
#[derive(Debug, Default, Clone, PartialEq)]
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

#[derive(Debug)]
pub enum SchemaCreateBuildError {
    MissingField(String),
}

impl StdError for SchemaCreateBuildError {
    fn description(&self) -> &str {
        match *self {
            SchemaCreateBuildError::MissingField(ref msg) => msg,
        }
    }

    fn cause(&self) -> Option<&dyn StdError> {
        match *self {
            SchemaCreateBuildError::MissingField(_) => None,
        }
    }
}

impl std::fmt::Display for SchemaCreateBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            SchemaCreateBuildError::MissingField(ref s) => write!(f, "MissingField: {}", s),
        }
    }
}

/// Builder used to create a SchemaPayload
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

    pub fn build(self) -> Result<SchemaCreateAction, SchemaCreateBuildError> {
        let schema_name = self.schema_name.ok_or_else(|| {
            SchemaCreateBuildError::MissingField("'schema_name' field is required".to_string())
        })?;

        let description = self.description.unwrap_or_default();

        let properties = {
            if !self.properties.is_empty() {
                self.properties
            } else {
                return Err(SchemaCreateBuildError::MissingField(
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

#[derive(Debug, Default, Clone, PartialEq)]
pub struct SchemaUpdateAction {
    schema_name: String,
    properties: Vec<PropertyDefinition>,
}

/// Native implementation for SchemaUpdateAction
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

#[derive(Debug)]
pub enum SchemaUpdateBuildError {
    MissingField(String),
}

impl StdError for SchemaUpdateBuildError {
    fn description(&self) -> &str {
        match *self {
            SchemaUpdateBuildError::MissingField(ref msg) => msg,
        }
    }

    fn cause(&self) -> Option<&dyn StdError> {
        match *self {
            SchemaUpdateBuildError::MissingField(_) => None,
        }
    }
}

impl std::fmt::Display for SchemaUpdateBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            SchemaUpdateBuildError::MissingField(ref s) => write!(f, "MissingField: {}", s),
        }
    }
}

/// Builder used to create a SchemaPayload
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

    pub fn build(self) -> Result<SchemaUpdateAction, SchemaUpdateBuildError> {
        let schema_name = self.schema_name.ok_or_else(|| {
            SchemaUpdateBuildError::MissingField("'schema field is required".to_string())
        })?;

        let properties = {
            if !self.properties.is_empty() {
                self.properties
            } else {
                return Err(SchemaUpdateBuildError::MissingField(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::schema::state::{DataType, PropertyDefinitionBuilder};

    #[test]
    // check that a schema create action is built correctly
    fn check_schema_create_action() {
        let builder = PropertyDefinitionBuilder::new();
        let property_definition = builder
            .with_name("TEST".to_string())
            .with_data_type(DataType::String)
            .with_description("Optional".to_string())
            .build()
            .unwrap();

        let builder = SchemaCreateBuilder::new();
        let action = builder
            .with_schema_name("TestSchema".to_string())
            .with_description("Test Schema".to_string())
            .with_properties(vec![property_definition.clone()])
            .build()
            .unwrap();

        assert_eq!(action.schema_name, "TestSchema");
        assert_eq!(action.description, "Test Schema");
        assert_eq!(action.properties, vec![property_definition]);
    }

    #[test]
    // check that a schema create can be converted to bytes and back
    fn check_schema_create_bytes() {
        let builder = PropertyDefinitionBuilder::new();
        let property_definition = builder
            .with_name("TEST".to_string())
            .with_data_type(DataType::String)
            .with_description("Optional".to_string())
            .build()
            .unwrap();

        let builder = SchemaCreateBuilder::new();
        let original = builder
            .with_schema_name("TestSchema".to_string())
            .with_description("Test Schema".to_string())
            .with_properties(vec![property_definition.clone()])
            .build()
            .unwrap();

        let bytes = original.clone().into_bytes().unwrap();

        let create = SchemaCreateAction::from_bytes(&bytes).unwrap();
        assert_eq!(create, original);
    }

    #[test]
    // check that a schema update action is built correctly
    fn check_schema_update_action() {
        let builder = PropertyDefinitionBuilder::new();
        let property_definition = builder
            .with_name("TEST".to_string())
            .with_data_type(DataType::String)
            .with_description("Optional".to_string())
            .build()
            .unwrap();

        let builder = SchemaUpdateBuilder::new();
        let action = builder
            .with_schema_name("TestSchema".to_string())
            .with_properties(vec![property_definition.clone()])
            .build()
            .unwrap();

        assert_eq!(action.schema_name, "TestSchema");
        assert_eq!(action.properties, vec![property_definition]);
    }

    #[test]
    // check that a schema update can be converted to bytes and back
    fn check_schema_update_bytes() {
        let builder = PropertyDefinitionBuilder::new();
        let property_definition = builder
            .with_name("TEST".to_string())
            .with_data_type(DataType::String)
            .with_description("Optional".to_string())
            .build()
            .unwrap();

        let builder = SchemaUpdateBuilder::new();
        let original = builder
            .with_schema_name("TestSchema".to_string())
            .with_properties(vec![property_definition.clone()])
            .build()
            .unwrap();

        let bytes = original.clone().into_bytes().unwrap();

        let update = SchemaUpdateAction::from_bytes(&bytes).unwrap();
        assert_eq!(update, original);
    }

    #[test]
    // check that a schema payload with create action is built correctly
    fn check_schema_create_action_payload() {
        let builder = PropertyDefinitionBuilder::new();
        let property_definition = builder
            .with_name("TEST".to_string())
            .with_data_type(DataType::String)
            .with_description("Optional".to_string())
            .build()
            .unwrap();

        let builder = SchemaCreateBuilder::new();
        let action = builder
            .with_schema_name("TestSchema".to_string())
            .with_description("Test Schema".to_string())
            .with_properties(vec![property_definition.clone()])
            .build()
            .unwrap();

        let builder = SchemaPayloadBuilder::new();
        let payload = builder
            .with_action(Action::SchemaCreate(action.clone()))
            .build()
            .unwrap();

        assert_eq!(payload.action, Action::SchemaCreate(action));
    }

    #[test]
    // check that a schema payload with update action is built correctly
    fn check_schema_update_action_payload() {
        let builder = PropertyDefinitionBuilder::new();
        let property_definition = builder
            .with_name("TEST".to_string())
            .with_data_type(DataType::String)
            .with_description("Optional".to_string())
            .build()
            .unwrap();

        let builder = SchemaUpdateBuilder::new();
        let action = builder
            .with_schema_name("TestSchema".to_string())
            .with_properties(vec![property_definition.clone()])
            .build()
            .unwrap();

        let builder = SchemaPayloadBuilder::new();
        let payload = builder
            .with_action(Action::SchemaUpdate(action.clone()))
            .build()
            .unwrap();

        assert_eq!(payload.action, Action::SchemaUpdate(action));
    }

    #[test]
    // check that a schema payload can be converted to bytes and back
    fn check_schema_payload_bytes() {
        let builder = PropertyDefinitionBuilder::new();
        let property_definition = builder
            .with_name("TEST".to_string())
            .with_data_type(DataType::String)
            .with_description("Optional".to_string())
            .build()
            .unwrap();

        let builder = SchemaUpdateBuilder::new();
        let action = builder
            .with_schema_name("TestSchema".to_string())
            .with_properties(vec![property_definition.clone()])
            .build()
            .unwrap();

        let builder = SchemaPayloadBuilder::new();
        let original = builder
            .with_action(Action::SchemaUpdate(action))
            .build()
            .unwrap();

        let bytes = original.clone().into_bytes().unwrap();

        let payload = SchemaPayload::from_bytes(&bytes).unwrap();
        assert_eq!(payload, original);
    }
}
