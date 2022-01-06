// Copyright (c) 2019 Target Brands, Inc.
// Copyright 2021 Cargill Incorporated
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

use std::env;

use crate::transaction::product_batch_builder;
use cylinder::Signer;
use grid_sdk::client::product::{
    Product as GridProduct, ProductClient, PropertyValue as GridPropertyValue,
};
use grid_sdk::client::schema::{DataType, PropertyDefinition, SchemaClient};
use grid_sdk::data_validation::validate_gdsn_3_1;
use grid_sdk::pike::addressing::GRID_PIKE_NAMESPACE;
use grid_sdk::product::addressing::GRID_PRODUCT_NAMESPACE;
use grid_sdk::product::gdsn::{get_trade_items_from_xml, GDSN_3_1_PROPERTY_NAME};
use grid_sdk::protocol::product::payload::{
    Action, ProductCreateAction, ProductCreateActionBuilder, ProductDeleteAction,
    ProductPayloadBuilder, ProductUpdateAction, ProductUpdateActionBuilder,
};
use grid_sdk::protocol::product::state::ProductNamespace;
use grid_sdk::protocol::schema::state::{LatLongBuilder, PropertyValue, PropertyValueBuilder};
use grid_sdk::protos::IntoProto;
use grid_sdk::schema::addressing::GRID_SCHEMA_NAMESPACE;

use crate::error::CliError;
use serde::Deserialize;

use std::borrow::Borrow;
use std::{
    collections::HashMap,
    fs::File,
    io::prelude::*,
    time::{SystemTime, UNIX_EPOCH},
};

use super::DEFAULT_SCHEMA_DIR;

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
 * signer - Signer for the agent
 * wait - Time in seconds to wait for commit
 * path - Path to the yaml file that contains the product descriptions
 */
pub fn do_create_products(
    client: Box<dyn ProductClient>,
    signer: Box<dyn Signer>,
    wait: u64,
    actions: Vec<ProductCreateAction>,
    service_id: Option<&str>,
) -> Result<(), CliError> {
    submit_payloads(
        client,
        signer,
        wait,
        actions.into_iter().map(Action::ProductCreate).collect(),
        service_id,
    )
}

/**
 * Update an existing product
 *
 * url - Url for the REST API
 * signer - Signer for the agent
 * wait - Time in seconds to wait for commit
 * path - Path to the yaml file that contains the product descriptions
 */
pub fn do_update_products(
    client: Box<dyn ProductClient>,
    signer: Box<dyn Signer>,
    wait: u64,
    actions: Vec<ProductUpdateAction>,
    service_id: Option<&str>,
) -> Result<(), CliError> {
    submit_payloads(
        client,
        signer,
        wait,
        actions.into_iter().map(Action::ProductUpdate).collect(),
        service_id,
    )
}

/**
 * Delete an existing product
 *
 * url - Url for the REST API
 * signer - Signer for the agent
 * wait - Time in seconds to wait for commit
 * path - Path to the yaml file that contains the product descriptions
 */
pub fn do_delete_products(
    client: Box<dyn ProductClient>,
    signer: Box<dyn Signer>,
    wait: u64,
    action: ProductDeleteAction,
    service_id: Option<&str>,
) -> Result<(), CliError> {
    submit_payloads(
        client,
        signer,
        wait,
        vec![Action::ProductDelete(action)],
        service_id,
    )
}

/**
 * Print all products in state
 *
 * url - Url for the REST API
 */
pub fn do_list_products(
    client: Box<dyn ProductClient>,
    service_id: Option<&str>,
) -> Result<(), CliError> {
    let products = client.list_products(service_id)?;
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
    client: Box<dyn ProductClient>,
    product_id: String,
    service_id: Option<&str>,
) -> Result<(), CliError> {
    let product = client.get_product(product_id, service_id)?;
    display_product(&product);
    Ok(())
}

enum ProductFileType {
    Gdsn3_1,
    SchemaBasedDefinition,
}

fn determine_file_type(path: &str) -> Result<ProductFileType, CliError> {
    let extension = std::path::Path::new(&path).extension();

    match extension {
        None => Err(CliError::UserError(format!(
            "Unable to determine file extension: {}",
            path
        ))),
        Some(os_str) => match os_str.to_str() {
            None => {
                return Err(CliError::UserError(format!(
                    "Unable to determine file extension: {}",
                    path
                )))
            }
            Some("yaml") | Some("yml") => Ok(ProductFileType::SchemaBasedDefinition),
            Some("xml") => Ok(ProductFileType::Gdsn3_1),
            Some(_) => Err(CliError::UserError(format!(
                "File has an unsupported format: {}, Accepted formats: GDSN 3.1 XML, Grid \
                    Product YAML definition",
                path
            ))),
        },
    }
}

