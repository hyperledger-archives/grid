/*
 * Copyright 2020 Cargill Incorporated
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * -----------------------------------------------------------------------------
 */

use std::convert::TryInto;
use std::time::Duration;

#[cfg(feature = "location")]
use grid_sdk::locations::addressing::GRID_LOCATION_NAMESPACE;
#[cfg(feature = "pike")]
use grid_sdk::pike::addressing::PIKE_NAMESPACE;
#[cfg(feature = "product")]
use grid_sdk::products::addressing::GRID_PRODUCT_NAMESPACE;
#[cfg(feature = "purchase-order")]
use grid_sdk::purchase_order::addressing::GRID_PURCHASE_ORDER_NAMESPACE;
#[cfg(feature = "schema")]
use grid_sdk::schemas::addressing::GRID_SCHEMA_NAMESPACE;

use sabre_sdk::protocol::payload::{
    CreateContractActionBuilder, CreateContractRegistryActionBuilder,
    CreateNamespaceRegistryActionBuilder, CreateNamespaceRegistryPermissionActionBuilder,
};
use sawtooth_sdk::signing::{
    create_context, secp256k1::Secp256k1PrivateKey, transact::TransactSigner,
    Signer as SawtoothSigner,
};
use scabbard::client::{ScabbardClient, ServiceId};
#[cfg(any(
    feature = "location",
    feature = "pike",
    feature = "product",
    feature = "purchase-order",
    feature = "schema"
))]
use transact::contract::archive::default_scar_path;
use transact::{
    contract::archive::SmartContractArchive,
    protocol::{batch::BatchBuilder, transaction::Transaction},
    signing::Signer,
};

use crate::splinter::app_auth_handler::error::AppAuthHandlerError;

const SCABBARD_SUBMISSION_WAIT_SECS: u64 = 10;

pub fn setup_grid(
    scabbard_admin_key: &str,
    proposed_admin_pubkeys: Vec<String>,
    splinterd_url: &str,
    service_id: &str,
    circuit_id: &str,
) -> Result<(), AppAuthHandlerError> {
    #[cfg(any(
        feature = "location",
        feature = "pike",
        feature = "product",
        feature = "purchase-order",
        feature = "schema"
    ))]
    let version = env!("CARGO_PKG_VERSION");

    let signer = new_signer(&scabbard_admin_key)?;

    // The node with the first key in the list of scabbard admins is responsible for setting up xo
    let public_key = bytes_to_hex_str(signer.public_key());
    let is_submitter = match proposed_admin_pubkeys.get(0) {
        Some(submitting_key) => &public_key == submitting_key,
        None => false,
    };
    if !is_submitter {
        return Ok(());
    }

    let mut txns = Vec::new();

    // Make Pike transactions
    #[cfg(feature = "pike")]
    make_pike_txns(&mut txns, &version, &signer)?;

    // Make Product transactions
    #[cfg(feature = "product")]
    make_product_txns(&mut txns, &version, &signer)?;

    // Make Location transactions
    #[cfg(feature = "location")]
    make_location_txns(&mut txns, &version, &signer)?;

    // Make schema transactions
    #[cfg(feature = "schema")]
    make_schema_txns(&mut txns, &version, &signer)?;

    // Make Purchase Order transactions
    #[cfg(feature = "purchase-order")]
    make_purchase_order_txns(&mut txns, &version, &signer)?;

    let batch = BatchBuilder::new().with_transactions(txns).build(&signer)?;

    ScabbardClient::new(&splinterd_url)
        .submit(
            &ServiceId::new(circuit_id, service_id),
            vec![batch],
            Some(Duration::from_secs(SCABBARD_SUBMISSION_WAIT_SECS)),
        )
        .map_err(|err| AppAuthHandlerError::BatchSubmitError(err.to_string()))?;

    Ok(())
}

#[cfg(feature = "pike")]
fn make_pike_txns(
    txns: &mut Vec<Transaction>,
    version: &str,
    signer: &TransactSigner,
) -> Result<(), AppAuthHandlerError> {
    let pike_contract =
        SmartContractArchive::from_scar_file("grid_pike", &version, &default_scar_path())?;
    let pike_contract_registry_txn =
        make_contract_registry_txn(signer, &pike_contract.metadata.name)?;
    let pike_contract_txn = make_upload_contract_txn(signer, &pike_contract, PIKE_NAMESPACE)?;
    let pike_namespace_registry_txn = make_namespace_registry_txn(signer, PIKE_NAMESPACE)?;
    let pike_namespace_permissions_txn =
        make_namespace_permissions_txn(signer, &pike_contract, PIKE_NAMESPACE)?;

    txns.append(&mut vec![
        pike_contract_registry_txn,
        pike_contract_txn,
        pike_namespace_registry_txn,
        pike_namespace_permissions_txn,
    ]);

    Ok(())
}

