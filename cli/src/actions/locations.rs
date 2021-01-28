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
    agents::addressing::PIKE_NAMESPACE,
    locations::addressing::GRID_LOCATION_NAMESPACE,
    protocol::{
        location::payload::{
            Action, LocationCreateAction, LocationCreateActionBuilder, LocationDeleteAction,
            LocationNamespace, LocationPayloadBuilder, LocationUpdateAction,
            LocationUpdateActionBuilder,
        },
        schema::state::{LatLongBuilder, PropertyValue, PropertyValueBuilder},
    },
    protos::IntoProto,
};
use reqwest::Client;
use serde::Deserialize;

use crate::error::CliError;
use crate::http::submit_batches;
use crate::{
    actions::schemas::{self, get_schema, GridPropertyDefinitionSlice},
    transaction::location_batch_builder,
};

pub fn do_create_location(
    url: &str,
    key: Option<String>,
    wait: u64,
    actions: Vec<LocationCreateAction>,
    service_id: Option<&str>,
) -> Result<(), CliError> {
    submit_payloads(
        url,
        key,
        wait,
        actions.into_iter().map(Action::LocationCreate).collect(),
        service_id,
    )
}

pub fn do_update_location(
    url: &str,
    key: Option<String>,
    wait: u64,
    actions: Vec<LocationUpdateAction>,
    service_id: Option<&str>,
) -> Result<(), CliError> {
    submit_payloads(
        url,
        key,
        wait,
        actions.into_iter().map(Action::LocationUpdate).collect(),
        service_id,
    )
}

pub fn do_delete_location(
    url: &str,
    key: Option<String>,
    wait: u64,
    action: LocationDeleteAction,
    service_id: Option<&str>,
) -> Result<(), CliError> {
    submit_payloads(
        url,
        key,
        wait,
        vec![Action::LocationDelete(action)],
        service_id,
    )
}

pub fn do_list_locations(url: &str, service_id: Option<&str>) -> Result<(), CliError> {
    let client = Client::new();
    let mut final_url = format!("{}/location", url);
    if let Some(service_id) = service_id {
        final_url = format!("{}?service_id={}", final_url, service_id);
    }
    let mut response = client.get(&final_url).send()?;

    if !response.status().is_success() {
        return Err(CliError::DaemonError(response.text()?));
    }

    let locations = response.json::<Vec<LocationSlice>>()?;
    display_locations_info(&locations);
    Ok(())
}

pub fn do_show_location(
    url: &str,
    location_id: &str,
    service_id: Option<&str>,
) -> Result<(), CliError> {
    let client = Client::new();
    let mut final_url = format!("{}/location/{}", url, location_id);
    if let Some(service_id) = service_id {
        final_url = format!("{}?service_id={}", final_url, service_id);
    }

    let mut response = client.get(&final_url).send()?;

    if !response.status().is_success() {
        return Err(CliError::DaemonError(response.text()?));
    }

    let location = response.json::<LocationSlice>()?;

    display_location(&location);

    Ok(())
}

fn submit_payloads(
    url: &str,
    key: Option<String>,
    wait: u64,
    actions: Vec<Action>,
    service_id: Option<&str>,
) -> Result<(), CliError> {
    let mut builder = location_batch_builder(key);

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
                PIKE_NAMESPACE.to_string(),
                GRID_LOCATION_NAMESPACE.to_string(),
            ],
            &[GRID_LOCATION_NAMESPACE.to_string()],
        )?;
    }

    let batches = builder.create_batch_list();

    submit_batches(url, wait, &batches, service_id)
}

pub fn create_location_payloads_from_file(
    path: &str,
    url: &str,
    service_id: Option<&str>,
) -> Result<Vec<LocationCreateAction>, CliError> {
    let file = std::fs::File::open(path)?;
    let ymls: Vec<LocationCreateYaml> = serde_yaml::from_reader(&file)?;

    let mut payloads = Vec::new();

    for yml in ymls {
        let namespace = match yml.namespace {
            Namespace::GS1 => "gs1_location",
        };
        let schema = get_schema(url, namespace, service_id)?;
        payloads.push(yml.into_payload(schema.properties)?);
    }

    Ok(payloads)
}

pub fn update_location_payloads_from_file(
    path: &str,
    url: &str,
    service_id: Option<&str>,
) -> Result<Vec<LocationUpdateAction>, CliError> {
    let file = std::fs::File::open(path)?;
    let ymls: Vec<LocationUpdateYaml> = serde_yaml::from_reader(&file)?;

    let mut payloads = Vec::new();

    for yml in ymls {
        let namespace = match yml.namespace {
            Namespace::GS1 => "gs1_location",
        };
        let schema = get_schema(url, namespace, service_id)?;
        payloads.push(yml.into_payload(schema.properties)?);
    }

    Ok(payloads)
}

