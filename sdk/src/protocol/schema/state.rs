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

/// Native implementation of DataType enum
#[derive(Debug, Clone, PartialEq)]
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
        proto_lat_long.set_latitude(lat_long.latitude().clone());
        proto_lat_long.set_longitude(lat_long.longitude().clone());
        Ok(proto_lat_long)
    }
}

impl IntoProto<protos::schema_state::LatLong> for LatLong {}
impl IntoNative<LatLong> for protos::schema_state::LatLong {}

/// Native implementation of PropertyDefinition
#[derive(Debug, Clone, PartialEq)]
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
        proto_property_definition.set_required(property_definition.required().clone());
        proto_property_definition.set_description(property_definition.description().to_string());
        proto_property_definition
            .set_number_exponent(property_definition.number_exponent().clone());
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
        let proto: protos::schema_state::PropertyDefinition = protobuf::parse_from_bytes(bytes)
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

#[derive(Debug)]
pub enum PropertyDefinitionBuildError {
    MissingField(String),
    EmptyVec(String),
}

impl StdError for PropertyDefinitionBuildError {
    fn description(&self) -> &str {
        match *self {
            PropertyDefinitionBuildError::MissingField(ref msg) => msg,
            PropertyDefinitionBuildError::EmptyVec(ref msg) => msg,
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            PropertyDefinitionBuildError::MissingField(_) => None,
            PropertyDefinitionBuildError::EmptyVec(_) => None,
        }
    }
}

impl std::fmt::Display for PropertyDefinitionBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            PropertyDefinitionBuildError::MissingField(ref s) => write!(f, "MissingField: {}", s),
            PropertyDefinitionBuildError::EmptyVec(ref s) => write!(f, "EmptyVec: {}", s),
        }
    }
}

/// Builder used to create a PropertyDefinition
#[derive(Default, Clone, PartialEq)]
pub struct PropertyDefinitionBuilder {
    pub name: Option<String>,
    pub data_type: Option<DataType>,
    pub required: Option<bool>,
    pub description: Option<String>,
    pub number_exponent: Option<i32>,
    pub enum_options: Vec<String>,
    pub struct_properties: Vec<PropertyDefinition>,
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

