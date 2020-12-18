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
    FromBytes, FromNative, FromProto, IntoBytes, IntoNative, IntoProto, ProtoConversionError,
};

use super::BuilderError;

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

#[derive(Default, Clone, PartialEq)]
pub struct PropertyDefinitionBuilder {
    name: Option<String>,
    data_type: Option<DataType>,
    required: Option<bool>,
    description: Option<String>,
    number_exponent: Option<i32>,
    enum_options: Vec<String>,
    struct_properties: Vec<PropertyDefinition>,
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

    pub fn build(self) -> Result<PropertyDefinition, BuilderError> {
        let name = self
            .name
            .ok_or_else(|| BuilderError::MissingField("'name' field is required".to_string()))?;

        let data_type = self.data_type.ok_or_else(|| {
            BuilderError::MissingField("'data_type' field is required".to_string())
        })?;

        let required = self.required.unwrap_or(false);
        let description = self.description.unwrap_or_default();

        let number_exponent = {
            if data_type == DataType::Number {
                self.number_exponent.ok_or_else(|| {
                    BuilderError::MissingField("'number_exponent' field is required".to_string())
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
                    return Err(BuilderError::EmptyVec(
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
                    return Err(BuilderError::EmptyVec(
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

#[derive(Default, Clone)]
pub struct PropertyValueBuilder {
    name: Option<String>,
    data_type: Option<DataType>,
    bytes_value: Option<Vec<u8>>,
    boolean_value: Option<bool>,
    number_value: Option<i64>,
    string_value: Option<String>,
    enum_value: Option<u32>,
    struct_values: Vec<PropertyValue>,
    lat_long_value: Option<LatLong>,
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

    pub fn build(self) -> Result<PropertyValue, BuilderError> {
        let name = self
            .name
            .ok_or_else(|| BuilderError::MissingField("'name' field is required".to_string()))?;

        let data_type = self.data_type.ok_or_else(|| {
            BuilderError::MissingField("'data_type' field is required".to_string())
        })?;

        let bytes_value = {
            if data_type == DataType::Bytes {
                self.bytes_value.ok_or_else(|| {
                    BuilderError::MissingField("'bytes_value' field is required".to_string())
                })?
            } else {
                vec![]
            }
        };

        let boolean_value = {
            if data_type == DataType::Boolean {
                self.boolean_value.ok_or_else(|| {
                    BuilderError::MissingField("'boolean_value' field is required".to_string())
                })?
            } else {
                false
            }
        };

        let number_value = {
            if data_type == DataType::Number {
                self.number_value.ok_or_else(|| {
                    BuilderError::MissingField("'number_value' field is required".to_string())
                })?
            } else {
                0
            }
        };

        let string_value = {
            if data_type == DataType::String {
                self.string_value.ok_or_else(|| {
                    BuilderError::MissingField("'string_value' field is required".to_string())
                })?
            } else {
                "".to_string()
            }
        };

        let enum_value = {
            if data_type == DataType::Enum {
                self.enum_value.ok_or_else(|| {
                    BuilderError::MissingField("'enum_value' field is required".to_string())
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
                    return Err(BuilderError::MissingField(
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
                    BuilderError::MissingField("'lat_long_value' field is required".to_string())
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
    latitude: i64,
    longitude: i64,
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
