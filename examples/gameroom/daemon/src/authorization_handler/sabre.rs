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

use std::convert::TryInto;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use futures::future::{self, Future};
use futures::stream::Stream;
use hyper::{Body, Client, Request, StatusCode};
use sabre_sdk::protocol::{
    compute_contract_address,
    payload::{
        CreateContractActionBuilder, CreateContractRegistryActionBuilder,
        CreateNamespaceRegistryActionBuilder, CreateNamespaceRegistryPermissionActionBuilder,
    },
};
use sawtooth_sdk::signing::{
    create_context, secp256k1::Secp256k1PrivateKey, transact::TransactSigner,
    Signer as SawtoothSigner,
};
use splinter::protocol;
use transact::{
    protocol::{batch::BatchBuilder, transaction::Transaction},
    protos::IntoBytes,
    signing::Signer,
};

use super::AppAuthHandlerError;

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
    let signer = new_signer(private_key)?;

    // The node with the first key in the list of scabbard admins is responsible for setting up xo
    let public_key = bytes_to_hex_str(signer.public_key());
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
        create_xo_namespace_registry_txn(scabbard_admin_keys, &signer)?,
        xo_namespace_permissions_txn(&signer)?,
    ];
    let batch = BatchBuilder::new().with_transactions(txns).build(&signer)?;
    let payload = vec![batch].into_bytes()?;

    // Submit the batch to the scabbard service
    let body_stream = futures::stream::once::<_, std::io::Error>(Ok(payload));
    let req = Request::builder()
        .uri(format!(
            "{}/scabbard/{}/{}/batches",
            splinterd_url, circuit_id, service_id
        ))
        .method("POST")
        .header(
            "SplinterProtocolVersion",
            protocol::SCABBARD_PROTOCOL_VERSION.to_string(),
        )
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

fn new_signer(private_key: &str) -> Result<TransactSigner, AppAuthHandlerError> {
    let context = create_context("secp256k1")?;
    let private_key = Box::new(Secp256k1PrivateKey::from_hex(private_key)?);
    Ok(SawtoothSigner::new_boxed(context, private_key).try_into()?)
}

fn create_contract_registry_txn(
    owners: Vec<String>,
    signer: &dyn Signer,
) -> Result<Transaction, AppAuthHandlerError> {
    Ok(CreateContractRegistryActionBuilder::new()
        .with_name(XO_NAME.into())
        .with_owners(owners)
        .into_payload_builder()?
        .into_transaction_builder(signer)?
        .build(signer)?)
}

fn upload_contract_txn(signer: &dyn Signer) -> Result<Transaction, AppAuthHandlerError> {
    let contract_path = Path::new(XO_CONTRACT_PATH);
    let contract_file = File::open(contract_path).map_err(|err| {
        AppAuthHandlerError::SabreError(format!("Failed to load contract: {}", err))
    })?;
    let mut buf_reader = BufReader::new(contract_file);
    let mut contract = Vec::new();
    buf_reader.read_to_end(&mut contract).map_err(|err| {
        AppAuthHandlerError::SabreError(format!("IoError while reading contract: {}", err))
    })?;

    let action_addresses = vec![XO_PREFIX.into()];

    Ok(CreateContractActionBuilder::new()
        .with_name(XO_NAME.into())
        .with_version(XO_VERSION.into())
        .with_inputs(action_addresses.clone())
        .with_outputs(action_addresses)
        .with_contract(contract)
        .into_payload_builder()?
        .into_transaction_builder(signer)?
        .build(signer)?)
}

fn create_xo_namespace_registry_txn(
    owners: Vec<String>,
    signer: &dyn Signer,
) -> Result<Transaction, AppAuthHandlerError> {
    Ok(CreateNamespaceRegistryActionBuilder::new()
        .with_namespace(XO_PREFIX.into())
        .with_owners(owners)
        .into_payload_builder()?
        .into_transaction_builder(signer)?
        .build(signer)?)
}

fn xo_namespace_permissions_txn(signer: &dyn Signer) -> Result<Transaction, AppAuthHandlerError> {
    Ok(CreateNamespaceRegistryPermissionActionBuilder::new()
        .with_namespace(XO_PREFIX.into())
        .with_contract_name(XO_NAME.into())
        .with_read(true)
        .with_write(true)
        .into_payload_builder()?
        .into_transaction_builder(signer)?
        .build(signer)?)
}

pub fn get_xo_contract_address() -> Result<String, AppAuthHandlerError> {
    Ok(bytes_to_hex_str(&compute_contract_address(
        XO_NAME, XO_VERSION,
    )?))
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
