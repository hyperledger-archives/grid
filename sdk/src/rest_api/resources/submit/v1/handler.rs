// Copyright 2018-2021 Cargill Incorporated
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

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use crypto::digest::Digest;
use crypto::sha2::Sha512;
use cylinder::{load_key, secp256k1::Secp256k1Context, Context, PrivateKey};
use protobuf::Message;
use sabre_sdk::{
    protocol::payload::ExecuteContractActionBuilder, protos::IntoBytes as SabreIntoBytes,
};
use sawtooth_sdk::messages::{batch, transaction};

use super::payloads::{Batch, SubmitBatchRequest, SubmitBatchResponse};
use crate::batches::{
    store::{Batch as DbBatch, BatchStoreError},
    BatchStore,
};
use crate::protos::IntoBytes;
use crate::rest_api::resources::error::ErrorResponse;

const SABRE_FAMILY_NAME: &str = "sabre";
const SABRE_FAMILY_VERSION: &str = "0.5";
const SABRE_NAMESPACE_REGISTRY_PREFIX: &str = "00ec00";
const SABRE_CONTRACT_REGISTRY_PREFIX: &str = "00ec01";
const SABRE_CONTRACT_PREFIX: &str = "00ec02";

pub async fn submit_batches(
    key_file_name: &str,
    batch_store: Arc<dyn BatchStore>,
    request: SubmitBatchRequest,
) -> Result<SubmitBatchResponse, ErrorResponse> {
    let private_key = load_key(key_file_name, &[PathBuf::from("/etc/grid/keys")])
        .map_err(|err| {
            error!("{}", err);
            ErrorResponse::internal_error(Box::new(err))
        })?
        .ok_or_else(|| {
            error!("Signing key not found");
            ErrorResponse::new(500, "Signing key not found")
        })?;

    let db_batches = batches_into_bytes(private_key, request.batches)?;

    let mut ids = Vec::new();

    for db_batch in db_batches {
        let id = db_batch.id().to_string();

        batch_store.add_batch(db_batch).map_err(|err| match err {
            BatchStoreError::ConstraintViolationError(err) => {
                ErrorResponse::new(400, &format!("{}", err))
            }
            BatchStoreError::ResourceTemporarilyUnavailableError(_) => {
                ErrorResponse::new(503, "Service unavailable")
            }
            err => {
                error!("{}", err);
                ErrorResponse::internal_error(Box::new(err))
            }
        })?;

        ids.push(id);
    }

    Ok(SubmitBatchResponse::new(ids))
}

