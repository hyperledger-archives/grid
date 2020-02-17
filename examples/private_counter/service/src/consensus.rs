// Copyright 2018-2020 Cargill Incorporated
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

use std::convert::TryInto;
use std::sync::{Arc, Mutex};

use protobuf::Message;
use uuid::Uuid;

use splinter::consensus::{
    error::{ConsensusSendError, ProposalManagerError},
    ConsensusMessage, ConsensusNetworkSender, PeerId, Proposal, ProposalId, ProposalManager,
    ProposalUpdate,
};
use splinter::network::sender::SendRequest;

use crate::error::ServiceError;
use crate::protos::private_counter::{
    PrivateCounterMessage, PrivateCounterMessage_Type, ProposedIncrement,
};
use crate::{create_circuit_direct_msg, hash, to_hex, write_u32, ServiceState};

pub struct PrivateCounterProposalManager {
    state: Arc<Mutex<ServiceState>>,
}

impl PrivateCounterProposalManager {
    pub fn new(state: Arc<Mutex<ServiceState>>) -> Self {
        PrivateCounterProposalManager { state }
    }
}

impl ProposalManager for PrivateCounterProposalManager {
    fn create_proposal(
        &self,
        _previous_proposal_id: Option<ProposalId>,
        consensus_data: Vec<u8>,
    ) -> Result<(), ProposalManagerError> {
        let state = self.state.lock().expect("State lock has been poisoned");
        match state.proposed_increments.keys().next() {
            Some(expected_hash) => {
                // Cheating a bit here by not setting the ID properly (isn't a hash of previous_id,
                // proposal_height, and summary), but none of this really matters with 2-phase
                // consensus. The ID is the expected hash, and it is also used as the key for the
                // proposed increment hashmap. This example will not work with forking consensus,
                // because it does not track previously accepted proposals.
                let mut proposal = Proposal::default();
                proposal.id = expected_hash.clone().into();
                proposal.summary = expected_hash.clone();
                proposal.consensus_data = consensus_data;

                // This unwrap can't fail because the key was taken from the HashMap
                let increment = *state.proposed_increments.get(expected_hash).unwrap();

                // Send the proposal to the other services
                let mut proposed_increment = ProposedIncrement::new();
                proposed_increment.set_increment(increment);
                proposed_increment.set_expected_hash(expected_hash.to_vec());
                let mut msg = PrivateCounterMessage::new();
                msg.set_message_type(PrivateCounterMessage_Type::PROPOSED_INCREMENT);
                msg.set_proposed_increment(proposed_increment);

                for verifier in &state.verifiers {
                    state
                        .service_sender
                        .send(SendRequest::new(
                            state.peer_id.clone(),
                            create_circuit_direct_msg(
                                state.circuit.clone(),
                                state.service_id.clone(),
                                verifier.clone(),
                                msg.write_to_bytes().map_err(ServiceError::from)?,
                                Uuid::new_v4().to_string(),
                            )?,
                        ))
                        .map_err(ServiceError::from)?;
                }

                state
                    .proposal_update_sender
                    .send(ProposalUpdate::ProposalCreated(Some(proposal)))?;
            }
            None => {
                state
                    .proposal_update_sender
                    .send(ProposalUpdate::ProposalCreated(None))?;
            }
        }

        Ok(())
    }

    fn check_proposal(&self, id: &ProposalId) -> Result<(), ProposalManagerError> {
        let state = self.state.lock().expect("State lock has been poisoned");

        match state.proposed_increments.get(id.as_ref()) {
            Some(increment) => {
                let check_result = hash(&write_u32(state.counter + increment)?);
                // proposal id == expected hash
                if check_result.as_slice() != id.as_ref() {
                    warn!(
                        "Hash mismatch: expected {} but was {}",
                        to_hex(id.as_ref()),
                        to_hex(&check_result),
                    );
                    warn!(
                        "In our state: {} + {} = {}",
                        state.counter,
                        increment,
                        state.counter + increment,
                    );

                    state
                        .proposal_update_sender
                        .send(ProposalUpdate::ProposalInvalid(id.clone()))?;
                } else {
                    state
                        .proposal_update_sender
                        .send(ProposalUpdate::ProposalValid(id.clone()))?;
                }
            }
            None => {
                warn!("Checked unknown proposal: {:?}", id);
                state
                    .proposal_update_sender
                    .send(ProposalUpdate::ProposalInvalid(id.clone()))?;
            }
        }

        Ok(())
    }

