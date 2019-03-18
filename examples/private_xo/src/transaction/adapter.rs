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

use std::fmt::Write as FmtWrite;

use sawtooth_sdk::messages::processor::TpProcessRequest;
use sawtooth_sdk::messages::transaction::TransactionHeader as SawtoothTxnHeader;
use sawtooth_sdk::processor::handler::{
    ApplyError as SawtoothApplyError, ContextError as SawtoothContextError,
    TransactionContext as SawtoothContext, TransactionHandler as SawtoothHandler,
};

use transact::handler::{ApplyError, ContextError, TransactionContext, TransactionHandler};
use transact::protocol::transaction::{TransactionHeader, TransactionPair};

pub struct SawtoothToTransactHandlerAdapter<H: SawtoothHandler> {
    family_name: String,
    family_versions: Vec<String>,
    handler: H,
}

impl<H: SawtoothHandler> SawtoothToTransactHandlerAdapter<H> {
    pub fn new(handler: H) -> Self {
        SawtoothToTransactHandlerAdapter {
            family_name: handler.family_name().clone(),
            family_versions: handler.family_versions().clone(),
            handler,
        }
    }
}

impl<H: SawtoothHandler + Sync + Send> TransactionHandler for SawtoothToTransactHandlerAdapter<H> {
    fn family_name(&self) -> &str {
        &self.family_name
    }

    fn family_versions(&self) -> &[String] {
        &self.family_versions
    }

    fn apply(
        &self,
        transaction_pair: &TransactionPair,
        context: &mut dyn TransactionContext,
    ) -> Result<(), ApplyError> {
        let request = txn_pair_to_process_request(transaction_pair);
        let mut context_adapter = TransactToSawtoothContextAdapter::new(context);
        self.handler
            .apply(&request, &mut context_adapter)
            .map_err(|err| match err {
                SawtoothApplyError::InvalidTransaction(error_message) => {
                    ApplyError::InvalidTransaction(error_message)
                }
                SawtoothApplyError::InternalError(error_message) => {
                    ApplyError::InternalError(error_message)
                }
            })
    }
}

struct TransactToSawtoothContextAdapter<'a> {
    transact_context: &'a TransactionContext,
}

impl<'a> TransactToSawtoothContextAdapter<'a> {
    fn new(transact_context: &'a TransactionContext) -> Self {
        TransactToSawtoothContextAdapter { transact_context }
    }
}

impl<'a> SawtoothContext for TransactToSawtoothContextAdapter<'a> {
    fn get_state_entry(&self, address: &str) -> Result<Option<Vec<u8>>, SawtoothContextError> {
        let results = self
            .transact_context
            .get_state_entries(&[address.to_owned()])
            .map_err(to_context_error)?;

        // take the first item, if it exists
        Ok(results.into_iter().next().map(|(_, v)| v))
    }

    fn get_state_entries(
        &self,
        addresses: &[String],
    ) -> Result<Vec<(String, Vec<u8>)>, SawtoothContextError> {
        self.transact_context
            .get_state_entries(addresses)
            .map_err(to_context_error)
    }

    fn set_state_entry(&self, address: String, data: Vec<u8>) -> Result<(), SawtoothContextError> {
        self.set_state_entries(vec![(address, data)])
    }

    fn set_state_entries(
        &self,
        entries: Vec<(String, Vec<u8>)>,
    ) -> Result<(), SawtoothContextError> {
        self.transact_context
            .set_state_entries(entries)
            .map_err(to_context_error)
    }

    fn delete_state_entry(&self, address: &str) -> Result<Option<String>, SawtoothContextError> {
        Ok(self
            .delete_state_entries(&[address.to_owned()])?
            .into_iter()
            .next())
    }

    fn delete_state_entries(
        &self,
        addresses: &[String],
    ) -> Result<Vec<String>, SawtoothContextError> {
        self.transact_context
            .delete_state_entries(addresses)
            .map_err(to_context_error)
    }

    fn add_receipt_data(&self, data: &[u8]) -> Result<(), SawtoothContextError> {
        self.transact_context
            .add_receipt_data(data.to_vec())
            .map_err(to_context_error)
    }

    fn add_event(
        &self,
        event_type: String,
        attributes: Vec<(String, String)>,
        data: &[u8],
    ) -> Result<(), SawtoothContextError> {
        self.transact_context
            .add_event(event_type, attributes, data.to_vec())
            .map_err(to_context_error)
    }
}

fn txn_pair_to_process_request(transaction_pair: &TransactionPair) -> TpProcessRequest {
    let mut process_request = TpProcessRequest::new();

    let header = as_sawtooth_header(transaction_pair.header());
    process_request.set_header(header);

    let txn = transaction_pair.transaction();
    process_request.set_payload(txn.payload().to_vec());
    process_request.set_signature(txn.header_signature().to_owned());

    process_request
}

fn as_sawtooth_header(header: &TransactionHeader) -> SawtoothTxnHeader {
    let mut sawtooth_header = SawtoothTxnHeader::new();

    sawtooth_header.set_family_name(header.family_name().to_owned());
    sawtooth_header.set_family_version(header.family_version().to_owned());
    sawtooth_header.set_signer_public_key(to_hex(&header.signer_public_key()));
    sawtooth_header.set_batcher_public_key(to_hex(&header.batcher_public_key()));
    sawtooth_header.set_dependencies(header.dependencies().iter().map(to_hex).collect());
    sawtooth_header.set_inputs(header.inputs().iter().map(to_hex).collect());
    sawtooth_header.set_outputs(header.outputs().iter().map(to_hex).collect());
    sawtooth_header.set_nonce(to_hex(&header.nonce()));

    sawtooth_header
}

fn to_hex<T: AsRef<[u8]>>(bytes: &T) -> String {
    let bytes = bytes.as_ref();
    let mut buf = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        write!(&mut buf, "{:0x}", b).unwrap(); // this can't fail
    }
    buf
}

fn to_context_error(err: ContextError) -> SawtoothContextError {
    SawtoothContextError::ReceiveError(Box::new(err))
}
