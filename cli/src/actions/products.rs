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

use crate::http::submit_batches;
use crate::transaction::{
    product_batch_builder, GRID_PRODUCT_NAMESPACE, GRID_SCHEMA_NAMESPACE, PIKE_NAMESPACE,
};
use grid_sdk::protocol::product::payload::{
    Action, ProductCreateAction, ProductCreateActionBuilder, ProductDeleteActionBuilder,
    ProductPayload, ProductPayloadBuilder, ProductUpdateAction, ProductUpdateActionBuilder,
};
use grid_sdk::protocol::product::state::ProductType;
use grid_sdk::protocol::schema::state::PropertyValue;
use grid_sdk::protos::IntoProto;
use reqwest::Client;

use crate::error::CliError;
use serde::Deserialize;

use crate::yaml_parser::{
    parse_value_as_product_type, parse_value_as_repeated_property_values, parse_value_as_sequence,
    parse_value_as_string,
};

use sawtooth_sdk::messages::batch::BatchList;
use serde_yaml::Mapping;
use std::time::{SystemTime, SystemTimeError, UNIX_EPOCH};

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
 * Print the fields for a given product
 *
 * product - Product to be printed
 */
pub fn display_product(product: &GridProduct) {
    println!(
        "Product Id: {:?}\n Product Type: {:?}\n Owner: {:?}\n Properties:",
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
    let products = client.get(&final_url).send()?.json::<Vec<GridProduct>>()?;
    products.iter().for_each(|product| display_product(product));
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
    let product = client.get(&final_url).send()?.json::<GridProduct>()?;
    display_product(&product);
    Ok(())
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
    path: &str,
    service_id: Option<String>,
) -> Result<(), CliError> {
    let payloads = parse_product_yaml(path, Action::ProductCreate(ProductCreateAction::default()))?;
    let batch_list = build_batches_from_payloads(payloads, key)?;
    submit_batches(url, wait, &batch_list, service_id.as_deref())
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
    path: &str,
    service_id: Option<String>,
) -> Result<(), CliError> {
    let payloads = parse_product_yaml(path, Action::ProductUpdate(ProductUpdateAction::default()))?;
    let batch_list = build_batches_from_payloads(payloads, key)?;
    submit_batches(url, wait, &batch_list, service_id.as_deref())
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
    product_id: &str,
    product_type: &str,
    service_id: Option<String>,
) -> Result<(), CliError> {
    let parsed_product_type = parse_value_as_product_type(product_type)?;
    let payloads = vec![generate_delete_product_payload(
        parsed_product_type,
        product_id,
    )?];
    let batch_list = build_batches_from_payloads(payloads, key)?;
    submit_batches(url, wait, &batch_list, service_id.as_deref())
}

/**
 * Build a batch from our Product Payloads. The CLI is responsible for batch creation.
 *
 * payloads - Product payloads
 * key - Signing key of the agent
 */
pub fn build_batches_from_payloads(
    payloads: Vec<ProductPayload>,
    key: Option<String>,
) -> Result<BatchList, CliError> {
    let mut batch_list_builder = product_batch_builder(key);
    for payload in payloads {
        batch_list_builder = batch_list_builder.add_transaction(
            &payload.into_proto()?,
            &[
                PIKE_NAMESPACE.to_string(),
                GRID_SCHEMA_NAMESPACE.to_string(),
                GRID_PRODUCT_NAMESPACE.to_string(),
            ],
            &[GRID_PRODUCT_NAMESPACE.to_string()],
        )?;
    }

    Ok(batch_list_builder.create_batch_list())
}

/**
 * Iterate through a list of products in a yaml file to build our payloads.
 *
 * path: Path to the yaml file
 * action: Determines the type of product payload to generate
 */
fn parse_product_yaml(path: &str, action: Action) -> Result<Vec<ProductPayload>, CliError> {
    let file = std::fs::File::open(path)?;
    let products_yaml: Vec<Mapping> = serde_yaml::from_reader(file)?;

    match action {
        Action::ProductCreate(_) => products_yaml
            .iter()
            .map(|product_yaml| {
                let product_id =
                    parse_value_as_string(product_yaml, "product_id")?.ok_or_else(|| {
                        CliError::InvalidYamlError(
                            "Missing `product_id` field for Product.".to_string(),
                        )
                    })?;

                let product_type = parse_value_as_product_type(
                    &parse_value_as_string(product_yaml, "product_type")?.ok_or_else(|| {
                        CliError::InvalidYamlError(
                            "Missing `product_type` field for property definition.".to_string(),
                        )
                    })?,
                )?;

                let owner = parse_value_as_string(product_yaml, "owner")?.ok_or_else(|| {
                    CliError::InvalidYamlError("Missing `owner` field for Product.".to_string())
                })?;

                let properties =
                    parse_value_as_sequence(product_yaml, "properties")?.ok_or_else(|| {
                        CliError::InvalidYamlError(
                            "Product is missing `properties` field.".to_string(),
                        )
                    })?;

                let property_values = parse_value_as_repeated_property_values(&properties)?;

                generate_create_product_payload(product_type, &product_id, &owner, &property_values)
            })
            .collect::<Result<Vec<ProductPayload>, _>>(),
        Action::ProductUpdate(_) => products_yaml
            .iter()
            .map(|product_yaml| {
                let product_id =
                    parse_value_as_string(product_yaml, "product_id")?.ok_or_else(|| {
                        CliError::InvalidYamlError(
                            "Missing `product_id` field for Product.".to_string(),
                        )
                    })?;

                let product_type = parse_value_as_product_type(
                    &parse_value_as_string(product_yaml, "product_type")?.ok_or_else(|| {
                        CliError::InvalidYamlError(
                            "Missing `product_type` field for property definition.".to_string(),
                        )
                    })?,
                )?;

                let properties =
                    parse_value_as_sequence(product_yaml, "properties")?.ok_or_else(|| {
                        CliError::InvalidYamlError(
                            "Product is missing `properties` field.".to_string(),
                        )
                    })?;

                let property_values = parse_value_as_repeated_property_values(&properties)?;

                generate_update_product_payload(product_type, &product_id, &property_values)
            })
            .collect::<Result<Vec<ProductPayload>, _>>(),
        Action::ProductDelete(_) => Err(CliError::UserError("To delete a product pass the arguments to the command line directly rather than using a Yaml file.".to_string()))
    }
}

/**
 * Generate the payload needed to create a new product
 *
 * product_type - e.g. GS1
 * product_id - e.g. GTIN
 * owner - Identifier of the organization responsible for maintaining the product
 * properties - One or more property values
 */
fn generate_create_product_payload(
    product_type: ProductType,
    product_id: &str,
    owner: &str,
    properties: &[PropertyValue],
) -> Result<ProductPayload, CliError> {
    let product_payload = ProductPayloadBuilder::new();

    let product_create_action_builder = ProductCreateActionBuilder::new()
        .with_product_id(product_id.to_string())
        .with_product_type(product_type)
        .with_owner(owner.to_string())
        .with_properties(properties.to_vec());

    let product_create_action = product_create_action_builder.build().map_err(|err| {
        CliError::PayloadError(format!("Failed to build product create payload: {}", err))
    })?;

    let timestamp = match get_unix_utc_timestamp() {
        Ok(timestamp) => timestamp,
        Err(err) => {
            return Err(CliError::PayloadError(format!(
                "Failed to build product create payload: {}",
                err
            )))
        }
    };

    product_payload
        .with_timestamp(timestamp)
        .with_action(Action::ProductCreate(product_create_action))
        .build()
        .map_err(|err| CliError::PayloadError(format!("Failed to build product payload: {}", err)))
}

/**
 * Generate the payload needed to update an existing product
 *
 * product_type - e.g. GS1
 * product_id - e.g. GTIN
 * properties - One or more property values
 */
fn generate_update_product_payload(
    product_type: ProductType,
    product_id: &str,
    properties: &[PropertyValue],
) -> Result<ProductPayload, CliError> {
    let product_payload = ProductPayloadBuilder::new();

    let product_update_action_builder = ProductUpdateActionBuilder::new()
        .with_product_id(product_id.to_string())
        .with_product_type(product_type)
        .with_properties(properties.to_vec());

    let product_update_action = product_update_action_builder.build().map_err(|err| {
        CliError::PayloadError(format!("Failed to build product update payload: {}", err))
    })?;

    let timestamp = match get_unix_utc_timestamp() {
        Ok(timestamp) => timestamp,
        Err(err) => {
            return Err(CliError::PayloadError(format!(
                "Failed to build product update payload: {}",
                err
            )))
        }
    };

    product_payload
        .with_timestamp(timestamp)
        .with_action(Action::ProductUpdate(product_update_action))
        .build()
        .map_err(|err| {
            CliError::PayloadError(format!("Failed to build product update payload: {}", err))
        })
}

/**
 * Generate the payload needed to delete an existing product
 *
 * product_type - e.g. GS1
 * product_id - e.g. GTIN
 */
fn generate_delete_product_payload(
    product_type: ProductType,
    product_id: &str,
) -> Result<ProductPayload, CliError> {
    let product_payload = ProductPayloadBuilder::new();

    let product_delete_action_builder = ProductDeleteActionBuilder::new()
        .with_product_id(product_id.to_string())
        .with_product_type(product_type);

    let product_delete_action = product_delete_action_builder.build().map_err(|err| {
        CliError::PayloadError(format!("Failed to build product delete payload: {}", err))
    })?;

    let timestamp = match get_unix_utc_timestamp() {
        Ok(timestamp) => timestamp,
        Err(err) => {
            return Err(CliError::PayloadError(format!(
                "Failed to build product delete payload: {}",
                err
            )))
        }
    };

    product_payload
        .with_action(Action::ProductDelete(product_delete_action))
        .with_timestamp(timestamp)
        .build()
        .map_err(|err| {
            CliError::PayloadError(format!("Failed to build product delete payload: {}", err))
        })
}

fn get_unix_utc_timestamp() -> Result<u64, SystemTimeError> {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => Ok(duration.as_secs()),
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use grid_sdk::protocol::product::payload::{Action, ProductPayload};
    use grid_sdk::protocol::product::state::ProductType;
    use grid_sdk::protocol::schema::state::{DataType, PropertyValueBuilder};
    use std::fs::{remove_file, File};
    use std::io::Write;
    use std::{env, panic, thread};

    static EXAMPLE_PRODUCT_YAML: &[u8; 288] = br##"- product_type: "GS1"
  product_id: "723382885088"
  owner: "314156"
  properties:
    - name: "length"
      data_type: "NUMBER"
      number_value: 8
    - name: "width"
      data_type: "NUMBER"
      number_value: 11
    - name: "depth"
      data_type: "NUMBER"
      number_value: 1"##;

    /*
     * Verifies parse_product_yaml returns valids ProductPayload with ProductCreateAction set from a yaml
     * containing a multiple Product definitions
     */
    #[test]
    fn test_valid_yaml_create_product() {
        run_test(|test_yaml_file_path| {
            write_yaml_file(test_yaml_file_path);

            let payload = parse_product_yaml(
                test_yaml_file_path,
                Action::ProductCreate(ProductCreateAction::default()),
            )
            .expect("Error parsing yaml");

            assert_eq!(make_create_product_payload(), payload[0]);
        })
    }

    /*
     * Verifies parse_product_yaml returns valids ProductPayload with ProductUpdateAction set from a yaml
     * containing a multiple Product definitions
     */
    #[test]
    fn test_valid_yaml_update_multiple_products() {
        run_test(|test_yaml_file_path| {
            write_yaml_file(test_yaml_file_path);

            let payload = parse_product_yaml(
                test_yaml_file_path,
                Action::ProductUpdate(ProductUpdateAction::default()),
            )
            .expect("Error parsing yaml");

            assert_eq!(make_update_product_payload(), payload[0]);
        })
    }

    fn write_yaml_file(file_path: &str) {
        let mut file = File::create(file_path).expect("Error creating test product yaml file.");

        file.write_all(EXAMPLE_PRODUCT_YAML)
            .expect("Error writting example product yaml.");
    }

    fn make_update_product_payload() -> ProductPayload {
        generate_update_product_payload(ProductType::GS1, "723382885088", &create_property_values())
            .unwrap()
    }

    fn make_create_product_payload() -> ProductPayload {
        generate_create_product_payload(
            ProductType::GS1,
            "723382885088",
            "314156",
            &create_property_values(),
        )
        .unwrap()
    }

    fn create_property_values() -> Vec<PropertyValue> {
        vec![
            make_number_property_value("length", 8),
            make_number_property_value("width", 11),
            make_number_property_value("depth", 1),
        ]
    }

    fn make_number_property_value(name: &str, number_value: i64) -> PropertyValue {
        PropertyValueBuilder::new()
            .with_name(name.to_string())
            .with_data_type(DataType::Number)
            .with_number_value(number_value)
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
        temp_dir.push(format!("test_parse_product-{:?}.yaml", thread_id));
        temp_dir.to_str().unwrap().to_string()
    }
}