pub fn create_product_payloads_from_file(
    paths: Vec<&str>,
    client: Box<dyn SchemaClient>,
    service_id: Option<&str>,
    owner: Option<&str>,
) -> Result<Vec<ProductCreateAction>, CliError> {
    let mut total_payloads: Vec<ProductCreateAction> = Vec::new();

    for path in paths {
        let file_type = determine_file_type(path)?;

        let file_payloads = match file_type {
            ProductFileType::Gdsn3_1 => {
                let owner = owner.ok_or_else(|| {
                    CliError::ActionError(
                        "'--owner' argument is required for product creation with GDSN XML files"
                            .to_string(),
                    )
                })?;
                create_product_payloads_from_xml(path, owner)?
            }
            ProductFileType::SchemaBasedDefinition => {
                create_product_payloads_from_yaml(path, client.borrow(), service_id)?
            }
        };

        for file_payload in file_payloads {
            let mut product_id_exists = false;

            for payload in total_payloads.iter_mut() {
                if payload.product_id() == file_payload.product_id() {
                    check_duplicate_properties(
                        payload.product_id().to_string(),
                        file_payload.properties().to_vec(),
                        payload.properties().to_vec(),
                    )?;

                    product_id_exists = true;
                    let mut combined_properties: Vec<PropertyValue> = payload.properties().to_vec();
                    combined_properties.append(&mut file_payload.properties().to_vec());
                    let payload_with_combined_properties = ProductCreateActionBuilder::new()
                        .with_product_id(payload.product_id().to_string())
                        .with_owner(payload.owner().to_string())
                        .with_product_namespace(payload.product_namespace().clone())
                        .with_properties(combined_properties)
                        .build()
                        .map_err(|err| CliError::PayloadError(format!("{}", err)))?;
                    *payload = payload_with_combined_properties;
                }
            }
            if !product_id_exists {
                total_payloads.push(file_payload);
            }
        }
    }

    Ok(total_payloads)
}

pub fn update_product_payloads_from_file(
    paths: Vec<&str>,
    client: Box<dyn SchemaClient>,
    service_id: Option<&str>,
) -> Result<Vec<ProductUpdateAction>, CliError> {
    let mut total_payloads: Vec<ProductUpdateAction> = Vec::new();

    for path in paths {
        let file_type = determine_file_type(path)?;

        let file_payloads = match file_type {
            ProductFileType::Gdsn3_1 => update_product_payloads_from_xml(path)?,
            ProductFileType::SchemaBasedDefinition => {
                update_product_payloads_from_yaml(path, client.borrow(), service_id)?
            }
        };

        for file_payload in file_payloads {
            let mut product_id_exists = false;

            for payload in total_payloads.iter_mut() {
                if payload.product_id() == file_payload.product_id() {
                    check_duplicate_properties(
                        payload.product_id().to_string(),
                        file_payload.properties().to_vec(),
                        payload.properties().to_vec(),
                    )?;

                    product_id_exists = true;
                    let mut combined_properties: Vec<PropertyValue> = payload.properties().to_vec();
                    combined_properties.append(&mut file_payload.properties().to_vec());
                    let payload_with_combined_properties = ProductUpdateActionBuilder::new()
                        .with_product_id(payload.product_id().to_string())
                        .with_product_namespace(payload.product_namespace().clone())
                        .with_properties(combined_properties)
                        .build()
                        .map_err(|err| CliError::PayloadError(format!("{}", err)))?;
                    *payload = payload_with_combined_properties;
                }
            }
            if !product_id_exists {
                total_payloads.push(file_payload);
            }
        }
    }

    Ok(total_payloads)
}

pub fn create_product_payloads_from_xml(
    path: &str,
    owner: &str,
) -> Result<Vec<ProductCreateAction>, CliError> {
    let trade_items = get_trade_items_from_xml(path)?;
    let data_validation_dir = get_product_schema_dir();
    validate_gdsn_3_1(path, true, &data_validation_dir)?;

    let mut payloads = Vec::new();

    for trade_item in trade_items {
        payloads.push(
            trade_item
                .into_create_payload(owner)
                .map_err(|err| CliError::PayloadError(format!("{}", err)))?,
        );
    }
    Ok(payloads)
}

pub fn create_product_payloads_from_yaml(
    path: &str,
    client: &dyn SchemaClient,
    service_id: Option<&str>,
) -> Result<Vec<ProductCreateAction>, CliError> {
    let file = std::fs::File::open(path)?;
    let ymls: Vec<ProductCreateYaml> = serde_yaml::from_reader(&file)?;

    let mut payloads = Vec::new();

    for yml in ymls {
        let namespace = match yml.product_namespace {
            Namespace::Gs1 => "gs1_product".to_string(),
        };
        let schema = client.get_schema(namespace, service_id)?;
        payloads.push(yml.into_payload(schema.properties)?);
    }

    Ok(payloads)
}

