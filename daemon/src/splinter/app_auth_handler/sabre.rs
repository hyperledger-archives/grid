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

use std::time::Instant;

use crypto::digest::Digest;
use crypto::sha2::Sha512;
use protobuf::Message;
use sabre_sdk::protocol::payload::{
    Action, CreateContractActionBuilder, CreateContractRegistryActionBuilder,
    CreateNamespaceRegistryActionBuilder, CreateNamespaceRegistryPermissionActionBuilder,
    SabrePayloadBuilder,
};
use sabre_sdk::protocol::ADMINISTRATORS_SETTING_ADDRESS;
use sabre_sdk::protos::IntoBytes as SabreIntoBytes;
use sawtooth_sdk::messages::batch::{Batch, BatchHeader, BatchList};
use sawtooth_sdk::messages::transaction::{Transaction, TransactionHeader};
use sawtooth_sdk::signing::secp256k1::Secp256k1PrivateKey;
use sawtooth_sdk::signing::{create_context, CryptoFactory, Signer};
use splinter::service::scabbard::client::{SabreSmartContractDefinition, ScabbardClient};

use crate::splinter::app_auth_handler::error::AppAuthHandlerError;

const SCABBARD_SUBMISSION_WAIT_SECS: u64 = 10;

/// The namespace registry prefix for global state (00ec00)
const NAMESPACE_REGISTRY_PREFIX: &str = "00ec00";

/// The contract registry prefix for global state (00ec01)
const CONTRACT_REGISTRY_PREFIX: &str = "00ec01";

/// The contract prefix for global state (00ec02)
const CONTRACT_PREFIX: &str = "00ec02";

/// The smart permission prefix for global state (00ec03)
const SMART_PERMISSION_PREFIX: &str = "00ec03";

/// Sabre constants
const SABRE_FAMILY_NAME: &str = "sabre";
const SABRE_FAMILY_VERSION: &str = "0.4";

// Pike constants
const PIKE_PREFIX: &str = "cad11d";
const PIKE_CONTRACT_FILENAME: &str = "grid-pike_0.1.0-dev.scar";

// Product constants
const PRODUCT_PREFIX: &str = "621dee";
const PRODUCT_CONTRACT_FILENAME: &str = "grid-product_0.1.0-dev.scar";

pub fn setup_grid(
    scabbard_admin_key: &str,
    proposed_admin_pubkeys: Vec<String>,
    splinterd_url: &str,
    service_id: &str,
    circuit_id: &str,
) -> Result<(), AppAuthHandlerError> {
    let context = create_context("secp256k1")?;
    let factory = CryptoFactory::new(&*context);
    let private_key = Secp256k1PrivateKey::from_hex(&scabbard_admin_key)?;
    let signer = factory.new_signer(&private_key);

    // The node with the first key in the list of scabbard admins is responsible for setting up xo
    let public_key = signer.get_public_key()?.as_hex();
    let is_submitter = match proposed_admin_pubkeys.get(0) {
        Some(submitting_key) => &public_key == submitting_key,
        None => false,
    };
    if !is_submitter {
        return Ok(());
    }

    // Make Pike transactions
    let pike_contract = SabreSmartContractDefinition::new_from_scar(&format!(
        "/usr/share/scar/{}",
        PIKE_CONTRACT_FILENAME
    ))?;
    let pike_contract_registry_txn =
        make_contract_registry_txn(&signer, &pike_contract.metadata.name)?;
    let pike_contract_txn = make_upload_contract_txn(
        &signer,
        &pike_contract.metadata.name,
        &pike_contract.metadata.version,
        pike_contract.contract,
        PIKE_PREFIX,
    )?;
    let pike_namespace_registry_txn = make_namespace_registry_txn(&signer, PIKE_PREFIX)?;
    let pike_namespace_permissions_txn =
        make_namespace_permissions_txn(&signer, &pike_contract.metadata.name, PIKE_PREFIX)?;

    // Make Product transactions
    let product_contract = SabreSmartContractDefinition::new_from_scar(&format!(
        "/usr/share/scar/{}",
        PRODUCT_CONTRACT_FILENAME
    ))?;
    let product_contract_registry_txn =
        make_contract_registry_txn(&signer, &product_contract.metadata.name)?;
    let product_contract_txn = make_upload_contract_txn(
        &signer,
        &product_contract.metadata.name,
        &product_contract.metadata.version,
        product_contract.contract,
        PRODUCT_PREFIX,
    )?;
    let product_namespace_registry_txn = make_namespace_registry_txn(&signer, PRODUCT_PREFIX)?;
    let product_namespace_permissions_txn =
        make_namespace_permissions_txn(&signer, &product_contract.metadata.name, PRODUCT_PREFIX)?;
    let product_pike_namespace_permissions_txn =
        make_namespace_permissions_txn(&signer, &product_contract.metadata.name, PIKE_PREFIX)?;

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
    ];

    let batch = create_batch(txns, &signer)?;
    let batch_list = create_batch_list_from_one(batch);

    ScabbardClient::new(&splinterd_url)
        .submit(
            circuit_id,
            service_id,
            batch_list,
            Some(SCABBARD_SUBMISSION_WAIT_SECS),
        )
        .map_err(|err| AppAuthHandlerError::BatchSubmitError(err.to_string()))?;

    Ok(())
}

