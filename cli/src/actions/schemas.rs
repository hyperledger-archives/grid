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

use crate::http::submit_batches;
use crate::transaction::{schema_batch_builder, GRID_SCHEMA_NAMESPACE, PIKE_NAMESPACE};
use grid_sdk::protocol::schema::payload::{
    Action, SchemaCreateBuilder, SchemaPayload, SchemaPayloadBuilder, SchemaUpdateBuilder,
};
use grid_sdk::protocol::schema::state::{DataType, PropertyDefinition, PropertyDefinitionBuilder};
use grid_sdk::protos::IntoProto;

use crate::error::CliError;
use serde_yaml::{Mapping, Sequence, Value};

pub fn do_create_schemas(
    url: &str,
    key: Option<String>,
    wait: u64,
    path: &str,
) -> Result<(), CliError> {
    let payloads = parse_yaml(path, Action::SchemaCreate)?;
    let mut batch_list_builder = schema_batch_builder(key);
    for payload in payloads {
        batch_list_builder = batch_list_builder.add_transaction(
            &payload.into_proto()?,
            &[
                PIKE_NAMESPACE.to_string(),
                GRID_SCHEMA_NAMESPACE.to_string(),
            ],
            &[GRID_SCHEMA_NAMESPACE.to_string()],
        )?;
    }

    let batch_list = batch_list_builder.create_batch_list();

    submit_batches(url, wait, &batch_list)
}

pub fn do_update_schemas(
    url: &str,
    key: Option<String>,
    wait: u64,
    path: &str,
) -> Result<(), CliError> {
    let payloads = parse_yaml(path, Action::SchemaUpdate)?;
    let mut batch_list_builder = schema_batch_builder(key);
    for payload in payloads {
        batch_list_builder = batch_list_builder.add_transaction(
            &payload.into_proto()?,
            &[
                PIKE_NAMESPACE.to_string(),
                GRID_SCHEMA_NAMESPACE.to_string(),
            ],
            &[GRID_SCHEMA_NAMESPACE.to_string()],
        )?;
    }

    let batch_list = batch_list_builder.create_batch_list();

    submit_batches(url, wait, &batch_list)
}

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
        "lat_long" => Ok(DataType::LatLong),
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

#[cfg(test)]
mod test {
    use super::*;
    use grid_sdk::protocol::schema::payload::{Action, SchemaPayload};
    use grid_sdk::protocol::schema::state::{
        DataType, PropertyDefinition, PropertyDefinitionBuilder,
    };
    use serde_yaml::{Mapping, Value};
    use std::env;
    use std::fs::{remove_file, File};
    use std::io::Write;
    use std::panic;
    use std::thread;

    static LIGHTBULB_YAML_EXAMPLE: &[u8; 800] = br##"- name: "Lightbulb"
  description: "Example Lightbulb schema"
  properties:
    - name: "size"
      data_type: NUMBER
      description: "Lightbulb radius, in millimeters"
      number_exponent: 0
      required: true
    - name: "bulb_type"
      data_type: ENUM
      description: "Bulb type"
      enum_options: ["filament", "CF", "LED"]
      required: true
    - name: "energy_rating"
      data_type: NUMBER
      description: "EnergyStar energy rating"
      number_exponent: 0
    - name: "color"
      data_type: STRUCT
      description: "A named RGB Color value"
      struct_properties:
            - name: 'name'
              data_type: STRING
              description: "Color name"
            - name: 'rgb_hex'
              data_type: STRING
              description: "RGB value" "##;

    static PHONE_YAML_EXAMPLE: &[u8; 630] = br##"
- name: "Phone"
  description: "Example phone schema"
  properties:
      - name: "brand"
        data_type: STRING
        description: "Name of the brand"
        required: true
      - name: "operation_system"
        data_type: ENUM
        description: "Phone's operation system"
        enum_options: ["Android", "iOS"]
        required: true
      - name: "price"
        data_type: NUMBER
        description: "Price of phone rounded to the nearest dollar"
        number_exponent: 0
      - name: "manufacturer_location"
        data_type: lat_long
        description: "Location where manufacturer is headquarted.""##;

    ///
    /// Verifies parse_yaml returns a valid SchemaPayload with SchemaCreateAction set from a yaml
    /// containing a single schema definition
    ///
    #[test]
    fn test_valid_yaml_create_one_schema() {
        run_test(|test_yaml_file_path| {
            let mut file =
                File::create(test_yaml_file_path).expect("Error creating test schema yaml file.");

            file.write_all(LIGHTBULB_YAML_EXAMPLE)
                .expect("Error writting example schema.");

            let payload =
                parse_yaml(test_yaml_file_path, Action::SchemaCreate).expect("Error parsing yaml");

            assert_eq!(make_create_schema_payload_1(), payload[0]);
        })
    }

