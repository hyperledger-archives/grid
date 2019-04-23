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

use crate::error::CliError;
use grid_sdk::protocol::schema::payload::{
    Action, SchemaCreateBuilder, SchemaPayload, SchemaPayloadBuilder, SchemaUpdateBuilder,
};
use grid_sdk::protocol::schema::state::{DataType, PropertyDefinition, PropertyDefinitionBuilder};
use serde_yaml::{Mapping, Sequence, Value};

fn parse_yaml(path: &str, action: Action) -> Result<Vec<SchemaPayload>, CliError> {
    let file = std::fs::File::open(path)?;
    let schemas_yaml: Vec<Mapping> = serde_yaml::from_reader(file)?;

    match action {
        Action::SchemaCreate => schemas_yaml
            .iter()
            .map(|schema_yaml| {
                let properties =
                    parse_value_as_sequence(schema_yaml, "properties")?.ok_or_else(|| {
                        CliError::InvalidYamlError(
                            "Schema definition is missing `properties` field.".to_string(),
                        )
                    })?;
                let property_definitions = parse_properties(&properties)?;
                let schema_name = parse_value_as_string(schema_yaml, "name")?.ok_or_else(|| {
                    CliError::InvalidYamlError("Missing `name` field for schema.".to_string())
                })?;
                let schema_description = parse_value_as_string(schema_yaml, "description")?;

                generate_create_schema_payload(
                    &schema_name,
                    &property_definitions,
                    schema_description,
                )
            })
            .collect::<Result<Vec<SchemaPayload>, _>>(),

        Action::SchemaUpdate => schemas_yaml
            .iter()
            .map(|schema_yaml| {
                let properties =
                    parse_value_as_sequence(schema_yaml, "properties")?.ok_or_else(|| {
                        CliError::InvalidYamlError(
                            "Schema definition is missing `properties` field.".to_string(),
                        )
                    })?;
                let property_definitions = parse_properties(&properties)?;
                let schema_name = parse_value_as_string(schema_yaml, "name")?.ok_or_else(|| {
                    CliError::InvalidYamlError("Missing `name` field for schema.".to_string())
                })?;

                generate_update_schema_payload(&schema_name, &property_definitions)
            })
            .collect::<Result<Vec<SchemaPayload>, _>>(),
    }
}

fn generate_create_schema_payload(
    name: &str,
    properties: &[PropertyDefinition],
    description: Option<String>,
) -> Result<SchemaPayload, CliError> {
    let mut schema_paylod = SchemaPayloadBuilder::new();
    schema_paylod = schema_paylod.with_action(Action::SchemaCreate);

    let mut schema_create_action_builder = SchemaCreateBuilder::new()
        .with_schema_name(name.to_string())
        .with_properties(properties.to_vec());

    schema_create_action_builder = match description {
        Some(description) => schema_create_action_builder.with_description(description),
        None => schema_create_action_builder,
    };

    let schema_create_action = schema_create_action_builder.build().map_err(|err| {
        CliError::PayloadError(format!("Failed to build schema payload: {}", err))
    })?;

    schema_paylod = schema_paylod.with_schema_create(schema_create_action);
    schema_paylod
        .build()
        .map_err(|err| CliError::PayloadError(format!("Failed to build schema payload: {}", err)))
}

fn generate_update_schema_payload(
    name: &str,
    properties: &[PropertyDefinition],
) -> Result<SchemaPayload, CliError> {
    let mut schema_paylod = SchemaPayloadBuilder::new();
    schema_paylod = schema_paylod.with_action(Action::SchemaUpdate);

    let schema_update_action_builder = SchemaUpdateBuilder::new()
        .with_schema_name(name.to_string())
        .with_properties(properties.to_vec());

    let schema_update_action = schema_update_action_builder.build().map_err(|err| {
        CliError::PayloadError(format!("Failed to build schema payload: {}", err))
    })?;

    schema_paylod = schema_paylod.with_schema_update(schema_update_action);
    schema_paylod
        .build()
        .map_err(|err| CliError::PayloadError(format!("Failed to build schema payload: {}", err)))
}

fn parse_properties(properties: &[Value]) -> Result<Vec<PropertyDefinition>, CliError> {
    properties
        .iter()
        .map(|value| {
            let property = value.as_mapping().ok_or_else(|| {
                CliError::InvalidYamlError(
                    "Failed to parse schema property definition.".to_string(),
                )
            })?;
            parse_property_definition(property)
        })
        .collect()
}

