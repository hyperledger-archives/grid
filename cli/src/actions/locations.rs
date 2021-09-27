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

use std::{
    collections::HashMap,
    fs::File,
    io::prelude::*,
    time::{SystemTime, UNIX_EPOCH},
};

use grid_sdk::{
    client::location::{Location, LocationClient},
    client::schema::{DataType, PropertyDefinition, SchemaClient},
    location::addressing::GRID_LOCATION_NAMESPACE,
    pike::addressing::GRID_PIKE_NAMESPACE,
    protocol::{
        location::payload::{
            Action, LocationCreateAction, LocationCreateActionBuilder, LocationDeleteAction,
            LocationNamespace, LocationPayloadBuilder, LocationUpdateAction,
            LocationUpdateActionBuilder,
        },
        schema::state::{LatLongBuilder, PropertyValue, PropertyValueBuilder},
    },
    protos::IntoProto,
    schema::addressing::GRID_SCHEMA_NAMESPACE,
};

use cylinder::Signer;
use serde::Deserialize;

use crate::error::CliError;
use crate::transaction::location_batch_builder;

pub fn do_create_location(
    client: Box<dyn LocationClient>,
    signer: Box<dyn Signer>,
    wait: u64,
    actions: Vec<LocationCreateAction>,
    service_id: Option<&str>,
) -> Result<(), CliError> {
    submit_payloads(
        client,
        signer,
        wait,
        actions.into_iter().map(Action::LocationCreate).collect(),
        service_id,
    )
}

pub fn do_update_location(
    client: Box<dyn LocationClient>,
    signer: Box<dyn Signer>,
    wait: u64,
    actions: Vec<LocationUpdateAction>,
    service_id: Option<&str>,
) -> Result<(), CliError> {
    submit_payloads(
        client,
        signer,
        wait,
        actions.into_iter().map(Action::LocationUpdate).collect(),
        service_id,
    )
}

pub fn do_delete_location(
    client: Box<dyn LocationClient>,
    signer: Box<dyn Signer>,
    wait: u64,
    action: LocationDeleteAction,
    service_id: Option<&str>,
) -> Result<(), CliError> {
    submit_payloads(
        client,
        signer,
        wait,
        vec![Action::LocationDelete(action)],
        service_id,
    )
}

pub fn do_list_locations(
    client: Box<dyn LocationClient>,
    service_id: Option<&str>,
) -> Result<(), CliError> {
    let locations = client.list_locations(service_id)?;
    display_locations_info(&locations);
    Ok(())
}

pub fn do_show_location(
    client: Box<dyn LocationClient>,
    location_id: &str,
    service_id: Option<&str>,
) -> Result<(), CliError> {
    let location = client.get_location(location_id.into(), service_id)?;
    display_location(&location);
    Ok(())
}

fn submit_payloads(
    client: Box<dyn LocationClient>,
    signer: Box<dyn Signer>,
    wait: u64,
    actions: Vec<Action>,
    service_id: Option<&str>,
) -> Result<(), CliError> {
    let mut builder = location_batch_builder(signer);

    for action in actions {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .map_err(|err| CliError::PayloadError(format!("{}", err)))?;

        let action = LocationPayloadBuilder::new()
            .with_action(action)
            .with_timestamp(timestamp)
            .build()
            .map_err(|err| CliError::PayloadError(format!("{}", err)))?;

        builder.add_transaction(
            &action.into_proto()?,
            &[
                GRID_SCHEMA_NAMESPACE.to_string(),
                GRID_PIKE_NAMESPACE.to_string(),
                GRID_LOCATION_NAMESPACE.to_string(),
            ],
            &[GRID_LOCATION_NAMESPACE.to_string()],
        )?;
    }

    let batches = builder.create_batch_list();

    client.post_batches(wait, &batches, service_id)?;
    Ok(())
}

pub fn create_location_payloads_from_file(
    path: &str,
    client: Box<dyn SchemaClient>,
    service_id: Option<&str>,
) -> Result<Vec<LocationCreateAction>, CliError> {
    let file = std::fs::File::open(path)?;
    let ymls: Vec<LocationCreateYaml> = serde_yaml::from_reader(&file)?;

    let mut payloads = Vec::new();

    for yml in ymls {
        let namespace = match yml.namespace {
            Namespace::Gs1 => "gs1_location".to_string(),
        };
        let schema = client.get_schema(namespace, service_id)?;
        payloads.push(yml.into_payload(schema.properties)?);
    }

    Ok(payloads)
}