    ///
    /// Verifies parse_yaml returns valids SchemaPayload with SchemaCreateAction set from a yaml
    /// containing a multiple schema definitions
    ///
    #[test]
    fn test_valid_yaml_create_multiple_schemas() {
        run_test(|test_yaml_file_path| {
            let mut file =
                File::create(test_yaml_file_path).expect("Error creating test schema yaml file.");
            file.write(LIGHTBULB_YAML_EXAMPLE)
                .expect("Error writting example schema.");
            file.write(PHONE_YAML_EXAMPLE)
                .expect("Error writting example schema.");

            let payload =
                parse_yaml(test_yaml_file_path, Action::SchemaCreate).expect("Error parsing yaml");

            assert_eq!(make_create_schema_payload_1(), payload[0]);
            assert_eq!(make_create_schema_payload_2(), payload[1]);
        })
    }

    ///
    /// Verifies parse_yaml returns a valid SchemaPayload with SchemaUpdateAction set from a yaml
    /// containing a single schema definition
    ///
    #[test]
    fn test_valid_yaml_update_one_schema() {
        run_test(|test_yaml_file_path| {
            let mut file =
                File::create(test_yaml_file_path).expect("Error creating test schema yaml file.");

            file.write_all(LIGHTBULB_YAML_EXAMPLE)
                .expect("Error writting example schema.");

            let payload =
                parse_yaml(test_yaml_file_path, Action::SchemaUpdate).expect("Error parsing yaml");

            assert_eq!(make_update_schema_payload_1(), payload[0]);
        })
    }

    ///
    /// Verifies parse_yaml returns valids SchemaPayload with SchemaUpdateAction set from a yaml
    /// containing a multiple schema definitions
    ///
    #[test]
    fn test_valid_yaml_update_multiple_schemas() {
        run_test(|test_yaml_file_path| {
            let mut file =
                File::create(test_yaml_file_path).expect("Error creating test schema yaml file.");
            file.write(LIGHTBULB_YAML_EXAMPLE)
                .expect("Error writting example schema.");
            file.write(PHONE_YAML_EXAMPLE)
                .expect("Error writting example schema.");

            let payload =
                parse_yaml(test_yaml_file_path, Action::SchemaUpdate).expect("Error parsing yaml");

            assert_eq!(make_update_schema_payload_1(), payload[0]);
            assert_eq!(make_update_schema_payload_2(), payload[1]);
        })
    }

    ///
    /// Verifies parse_data_type returns the expected data_types for valid inputs and returns an
    /// error for a invalid input
    ///
    #[test]
    fn test_parse_data_type() {
        // Check the method returns the expected data types for each valid input
        assert_eq!(parse_data_type("string").unwrap(), DataType::String);
        assert_eq!(parse_data_type("bytes").unwrap(), DataType::Bytes);
        assert_eq!(parse_data_type("NUMBER").unwrap(), DataType::Number);
        assert_eq!(parse_data_type("enum").unwrap(), DataType::Enum);
        assert_eq!(parse_data_type("Struct").unwrap(), DataType::Struct);
        assert_eq!(parse_data_type("lat_long").unwrap(), DataType::LatLong);

        // Check the method returns an error for an invalid input
        assert!(parse_data_type("not_a_valid_type").is_err());
    }

    ///
    /// Verifies parse_value_as_sequence can parse Values as Sequence for valid inputs
    ///  and returns an error for invalid inputs
    ///
    #[test]
    fn test_parse_value_as_sequence() {
        let key = "sequence".to_string();
        let mut property_valid = Mapping::new();

        // Check method can properly parse a sequence value
        let sequence = Vec::<Value>::new();
        property_valid.insert(
            Value::String(key.clone()),
            Value::Sequence(sequence.clone()),
        );
        assert_eq!(
            parse_value_as_sequence(&property_valid, &key).unwrap(),
            Some(sequence)
        );

        // Check method returns an error when key is found but value cannot be parsed into a
        // sequence
        let mut property_invalid = Mapping::new();
        property_invalid.insert(
            Value::String(key.clone()),
            Value::String("not a sequence".to_string()),
        );
        assert!(parse_value_as_sequence(&property_invalid, &key).is_err());

        // Check method returns Ok(None) when key is not found
        assert!(parse_value_as_sequence(&Mapping::new(), &key)
            .unwrap()
            .is_none());
    }