fn make_contract_registry_txn(
    signer: &Signer,
    name: &str,
) -> Result<Transaction, AppAuthHandlerError> {
    let action = CreateContractRegistryActionBuilder::new()
        .with_name(String::from(name))
        .with_owners(vec![signer.get_public_key()?.as_hex()])
        .build()?;
    let payload = SabrePayloadBuilder::new()
        .with_action(Action::CreateContractRegistry(action))
        .build()?
        .into_bytes()?;
    let addresses = vec![
        compute_contract_registry_address(name),
        ADMINISTRATORS_SETTING_ADDRESS.into(),
    ];

    create_txn(signer, addresses, payload)
}

fn make_upload_contract_txn(
    signer: &Signer,
    name: &str,
    version: &str,
    contract: Vec<u8>,
    contract_prefix: &str,
) -> Result<Transaction, AppAuthHandlerError> {
    let action_addresses = vec![
        SMART_PERMISSION_PREFIX.into(),
        PIKE_PREFIX.into(),
        contract_prefix.into(),
    ];
    let action = CreateContractActionBuilder::new()
        .with_name(name.into())
        .with_version(version.into())
        .with_inputs(action_addresses.clone())
        .with_outputs(action_addresses)
        .with_contract(contract)
        .build()?;
    let payload = SabrePayloadBuilder::new()
        .with_action(Action::CreateContract(action))
        .build()?
        .into_bytes()?;
    let addresses = vec![
        compute_contract_registry_address(name),
        compute_contract_address(name, version),
    ];

    create_txn(signer, addresses, payload)
}

fn make_namespace_registry_txn(
    signer: &Signer,
    contract_prefix: &str,
) -> Result<Transaction, AppAuthHandlerError> {
    let action = CreateNamespaceRegistryActionBuilder::new()
        .with_namespace(contract_prefix.into())
        .with_owners(vec![signer.get_public_key()?.as_hex()])
        .build()?;
    let payload = SabrePayloadBuilder::new()
        .with_action(Action::CreateNamespaceRegistry(action))
        .build()?
        .into_bytes()?;
    let addresses = vec![
        compute_namespace_registry_address(contract_prefix)?,
        ADMINISTRATORS_SETTING_ADDRESS.into(),
    ];

    create_txn(signer, addresses, payload)
}

fn create_txn(
    signer: &Signer,
    addresses: Vec<String>,
    payload: Vec<u8>,
) -> Result<Transaction, AppAuthHandlerError> {
    let public_key = signer.get_public_key()?.as_hex();

    let mut txn = Transaction::new();
    let mut txn_header = TransactionHeader::new();

    txn_header.set_family_name(String::from(SABRE_FAMILY_NAME));
    txn_header.set_family_version(String::from(SABRE_FAMILY_VERSION));
    txn_header.set_nonce(create_nonce());
    txn_header.set_signer_public_key(public_key.clone());
    txn_header.set_batcher_public_key(public_key);
    txn_header.set_inputs(protobuf::RepeatedField::from_vec(addresses.clone()));
    txn_header.set_outputs(protobuf::RepeatedField::from_vec(addresses));

    let mut sha = Sha512::new();
    sha.input(&payload);
    let hash: &mut [u8] = &mut [0; 64];
    sha.result(hash);
    txn_header.set_payload_sha512(bytes_to_hex_str(hash));
    txn.set_payload(payload);

    let txn_header_bytes = txn_header.write_to_bytes().map_err(|err| {
        AppAuthHandlerError::SawtoothError(format!(
            "failed to serialize transaction header to bytes: {}",
            err
        ))
    })?;
    txn.set_header(txn_header_bytes.clone());

    let b: &[u8] = &txn_header_bytes;
    txn.set_header_signature(signer.sign(b)?);

    Ok(txn)
}