fn batches_into_bytes(
    private_key: PrivateKey,
    batches_from_request: Vec<Batch>,
) -> Result<Vec<DbBatch>, ErrorResponse> {
    let mut batches = Vec::new();
    for batch in batches_from_request {
        let mut transactions = Vec::new();

        let context = Secp256k1Context::new();

        let public_key = context
            .get_public_key(&private_key)
            .map_err(|err| {
                error!("{}", err);
                ErrorResponse::internal_error(Box::new(err))
            })?
            .as_hex();

        let signer = context.new_signer(private_key.clone());

        for transaction in batch.transactions {
            let payload_bytes = transaction.payload.into_bytes().map_err(|err| {
                error!("{}", err);
                ErrorResponse::internal_error(Box::new(err))
            })?;

            let sabre_payload = ExecuteContractActionBuilder::new()
                .with_name(transaction.family_name.to_string())
                .with_version(transaction.version.to_string())
                .with_inputs(transaction.inputs.to_vec())
                .with_outputs(transaction.outputs.to_vec())
                .with_payload(payload_bytes)
                .into_payload_builder()
                .map_err(|err| {
                    error!("{}", err);
                    ErrorResponse::internal_error(Box::new(err))
                })?
                .build()
                .map_err(|err| {
                    error!("{}", err);
                    ErrorResponse::internal_error(Box::new(err))
                })?;

            let mut input_addresses = vec![
                compute_contract_registry_address(&transaction.family_name),
                compute_contract_address(&transaction.family_name, &transaction.version),
            ];

            for input in transaction.inputs.clone() {
                let namespace = match input.get(..6) {
                    Some(namespace) => namespace,
                    None => {
                        return Err(ErrorResponse::new(
                            400,
                            &format!("Input must be at least 6 characters long: {}", input),
                        ));
                    }
                };

                input_addresses.push(compute_namespace_registry_address(namespace)?);
            }
            input_addresses.append(&mut transaction.inputs.to_vec());

            let mut output_addresses = vec![
                compute_contract_registry_address(&transaction.family_name),
                compute_contract_address(&transaction.family_name, &transaction.version),
            ];

            for output in transaction.outputs.clone() {
                let namespace = match output.get(..6) {
                    Some(namespace) => namespace,
                    None => {
                        return Err(ErrorResponse::new(
                            400,
                            &format!("Output must be at least 6 characters long: {}", output),
                        ));
                    }
                };

                output_addresses.push(compute_namespace_registry_address(namespace)?);
            }
            output_addresses.append(&mut transaction.outputs.to_vec());

            let mut txn = transaction::Transaction::new();
            let mut txn_header = transaction::TransactionHeader::new();

            txn_header.set_family_name(SABRE_FAMILY_NAME.into());
            txn_header.set_family_version(SABRE_FAMILY_VERSION.into());
            txn_header.set_nonce(create_nonce());
            txn_header.set_signer_public_key(public_key.clone());
            txn_header.set_batcher_public_key(public_key.clone());

            txn_header.set_inputs(protobuf::RepeatedField::from_vec(input_addresses));
            txn_header.set_outputs(protobuf::RepeatedField::from_vec(output_addresses));

            let payload_bytes = sabre_payload.into_bytes().map_err(|err| {
                error!("{}", err);
                ErrorResponse::internal_error(Box::new(err))
            })?;
            let mut sha = Sha512::new();
            sha.input(&payload_bytes);
            let hash: &mut [u8] = &mut [0; 64];
            sha.result(hash);
            txn_header.set_payload_sha512(bytes_to_hex_str(hash));
            txn.set_payload(payload_bytes.to_vec());

            let txn_header_bytes = txn_header.write_to_bytes().map_err(|err| {
                error!("{}", err);
                ErrorResponse::internal_error(Box::new(err))
            })?;
            txn.set_header(txn_header_bytes.clone());

            let b: &[u8] = &txn_header_bytes;
            txn.set_header_signature(
                signer
                    .sign(b)
                    .map_err(|err| {
                        error!("{}", err);
                        ErrorResponse::internal_error(Box::new(err))
                    })?
                    .as_hex(),
            );

            transactions.push(txn);
        }

        let mut batch = batch::Batch::new();
        let mut batch_header = batch::BatchHeader::new();

        batch_header.set_transaction_ids(protobuf::RepeatedField::from_vec(
            transactions
                .iter()
                .map(|txn| txn.header_signature.clone())
                .collect(),
        ));
        batch_header.set_signer_public_key(public_key);
        batch.set_transactions(protobuf::RepeatedField::from_vec(transactions));

        let batch_header_bytes = batch_header.write_to_bytes().map_err(|err| {
            error!("{}", err);
            ErrorResponse::internal_error(Box::new(err))
        })?;
        batch.set_header(batch_header_bytes.clone());

        let header_signature = signer
            .sign(&batch_header_bytes)
            .map_err(|err| {
                error!("{}", err);
                ErrorResponse::internal_error(Box::new(err))
            })?
            .as_hex();

        batch.set_header_signature(header_signature.clone());

        let mut batch_list = batch::BatchList::new();
        batch_list.set_batches(protobuf::RepeatedField::from_vec(vec![batch]));

        let bytes = batch_list.write_to_bytes().map_err(|err| {
            error!("{}", err);
            ErrorResponse::internal_error(Box::new(err))
        })?;

        batches.push(DbBatch::from_bytes(&header_signature, &bytes));
    }

    Ok(batches)
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

/// Returns a state address for a given sabre contract registry
///
/// # Arguments
///
/// * `name` - the name of the contract registry
fn compute_contract_registry_address(name: &str) -> String {
    let hash: &mut [u8] = &mut [0; 64];

    let mut sha = Sha512::new();
    sha.input(name.as_bytes());
    sha.result(hash);

    String::from(SABRE_CONTRACT_REGISTRY_PREFIX) + &bytes_to_hex_str(hash)[..64]
}

/// Returns a state address for a given sabre contract
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

    String::from(SABRE_CONTRACT_PREFIX) + &bytes_to_hex_str(hash)[..64]
}

/// Returns a state address for a given namespace registry
///
/// # Arguments
///
/// * `namespace` - the address prefix for this namespace
fn compute_namespace_registry_address(namespace: &str) -> Result<String, ErrorResponse> {
    let prefix = match namespace.get(..6) {
        Some(x) => x,
        None => {
            return Err(ErrorResponse::new(
                400,
                &format!(
                    "Namespace must be at least 6 characters long: {}",
                    namespace
                ),
            ));
        }
    };

    let hash: &mut [u8] = &mut [0; 64];

    let mut sha = Sha512::new();
    sha.input(prefix.as_bytes());
    sha.result(hash);

    Ok(String::from(SABRE_NAMESPACE_REGISTRY_PREFIX) + &bytes_to_hex_str(hash)[..64])
}