    fn accept_proposal(
        &self,
        id: &ProposalId,
        _consensus_data: Option<Vec<u8>>,
    ) -> Result<(), ProposalManagerError> {
        let mut state = self.state.lock().expect("State lock has been poisoned");

        match state.proposed_increments.remove(id.as_ref()) {
            Some(increment) => {
                let prev = state.counter;
                state.counter += increment;
                info!("Committed count increment: {} -> {}", prev, state.counter);

                state
                    .proposal_update_sender
                    .send(ProposalUpdate::ProposalAccepted(id.clone()))?;
            }
            None => state
                .proposal_update_sender
                .send(ProposalUpdate::ProposalAcceptFailed(
                    id.clone(),
                    "unknown proposal".into(),
                ))?,
        }

        Ok(())
    }

    fn reject_proposal(&self, id: &ProposalId) -> Result<(), ProposalManagerError> {
        match self
            .state
            .lock()
            .expect("State lock has been poisoned")
            .proposed_increments
            .remove(id.as_ref())
        {
            Some(_) => {
                info!("Rejected count increment");
            }
            None => warn!("Rejected unknown proposal"),
        }

        Ok(())
    }
}

pub struct PrivateCounterNetworkSender {
    state: Arc<Mutex<ServiceState>>,
}

impl PrivateCounterNetworkSender {
    pub fn new(state: Arc<Mutex<ServiceState>>) -> Self {
        PrivateCounterNetworkSender { state }
    }
}

impl ConsensusNetworkSender for PrivateCounterNetworkSender {
    fn send_to(&self, peer_id: &PeerId, message: Vec<u8>) -> Result<(), ConsensusSendError> {
        let state = self.state.lock().expect("State lock has been poisoned");

        let consensus_message = ConsensusMessage::new(message, state.service_id.as_bytes().into());
        let mut msg = PrivateCounterMessage::new();
        msg.set_message_type(PrivateCounterMessage_Type::CONSENSUS_MESSAGE);
        msg.set_consensus_message(consensus_message.try_into()?);

        state
            .service_sender
            .send(SendRequest::new(
                state.peer_id.clone(),
                create_circuit_direct_msg(
                    state.circuit.clone(),
                    state.service_id.clone(),
                    String::from_utf8(peer_id.clone().into())
                        .map_err(|err| ConsensusSendError::Internal(Box::new(err)))?,
                    msg.write_to_bytes()
                        .map_err(|err| ConsensusSendError::Internal(Box::new(err)))?,
                    Uuid::new_v4().to_string(),
                )
                .map_err(|err| ConsensusSendError::Internal(Box::new(err)))?,
            ))
            .map_err(|err| ConsensusSendError::Internal(Box::new(ServiceError::from(err))))?;

        Ok(())
    }

    fn broadcast(&self, message: Vec<u8>) -> Result<(), ConsensusSendError> {
        let state = self.state.lock().expect("State lock has been poisoned");

        let consensus_message = ConsensusMessage::new(message, state.service_id.as_bytes().into());
        let mut msg = PrivateCounterMessage::new();
        msg.set_message_type(PrivateCounterMessage_Type::CONSENSUS_MESSAGE);
        msg.set_consensus_message(consensus_message.try_into()?);

        for verifier in &state.verifiers {
            state
                .service_sender
                .send(SendRequest::new(
                    state.peer_id.clone(),
                    create_circuit_direct_msg(
                        state.circuit.clone(),
                        state.service_id.clone(),
                        verifier.clone(),
                        msg.write_to_bytes()
                            .map_err(|err| ConsensusSendError::Internal(Box::new(err)))?,
                        Uuid::new_v4().to_string(),
                    )
                    .map_err(|err| ConsensusSendError::Internal(Box::new(err)))?,
                ))
                .map_err(|err| ConsensusSendError::Internal(Box::new(ServiceError::from(err))))?;
        }

        Ok(())
    }
}