    pub fn build(self) -> Result<PropertyDefinition, PropertyDefinitionBuildError> {
        let name = self.name.ok_or_else(|| {
            PropertyDefinitionBuildError::MissingField("'name' field is required".to_string())
        })?;

        let data_type = self.data_type.ok_or_else(|| {
            PropertyDefinitionBuildError::MissingField("'data_type' field is required".to_string())
        })?;

        let required = self.required.unwrap_or_else(|| false);
        let description = self.description.unwrap_or_default();

        let number_exponent = {
            if data_type == DataType::Number {
                self.number_exponent.ok_or_else(|| {
                    PropertyDefinitionBuildError::MissingField(
                        "'number_exponent' field is required".to_string(),
                    )
                })?
            } else {
                0 as i32
            }
        };

        let enum_options = {
            if data_type == DataType::Enum {
                if !self.enum_options.is_empty() {
                    self.enum_options
                } else {
                    return Err(PropertyDefinitionBuildError::EmptyVec(
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
                    return Err(PropertyDefinitionBuildError::EmptyVec(
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

/// Native implementation of Schema
#[derive(Debug, Clone, PartialEq)]
pub struct Schema {
    name: String,
    description: String,
    owner: String,
    properties: Vec<PropertyDefinition>,
}

impl Schema {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn owner(&self) -> &str {
        &self.owner
    }

    pub fn properties(&self) -> &[PropertyDefinition] {
        &self.properties
    }
}

impl FromProto<protos::schema_state::Schema> for Schema {
    fn from_proto(schema: protos::schema_state::Schema) -> Result<Self, ProtoConversionError> {
        Ok(Schema {
            name: schema.get_name().to_string(),
            description: schema.get_description().to_string(),
            owner: schema.get_owner().to_string(),
            properties: schema
                .get_properties()
                .to_vec()
                .into_iter()
                .map(PropertyDefinition::from_proto)
                .collect::<Result<Vec<PropertyDefinition>, ProtoConversionError>>()?,
        })
    }
}

impl FromNative<Schema> for protos::schema_state::Schema {
    fn from_native(schema: Schema) -> Result<Self, ProtoConversionError> {
        let mut proto_schema = protos::schema_state::Schema::new();
        proto_schema.set_name(schema.name().to_string());
        proto_schema.set_description(schema.description().to_string());
        proto_schema.set_owner(schema.owner().to_string());
        proto_schema.set_properties(RepeatedField::from_vec(
            schema
                .properties()
                .to_vec()
                .into_iter()
                .map(PropertyDefinition::into_proto)
                .collect::<Result<Vec<protos::schema_state::PropertyDefinition>, ProtoConversionError>>()?,
        ));
        Ok(proto_schema)
    }
}

impl FromBytes<Schema> for Schema {
    fn from_bytes(bytes: &[u8]) -> Result<Schema, ProtoConversionError> {
        let proto: protos::schema_state::Schema =
            protobuf::parse_from_bytes(bytes).map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get Schema from bytes".to_string(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for Schema {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError("Unable to get bytes from Schema".to_string())
        })?;
        Ok(bytes)
    }
}

impl IntoProto<protos::schema_state::Schema> for Schema {}
impl IntoNative<Schema> for protos::schema_state::Schema {}

#[derive(Debug)]
pub enum SchemaBuildError {
    MissingField(String),
}

impl StdError for SchemaBuildError {
    fn description(&self) -> &str {
        match *self {
            SchemaBuildError::MissingField(ref msg) => msg,
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            SchemaBuildError::MissingField(_) => None,
        }
    }
}

impl std::fmt::Display for SchemaBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            SchemaBuildError::MissingField(ref s) => write!(f, "MissingField: {}", s),
        }
    }
}

/// Builder used to create a Schema
#[derive(Default, Clone)]
pub struct SchemaBuilder {
    pub name: Option<String>,
    pub description: Option<String>,
    pub owner: Option<String>,
    pub properties: Vec<PropertyDefinition>,
}

impl SchemaBuilder {
    pub fn new() -> Self {
        SchemaBuilder::default()
    }

    pub fn with_name(mut self, name: String) -> SchemaBuilder {
        self.name = Some(name);
        self
    }

    pub fn with_description(mut self, description: String) -> SchemaBuilder {
        self.description = Some(description);
        self
    }

    pub fn with_owner(mut self, owner: String) -> SchemaBuilder {
        self.owner = Some(owner);
        self
    }

    pub fn with_properties(mut self, properties: Vec<PropertyDefinition>) -> SchemaBuilder {
        self.properties = properties;
        self
    }

    pub fn build(self) -> Result<Schema, SchemaBuildError> {
        let name = self.name.ok_or_else(|| {
            SchemaBuildError::MissingField("'name' field is required".to_string())
        })?;

        let owner = self.owner.ok_or_else(|| {
            SchemaBuildError::MissingField("'owner' field is required".to_string())
        })?;

        let description = self.description.unwrap_or_else(|| "".to_string());
        let properties = {
            if !self.properties.is_empty() {
                self.properties
            } else {
                return Err(SchemaBuildError::MissingField(
                    "'properties' field is required".to_string(),
                ));
            }
        };

        Ok(Schema {
            name,
            description,
            owner,
            properties,
        })
    }
}

/// Native implementation of SchemaList
#[derive(Debug, Clone, PartialEq)]
pub struct SchemaList {
    schemas: Vec<Schema>,
}

impl SchemaList {
    pub fn schemas(&self) -> &[Schema] {
        &self.schemas
    }
}

impl FromProto<protos::schema_state::SchemaList> for SchemaList {
    fn from_proto(
        schema_list: protos::schema_state::SchemaList,
    ) -> Result<Self, ProtoConversionError> {
        Ok(SchemaList {
            schemas: schema_list
                .get_schemas()
                .to_vec()
                .into_iter()
                .map(Schema::from_proto)
                .collect::<Result<Vec<Schema>, ProtoConversionError>>()?,
        })
    }
}

impl FromNative<SchemaList> for protos::schema_state::SchemaList {
    fn from_native(schema_list: SchemaList) -> Result<Self, ProtoConversionError> {
        let mut schema_list_proto = protos::schema_state::SchemaList::new();

        schema_list_proto.set_schemas(RepeatedField::from_vec(
            schema_list
                .schemas()
                .to_vec()
                .into_iter()
                .map(Schema::into_proto)
                .collect::<Result<Vec<protos::schema_state::Schema>, ProtoConversionError>>()?,
        ));

        Ok(schema_list_proto)
    }
}

impl FromBytes<SchemaList> for SchemaList {
    fn from_bytes(bytes: &[u8]) -> Result<SchemaList, ProtoConversionError> {
        let proto: protos::schema_state::SchemaList =
            protobuf::parse_from_bytes(bytes).map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get SchemaList from bytes".to_string(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for SchemaList {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from SchemaList".to_string(),
            )
        })?;
        Ok(bytes)
    }
}

impl IntoProto<protos::schema_state::SchemaList> for SchemaList {}
impl IntoNative<SchemaList> for protos::schema_state::SchemaList {}

#[derive(Debug)]
pub enum SchemaListBuildError {
    MissingField(String),
}

impl StdError for SchemaListBuildError {
    fn description(&self) -> &str {
        match *self {
            SchemaListBuildError::MissingField(ref msg) => msg,
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            SchemaListBuildError::MissingField(_) => None,
        }
    }
}

impl std::fmt::Display for SchemaListBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            SchemaListBuildError::MissingField(ref s) => write!(f, "MissingField: {}", s),
        }
    }
}