pub fn update_product_payloads_from_xml(path: &str) -> Result<Vec<ProductUpdateAction>, CliError> {
    let trade_items = get_trade_items_from_xml(path)?;
    let data_validation_dir = get_product_schema_dir();
    validate_gdsn_3_1(path, true, &data_validation_dir)?;

    let mut payloads = Vec::new();

    for trade_item in trade_items {
        payloads.push(
            trade_item
                .into_update_payload()
                .map_err(|err| CliError::PayloadError(format!("{}", err)))?,
        )
    }
    Ok(payloads)
}

pub fn update_product_payloads_from_yaml(
    path: &str,
    client: &dyn SchemaClient,
    service_id: Option<&str>,
) -> Result<Vec<ProductUpdateAction>, CliError> {
    let file = std::fs::File::open(path)?;
    let ymls: Vec<ProductUpdateYaml> = serde_yaml::from_reader(&file)?;

    let mut payloads = Vec::new();

    for yml in ymls {
        let namespace = match yml.product_namespace {
            Namespace::Gs1 => "gs1_product".to_string(),
        };
        let schema = client.get_schema(namespace, service_id)?;
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
        definitions: Vec<PropertyDefinition>,
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

fn check_duplicate_properties(
    product_id: String,
    props1: Vec<PropertyValue>,
    props2: Vec<PropertyValue>,
) -> Result<(), CliError> {
    props1.iter().try_for_each(|prop1| {
        props2.iter().try_for_each(|prop2| {
            if prop1.name() == prop2.name() {
                return Err(CliError::UserError(format!(
                    "Duplicate property for {}: {}",
                    product_id,
                    prop1.name()
                )));
            }
            Ok(())
        })
    })
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
        definitions: Vec<PropertyDefinition>,
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
    definitions: Vec<PropertyDefinition>,
) -> Result<Vec<PropertyValue>, CliError> {
    let mut property_values = Vec::new();

    for def in definitions {
        let value = if let Some(value) = properties.get(&def.name) {
            value
        } else if !def.required {
            continue;
        } else {
            if def.name == GDSN_3_1_PROPERTY_NAME {
                continue;
            }
            return Err(CliError::UserError(format!("Field {} not found", def.name)));
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

                property_values.push(property_value);
            }
            DataType::Boolean => {
                let property_value = PropertyValueBuilder::new()
                    .with_name(def.name.clone())
                    .with_data_type(def.data_type.into())
                    .with_boolean_value(serde_yaml::from_value(value.clone())?)
                    .build()
                    .map_err(|err| CliError::PayloadError(format!("{}", err)))?;
                property_values.push(property_value);
            }
            DataType::Number => {
                let property_value = PropertyValueBuilder::new()
                    .with_name(def.name.clone())
                    .with_data_type(def.data_type.into())
                    .with_number_value(serde_yaml::from_value(value.clone())?)
                    .build()
                    .map_err(|err| CliError::PayloadError(format!("{}", err)))?;
                property_values.push(property_value);
            }
            DataType::String => {
                let property_value = PropertyValueBuilder::new()
                    .with_name(def.name.clone())
                    .with_data_type(def.data_type.into())
                    .with_string_value(serde_yaml::from_value(value.clone())?)
                    .build()
                    .map_err(|err| CliError::PayloadError(format!("{}", err)))?;
                property_values.push(property_value);
            }
            DataType::Enum => {
                let property_value = PropertyValueBuilder::new()
                    .with_name(def.name.clone())
                    .with_data_type(def.data_type.into())
                    .with_enum_value(serde_yaml::from_value(value.clone())?)
                    .build()
                    .map_err(|err| CliError::PayloadError(format!("{}", err)))?;
                property_values.push(property_value);
            }
            DataType::Struct => {
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

                property_values.push(property_value);
            }
        }
    }

    Ok(property_values)
}

fn submit_payloads(
    client: Box<dyn ProductClient>,
    signer: Box<dyn Signer>,
    wait: u64,
    actions: Vec<Action>,
    service_id: Option<&str>,
) -> Result<(), CliError> {
    let mut builder = product_batch_builder(signer);

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
                GRID_PIKE_NAMESPACE.to_string(),
                GRID_SCHEMA_NAMESPACE.to_string(),
                GRID_PRODUCT_NAMESPACE.to_string(),
            ],
            &[GRID_PRODUCT_NAMESPACE.to_string()],
        )?;
    }

    let batches = builder.create_batch_list();

    client.post_batches(wait, &batches, service_id)?;
    Ok(())
}

#[derive(Deserialize, Debug)]
pub enum Namespace {
    #[serde(rename = "GS1")]
    Gs1,
}

impl From<Namespace> for ProductNamespace {
    fn from(namespace: Namespace) -> Self {
        match namespace {
            Namespace::Gs1 => ProductNamespace::Gs1,
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

fn get_product_schema_dir() -> String {
    env::var("GRID_PRODUCT_SCHEMA_DIR")
        .unwrap_or_else(|_| DEFAULT_SCHEMA_DIR.to_string() + "/product")
}