#[cfg(feature = "product")]
fn make_product_txns(
    txns: &mut Vec<Transaction>,
    version: &str,
    signer: &TransactSigner,
) -> Result<(), AppAuthHandlerError> {
    let product_contract =
        SmartContractArchive::from_scar_file("grid-product", &version, &default_scar_path())?;
    let product_contract_registry_txn =
        make_contract_registry_txn(signer, &product_contract.metadata.name)?;
    let product_contract_txn =
        make_upload_contract_txn(signer, &product_contract, GRID_PRODUCT_NAMESPACE)?;
    let product_namespace_registry_txn =
        make_namespace_registry_txn(signer, GRID_PRODUCT_NAMESPACE)?;
    let product_namespace_permissions_txn =
        make_namespace_permissions_txn(signer, &product_contract, GRID_PRODUCT_NAMESPACE)?;
    let product_pike_namespace_permissions_txn =
        make_namespace_permissions_txn(signer, &product_contract, PIKE_NAMESPACE)?;
    let product_schema_namespace_permissions_txn =
        make_namespace_permissions_txn(signer, &product_contract, GRID_SCHEMA_NAMESPACE)?;

    txns.append(&mut vec![
        product_contract_registry_txn,
        product_contract_txn,
        product_namespace_registry_txn,
        product_namespace_permissions_txn,
        product_pike_namespace_permissions_txn,
        product_schema_namespace_permissions_txn,
    ]);

    Ok(())
}

#[cfg(feature = "location")]
fn make_location_txns(
    txns: &mut Vec<Transaction>,
    version: &str,
    signer: &TransactSigner,
) -> Result<(), AppAuthHandlerError> {
    let location_contract =
        SmartContractArchive::from_scar_file("grid-location", &version, &default_scar_path())?;
    let location_contract_registry_txn =
        make_contract_registry_txn(signer, &location_contract.metadata.name)?;
    let location_contract_txn =
        make_upload_contract_txn(signer, &location_contract, GRID_LOCATION_NAMESPACE)?;
    let location_namespace_registry_txn =
        make_namespace_registry_txn(signer, GRID_LOCATION_NAMESPACE)?;
    let location_namespace_permissions_txn =
        make_namespace_permissions_txn(signer, &location_contract, GRID_LOCATION_NAMESPACE)?;
    let location_pike_namespace_permissions_txn =
        make_namespace_permissions_txn(signer, &location_contract, PIKE_NAMESPACE)?;
    let location_schema_namespace_permissions_txn =
        make_namespace_permissions_txn(signer, &location_contract, GRID_SCHEMA_NAMESPACE)?;

    txns.append(&mut vec![
        location_contract_registry_txn,
        location_contract_txn,
        location_namespace_registry_txn,
        location_namespace_permissions_txn,
        location_pike_namespace_permissions_txn,
        location_schema_namespace_permissions_txn,
    ]);

    Ok(())
}

#[cfg(feature = "schema")]
fn make_schema_txns(
    txns: &mut Vec<Transaction>,
    version: &str,
    signer: &TransactSigner,
) -> Result<(), AppAuthHandlerError> {
    let schema_contract =
        SmartContractArchive::from_scar_file("grid-schema", &version, &default_scar_path())?;
    let schema_contract_registry_txn =
        make_contract_registry_txn(signer, &schema_contract.metadata.name)?;
    let schema_contract_txn =
        make_upload_contract_txn(signer, &schema_contract, GRID_SCHEMA_NAMESPACE)?;
    let schema_namespace_registry_txn = make_namespace_registry_txn(signer, GRID_SCHEMA_NAMESPACE)?;
    let schema_namespace_permissions_txn =
        make_namespace_permissions_txn(signer, &schema_contract, GRID_SCHEMA_NAMESPACE)?;
    let schema_pike_namespace_permissions_txn =
        make_namespace_permissions_txn(signer, &schema_contract, PIKE_NAMESPACE)?;

    txns.append(&mut vec![
        schema_contract_registry_txn,
        schema_contract_txn,
        schema_namespace_registry_txn,
        schema_namespace_permissions_txn,
        schema_pike_namespace_permissions_txn,
    ]);

    Ok(())
}

