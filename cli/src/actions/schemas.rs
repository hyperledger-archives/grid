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
use crate::yaml_parser::{
    parse_value_as_boolean, parse_value_as_data_type, parse_value_as_i32, parse_value_as_sequence,
    parse_value_as_string, parse_value_as_vec_string,
};
use grid_sdk::protocol::schema::payload::{
    Action, SchemaCreateAction, SchemaCreateBuilder, SchemaPayload, SchemaPayloadBuilder,
    SchemaUpdateAction, SchemaUpdateBuilder,
};
use grid_sdk::protocol::schema::state::{DataType, PropertyDefinition, PropertyDefinitionBuilder};
use grid_sdk::protos::IntoProto;
use reqwest::Client;

use crate::error::CliError;
use serde::Deserialize;
use serde_yaml::{Mapping, Value};

#[derive(Debug, Deserialize)]
pub struct GridSchemaSlice {
    pub name: String,
    pub description: String,
    pub owner: String,
    pub properties: Vec<GridPropertyDefinitionSlice>,
}

#[derive(Debug, Deserialize)]
pub struct GridPropertyDefinitionSlice {
    pub name: String,
    pub schema_name: String,
    pub data_type: String,
    pub required: bool,
    pub description: String,
    pub number_exponent: i64,
    pub enum_options: Vec<String>,
    pub struct_properties: Vec<String>,
}

pub fn display_schema(schema: &GridSchemaSlice) {
    println!(
        "Name: {:?}\n Description: {:?}\n Owner: {:?}\n Properties:",
        schema.name, schema.description, schema.owner,
    );
    display_schema_property_definitions(&schema.properties);
}

pub fn display_schema_property_definitions(properties: &[GridPropertyDefinitionSlice]) {
    properties.iter().for_each(|def| {
        println!(
            "\tName: {:?}\n\t Data Type: {:?}\n\t Required: {:?}\n\t Description: {:?}
        Number Exponent: {:?}\n\t Enum Options: {:?}\n\t Struct Properties: {:?}",
            def.name,
            def.data_type,
            def.required,
            def.description,
            def.number_exponent,
            def.enum_options,
            def.struct_properties,
        );
    });
}

pub fn do_list_schemas(url: &str, service_id: Option<String>) -> Result<(), CliError> {
    let client = Client::new();
    let mut final_url = format!("{}/schema", url);
    if let Some(service_id) = service_id {
        final_url = format!("{}?service_id={}", final_url, service_id);
    }
    let schemas = client
        .get(&final_url)
        .send()?
        .json::<Vec<GridSchemaSlice>>()?;
    schemas.iter().for_each(|schema| display_schema(schema));
    Ok(())
}

pub fn do_show_schema(url: &str, name: &str, service_id: Option<String>) -> Result<(), CliError> {
    let client = Client::new();
    let mut final_url = format!("{}/schema/{}", url, name);
    if let Some(service_id) = service_id {
        final_url = format!("{}?service_id={}", final_url, service_id);
    }
    let schema = client.get(&final_url).send()?.json::<GridSchemaSlice>()?;
    display_schema(&schema);
    Ok(())
}

pub fn do_create_schemas(
    url: &str,
    key: Option<String>,
    wait: u64,
    path: &str,
    service_id: Option<String>,
) -> Result<(), CliError> {
    let payloads = parse_yaml(path, Action::SchemaCreate(SchemaCreateAction::default()))?;
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

    submit_batches(url, wait, &batch_list, service_id.as_deref())
}

pub fn do_update_schemas(
    url: &str,
    key: Option<String>,
    wait: u64,
    path: &str,
    service_id: Option<String>,
) -> Result<(), CliError> {
    let payloads = parse_yaml(path, Action::SchemaUpdate(SchemaUpdateAction::default()))?;
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

    submit_batches(url, wait, &batch_list, service_id.as_deref())
}

fn parse_yaml(path: &str, action: Action) -> Result<Vec<SchemaPayload>, CliError> {
    let file = std::fs::File::open(path)?;
    let schemas_yaml: Vec<Mapping> = serde_yaml::from_reader(file)?;

    match action {
        Action::SchemaCreate(_) => schemas_yaml
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

        Action::SchemaUpdate(_) => schemas_yaml
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

    schema_paylod = schema_paylod.with_action(Action::SchemaCreate(schema_create_action));
    schema_paylod
        .build()
        .map_err(|err| CliError::PayloadError(format!("Failed to build schema payload: {}", err)))
}

fn generate_update_schema_payload(
    name: &str,
    properties: &[PropertyDefinition],
) -> Result<SchemaPayload, CliError> {
    let mut schema_paylod = SchemaPayloadBuilder::new();

    let schema_update_action_builder = SchemaUpdateBuilder::new()
        .with_schema_name(name.to_string())
        .with_properties(properties.to_vec());

    let schema_update_action = schema_update_action_builder.build().map_err(|err| {
        CliError::PayloadError(format!("Failed to build schema payload: {}", err))
    })?;

    schema_paylod = schema_paylod.with_action(Action::SchemaUpdate(schema_update_action));
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
    let data_type = parse_value_as_data_type(
        &parse_value_as_string(property, "data_type")?.ok_or_else(|| {
            CliError::InvalidYamlError(
                "Missing `data_type` field for property definition.".to_string(),
            )
        })?,
    )?;

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

#[cfg(test)]
mod test {
    use super::*;
    use grid_sdk::protocol::schema::payload::{Action, SchemaPayload};
    use grid_sdk::protocol::schema::state::{
        DataType, PropertyDefinition, PropertyDefinitionBuilder,
    };
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

    static PHONE_YAML_EXAMPLE: &[u8; 625] = br##"
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

            let payload = parse_yaml(
                test_yaml_file_path,
                Action::SchemaCreate(SchemaCreateAction::default()),
            )
            .expect("Error parsing yaml");

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

            let payload = parse_yaml(
                test_yaml_file_path,
                Action::SchemaCreate(SchemaCreateAction::default()),
            )
            .expect("Error parsing yaml");

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

            let payload = parse_yaml(
                test_yaml_file_path,
                Action::SchemaUpdate(SchemaUpdateAction::default()),
            )
            .expect("Error parsing yaml");

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

            let payload = parse_yaml(
                test_yaml_file_path,
                Action::SchemaUpdate(SchemaUpdateAction::default()),
            )
            .expect("Error parsing yaml");

            assert_eq!(make_update_schema_payload_1(), payload[0]);
            assert_eq!(make_update_schema_payload_2(), payload[1]);
        })
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