/// Builder used to create a SchemaList
#[derive(Default, Clone)]
pub struct SchemaListBuilder {
    pub schemas: Vec<Schema>,
}

impl SchemaListBuilder {
    pub fn new() -> Self {
        SchemaListBuilder::default()
    }

    pub fn with_schemas(mut self, schemas: Vec<Schema>) -> SchemaListBuilder {
        self.schemas = schemas;
        self
    }

    pub fn build(self) -> Result<SchemaList, SchemaListBuildError> {
        let schemas = {
            if self.schemas.is_empty() {
                return Err(SchemaListBuildError::MissingField(
                    "'schemas' cannot be empty".to_string(),
                ));
            } else {
                self.schemas
            }
        };

        Ok(SchemaList { schemas })
    }
}

/// Native implementation of PropertyValue
#[derive(Debug, Clone, PartialEq)]
pub struct PropertyValue {
    name: String,
    data_type: DataType,
    bytes_value: Vec<u8>,
    boolean_value: bool,
    number_value: i64,
    string_value: String,
    enum_value: u32,
    struct_values: Vec<PropertyValue>,
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
        })
    }
}

impl FromNative<PropertyValue> for protos::schema_state::PropertyValue {
    fn from_native(property_value: PropertyValue) -> Result<Self, ProtoConversionError> {
        let mut proto_property_value = protos::schema_state::PropertyValue::new();
        proto_property_value.set_name(property_value.name().to_string());
        proto_property_value.set_data_type(property_value.data_type().clone().into_proto()?);
        proto_property_value.set_bytes_value(property_value.bytes_value().to_vec());
        proto_property_value.set_boolean_value(property_value.boolean_value().clone());
        proto_property_value.set_number_value(property_value.number_value().clone());
        proto_property_value.set_string_value(property_value.string_value().to_string());
        proto_property_value.set_enum_value(property_value.enum_value().clone());
        proto_property_value.set_struct_values(RepeatedField::from_vec(
            property_value
                .struct_values()
                .to_vec()
                .into_iter()
                .map(PropertyValue::into_proto)
                .collect::<Result<Vec<protos::schema_state::PropertyValue>, ProtoConversionError>>(
                )?,
        ));
        Ok(proto_property_value)
    }
}

impl FromBytes<PropertyValue> for PropertyValue {
    fn from_bytes(bytes: &[u8]) -> Result<PropertyValue, ProtoConversionError> {
        let proto: protos::schema_state::PropertyValue = protobuf::parse_from_bytes(bytes)
            .map_err(|_| {
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

impl IntoProto<protos::schema_state::PropertyValue> for PropertyValue {}
impl IntoNative<PropertyValue> for protos::schema_state::PropertyValue {}

#[derive(Debug)]
pub enum PropertyValueBuildError {
    MissingField(String),
}

impl StdError for PropertyValueBuildError {
    fn description(&self) -> &str {
        match *self {
            PropertyValueBuildError::MissingField(ref msg) => msg,
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            PropertyValueBuildError::MissingField(_) => None,
        }
    }
}

impl std::fmt::Display for PropertyValueBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            PropertyValueBuildError::MissingField(ref s) => write!(f, "MissingField: {}", s),
        }
    }
}

/// Builder used to create a PropertyValue
#[derive(Default, Clone)]
pub struct PropertyValueBuilder {
    pub name: Option<String>,
    pub data_type: Option<DataType>,
    pub bytes_value: Option<Vec<u8>>,
    pub boolean_value: Option<bool>,
    pub number_value: Option<i64>,
    pub string_value: Option<String>,
    pub enum_value: Option<u32>,
    pub struct_values: Vec<PropertyValue>,
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