pub fn update_location_payloads_from_file(
    path: &str,
    client: Box<dyn SchemaClient>,
    service_id: Option<&str>,
) -> Result<Vec<LocationUpdateAction>, CliError> {
    let file = std::fs::File::open(path)?;
    let ymls: Vec<LocationUpdateYaml> = serde_yaml::from_reader(&file)?;

    let mut payloads = Vec::new();

    for yml in ymls {
        let namespace = match yml.namespace {
            Namespace::Gs1 => "gs1_location".to_string(),
        };
        let schema = client.get_schema(namespace, service_id)?;
        payloads.push(yml.into_payload(schema.properties)?);
    }

    Ok(payloads)
}

fn yaml_to_property_values(
    properties: &HashMap<String, serde_yaml::Value>,
    definitions: Vec<PropertyDefinition>,
) -> Result<Vec<PropertyValue>, CliError> {
    let mut property_values = Vec::new();
    let mut property_error_messages = Vec::new();

    for def in definitions {
        match get_property_value_from_property_definition(properties, def) {
            Ok(Some(property_value)) => property_values.push(property_value),
            Ok(None) => (),
            Err(CliError::YamlProcessingError(mut messages)) => {
                property_error_messages.append(&mut messages);
            }
            Err(CliError::PayloadError(message)) => {
                property_error_messages.push(message);
            }
            Err(err) => {
                return Err(err);
            }
        }
    }

    if !property_error_messages.is_empty() {
        Err(CliError::YamlProcessingError(property_error_messages))
    } else {
        Ok(property_values)
    }
}

fn get_property_value_from_property_definition(
    properties: &HashMap<String, serde_yaml::Value>,
    def: PropertyDefinition,
) -> Result<Option<PropertyValue>, CliError> {
    let value = if let Some(value) = properties.get(&def.name) {
        value
    } else if !def.required {
        return Ok(None);
    } else {
        return Err(CliError::PayloadError(format!(
            "Field {} not found",
            def.name
        )));
    };

    match def.data_type {
        DataType::Bytes => {
            let mut f = File::open(&serde_yaml::from_value::<String>(value.clone())?)?;
            let mut buffer = Vec::new();
            f.read_to_end(&mut buffer)?;

            let property_value = PropertyValueBuilder::new()
                .with_name(def.name)
                .with_data_type(def.data_type.into())
                .with_bytes_value(buffer)
                .build()
                .map_err(|err| CliError::PayloadError(format!("{}", err)))?;

            Ok(Some(property_value))
        }
        DataType::Boolean => {
            let property_value = PropertyValueBuilder::new()
                .with_name(def.name.clone())
                .with_data_type(def.data_type.into())
                .with_boolean_value(serde_yaml::from_value(value.clone())?)
                .build()
                .map_err(|err| CliError::PayloadError(format!("{}", err)))?;
            Ok(Some(property_value))
        }
        DataType::Number => {
            let property_value = PropertyValueBuilder::new()
                .with_name(def.name.clone())
                .with_data_type(def.data_type.into())
                .with_number_value(serde_yaml::from_value(value.clone())?)
                .build()
                .map_err(|err| CliError::PayloadError(format!("{}", err)))?;
            Ok(Some(property_value))
        }
        DataType::String => {
            let property_value = PropertyValueBuilder::new()
                .with_name(def.name.clone())
                .with_data_type(def.data_type.into())
                .with_string_value(serde_yaml::from_value(value.clone())?)
                .build()
                .map_err(|err| CliError::PayloadError(format!("{}", err)))?;
            Ok(Some(property_value))
        }
        DataType::Enum => {
            let property_value = PropertyValueBuilder::new()
                .with_name(def.name.clone())
                .with_data_type(def.data_type.into())
                .with_enum_value(serde_yaml::from_value(value.clone())?)
                .build()
                .map_err(|err| CliError::PayloadError(format!("{}", err)))?;
            Ok(Some(property_value))
        }
        DataType::Struct => {
            let properties: HashMap<String, serde_yaml::Value> =
                serde_yaml::from_value(value.clone())?;
            let property_value = PropertyValueBuilder::new()
                .with_name(def.name.clone())
                .with_data_type(def.data_type.into())
                .with_struct_values(yaml_to_property_values(&properties, def.struct_properties)?)
                .build()
                .map_err(|err| CliError::PayloadError(format!("{}", err)))?;
            Ok(Some(property_value))
        }
        DataType::LatLong => {
            let lat_long = serde_yaml::from_value::<String>(value.clone())?
                .split(',')
                .map(|x| {
                    x.parse::<i64>()
                        .map_err(|err| CliError::PayloadError(format!("{}", err)))
                })
                .collect::<Result<Vec<i64>, CliError>>()?;

            if lat_long.len() != 2 {
                return Err(CliError::PayloadError(format!(
                    "{:?} is not a valid latitude longitude",
                    lat_long
                )));
            }

            let lat_long = LatLongBuilder::new()
                .with_lat_long(lat_long[0], lat_long[1])
                .build()
                .map_err(|err| CliError::PayloadError(format!("{}", err)))?;

            let property_value = PropertyValueBuilder::new()
                .with_name(def.name)
                .with_data_type(def.data_type.into())
                .with_lat_long_value(lat_long)
                .build()
                .map_err(|err| CliError::PayloadError(format!("{}", err)))?;

            Ok(Some(property_value))
        }
    }
}