fn yaml_to_property_values(
    properties: &HashMap<String, serde_yaml::Value>,
    definitions: Vec<GridPropertyDefinitionSlice>,
) -> Result<Vec<PropertyValue>, CliError> {
    let mut property_values = Vec::new();

    for def in definitions {
        let value = if let Some(value) = properties.get(&def.name) {
            value
        } else if !def.required {
            continue;
        } else {
            return Err(CliError::PayloadError(format!(
                "Field {} not found",
                def.name
            )));
        };

        match def.data_type {
            schemas::DataType::Bytes => {
                let mut f = File::open(&serde_yaml::from_value::<String>(value.clone())?)?;
                let mut buffer = Vec::new();
                f.read_to_end(&mut buffer)?;

                let property_value = PropertyValueBuilder::new()
                    .with_name(def.name)
                    .with_data_type(def.data_type.into())
                    .with_bytes_value(buffer)
                    .build()
                    .map_err(|err| CliError::PayloadError(format!("{}", err)))?;

                property_values.push(property_value);
            }
            schemas::DataType::Boolean => {
                let property_value = PropertyValueBuilder::new()
                    .with_name(def.name.clone())
                    .with_data_type(def.data_type.into())
                    .with_boolean_value(serde_yaml::from_value(value.clone())?)
                    .build()
                    .map_err(|err| CliError::PayloadError(format!("{}", err)))?;
                property_values.push(property_value);
            }
            schemas::DataType::Number => {
                let property_value = PropertyValueBuilder::new()
                    .with_name(def.name.clone())
                    .with_data_type(def.data_type.into())
                    .with_number_value(serde_yaml::from_value(value.clone())?)
                    .build()
                    .map_err(|err| CliError::PayloadError(format!("{}", err)))?;
                property_values.push(property_value);
            }
            schemas::DataType::String => {
                let property_value = PropertyValueBuilder::new()
                    .with_name(def.name.clone())
                    .with_data_type(def.data_type.into())
                    .with_string_value(serde_yaml::from_value(value.clone())?)
                    .build()
                    .map_err(|err| CliError::PayloadError(format!("{}", err)))?;
                property_values.push(property_value);
            }
            schemas::DataType::Enum => {
                let property_value = PropertyValueBuilder::new()
                    .with_name(def.name.clone())
                    .with_data_type(def.data_type.into())
                    .with_enum_value(serde_yaml::from_value(value.clone())?)
                    .build()
                    .map_err(|err| CliError::PayloadError(format!("{}", err)))?;
                property_values.push(property_value);
            }
            schemas::DataType::Struct => {
                let properties: HashMap<String, serde_yaml::Value> =
                    serde_yaml::from_value(value.clone())?;
                let property_value = PropertyValueBuilder::new()
                    .with_name(def.name.clone())
                    .with_data_type(def.data_type.into())
                    .with_struct_values(yaml_to_property_values(
                        &properties,
                        def.struct_properties,
                    )?)
                    .build()
                    .map_err(|err| CliError::PayloadError(format!("{}", err)))?;
                property_values.push(property_value);
            }
            schemas::DataType::LatLong => {
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

                property_values.push(property_value);
            }
        }
    }

    Ok(property_values)
}

fn display_locations_info(locations: &[LocationSlice]) {
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

fn display_location(location: &LocationSlice) {
    println!(
        "Location ID: {}\nNamespace: {}\nOwner: {}\nProperties",
        location.location_id, location.location_namespace, location.owner,
    );

    location.properties.iter().for_each(|p| match p.data_type {
        schemas::DataType::Bytes => {
            println!("{}: {:?}", p.name, p.bytes_value.as_ref().unwrap());
        }
        schemas::DataType::Boolean => {
            println!("{}: {:?}", p.name, p.boolean_value.as_ref().unwrap());
        }
        schemas::DataType::Number => {
            println!("{}: {:?}", p.name, p.number_value.as_ref().unwrap());
        }
        schemas::DataType::String => {
            println!("{}: {:?}", p.name, p.string_value.as_ref().unwrap());
        }
        schemas::DataType::Enum => {
            println!("{}: {:?}", p.name, p.enum_value.as_ref().unwrap());
        }
        schemas::DataType::Struct => {
            println!("{}: {:?}", p.name, p.struct_values.as_ref().unwrap());
        }
        schemas::DataType::LatLong => {
            println!(
                "{}: {}, {}",
                p.name,
                p.lat_long_value.as_ref().unwrap().latitude,
                p.lat_long_value.as_ref().unwrap().longitude
            );
        }
    });
}

#[derive(Debug, Deserialize)]
pub struct LocationSlice {
    pub location_id: String,
    pub location_namespace: String,
    pub owner: String,
    pub properties: Vec<LocationPropertyValueSlice>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LocationPropertyValueSlice {
    pub name: String,
    pub data_type: schemas::DataType,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_id: Option<String>,
    pub bytes_value: Option<Vec<u8>>,
    pub boolean_value: Option<bool>,
    pub number_value: Option<i64>,
    pub string_value: Option<String>,
    pub enum_value: Option<i32>,
    pub struct_values: Option<Vec<String>>,
    pub lat_long_value: Option<LatLongSlice>,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct LatLongSlice {
    pub latitude: i64,
    pub longitude: i64,
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
        definitions: Vec<GridPropertyDefinitionSlice>,
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
        definitions: Vec<GridPropertyDefinitionSlice>,
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
    GS1,
}

impl Into<LocationNamespace> for Namespace {
    fn into(self) -> LocationNamespace {
        match self {
            Namespace::GS1 => LocationNamespace::GS1,
        }
    }
}

impl Into<String> for Namespace {
    fn into(self) -> String {
        match self {
            Namespace::GS1 => "GS1".to_string(),
        }
    }
}
