// Copyright (c) 2019 Target Brands, Inc.
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

use crate::actions::schemas::{self, get_schema, GridPropertyDefinitionSlice};
use crate::http::submit_batches;
use crate::transaction::product_batch_builder;
use grid_sdk::agents::addressing::PIKE_NAMESPACE;
use grid_sdk::products::addressing::GRID_PRODUCT_NAMESPACE;
use grid_sdk::protocol::product::payload::{
    Action, ProductCreateAction, ProductCreateActionBuilder, ProductDeleteAction,
    ProductPayloadBuilder, ProductUpdateAction, ProductUpdateActionBuilder,
};
use grid_sdk::protocol::product::state::ProductNamespace;
use grid_sdk::protocol::schema::state::{LatLongBuilder, PropertyValue, PropertyValueBuilder};
use grid_sdk::protos::IntoProto;
use grid_sdk::schemas::addressing::GRID_SCHEMA_NAMESPACE;
use reqwest::Client;

use crate::error::CliError;
use serde::Deserialize;

use std::{
    collections::HashMap,
    fs::File,
    io::prelude::*,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Deserialize)]
pub struct GridProduct {
    pub product_id: String,
    pub product_namespace: String,
    pub owner: String,
    pub properties: Vec<GridPropertyValue>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GridPropertyValue {
    pub name: String,
    pub data_type: String,
    pub bytes_value: Option<Vec<u8>>,
    pub boolean_value: Option<bool>,
    pub number_value: Option<i64>,
    pub string_value: Option<String>,
    pub enum_value: Option<u32>,
    pub struct_values: Option<Vec<String>>,
    pub lat_long_value: Option<LatLong>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LatLong {
    latitude: i64,
    longitude: i64,
}

/**
 * Prints basic info for products
 *
 * products - Products to be printed
 */
pub fn display_products_info(products: &[GridProduct]) {
    // GTINs are always 14 characters long
    const ID_LENGTH: usize = 14;
    // Column header "namespace" is longer than the namespace strings in practice
    const NAMESPACE_LENGTH: usize = "NAMESPACE".len();
    // Minimum width of the "owner" column. This is required because of Rust linting
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
    products.iter().for_each(|product| {
        println!(
            "{:<length_id$} {:<length_namespace$.length_namespace$} {:<length_owner$}",
            product.product_id,
            product.product_namespace,
            product.owner,
            length_id = ID_LENGTH,
            length_namespace = NAMESPACE_LENGTH,
            length_owner = OWNER_MIN
        )
    });
}

/**
 * Print the fields for a given product
 *
 * product - Product to be printed
 */
pub fn display_product(product: &GridProduct) {
    println!(
        "Product Id: {:?}\n Product Namespace: {:?}\n Owner: {:?}\n Properties:",
        product.product_id, product.product_namespace, product.owner,
    );
    display_product_property_definitions(&product.properties);
}

/**
 * Iterate through all fields of a Property Value and print the given value
 *
 * properties - Property values to be printed
 */
pub fn display_product_property_definitions(properties: &[GridPropertyValue]) {
    properties.iter().for_each(|def| {
        println!(
            "\tProperty Name: {:?}\n\t Data Type: {:?}\n\t Bytes Value: {:?}\n\t Boolean Value: {:?}
        Number Value: {:?}\n\t String Value: {:?}\n\t Enum Value: {:?}\n\t Struct Values: {:?}\n\t Lat/Lon Values: {:?}\n\t",
            def.name,
            def.data_type,
            def.bytes_value,
            def.boolean_value,
            def.number_value,
            def.string_value,
            def.enum_value,
            def.struct_values,
            def.lat_long_value,
        );
    })
}

/**
 * Create a new product
 *
 * url - Url for the REST API
 * key - Signing key of the agent
 * wait - Time in seconds to wait for commit
 * path - Path to the yaml file that contains the product descriptions
 */
pub fn do_create_products(
    url: &str,
    key: Option<String>,
    wait: u64,
    actions: Vec<ProductCreateAction>,
    service_id: Option<String>,
) -> Result<(), CliError> {
    submit_payloads(
        url,
        key,
        wait,
        actions.into_iter().map(Action::ProductCreate).collect(),
        service_id.as_deref(),
    )
}

/**
 * Update an existing product
 *
 * url - Url for the REST API
 * key - Signing key of the agent
 * wait - Time in seconds to wait for commit
 * path - Path to the yaml file that contains the product descriptions
 */
pub fn do_update_products(
    url: &str,
    key: Option<String>,
    wait: u64,
    actions: Vec<ProductUpdateAction>,
    service_id: Option<String>,
) -> Result<(), CliError> {
    submit_payloads(
        url,
        key,
        wait,
        actions.into_iter().map(Action::ProductUpdate).collect(),
        service_id.as_deref(),
    )
}

/**
 * Delete an existing product
 *
 * url - Url for the REST API
 * key - Signing key of the agent
 * wait - Time in seconds to wait for commit
 * path - Path to the yaml file that contains the product descriptions
 */
pub fn do_delete_products(
    url: &str,
    key: Option<String>,
    wait: u64,
    action: ProductDeleteAction,
    service_id: Option<String>,
) -> Result<(), CliError> {
    submit_payloads(
        url,
        key,
        wait,
        vec![Action::ProductDelete(action)],
        service_id.as_deref(),
    )
}

/**
 * Print all products in state
 *
 * url - Url for the REST API
 */
pub fn do_list_products(url: &str, service_id: Option<String>) -> Result<(), CliError> {
    let client = Client::new();
    let mut final_url = format!("{}/product", url);
    if let Some(service_id) = service_id {
        final_url = format!("{}?service_id={}", final_url, service_id);
    }

    let mut response = client.get(&final_url).send()?;

    if !response.status().is_success() {
        return Err(CliError::DaemonError(response.text()?));
    }

    let products = response.json::<Vec<GridProduct>>()?;
    display_products_info(&products);
    Ok(())
}

/**
 * Print a single product in state
 *
 * url - Url for the REST API
 * product_id - e.g. GTIN
 */
pub fn do_show_products(
    url: &str,
    product_id: &str,
    service_id: Option<String>,
) -> Result<(), CliError> {
    let client = Client::new();
    let mut final_url = format!("{}/product/{}", url, product_id);
    if let Some(service_id) = service_id {
        final_url = format!("{}?service_id={}", final_url, service_id);
    }

    let mut response = client.get(&final_url).send()?;

    if !response.status().is_success() {
        return Err(CliError::DaemonError(response.text()?));
    }

    let product = response.json::<GridProduct>()?;
    display_product(&product);
    Ok(())
}

pub fn create_product_payloads_from_file(
    path: &str,
    url: &str,
    service_id: Option<&str>,
) -> Result<Vec<ProductCreateAction>, CliError> {
    let file = std::fs::File::open(path)?;
    let ymls: Vec<ProductCreateYaml> = serde_yaml::from_reader(&file)?;

    let mut payloads = Vec::new();

    for yml in ymls {
        let namespace = match yml.product_namespace {
            Namespace::GS1 => "gs1_product",
        };
        let schema = get_schema(url, namespace, service_id)?;
        payloads.push(yml.into_payload(schema.properties)?);
    }

    Ok(payloads)
}

pub fn update_product_payloads_from_file(
    path: &str,
    url: &str,
    service_id: Option<&str>,
) -> Result<Vec<ProductUpdateAction>, CliError> {
    let file = std::fs::File::open(path)?;
    let ymls: Vec<ProductUpdateYaml> = serde_yaml::from_reader(&file)?;

    let mut payloads = Vec::new();

    for yml in ymls {
        let namespace = match yml.product_namespace {
            Namespace::GS1 => "gs1_product",
        };
        let schema = get_schema(url, namespace, service_id)?;
        payloads.push(yml.into_payload(schema.properties)?);
    }

    Ok(payloads)
}

#[derive(Deserialize, Debug)]
pub struct ProductCreateYaml {
    product_id: String,
    owner: String,
    product_namespace: Namespace,
    properties: HashMap<String, serde_yaml::Value>,
}

impl ProductCreateYaml {
    pub fn into_payload(
        self,
        definitions: Vec<GridPropertyDefinitionSlice>,
    ) -> Result<ProductCreateAction, CliError> {
        let property_values = yaml_to_property_values(&self.properties, definitions)?;
        ProductCreateActionBuilder::new()
            .with_product_id(self.product_id)
            .with_owner(self.owner)
            .with_product_namespace(self.product_namespace.into())
            .with_properties(property_values)
            .build()
            .map_err(|err| CliError::PayloadError(format!("{}", err)))
    }
}

#[derive(Deserialize, Debug)]
pub struct ProductUpdateYaml {
    product_id: String,
    product_namespace: Namespace,
    properties: HashMap<String, serde_yaml::Value>,
}

impl ProductUpdateYaml {
    pub fn into_payload(
        self,
        definitions: Vec<GridPropertyDefinitionSlice>,
    ) -> Result<ProductUpdateAction, CliError> {
        let property_values = yaml_to_property_values(&self.properties, definitions)?;
        ProductUpdateActionBuilder::new()
            .with_product_id(self.product_id)
            .with_product_namespace(self.product_namespace.into())
            .with_properties(property_values)
            .build()
            .map_err(|err| CliError::PayloadError(format!("{}", err)))
    }
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

fn submit_payloads(
    url: &str,
    key: Option<String>,
    wait: u64,
    actions: Vec<Action>,
    service_id: Option<&str>,
) -> Result<(), CliError> {
    let mut builder = product_batch_builder(key);

    for action in actions {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .map_err(|err| CliError::PayloadError(format!("{}", err)))?;

        let action = ProductPayloadBuilder::new()
            .with_action(action)
            .with_timestamp(timestamp)
            .build()
            .map_err(|err| CliError::PayloadError(format!("{}", err)))?;

        builder.add_transaction(
            &action.into_proto()?,
            &[
                PIKE_NAMESPACE.to_string(),
                GRID_SCHEMA_NAMESPACE.to_string(),
                GRID_PRODUCT_NAMESPACE.to_string(),
            ],
            &[GRID_PRODUCT_NAMESPACE.to_string()],
        )?;
    }

    let batches = builder.create_batch_list();

    submit_batches(url, wait, &batches, service_id)
}

#[derive(Deserialize, Debug)]
pub enum Namespace {
    GS1,
}

impl Into<ProductNamespace> for Namespace {
    fn into(self) -> ProductNamespace {
        match self {
            Namespace::GS1 => ProductNamespace::GS1,
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