#[cfg(feature = "purchase-order")]
fn make_purchase_order_txns(
    txns: &mut Vec<Transaction>,
    version: &str,
    signer: &TransactSigner,
) -> Result<(), AppAuthHandlerError> {
    let purchase_order_contract = SmartContractArchive::from_scar_file(
        "grid-purchase-order",
        &version,
        &default_scar_path(),
    )?;
    let purchase_order_contract_registry_txn =
        make_contract_registry_txn(signer, &purchase_order_contract.metadata.name)?;
    let purchase_order_contract_txn = make_upload_contract_txn(
        signer,
        &purchase_order_contract,
        GRID_PURCHASE_ORDER_NAMESPACE,
    )?;
    let purchase_order_namespace_registry_txn =
        make_namespace_registry_txn(signer, GRID_PURCHASE_ORDER_NAMESPACE)?;
    let purchase_order_namespace_permissions_txn = make_namespace_permissions_txn(
        signer,
        &purchase_order_contract,
        GRID_PURCHASE_ORDER_NAMESPACE,
    )?;
    let purchase_order_pike_namespace_permissions_txn =
        make_namespace_permissions_txn(signer, &purchase_order_contract, PIKE_NAMESPACE)?;

    txns.append(&mut vec![
        purchase_order_contract_registry_txn,
        purchase_order_contract_txn,
        purchase_order_namespace_registry_txn,
        purchase_order_namespace_permissions_txn,
        purchase_order_pike_namespace_permissions_txn,
    ]);

    Ok(())
}

fn new_signer(private_key: &str) -> Result<TransactSigner, AppAuthHandlerError> {
    let context = create_context("secp256k1")?;
    let private_key = Box::new(Secp256k1PrivateKey::from_hex(private_key)?);
    Ok(SawtoothSigner::new_boxed(context, private_key).try_into()?)
}

#[cfg(any(
    feature = "location",
    feature = "pike",
    feature = "product",
    feature = "purchase-order",
    feature = "schema"
))]
fn make_contract_registry_txn(
    signer: &dyn Signer,
    name: &str,
) -> Result<Transaction, AppAuthHandlerError> {
    Ok(CreateContractRegistryActionBuilder::new()
        .with_name(String::from(name))
        .with_owners(vec![bytes_to_hex_str(signer.public_key())])
        .into_payload_builder()?
        .into_transaction_builder(signer)?
        .build(signer)?)
}

#[cfg(feature = "pike")]
fn make_upload_contract_txn(
    signer: &dyn Signer,
    contract: &SmartContractArchive,
    contract_prefix: &str,
) -> Result<Transaction, AppAuthHandlerError> {
    let action_addresses = vec![PIKE_NAMESPACE.into(), contract_prefix.into()];
    Ok(CreateContractActionBuilder::new()
        .with_name(contract.metadata.name.clone())
        .with_version(contract.metadata.version.clone())
        .with_inputs(action_addresses.clone())
        .with_outputs(action_addresses)
        .with_contract(contract.contract.clone())
        .into_payload_builder()?
        .into_transaction_builder(signer)?
        .build(signer)?)
}

#[cfg(any(
    feature = "location",
    feature = "pike",
    feature = "product",
    feature = "purchase-order",
    feature = "schema"
))]
fn make_namespace_registry_txn(
    signer: &dyn Signer,
    contract_prefix: &str,
) -> Result<Transaction, AppAuthHandlerError> {
    Ok(CreateNamespaceRegistryActionBuilder::new()
        .with_namespace(contract_prefix.into())
        .with_owners(vec![bytes_to_hex_str(signer.public_key())])
        .into_payload_builder()?
        .into_transaction_builder(signer)?
        .build(signer)?)
}

#[cfg(any(
    feature = "location",
    feature = "pike",
    feature = "product",
    feature = "purchase-order",
    feature = "schema"
))]
fn make_namespace_permissions_txn(
    signer: &dyn Signer,
    contract: &SmartContractArchive,
    contract_prefix: &str,
) -> Result<Transaction, AppAuthHandlerError> {
    Ok(CreateNamespaceRegistryPermissionActionBuilder::new()
        .with_namespace(contract_prefix.into())
        .with_contract_name(contract.metadata.name.clone())
        .with_read(true)
        .with_write(true)
        .into_payload_builder()?
        .into_transaction_builder(signer)?
        .build(signer)?)
}

/// Returns a hex string representation of the supplied bytes
///
/// # Arguments
///
/// * `b` - input bytes
fn bytes_to_hex_str(b: &[u8]) -> String {
    b.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join("")
}