    ///
    /// Verifies parse_value_as_string can parse Values as String for valid inputs
    ///  and returns an error for invalid inputs
    ///
    #[test]
    fn test_parse_value_as_string() {
        let key = "string".to_string();
        let string_value = "my string".to_string();

        // Check method can properly parse a string value
        let mut property_valid = Mapping::new();
        property_valid.insert(
            Value::String(key.clone()),
            Value::String(string_value.clone()),
        );
        assert_eq!(
            parse_value_as_string(&property_valid, &key).unwrap(),
            Some(string_value.clone())
        );

        // Check method returns an error when key is found but value cannot be parsed into a
        // string
        let mut property_invalid = Mapping::new();
        property_invalid.insert(Value::String(key.clone()), Value::Number(0.into()));
        assert!(parse_value_as_string(&property_invalid, &key).is_err());

        // Check method returns Ok(None) when key is not found
        assert!(parse_value_as_string(&Mapping::new(), &key)
            .unwrap()
            .is_none());
    }

    ///
    /// Verifies parse_value_as_boolean can parse Values as booleans for valid inputs
    ///  and returns an error for invalid inputs
    ///
    #[test]
    fn test_parse_value_as_boolean() {
        let key = "bool".to_string();
        let bool_value = true;

        // Check method can properly parse a boolean value
        let mut property_valid = Mapping::new();
        property_valid.insert(Value::String(key.clone()), Value::Bool(bool_value.clone()));
        assert_eq!(
            parse_value_as_boolean(&property_valid, &key).unwrap(),
            Some(bool_value.clone())
        );

        // Check method returns an error when key is found but value cannot be parsed into a
        // boolean
        let mut property_invalid = Mapping::new();
        property_invalid.insert(Value::String(key.clone()), Value::Number(0.into()));
        assert!(parse_value_as_boolean(&property_invalid, &key).is_err());

        // Check method returns Ok(None) when key is not found
        assert!(parse_value_as_boolean(&Mapping::new(), &key)
            .unwrap()
            .is_none());
    }

    ///
    /// Verifies parse_parse_value_as_i32 can parse Values as i32 for valid inputs
    ///  and returns an error for invalid inputs
    ///
    #[test]
    fn test_parse_value_as_i32() {
        let key = "number_i32".to_string();
        let number_value = 200;

        // Check method can properly parse a number value
        let mut property_valid = Mapping::new();
        property_valid.insert(
            Value::String(key.clone()),
            Value::Number(number_value.clone().into()),
        );
        assert_eq!(
            parse_value_as_i32(&property_valid, &key).unwrap(),
            Some(number_value.clone())
        );

        // Check method returns an error when key is found but value overflows an i32 capacity.
        let mut property_invalid = Mapping::new();
        property_invalid.insert(
            Value::String(key.clone()),
            Value::Number(3000000000_i64.into()),
        );
        assert!(parse_value_as_i32(&property_invalid, &key).is_err());

        // Check method returns an error when key is found but value cannot be parsed into a
        // number
        let mut property_invalid = Mapping::new();
        property_invalid.insert(
            Value::String(key.clone()),
            Value::String("Not a number".to_string()),
        );
        assert!(parse_value_as_i32(&property_invalid, &key).is_err());

        // Check method returns Ok(None) when key is not found
        assert!(parse_value_as_i32(&Mapping::new(), &key).unwrap().is_none());
    }

    ///
    /// Verifies pparse_value_as_vec_string can parse Values as Vec<String> for valid inputs
    ///  and returns an error for invalid inputs
    ///
    #[test]
    fn test_parse_value_as_vec_string() {
        let key = "vec".to_string();
        let string_value = "value".to_string();
        let sequence_value = vec![Value::String(string_value.clone())];

        // Check method can properly parse a Value into Vec<String>
        let mut property_valid = Mapping::new();
        property_valid.insert(Value::String(key.clone()), Value::Sequence(sequence_value));
        assert_eq!(
            parse_value_as_vec_string(&property_valid, &key).unwrap(),
            Some(vec![string_value.clone()])
        );

        // Check method returns an error when key is found but values inside the sequence cannot
        // be parsed into a string.
        let mut property_invalid = Mapping::new();
        let sequence_value_invalid = vec![Value::Number(0.into())];
        property_invalid.insert(
            Value::String(key.clone()),
            Value::Sequence(sequence_value_invalid),
        );
        assert!(parse_value_as_vec_string(&property_invalid, &key).is_err());

        // Check method returns an error when key is found but value cannot be parsed into a vec
        let mut property_invalid = Mapping::new();
        property_invalid.insert(
            Value::String(key.clone()),
            Value::String("Not a sequence".to_string()),
        );
        assert!(parse_value_as_vec_string(&property_invalid, &key).is_err());

        // Check method returns Ok(None) when key is not found
        assert!(parse_value_as_i32(&Mapping::new(), &key).unwrap().is_none());
    }

