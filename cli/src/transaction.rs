// Copyright 2019 Cargill Incorporated
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

//! Contains functions which assist with the creation of Batches and
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
use sawtooth_sdk::signing;

use crate::key;

use crate::CliError;

const PIKE_NAMESPACE: &str = "cad11d";
const PIKE_FAMILY_NAME: &str = "pike";
const PIKE_FAMILY_VERSION: &str = "0.1";

pub fn pike_batch_builder(key: Option<String>) -> BatchBuilder {
    BatchBuilder::new(
        PIKE_FAMILY_NAME,
        PIKE_FAMILY_VERSION,
        vec![PIKE_NAMESPACE.into()],
        key,
    )
}

#[derive(Clone)]
pub struct BatchBuilder {
    family_name: String,
    family_version: String,
    namespaces: Vec<String>,
    key_name: Option<String>,
    batches: Vec<Batch>,
}

impl BatchBuilder {
    pub fn new(
        family_name: &str,
        family_version: &str,
        namespaces: Vec<String>,
        key_name: Option<String>,
    ) -> BatchBuilder {
        BatchBuilder {
            family_name: family_name.to_string(),
            family_version: family_version.to_string(),
            namespaces,
            key_name,
            batches: Vec::new(),
        }
    }

    pub fn add_transaction<T: protobuf::Message>(&mut self, payload: &T) -> Result<Self, CliError> {
        let private_key = key::load_signing_key(self.key_name.clone())?;
        let context = signing::create_context("secp256k1")?;
        let public_key = context.get_public_key(&private_key)?.as_hex();
        let factory = signing::CryptoFactory::new(&*context);
        let signer = factory.new_signer(&private_key);

        let mut txn = Transaction::new();
        let mut txn_header = TransactionHeader::new();

        txn_header.set_family_name(self.family_name.clone());
        txn_header.set_family_version(self.family_version.clone());
        txn_header.set_nonce(create_nonce());
        txn_header.set_signer_public_key(public_key.clone());
        txn_header.set_batcher_public_key(public_key.clone());

        txn_header.set_inputs(protobuf::RepeatedField::from_vec(self.namespaces.clone()));
        txn_header.set_outputs(protobuf::RepeatedField::from_vec(self.namespaces.clone()));

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

        let mut batch = Batch::new();
        let mut batch_header = BatchHeader::new();

        batch_header.set_transaction_ids(protobuf::RepeatedField::from_vec(vec![txn
            .header_signature
            .clone()]));
        batch_header.set_signer_public_key(public_key.clone());
        batch.set_transactions(protobuf::RepeatedField::from_vec(vec![txn]));

        let batch_header_bytes = batch_header.write_to_bytes()?;
        batch.set_header(batch_header_bytes.clone());

        batch.set_header_signature(signer.sign(&batch_header_bytes)?);

        self.batches.push(batch);

        Ok(self.clone())
    }

    pub fn create_batch_list(&mut self) -> BatchList {
        let mut batch_list = BatchList::new();
        batch_list.set_batches(protobuf::RepeatedField::from_vec(self.batches.clone()));

        batch_list
    }
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
