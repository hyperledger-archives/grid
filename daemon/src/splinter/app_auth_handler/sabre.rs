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

use sabre_sdk::protocol::payload::{
    CreateContractActionBuilder, CreateContractRegistryActionBuilder,
    CreateNamespaceRegistryActionBuilder, CreateNamespaceRegistryPermissionActionBuilder,
};
use sawtooth_sdk::signing::{
    create_context, secp256k1::Secp256k1PrivateKey, transact::TransactSigner,
    Signer as SawtoothSigner,
};
use scabbard::client::{ScabbardClient, ServiceId};
use transact::{
    contract::archive::{default_scar_path, SmartContractArchive},
    protocol::{batch::BatchBuilder, transaction::Transaction},
    signing::Signer,
};

use crate::splinter::app_auth_handler::error::AppAuthHandlerError;

const SCABBARD_SUBMISSION_WAIT_SECS: u64 = 10;

// Pike constants
const PIKE_PREFIX: &str = "cad11d";
const PIKE_CONTRACT_NAME: &str = "grid-pike";
const PIKE_CONTRACT_VERSION_REQ: &str = "0.1.0-dev";

// Product constants
const PRODUCT_PREFIX: &str = "621dee02";
const PRODUCT_CONTRACT_NAME: &str = "grid-product";
const PRODUCT_CONTRACT_VERSION_REQ: &str = "0.1.0-dev";

// Schema constants
const SCHEMA_PREFIX: &str = "621dee01";
const SCHEMA_CONTRACT_NAME: &str = "grid-schema";
const SCHEMA_CONTRACT_VERSION_REQ: &str = "0.1.0-dev";

pub fn setup_grid(
    scabbard_admin_key: &str,
    proposed_admin_pubkeys: Vec<String>,
    splinterd_url: &str,
    service_id: &str,
    circuit_id: &str,
) -> Result<(), AppAuthHandlerError> {
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

    // Make Pike transactions
    let pike_contract = SmartContractArchive::from_scar_file(
        PIKE_CONTRACT_NAME,
        PIKE_CONTRACT_VERSION_REQ,
        &default_scar_path(),
    )?;
    let pike_contract_registry_txn =
        make_contract_registry_txn(&signer, &pike_contract.metadata.name)?;
    let pike_contract_txn = make_upload_contract_txn(&signer, &pike_contract, PIKE_PREFIX)?;
    let pike_namespace_registry_txn = make_namespace_registry_txn(&signer, PIKE_PREFIX)?;
    let pike_namespace_permissions_txn =
        make_namespace_permissions_txn(&signer, &pike_contract, PIKE_PREFIX)?;

    // Make Product transactions
    let product_contract = SmartContractArchive::from_scar_file(
        PRODUCT_CONTRACT_NAME,
        PRODUCT_CONTRACT_VERSION_REQ,
        &default_scar_path(),
    )?;
    let product_contract_registry_txn =
        make_contract_registry_txn(&signer, &product_contract.metadata.name)?;
    let product_contract_txn =
        make_upload_contract_txn(&signer, &product_contract, PRODUCT_PREFIX)?;
    let product_namespace_registry_txn = make_namespace_registry_txn(&signer, PRODUCT_PREFIX)?;
    let product_namespace_permissions_txn =
        make_namespace_permissions_txn(&signer, &product_contract, PRODUCT_PREFIX)?;
    let product_pike_namespace_permissions_txn =
        make_namespace_permissions_txn(&signer, &product_contract, PIKE_PREFIX)?;
    let product_schema_namespace_permissions_txn =
        make_namespace_permissions_txn(&signer, &product_contract, SCHEMA_PREFIX)?;

    // Make schema transactions
    let schema_contract = SmartContractArchive::from_scar_file(
        SCHEMA_CONTRACT_NAME,
        SCHEMA_CONTRACT_VERSION_REQ,
        &default_scar_path(),
    )?;
    let schema_contract_registry_txn =
        make_contract_registry_txn(&signer, &schema_contract.metadata.name)?;
    let schema_contract_txn = make_upload_contract_txn(&signer, &schema_contract, SCHEMA_PREFIX)?;
    let schema_namespace_registry_txn = make_namespace_registry_txn(&signer, SCHEMA_PREFIX)?;
    let schema_namespace_permissions_txn =
        make_namespace_permissions_txn(&signer, &schema_contract, SCHEMA_PREFIX)?;
    let schema_pike_namespace_permissions_txn =
        make_namespace_permissions_txn(&signer, &schema_contract, PIKE_PREFIX)?;

    let txns = vec![
        pike_contract_registry_txn,
        pike_contract_txn,
        pike_namespace_registry_txn,
        pike_namespace_permissions_txn,
        product_contract_registry_txn,
        product_contract_txn,
        product_namespace_registry_txn,
        product_pike_namespace_permissions_txn,
        product_namespace_permissions_txn,
        schema_contract_registry_txn,
        schema_contract_txn,
        schema_namespace_registry_txn,
        product_schema_namespace_permissions_txn,
        schema_pike_namespace_permissions_txn,
        schema_namespace_permissions_txn,
    ];
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

fn new_signer(private_key: &str) -> Result<TransactSigner, AppAuthHandlerError> {
    let context = create_context("secp256k1")?;
    let private_key = Box::new(Secp256k1PrivateKey::from_hex(private_key)?);
    Ok(SawtoothSigner::new_boxed(context, private_key).try_into()?)
}

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

fn make_upload_contract_txn(
    signer: &dyn Signer,
    contract: &SmartContractArchive,
    contract_prefix: &str,
) -> Result<Transaction, AppAuthHandlerError> {
    let action_addresses = vec![PIKE_PREFIX.into(), contract_prefix.into()];
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
