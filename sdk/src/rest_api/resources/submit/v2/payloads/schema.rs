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

use crate::rest_api::resources::submit::v2::error::BuilderError;

use std::error::Error as StdError;

use super::TransactionPayload;

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum SchemaAction {
    CreateSchema(CreateSchemaAction),
    UpdateSchema(UpdateSchemaAction),
}

impl SchemaAction {
    pub fn into_inner(self) -> Box<dyn TransactionPayload> {
        unimplemented!();
    }
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

    pub fn into_transaction_payload(self) -> Box<dyn TransactionPayload> {
        self.action.into_inner()
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct CreateSchemaAction {
    schema_name: String,
    description: String,
    properties: Vec<PropertyDefinition>,
}

impl CreateSchemaAction {
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

#[derive(Default, Clone)]
pub struct CreateSchemaBuilder {
    schema_name: Option<String>,
    description: Option<String>,
    properties: Vec<PropertyDefinition>,
}

impl CreateSchemaBuilder {
    pub fn new() -> Self {
        CreateSchemaBuilder::default()
    }

    pub fn with_schema_name(mut self, schema_name: String) -> CreateSchemaBuilder {
        self.schema_name = Some(schema_name);
        self
    }

    pub fn with_description(mut self, description: String) -> CreateSchemaBuilder {
        self.description = Some(description);
        self
    }

    pub fn with_properties(mut self, properties: Vec<PropertyDefinition>) -> CreateSchemaBuilder {
        self.properties = properties;
        self
    }

    pub fn build(self) -> Result<CreateSchemaAction, BuilderError> {
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

        Ok(CreateSchemaAction {
            schema_name,
            description,
            properties,
        })
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct UpdateSchemaAction {
    schema_name: String,
    properties: Vec<PropertyDefinition>,
}

impl UpdateSchemaAction {
    pub fn schema_name(&self) -> &str {
        &self.schema_name
    }

    pub fn properties(&self) -> &[PropertyDefinition] {
        &self.properties
    }
}

#[derive(Default, Clone)]
pub struct UpdateSchemaBuilder {
    schema_name: Option<String>,
    properties: Vec<PropertyDefinition>,
}

impl UpdateSchemaBuilder {
    pub fn new() -> Self {
        UpdateSchemaBuilder::default()
    }

    pub fn with_schema_name(mut self, schema_name: String) -> UpdateSchemaBuilder {
        self.schema_name = Some(schema_name);
        self
    }

    pub fn with_properties(mut self, properties: Vec<PropertyDefinition>) -> UpdateSchemaBuilder {
        self.properties = properties;
        self
    }

    pub fn build(self) -> Result<UpdateSchemaAction, BuilderError> {
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

        Ok(UpdateSchemaAction {
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

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
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

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum DataType {
    Bytes,
    Boolean,
    Number,
    String,
    Enum,
    Struct,
    LatLong,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
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

#[derive(Default, Clone, PartialEq, Eq)]
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

        if !(-90_000_000..=90_000_000).contains(&latitude) {
            Err(LatLongBuildError::InvalidLatitude(latitude))
        } else if !(-180_000_000..=180_000_000).contains(&longitude) {
            Err(LatLongBuildError::InvalidLongitude(longitude))
        } else {
            Ok(LatLong {
                latitude,
                longitude,
            })
        }
    }
}
