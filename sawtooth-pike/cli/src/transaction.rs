// Copyright 2018 Cargill Incorporated
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

//! Contains functions which assist with the creation of Pike Batches and
//! Transactions

use std::time::Instant;

use crypto::digest::Digest;
use crypto::sha2::Sha512;

use protobuf;
use protobuf::Message;

use sawtooth_sdk::messages::batch::Batch;
use sawtooth_sdk::messages::batch::BatchHeader;
use sawtooth_sdk::messages::batch::BatchList;
use sawtooth_sdk::messages::transaction::Transaction;
use sawtooth_sdk::messages::transaction::TransactionHeader;
use sawtooth_sdk::signing::Signer;

use addresser::{resource_to_byte, Resource};

use error::CliError;
use protos::payload;
use protos::payload::PikePayload_Action as Action;

/// The Pike transaction family name (pike)
const PIKE_FAMILY_NAME: &'static str = "pike";

/// The Pike transaction family version (0.1)
const PIKE_FAMILY_VERSION: &'static str = "0.1";

/// The Pike namespace prefix for global state (cad11d)
const PIKE_NAMESPACE: &'static str = "cad11d";

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

/// Returns a state address for a given agent name
///
/// # Arguments
///
/// * `name` - the agent's name
fn compute_agent_address(name: &str) -> String {
    let hash: &mut [u8] = &mut [0; 64];

    let mut sha = Sha512::new();
    sha.input(name.as_bytes());
    sha.result(hash);

    String::from(PIKE_NAMESPACE) + &resource_to_byte(Resource::AGENT)
        + &bytes_to_hex_str(hash)[..62]
}

/// Returns a state address for a given organization id
///
/// # Arguments
///
/// * `id` - the organization's id
fn compute_org_address(id: &str) -> String {
    let hash: &mut [u8] = &mut [0; 64];

    let mut sha = Sha512::new();
    sha.input(id.as_bytes());
    sha.result(hash);

    String::from(PIKE_NAMESPACE) + &resource_to_byte(Resource::ORG)
        + &bytes_to_hex_str(hash)[..62]
}

/// Returns a Transaction for the given Payload and Signer
///
/// # Arguments
///
/// * `payload` - a fully populated pike payload
/// * `signer` - the signer to be used to sign the transaction
/// * `public_key` - the public key associated with the signer
///
/// # Errors
///
/// If an error occurs during serialization of the provided payload or
/// internally created `TransactionHeader`, a `CliError::ProtobufError` is
/// returned.
///
/// If a signing error occurs, a `CliError::SigningError` is returned.
pub fn create_transaction(
    payload: &payload::PikePayload,
    signer: &Signer,
    public_key: &String,
) -> Result<Transaction, CliError> {
    let mut txn = Transaction::new();
    let mut txn_header = TransactionHeader::new();

    txn_header.set_family_name(String::from(PIKE_FAMILY_NAME));
    txn_header.set_family_version(String::from(PIKE_FAMILY_VERSION));
    txn_header.set_nonce(create_nonce());
    txn_header.set_signer_public_key(public_key.clone());
    txn_header.set_batcher_public_key(public_key.clone());

    let addresses = match payload.get_action() {
        Action::CREATE_AGENT => {
            let org_id = payload.get_create_agent().get_org_id();
            let agent_public_key = payload.get_create_agent().get_public_key();
            protobuf::RepeatedField::from_vec(vec![
                compute_agent_address(org_id),
                compute_agent_address(agent_public_key),
                compute_agent_address(public_key),
            ])
        }
        Action::UPDATE_AGENT => {
            let org_id = payload.get_update_agent().get_org_id();
            let agent_public_key = payload.get_update_agent().get_public_key();
            protobuf::RepeatedField::from_vec(vec![
                compute_agent_address(org_id),
                compute_agent_address(agent_public_key),
                compute_agent_address(public_key),
            ])
        }
        Action::CREATE_ORGANIZATION => {
            let id = payload.get_create_organization().get_id();
            protobuf::RepeatedField::from_vec(vec![
                compute_org_address(id),
                compute_agent_address(public_key),
            ])
        }
        Action::UPDATE_ORGANIZATION => {
            let id = payload.get_update_organization().get_id();
            protobuf::RepeatedField::from_vec(vec![
                compute_org_address(id),
                compute_agent_address(public_key),
            ])
        }
        _ => protobuf::RepeatedField::from_vec(vec![String::from(PIKE_NAMESPACE)]),
    };

    txn_header.set_inputs(addresses.clone());
    txn_header.set_outputs(addresses.clone());

    let payload_bytes = payload.write_to_bytes()?;
    let mut sha = Sha512::new();
    sha.input(&payload_bytes);
    let hash: &mut [u8] = &mut [0; 64];
    sha.result(hash);
    txn_header.set_payload_sha512(bytes_to_hex_str(hash));
    txn.set_payload(payload_bytes);

    let txn_header_bytes = txn_header.write_to_bytes()?;
    txn.set_header(txn_header_bytes.clone());

    let b: &[u8] = &txn_header_bytes;
    txn.set_header_signature(signer.sign(b)?);

    Ok(txn)
}

/// Returns a Batch for the given Transaction and Signer
///
/// # Arguments
///
/// * `txn` - a Transaction
/// * `signer` - the signer to be used to sign the transaction
/// * `public_key` - the public key associated with the signer
///
/// # Errors
///
/// If an error occurs during serialization of the provided Transaction or
/// internally created `BatchHeader`, a `CliError::ProtobufError` is
/// returned.
///
/// If a signing error occurs, a `CliError::SigningError` is returned.
pub fn create_batch(
    txn: Transaction,
    signer: &Signer,
    public_key: &String,
) -> Result<Batch, CliError> {
    let mut batch = Batch::new();
    let mut batch_header = BatchHeader::new();

    batch_header.set_transaction_ids(protobuf::RepeatedField::from_vec(vec![
        txn.header_signature.clone(),
    ]));
    batch_header.set_signer_public_key(public_key.clone());
    batch.set_transactions(protobuf::RepeatedField::from_vec(vec![txn]));

    let batch_header_bytes = batch_header.write_to_bytes()?;
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
    return batch_list;
}