    pub fn build(self) -> Result<PropertyValue, PropertyValueBuildError> {
        let name = self.name.ok_or_else(|| {
            PropertyValueBuildError::MissingField("'name' field is required".to_string())
        })?;

        let data_type = self.data_type.ok_or_else(|| {
            PropertyValueBuildError::MissingField("'data_type' field is required".to_string())
        })?;

        let bytes_value = {
            if data_type == DataType::Bytes {
                self.bytes_value.ok_or_else(|| {
                    PropertyValueBuildError::MissingField(
                        "'bytes_value' field is required".to_string(),
                    )
                })?
            } else {
                vec![]
            }
        };

        let boolean_value = {
            if data_type == DataType::Boolean {
                self.boolean_value.ok_or_else(|| {
                    PropertyValueBuildError::MissingField(
                        "'boolean_value' field is required".to_string(),
                    )
                })?
            } else {
                false
            }
        };

        let number_value = {
            if data_type == DataType::Number {
                self.number_value.ok_or_else(|| {
                    PropertyValueBuildError::MissingField(
                        "'number_value' field is required".to_string(),
                    )
                })?
            } else {
                0 as i64
            }
        };

        let string_value = {
            if data_type == DataType::String {
                self.string_value.ok_or_else(|| {
                    PropertyValueBuildError::MissingField(
                        "'string_value' field is required".to_string(),
                    )
                })?
            } else {
                "".to_string()
            }
        };

        let enum_value = {
            if data_type == DataType::Enum {
                self.enum_value.ok_or_else(|| {
                    PropertyValueBuildError::MissingField(
                        "'enum_value' field is required".to_string(),
                    )
                })?
            } else {
                0 as u32
            }
        };

        let struct_values = {
            if data_type == DataType::Struct {
                if !self.struct_values.is_empty() {
                    self.struct_values
                } else {
                    return Err(PropertyValueBuildError::MissingField(
                        "'struct_values' cannot be empty".to_string(),
                    ));
                }
            } else {
                self.struct_values
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
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // check that a property definition with a string data type is built correctly
    fn check_property_definition_builder_string() {
        let builder = PropertyDefinitionBuilder::new();
        let property_definition = builder
            .with_name("TEST".to_string())
            .with_data_type(DataType::String)
            .with_description("Optional".to_string())
            .build()
            .unwrap();

        assert_eq!(property_definition.name, "TEST");
        assert_eq!(property_definition.data_type, DataType::String);
        assert_eq!(property_definition.description, "Optional");
    }

    #[test]
    // check that a property definition with a enum data type is built correctly
    fn check_property_definition_builder_enum() {
        let builder = PropertyDefinitionBuilder::new();
        let property_definition = builder
            .with_name("TEST".to_string())
            .with_data_type(DataType::Enum)
            .with_description("Optional".to_string())
            .with_enum_options(vec![
                "One".to_string(),
                "Two".to_string(),
                "Three".to_string(),
            ])
            .build()
            .unwrap();

        assert_eq!(property_definition.name, "TEST");
        assert_eq!(property_definition.data_type, DataType::Enum);
        assert_eq!(property_definition.description, "Optional");
        assert_eq!(
            property_definition.enum_options,
            vec!["One".to_string(), "Two".to_string(), "Three".to_string()]
        );
    }

    #[test]
    // check that a property definitionwith a struct data type is built correctly
    fn check_property_definition_builder_struct() {
        let builder = PropertyDefinitionBuilder::new();
        let struct_string = builder
            .with_name("TEST".to_string())
            .with_data_type(DataType::Enum)
            .with_description("Optional".to_string())
            .with_enum_options(vec![
                "One".to_string(),
                "Two".to_string(),
                "Three".to_string(),
            ])
            .build()
            .unwrap();

        let builder = PropertyDefinitionBuilder::new();
        let property_definition = builder
            .with_name("TEST_STRUCT".to_string())
            .with_data_type(DataType::Struct)
            .with_description("Optional".to_string())
            .with_struct_properties(vec![struct_string.clone()])
            .build()
            .unwrap();

        assert_eq!(property_definition.name, "TEST_STRUCT");
        assert_eq!(property_definition.data_type, DataType::Struct);
        assert_eq!(property_definition.description, "Optional");
        assert_eq!(property_definition.struct_properties, vec![struct_string]);
    }

    #[test]
    // check that a property definition can be converted to bytes and back
    fn check_property_definition_bytes() {
        let builder = PropertyDefinitionBuilder::new();
        let original = builder
            .with_name("TEST".to_string())
            .with_data_type(DataType::String)
            .with_description("Optional".to_string())
            .build()
            .unwrap();

        let bytes = original.clone().into_bytes().unwrap();

        let property_definition = PropertyDefinition::from_bytes(&bytes).unwrap();
        assert_eq!(property_definition, original);
    }

    #[test]
    // check that a schema with a enum property is built correctly
    fn check_schema_builder() {
        let builder = PropertyDefinitionBuilder::new();
        let property_definition = builder
            .with_name("TEST".to_string())
            .with_data_type(DataType::Enum)
            .with_description("Optional".to_string())
            .with_enum_options(vec![
                "One".to_string(),
                "Two".to_string(),
                "Three".to_string(),
            ])
            .build()
            .unwrap();

        let builder = SchemaBuilder::new();
        let schema = builder
            .with_name("TestSchema".to_string())
            .with_description("Test Schema".to_string())
            .with_owner("owner".to_string())
            .with_properties(vec![property_definition.clone()])
            .build()
            .unwrap();

        assert_eq!(schema.name, "TestSchema");
        assert_eq!(schema.description, "Test Schema");
        assert_eq!(schema.owner, "owner");
        assert_eq!(schema.properties, vec![property_definition]);
    }

    #[test]
    // check that a schema can be converted to bytes and back
    fn check_schema_bytes() {
        let builder = PropertyDefinitionBuilder::new();
        let property_definition = builder
            .with_name("TEST".to_string())
            .with_data_type(DataType::Enum)
            .with_description("Optional".to_string())
            .with_enum_options(vec![
                "One".to_string(),
                "Two".to_string(),
                "Three".to_string(),
            ])
            .build()
            .unwrap();

        let builder = SchemaBuilder::new();
        let original = builder
            .with_name("TestSchema".to_string())
            .with_description("Test Schema".to_string())
            .with_owner("owner".to_string())
            .with_properties(vec![property_definition.clone()])
            .build()
            .unwrap();

        let bytes = original.clone().into_bytes().unwrap();

        let schema = Schema::from_bytes(&bytes).unwrap();
        assert_eq!(schema, original);
    }

    #[test]
    // check that a property value with a string data type is built correctly
    fn check_property_value_builder_string() {
        let builder = PropertyValueBuilder::new();
        let property_value = builder
            .with_name("TEST".to_string())
            .with_data_type(DataType::String)
            .with_string_value("String value".to_string())
            .build()
            .unwrap();

        assert_eq!(property_value.name, "TEST");
        assert_eq!(property_value.data_type, DataType::String);
        assert_eq!(property_value.string_value, "String value");
    }

    #[test]
    // check that a property value with a struct data type is built correctly
    fn check_property_value_builder_struct() {
        let builder = PropertyValueBuilder::new();
        let string_value = builder
            .with_name("TEST".to_string())
            .with_data_type(DataType::String)
            .with_string_value("String value".to_string())
            .build()
            .unwrap();

        let builder = PropertyValueBuilder::new();
        let property_value = builder
            .with_name("TEST_STRUCT".to_string())
            .with_data_type(DataType::Struct)
            .with_struct_values(vec![string_value.clone()])
            .build()
            .unwrap();

        assert_eq!(property_value.name, "TEST_STRUCT");
        assert_eq!(property_value.data_type, DataType::Struct);
        assert_eq!(property_value.struct_values, vec![string_value]);
    }

    #[test]
    // check that a property value can be converted to bytes and back
    fn check_property_value_bytes() {
        let builder = PropertyValueBuilder::new();
        let original = builder
            .with_name("TEST".to_string())
            .with_data_type(DataType::String)
            .with_string_value("String value".to_string())
            .build()
            .unwrap();

        let bytes = original.clone().into_bytes().unwrap();

        let property_value = PropertyValue::from_bytes(&bytes).unwrap();
        assert_eq!(property_value, original);
    }
}
