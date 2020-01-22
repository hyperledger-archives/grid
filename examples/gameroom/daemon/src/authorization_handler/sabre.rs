/*
 * Copyright 2018-2020 Cargill Incorporated
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

//! This module is based on the Sawtooth Sabre CLI.

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use std::time::Instant;

use crypto::digest::Digest;
use crypto::sha2::Sha512;
use futures::future::{self, Future};
use futures::stream::Stream;
use hyper::{Body, Client, Request, StatusCode};
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

use super::AppAuthHandlerError;

/// The Sawtooth Sabre transaction family name (sabre)
const SABRE_FAMILY_NAME: &str = "sabre";
/// The Sawtooth Sabre transaction family version (0.4)
const SABRE_FAMILY_VERSION: &str = "0.4";

/// The namespace registry prefix for global state (00ec00)
const NAMESPACE_REGISTRY_PREFIX: &str = "00ec00";

/// The contract registry prefix for global state (00ec01)
const CONTRACT_REGISTRY_PREFIX: &str = "00ec01";

/// The contract prefix for global state (00ec02)
const CONTRACT_PREFIX: &str = "00ec02";

/// The smart permission prefix for global state (00ec03)
const SMART_PERMISSION_PREFIX: &str = "00ec03";

const PIKE_PREFIX: &str = "cad11d";

const XO_NAME: &str = "xo";
const XO_VERSION: &str = "0.3.3";
pub const XO_PREFIX: &str = "5b7349";

const XO_CONTRACT_PATH: &str = "/var/lib/gameroomd/xo-tp-rust.wasm";

/// Create and submit the Sabre transactions to setup the XO smart contract.
pub fn setup_xo(
    private_key: &str,
    scabbard_admin_keys: Vec<String>,
    splinterd_url: &str,
    circuit_id: &str,
    service_id: &str,
) -> Result<Box<dyn Future<Item = (), Error = ()> + Send + 'static>, AppAuthHandlerError> {
    let context = create_context("secp256k1")?;
    let factory = CryptoFactory::new(&*context);
    let private_key = Secp256k1PrivateKey::from_hex(private_key)?;
    let signer = factory.new_signer(&private_key);

    // The node with the first key in the list of scabbard admins is responsible for setting up xo
    let public_key = signer.get_public_key()?.as_hex();
    let is_submitter = match scabbard_admin_keys.get(0) {
        Some(submitting_key) => &public_key == submitting_key,
        None => false,
    };
    if !is_submitter {
        return Ok(Box::new(future::ok(())));
    }

    // Create the transactions and batch them
    let txns = vec![
        create_contract_registry_txn(scabbard_admin_keys.clone(), &signer)?,
        upload_contract_txn(&signer)?,
        create_xo_namespace_registry_txn(scabbard_admin_keys.clone(), &signer)?,
        xo_namespace_permissions_txn(&signer)?,
        create_pike_namespace_registry_txn(scabbard_admin_keys, &signer)?,
        pike_namespace_permissions_txn(&signer)?,
    ];
    let batch = create_batch(txns, &signer)?;
    let batch_list = create_batch_list_from_one(batch);
    let payload = batch_list.write_to_bytes().map_err(|err| {
        AppAuthHandlerError::SawtoothError(format!("failed to serialize batch list: {}", err))
    })?;
    // Submit the batch to the scabbard service
    let body_stream = futures::stream::once::<_, std::io::Error>(Ok(payload));
    let req = Request::builder()
        .uri(format!(
            "{}/scabbard/{}/{}/batches",
            splinterd_url, circuit_id, service_id
        ))
        .method("POST")
        .body(Body::wrap_stream(body_stream))
        .map_err(|err| AppAuthHandlerError::BatchSubmitError(format!("{}", err)))?;

    let client = Client::new();

    Ok(Box::new(
        client
            .request(req)
            .then(|response| match response {
                Ok(res) => {
                    let status = res.status();
                    let body = res
                        .into_body()
                        .concat2()
                        .wait()
                        .map_err(|err| {
                            AppAuthHandlerError::BatchSubmitError(format!(
                                "The client encountered an error {}",
                                err
                            ))
                        })?
                        .to_vec();

                    match status {
                        StatusCode::ACCEPTED => Ok(()),
                        _ => Err(AppAuthHandlerError::BatchSubmitError(format!(
                            "The server returned an error. Status: {}, {}",
                            status,
                            String::from_utf8(body)?
                        ))),
                    }
                }
                Err(err) => Err(AppAuthHandlerError::BatchSubmitError(format!(
                    "The client encountered an error {}",
                    err
                ))),
            })
            .map_err(|_| ()),
    ))
}

fn create_contract_registry_txn(
    owners: Vec<String>,
    signer: &Signer,
) -> Result<Transaction, AppAuthHandlerError> {
    let action = CreateContractRegistryActionBuilder::new()
        .with_name(XO_NAME.into())
        .with_owners(owners)
        .build()?;
    let payload = SabrePayloadBuilder::new()
        .with_action(Action::CreateContractRegistry(action))
        .build()?
        .into_bytes()?;
    let addresses = vec![
        compute_contract_registry_address(XO_NAME),
        ADMINISTRATORS_SETTING_ADDRESS.into(),
    ];

    create_txn(addresses, payload, signer)
}

fn upload_contract_txn(signer: &Signer) -> Result<Transaction, AppAuthHandlerError> {
    let contract_path = Path::new(XO_CONTRACT_PATH);
    let contract_file = File::open(contract_path).map_err(|err| {
        AppAuthHandlerError::SabreError(format!("Failed to load contract: {}", err))
    })?;
    let mut buf_reader = BufReader::new(contract_file);
    let mut contract = Vec::new();
    buf_reader.read_to_end(&mut contract).map_err(|err| {
        AppAuthHandlerError::SabreError(format!("IoError while reading contract: {}", err))
    })?;

    let action_addresses = vec![
        SMART_PERMISSION_PREFIX.into(),
        PIKE_PREFIX.into(),
        XO_PREFIX.into(),
    ];
    let action = CreateContractActionBuilder::new()
        .with_name(XO_NAME.into())
        .with_version(XO_VERSION.into())
        .with_inputs(action_addresses.clone())
        .with_outputs(action_addresses)
        .with_contract(contract)
        .build()?;
    let payload = SabrePayloadBuilder::new()
        .with_action(Action::CreateContract(action))
        .build()?
        .into_bytes()?;
    let addresses = vec![
        compute_contract_registry_address(XO_NAME),
        compute_contract_address(XO_NAME, XO_VERSION),
    ];

    create_txn(addresses, payload, signer)
}

fn create_xo_namespace_registry_txn(
    owners: Vec<String>,
    signer: &Signer,
) -> Result<Transaction, AppAuthHandlerError> {
    let action = CreateNamespaceRegistryActionBuilder::new()
        .with_namespace(XO_PREFIX.into())
        .with_owners(owners)
        .build()?;
    let payload = SabrePayloadBuilder::new()
        .with_action(Action::CreateNamespaceRegistry(action))
        .build()?
        .into_bytes()?;
    let addresses = vec![
        compute_namespace_registry_address(XO_PREFIX)?,
        ADMINISTRATORS_SETTING_ADDRESS.into(),
    ];

    create_txn(addresses, payload, signer)
}

fn xo_namespace_permissions_txn(signer: &Signer) -> Result<Transaction, AppAuthHandlerError> {
    let action = CreateNamespaceRegistryPermissionActionBuilder::new()
        .with_namespace(XO_PREFIX.into())
        .with_contract_name(XO_NAME.into())
        .with_read(true)
        .with_write(true)
        .build()?;
    let payload = SabrePayloadBuilder::new()
        .with_action(Action::CreateNamespaceRegistryPermission(action))
        .build()?
        .into_bytes()?;
    let addresses = vec![
        compute_namespace_registry_address(XO_PREFIX)?,
        ADMINISTRATORS_SETTING_ADDRESS.into(),
    ];

    create_txn(addresses, payload, signer)
}

fn create_pike_namespace_registry_txn(
    owners: Vec<String>,
    signer: &Signer,
) -> Result<Transaction, AppAuthHandlerError> {
    let action = CreateNamespaceRegistryActionBuilder::new()
        .with_namespace(PIKE_PREFIX.into())
        .with_owners(owners)
        .build()?;
    let payload = SabrePayloadBuilder::new()
        .with_action(Action::CreateNamespaceRegistry(action))
        .build()?
        .into_bytes()?;
    let addresses = vec![
        compute_namespace_registry_address(PIKE_PREFIX)?,
        ADMINISTRATORS_SETTING_ADDRESS.into(),
    ];

    create_txn(addresses, payload, signer)
}

fn pike_namespace_permissions_txn(signer: &Signer) -> Result<Transaction, AppAuthHandlerError> {
    let action = CreateNamespaceRegistryPermissionActionBuilder::new()
        .with_namespace(PIKE_PREFIX.into())
        .with_contract_name(XO_NAME.into())
        .with_read(true)
        .with_write(false)
        .build()?;
    let payload = SabrePayloadBuilder::new()
        .with_action(Action::CreateNamespaceRegistryPermission(action))
        .build()?
        .into_bytes()?;
    let addresses = vec![
        compute_namespace_registry_address(PIKE_PREFIX)?,
        ADMINISTRATORS_SETTING_ADDRESS.into(),
    ];

    create_txn(addresses, payload, signer)
}

fn create_txn(
    addresses: Vec<String>,
    payload: Vec<u8>,
    signer: &Signer,
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

pub fn get_xo_contract_address() -> String {
    compute_contract_address(XO_NAME, XO_VERSION)
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