fn parse_property_definition(property: &Mapping) -> Result<PropertyDefinition, CliError> {
    let data_type = parse_data_type(&parse_value_as_string(property, "data_type")?.ok_or_else(
        || {
            CliError::InvalidYamlError(
                "Missing `data_type` field for property definition.".to_string(),
            )
        },
    )?)?;

    let mut property_definition = PropertyDefinitionBuilder::new()
        .with_name(parse_value_as_string(property, "name")?.ok_or_else(|| {
            CliError::InvalidYamlError("Missing `name` field for property definition.".to_string())
        })?)
        .with_data_type(data_type.clone());

    property_definition = match parse_value_as_string(property, "description")? {
        Some(description) => property_definition.with_description(description),
        None => property_definition,
    };

    property_definition = match parse_value_as_boolean(property, "required")? {
        Some(required) => property_definition.with_required(required),
        None => property_definition,
    };

    property_definition = match data_type {
        DataType::Number => property_definition.with_number_exponent(
            parse_value_as_i32(property, "number_exponent")?.ok_or_else(|| {
                CliError::InvalidYamlError(
                    "Missing `number_exponent` field for property definition with type NUMBER."
                        .to_string(),
                )
            })?,
        ),

        DataType::Struct => {
            let properties = parse_properties(
                property
                    .get(&Value::String("struct_properties".to_string()))
                    .unwrap()
                    .as_sequence()
                    .unwrap(),
            )?;
            property_definition.with_struct_properties(properties)
        }
        DataType::Enum => property_definition.with_enum_options(
            parse_value_as_vec_string(property, "enum_options")?.ok_or_else(|| {
                CliError::InvalidYamlError(
                    "Missing `enum_options` field for property definition with type ENUM."
                        .to_string(),
                )
            })?,
        ),
        _ => property_definition,
    };

    property_definition.build().map_err(|err| {
        CliError::PayloadError(format!("Failed to build property definition: {}", err))
    })
}

fn parse_data_type(data_type: &str) -> Result<DataType, CliError> {
    match data_type.to_lowercase().as_ref() {
        "string" => Ok(DataType::String),
        "bytes" => Ok(DataType::Bytes),
        "number" => Ok(DataType::Number),
        "enum" => Ok(DataType::Enum),
        "struct" => Ok(DataType::Struct),
        _ => Err(CliError::InvalidYamlError(format!(
            "Invalid data type for PropertyDefinition: {}",
            data_type
        ))),
    }
}

fn parse_value_as_sequence(property: &Mapping, key: &str) -> Result<Option<Sequence>, CliError> {
    match property.get(&Value::String(key.to_string())) {
        Some(value) => match value.as_sequence() {
            Some(value) => Ok(Some(value.to_vec())),
            None => Err(CliError::InvalidYamlError(format!(
                "Value of {} has an invalid format. Expected is a yaml list.",
                key
            ))),
        },
        None => Ok(None),
    }
}

fn parse_value_as_string(property: &Mapping, key: &str) -> Result<Option<String>, CliError> {
    match property.get(&Value::String(key.to_string())) {
        Some(value) => match value.as_str() {
            Some(val) => Ok(Some(val.to_string())),
            None => Err(CliError::InvalidYamlError(format!(
                "Value of {} has an invalid format. Expected is a yaml string.",
                key
            ))),
        },
        None => Ok(None),
    }
}

fn parse_value_as_boolean(property: &Mapping, key: &str) -> Result<Option<bool>, CliError> {
    match property.get(&Value::String(key.to_string())) {
        Some(value) => match value.as_bool() {
            Some(value) => Ok(Some(value)),
            None => Err(CliError::InvalidYamlError(format!(
                "Value of {} has an invalid format. Expected is a yaml boolean (true/false).",
                key
            ))),
        },
        None => Ok(None),
    }
}

fn parse_value_as_i32(property: &Mapping, key: &str) -> Result<Option<i32>, CliError> {
    match property.get(&Value::String(key.to_string())) {
        Some(value) => match value.as_i64() {
            Some(value) => Ok(Some(value.to_string().parse::<i32>().map_err(|_| {
                CliError::InvalidYamlError(format!(
                    "Failed to parse value of {} to 32 bit integer",
                    key
                ))
            })?)),
            None => Err(CliError::InvalidYamlError(format!(
                "Value of {} has an invalid format. Expected is a yaml integer.",
                key
            ))),
        },
        None => Ok(None),
    }
}

fn parse_value_as_vec_string(
    property: &Mapping,
    key: &str,
) -> Result<Option<Vec<String>>, CliError> {
    match property.get(&Value::String(key.to_string())) {
        Some(value) => match value.as_sequence() {
            Some(sequence) => Ok(Some(
                sequence
                    .iter()
                    .map(|value| match value.as_str() {
                        Some(value) => Ok(value.to_string()),
                        None => Err(CliError::InvalidYamlError(format!(
                            "Values in {} cannot be parsed to string.",
                            key
                        ))),
                    })
                    .collect::<Result<Vec<String>, CliError>>()?,
            )),
            None => Err(CliError::InvalidYamlError(format!(
                "Value of {} has an invalid format. Expected is a yaml list.",
                key
            ))),
        },
        None => Ok(None),
    }
}