fn make_namespace_permissions_txn(
    signer: &Signer,
    name: &str,
    contract_prefix: &str,
) -> Result<Transaction, AppAuthHandlerError> {
    let action = CreateNamespaceRegistryPermissionActionBuilder::new()
        .with_namespace(contract_prefix.into())
        .with_contract_name(name.into())
        .with_read(true)
        .with_write(false)
        .build()?;
    let payload = SabrePayloadBuilder::new()
        .with_action(Action::CreateNamespaceRegistryPermission(action))
        .build()?
        .into_bytes()?;
    let addresses = vec![
        compute_namespace_registry_address(PIKE_PREFIX)?,
        compute_namespace_registry_address(contract_prefix)?,
        ADMINISTRATORS_SETTING_ADDRESS.into(),
    ];

    create_txn(signer, addresses, payload)
}

/// Returns a Batch for the given Transactions and Signer
///
/// # Arguments
///
/// * `txns` - list of Transactions
/// * `signer` - the signer to be used to sign the transaction
/// * `public_key` - the public key associated with the signer
pub fn create_batch(txns: Vec<Transaction>, signer: &Signer) -> Result<Batch, AppAuthHandlerError> {
    let public_key = signer.get_public_key()?.as_hex();

    let mut batch = Batch::new();
    let mut batch_header = BatchHeader::new();

    batch_header.set_transaction_ids(protobuf::RepeatedField::from_vec(
        txns.iter()
            .map(|txn| txn.header_signature.clone())
            .collect(),
    ));
    batch_header.set_signer_public_key(public_key);
    batch.set_transactions(protobuf::RepeatedField::from_vec(txns));

    let batch_header_bytes = batch_header.write_to_bytes().map_err(|err| {
        AppAuthHandlerError::SawtoothError(format!(
            "failed to serialize batch header to bytes: {}",
            err
        ))
    })?;
    batch.set_header(batch_header_bytes.clone());

    let b: &[u8] = &batch_header_bytes;
    batch.set_header_signature(signer.sign(b)?);

    Ok(batch)
}

/// Returns a BatchList containing the provided Batch
///
/// # Arguments
///
/// * `batch` - a Batch
pub fn create_batch_list_from_one(batch: Batch) -> BatchList {
    let mut batch_list = BatchList::new();
    batch_list.set_batches(protobuf::RepeatedField::from_vec(vec![batch]));
    batch_list
}

/// Creates a nonce appropriate for a TransactionHeader
fn create_nonce() -> String {
    let elapsed = Instant::now().elapsed();
    format!("{}{}", elapsed.as_secs(), elapsed.subsec_nanos())
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

/// Returns a state address for a given namespace registry
///
/// # Arguments
///
/// * `namespace` - the address prefix for this namespace
fn compute_namespace_registry_address(namespace: &str) -> Result<String, AppAuthHandlerError> {
    let prefix = match namespace.get(..6) {
        Some(x) => x,
        None => {
            return Err(AppAuthHandlerError::SabreError(format!(
                "Namespace must be at least 6 characters long: {}",
                namespace
            )));
        }
    };

    let hash: &mut [u8] = &mut [0; 64];

    let mut sha = Sha512::new();
    sha.input(prefix.as_bytes());
    sha.result(hash);

    Ok(String::from(NAMESPACE_REGISTRY_PREFIX) + &bytes_to_hex_str(hash)[..64])
}

/// Returns a state address for a given contract registry
///
/// # Arguments
///
/// * `name` - the name of the contract registry
fn compute_contract_registry_address(name: &str) -> String {
    let hash: &mut [u8] = &mut [0; 64];

    let mut sha = Sha512::new();
    sha.input(name.as_bytes());
    sha.result(hash);

    String::from(CONTRACT_REGISTRY_PREFIX) + &bytes_to_hex_str(hash)[..64]
}

/// Returns a state address for a given contract
///
/// # Arguments
///
/// * `name` - the name of the contract
/// * `version` - the version of the contract
fn compute_contract_address(name: &str, version: &str) -> String {
    let hash: &mut [u8] = &mut [0; 64];

    let s = String::from(name) + "," + version;

    let mut sha = Sha512::new();
    sha.input(s.as_bytes());
    sha.result(hash);

    String::from(CONTRACT_PREFIX) + &bytes_to_hex_str(hash)[..64]
}
