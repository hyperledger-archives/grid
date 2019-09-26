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

use std::collections::{HashMap, HashSet, VecDeque};

use transact::protocol::batch::BatchPair;
use transact::protocol::transaction::{HashMethod, TransactionHeader};
use transact::protos::FromBytes;

use crate::consensus::ProposalId;
use crate::hex::parse_hex;
use crate::service::{ServiceError, ServiceNetworkSender};
use crate::signing::hash::HashVerifier;
use crate::signing::SignatureVerifier;

use super::state::{BatchHistory, ScabbardState};

/// Data structure used to store information that's shared between components in this service
pub struct ScabbardShared {
    /// Queue of batches that have been submitted locally via the REST API, but have not yet been
    /// proposed.
    batch_queue: VecDeque<BatchPair>,
    /// Used to send messages to other services; set when the service is started and unset when the
    /// service is stopped.
    network_sender: Option<Box<dyn ServiceNetworkSender>>,
    /// List of service IDs that this service is configured to communicate and share state with.
    peer_services: HashSet<String>,
    /// Tracks which batches are currently being evaluated, indexed by corresponding proposal IDs.
    proposed_batches: HashMap<ProposalId, BatchPair>,
    signature_verifier: Box<dyn SignatureVerifier>,
    state: ScabbardState,
}

impl ScabbardShared {
    pub fn new(
        batch_queue: VecDeque<BatchPair>,
        network_sender: Option<Box<dyn ServiceNetworkSender>>,
        peer_services: HashSet<String>,
        signature_verifier: Box<dyn SignatureVerifier>,
        state: ScabbardState,
    ) -> Self {
        ScabbardShared {
            batch_queue,
            network_sender,
            peer_services,
            proposed_batches: HashMap::new(),
            signature_verifier,
            state,
        }
    }

    pub fn add_batch_to_queue(&mut self, batch: BatchPair) {
        self.batch_queue.push_back(batch)
    }

    pub fn pop_batch_from_queue(&mut self) -> Option<BatchPair> {
        self.batch_queue.pop_front()
    }

    pub fn network_sender(&self) -> Option<&dyn ServiceNetworkSender> {
        self.network_sender.as_ref().map(|b| &**b)
    }

    pub fn set_network_sender(&mut self, sender: Box<dyn ServiceNetworkSender>) {
        self.network_sender = Some(sender)
    }

    pub fn take_network_sender(&mut self) -> Option<Box<dyn ServiceNetworkSender>> {
        self.network_sender.take()
    }

    pub fn peer_services(&self) -> &HashSet<String> {
        &self.peer_services
    }

    pub fn add_proposed_batch(
        &mut self,
        proposal_id: ProposalId,
        batch: BatchPair,
    ) -> Option<BatchPair> {
        self.proposed_batches.insert(proposal_id, batch)
    }

    pub fn get_proposed_batch(&self, proposal_id: &ProposalId) -> Option<&BatchPair> {
        self.proposed_batches.get(proposal_id)
    }

    pub fn remove_proposed_batch(&mut self, proposal_id: &ProposalId) -> Option<BatchPair> {
        self.proposed_batches.remove(&proposal_id)
    }

    pub fn state_mut(&mut self) -> &mut ScabbardState {
        &mut self.state
    }

    pub fn batch_history(&mut self) -> &mut BatchHistory {
        self.state.batch_history()
    }

    pub fn verify_batches(&self, batches: &[BatchPair]) -> Result<bool, ServiceError> {
        for batch in batches {
            let batch_pub_key = batch.header().signer_public_key();

            // Verify batch signature
            if !self
                .signature_verifier
                .verify(
                    batch.batch().header(),
                    parse_hex(batch.batch().header_signature())
                        .map_err(|err| ServiceError::UnableToHandleMessage(Box::new(err)))?
                        .as_slice(),
                    batch_pub_key,
                )
                .map_err(|err| ServiceError::UnableToHandleMessage(Box::new(err)))?
            {
                warn!(
                    "Batch failed signature verification: {}",
                    batch.batch().header_signature()
                );
                return Ok(false);
            }

            // Verify list of txn IDs in the batch header matches the txns in the batch (verify
            // length here, then verify IDs as each txn is verified)
            if batch.header().transaction_ids().len() != batch.batch().transactions().len() {
                warn!(
                    "Number of transactions in batch header does not match number of transactions
                     in batch: {}",
                    batch.batch().header_signature(),
                );
                return Ok(false);
            }

            // Verify all transactions in batch
            for (i, txn) in batch.batch().transactions().iter().enumerate() {
                let header = TransactionHeader::from_bytes(txn.header())
                    .map_err(|err| ServiceError::InvalidMessageFormat(Box::new(err)))?;

                // Verify this transaction matches the corresponding ID in the batch header
                let header_signature_bytes = parse_hex(txn.header_signature())
                    .map_err(|err| ServiceError::UnableToHandleMessage(Box::new(err)))?;
                if header_signature_bytes != batch.header().transaction_ids()[i] {
                    warn!(
                        "Transaction at index {} does not match corresponding transaction ID in
                         batch header: {}",
                        i,
                        batch.batch().header_signature(),
                    );
                    return Ok(false);
                }

                if header.batcher_public_key() != batch_pub_key {
                    warn!(
                        "Transaction batcher public key does not match batch signer public key -
                         txn: {}, batch: {}",
                        txn.header_signature(),
                        batch.batch().header_signature(),
                    );
                    return Ok(false);
                }

                if !self
                    .signature_verifier
                    .verify(
                        txn.header(),
                        parse_hex(txn.header_signature())
                            .map_err(|err| ServiceError::UnableToHandleMessage(Box::new(err)))?
                            .as_slice(),
                        header.signer_public_key(),
                    )
                    .map_err(|err| ServiceError::UnableToHandleMessage(Box::new(err)))?
                {
                    warn!(
                        "Transaction failed signature verification - txn: {}, batch: {}",
                        txn.header_signature(),
                        batch.batch().header_signature()
                    );
                    return Ok(false);
                }

                if !match header.payload_hash_method() {
                    HashMethod::SHA512 => HashVerifier
                        .verify(txn.payload(), header.payload_hash(), &[])
                        .map_err(|err| ServiceError::UnableToHandleMessage(Box::new(err)))?,
                } {
                    warn!(
                        "Transaction payload hash doesn't match payload - txn: {}, batch: {}",
                        txn.header_signature(),
                        batch.batch().header_signature()
                    );
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }
}