    fn make_create_schema_payload_1() -> SchemaPayload {
        generate_create_schema_payload(
            "Lightbulb",
            &create_lightbulb_property_definitions(),
            Some("Example Lightbulb schema".to_string()),
        )
        .unwrap()
    }

    fn make_create_schema_payload_2() -> SchemaPayload {
        generate_create_schema_payload(
            "Phone",
            &create_phone_property_definitions(),
            Some("Example phone schema".to_string()),
        )
        .unwrap()
    }

    fn make_update_schema_payload_1() -> SchemaPayload {
        generate_update_schema_payload("Lightbulb", &create_lightbulb_property_definitions())
            .unwrap()
    }

    fn make_update_schema_payload_2() -> SchemaPayload {
        generate_update_schema_payload("Phone", &create_phone_property_definitions()).unwrap()
    }

    fn create_lightbulb_property_definitions() -> Vec<PropertyDefinition> {
        vec![
            make_number_property_definition("size", 0, true, "Lightbulb radius, in millimeters"),
            make_enum_property_definition(
                "bulb_type",
                &["filament".to_string(), "CF".to_string(), "LED".to_string()],
                true,
                "Bulb type",
            ),
            make_number_property_definition("energy_rating", 0, false, "EnergyStar energy rating"),
            make_struct_property_definition(
                "color",
                &[
                    make_string_property_definition("name", false, "Color name"),
                    make_string_property_definition("rgb_hex", false, "RGB value"),
                ],
                false,
                "A named RGB Color value",
            ),
        ]
    }

    fn create_phone_property_definitions() -> Vec<PropertyDefinition> {
        vec![
            make_string_property_definition("brand", true, "Name of the brand"),
            make_enum_property_definition(
                "operation_system",
                &["Android".to_string(), "iOS".to_string()],
                true,
                "Phone's operation system",
            ),
            make_number_property_definition(
                "price",
                0,
                false,
                "Price of phone rounded to the nearest dollar",
            ),
            make_lat_long_property_definition(
                "manufacturer_location",
                false,
                "Location where manufacturer is headquarted.",
            ),
        ]
    }

    fn make_enum_property_definition(
        name: &str,
        options: &[String],
        required: bool,
        description: &str,
    ) -> PropertyDefinition {
        PropertyDefinitionBuilder::new()
            .with_name(name.to_string())
            .with_data_type(DataType::Enum)
            .with_enum_options(options.to_vec())
            .with_required(required)
            .with_description(description.to_string())
            .build()
            .unwrap()
    }

    fn make_number_property_definition(
        name: &str,
        number_exponent: i32,
        required: bool,
        description: &str,
    ) -> PropertyDefinition {
        PropertyDefinitionBuilder::new()
            .with_name(name.to_string())
            .with_data_type(DataType::Number)
            .with_number_exponent(number_exponent)
            .with_required(required)
            .with_description(description.to_string())
            .build()
            .unwrap()
    }

    fn make_struct_property_definition(
        name: &str,
        properties: &[PropertyDefinition],
        required: bool,
        description: &str,
    ) -> PropertyDefinition {
        PropertyDefinitionBuilder::new()
            .with_name(name.to_string())
            .with_data_type(DataType::Struct)
            .with_struct_properties(properties.to_vec())
            .with_required(required)
            .with_description(description.to_string())
            .build()
            .unwrap()
    }

    fn make_string_property_definition(
        name: &str,
        required: bool,
        description: &str,
    ) -> PropertyDefinition {
        PropertyDefinitionBuilder::new()
            .with_name(name.to_string())
            .with_data_type(DataType::String)
            .with_required(required)
            .with_description(description.to_string())
            .build()
            .unwrap()
    }

    fn make_lat_long_property_definition(
        name: &str,
        required: bool,
        description: &str,
    ) -> PropertyDefinition {
        PropertyDefinitionBuilder::new()
            .with_name(name.to_string())
            .with_data_type(DataType::LatLong)
            .with_required(required)
            .with_description(description.to_string())
            .build()
            .unwrap()
    }

    fn run_test<T>(test: T) -> ()
    where
        T: FnOnce(&str) -> () + panic::UnwindSafe,
    {
        let test_yaml_file = temp_yaml_file_path();

        let test_path = test_yaml_file.clone();
        let result = panic::catch_unwind(move || test(&test_path));

        remove_file(test_yaml_file).unwrap();

        assert!(result.is_ok())
    }

    fn temp_yaml_file_path() -> String {
        let mut temp_dir = env::temp_dir();

        let thread_id = thread::current().id();
        temp_dir.push(format!("test_parse_schema-{:?}.yaml", thread_id));
        temp_dir.to_str().unwrap().to_string()
    }
}