fn display_locations_info(locations: &[Location]) {
    // GLNs are always 13 characters
    const ID_LENGTH: usize = 13;
    // The column header "Namespace" will be longer than the values, in practice
    const NAMESPACE_LENGTH: usize = "NAMESPACE".len();
    // The min width of the owner column. This is required by the Rust linter
    const OWNER_MIN: usize = "OWNER".len();
    println!(
        "{:<length_id$} {:<length_namespace$.length_namespace$} {:<length_owner$}",
        "ID",
        "NAMESPACE",
        "OWNER",
        length_id = ID_LENGTH,
        length_namespace = NAMESPACE_LENGTH,
        length_owner = OWNER_MIN
    );
    locations.iter().for_each(|location| {
        println!(
            "{:<length_id$} {:<length_namespace$.length_namespace$} {:<length_owner$}",
            location.location_id,
            location.location_namespace,
            location.owner,
            length_id = ID_LENGTH,
            length_namespace = NAMESPACE_LENGTH,
            length_owner = OWNER_MIN
        )
    });
}

fn display_location(location: &Location) {
    println!(
        "Location ID: {}\nNamespace: {}\nOwner: {}\nProperties",
        location.location_id, location.location_namespace, location.owner,
    );

    location.properties.iter().for_each(|p| match p.data_type {
        DataType::Bytes => {
            println!("{}: {:?}", p.name, p.bytes_value.as_ref().unwrap());
        }
        DataType::Boolean => {
            println!("{}: {:?}", p.name, p.boolean_value.as_ref().unwrap());
        }
        DataType::Number => {
            println!("{}: {:?}", p.name, p.number_value.as_ref().unwrap());
        }
        DataType::String => {
            println!("{}: {:?}", p.name, p.string_value.as_ref().unwrap());
        }
        DataType::Enum => {
            println!("{}: {:?}", p.name, p.enum_value.as_ref().unwrap());
        }
        DataType::Struct => {
            println!("{}: {:?}", p.name, p.struct_values.as_ref().unwrap());
        }
        DataType::LatLong => {
            println!(
                "{}: {}, {}",
                p.name,
                p.lat_long_value.as_ref().unwrap().latitude,
                p.lat_long_value.as_ref().unwrap().longitude
            );
        }
    });
}

#[derive(Deserialize, Debug)]
pub struct LocationCreateYaml {
    location_id: String,
    owner: String,
    namespace: Namespace,
    properties: HashMap<String, serde_yaml::Value>,
}

impl LocationCreateYaml {
    pub fn into_payload(
        self,
        definitions: Vec<PropertyDefinition>,
    ) -> Result<LocationCreateAction, CliError> {
        let property_values = yaml_to_property_values(&self.properties, definitions)?;
        LocationCreateActionBuilder::new()
            .with_location_id(self.location_id)
            .with_owner(self.owner)
            .with_namespace(self.namespace.into())
            .with_properties(property_values)
            .build()
            .map_err(|err| CliError::PayloadError(format!("{}", err)))
    }
}

#[derive(Deserialize, Debug)]
pub struct LocationUpdateYaml {
    location_id: String,
    namespace: Namespace,
    properties: HashMap<String, serde_yaml::Value>,
}

impl LocationUpdateYaml {
    pub fn into_payload(
        self,
        definitions: Vec<PropertyDefinition>,
    ) -> Result<LocationUpdateAction, CliError> {
        let property_values = yaml_to_property_values(&self.properties, definitions)?;
        LocationUpdateActionBuilder::new()
            .with_location_id(self.location_id)
            .with_namespace(self.namespace.into())
            .with_properties(property_values)
            .build()
            .map_err(|err| CliError::PayloadError(format!("{}", err)))
    }
}

#[derive(Deserialize, Debug)]
pub enum Namespace {
    #[serde(rename = "GS1")]
    Gs1,
}

impl From<Namespace> for LocationNamespace {
    fn from(namespace: Namespace) -> Self {
        match namespace {
            Namespace::Gs1 => LocationNamespace::Gs1,
        }
    }
}

impl From<Namespace> for String {
    fn from(namespace: Namespace) -> Self {
        match namespace {
            Namespace::Gs1 => "GS1".to_string(),
        }
    }
}
