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

use std::collections::VecDeque;
use std::convert::TryInto;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use protobuf::Message;
use uuid::Uuid;

use splinter::consensus::{
    error::{ConsensusSendError, ProposalManagerError},
    ConsensusMessage, ConsensusNetworkSender, PeerId, Proposal, ProposalId, ProposalManager,
    ProposalUpdate,
};
use splinter::network::sender::SendRequest;
use transact::protos::{batch::Batch, FromProto};

use crate::protos::private_xo::{PrivateXoMessage, PrivateXoMessage_Type, ProposedBatch};
use crate::service::error::ServiceError;
use crate::service::{create_circuit_direct_msg, ServiceConfig};
use crate::transaction::XoState;

pub struct PrivateXoProposalManager {
    config: ServiceConfig,
    xo_state: XoState,
    pending_batches: Arc<Mutex<VecDeque<Batch>>>,
    pending_proposal: Arc<Mutex<Option<(Proposal, Batch)>>>,
    proposal_update_sender: Sender<ProposalUpdate>,
    service_sender: crossbeam_channel::Sender<SendRequest>,
}

impl PrivateXoProposalManager {
    pub fn new(
        config: ServiceConfig,
        xo_state: XoState,
        pending_batches: Arc<Mutex<VecDeque<Batch>>>,
        pending_proposal: Arc<Mutex<Option<(Proposal, Batch)>>>,
        proposal_update_sender: Sender<ProposalUpdate>,
        service_sender: crossbeam_channel::Sender<SendRequest>,
    ) -> Self {
        PrivateXoProposalManager {
            config,
            xo_state,
            pending_batches,
            pending_proposal,
            proposal_update_sender,
            service_sender,
        }
    }
}

impl ProposalManager for PrivateXoProposalManager {
    fn create_proposal(
        &self,
        _previous_proposal_id: Option<ProposalId>,
        _consensus_data: Vec<u8>,
    ) -> Result<(), ProposalManagerError> {
        if let Some(batch) = self
            .pending_batches
            .lock()
            .expect("pending batches lock poisoned")
            .pop_front()
        {
            let expected_hash = self
                .xo_state
                .propose_change(
                    transact::protocol::batch::Batch::from_proto(batch.clone())
                        .map_err(|err| ProposalManagerError::Internal(Box::new(err)))?,
                )
                .map_err(|err| ProposalManagerError::Internal(Box::new(err)))?;

            // Cheating a bit here by not setting the ID properly (isn't a hash of previous_id,
            // proposal_height, and summary), but none of this really matters with 2-phase
            // consensus. The ID is the expected hash. This example will not work with forking
            // consensus, because it does not track previously accepted proposals.
            let mut proposal = Proposal::default();
            proposal.id = expected_hash.as_bytes().into();

            *self
                .pending_proposal
                .lock()
                .expect("pending proposal lock poisoned") = Some((proposal.clone(), batch.clone()));

            // Send the proposal to the other services
            let mut proposed_batch = ProposedBatch::new();
            proposed_batch.set_batch(
                batch
                    .write_to_bytes()
                    .map_err(|err| ProposalManagerError::Internal(Box::new(err)))?,
            );
            proposed_batch.set_expected_hash(expected_hash.as_bytes().into());
            let mut msg = PrivateXoMessage::new();
            msg.set_message_type(PrivateXoMessage_Type::PROPOSED_BATCH);
            msg.set_proposed_batch(proposed_batch);

            for verifier in self.config.verifiers() {
                self.service_sender
                    .send(SendRequest::new(
                        self.config.peer_id().into(),
                        create_circuit_direct_msg(
                            self.config.circuit().into(),
                            self.config.service_id().into(),
                            verifier.clone(),
                            &msg,
                            Uuid::new_v4().to_string(),
                        )?,
                    ))
                    .map_err(ServiceError::from)?;
            }

            self.proposal_update_sender
                .send(ProposalUpdate::ProposalCreated(Some(proposal)))?;
        } else {
            self.proposal_update_sender
                .send(ProposalUpdate::ProposalCreated(None))?;
        }

        Ok(())
    }

    fn check_proposal(&self, id: &ProposalId) -> Result<(), ProposalManagerError> {
        let pending_proposal = self
            .pending_proposal
            .lock()
            .expect("pending proposal lock poisoned");

        match *pending_proposal {
            Some((ref proposal, ref batch)) if &proposal.id == id => {
                let hash = self
                    .xo_state
                    .propose_change(
                        transact::protocol::batch::Batch::from_proto(batch.clone())
                            .map_err(|err| ProposalManagerError::Internal(Box::new(err)))?,
                    )
                    .map_err(|err| ProposalManagerError::Internal(Box::new(err)))?;
                // proposal id == expected hash
                if hash.as_bytes() != id.as_ref() {
                    warn!("Hash mismatch: expected {} but was {}", id, hash);

                    self.proposal_update_sender
                        .send(ProposalUpdate::ProposalInvalid(id.clone()))?;
                } else {
                    self.proposal_update_sender
                        .send(ProposalUpdate::ProposalValid(id.clone()))?;
                }
            }
            _ => {
                warn!("checked proposal that isn't pending: {}", id);
                self.proposal_update_sender
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
        let mut pending_proposal = self
            .pending_proposal
            .lock()
            .expect("pending proposal lock poisoned");

        match *pending_proposal {
            Some((ref proposal, _)) if &proposal.id == id => match self.xo_state.commit() {
                Ok(_) => {
                    info!(
                        "Committed proposal {}",
                        pending_proposal.take().unwrap().0.id
                    );
                }
                Err(err) => {
                    self.proposal_update_sender
                        .send(ProposalUpdate::ProposalAcceptFailed(
                            id.clone(),
                            format!("failed to commit proposal: {}", err),
                        ))?
                }
            },
            _ => self
                .proposal_update_sender
                .send(ProposalUpdate::ProposalAcceptFailed(
                    id.clone(),
                    "not pending proposal".into(),
                ))?,
        }

        Ok(())
    }

    fn reject_proposal(&self, id: &ProposalId) -> Result<(), ProposalManagerError> {
        let mut pending_proposal = self
            .pending_proposal
            .lock()
            .expect("pending proposal lock poisoned");

        match *pending_proposal {
            Some((ref proposal, _)) if &proposal.id == id => match self.xo_state.rollback() {
                Ok(_) => {
                    info!(
                        "Rolled back proposal {}",
                        pending_proposal.take().unwrap().0.id
                    );
                }
                Err(err) => {
                    error!("Failed to roll back proposal: {}", err);
                }
            },
            _ => warn!("Rejected proposal that was not pending: {}", id),
        }

        Ok(())
    }
}

pub struct PrivateXoNetworkSender {
    config: ServiceConfig,
    service_sender: crossbeam_channel::Sender<SendRequest>,
}

impl PrivateXoNetworkSender {
    pub fn new(
        config: ServiceConfig,
        service_sender: crossbeam_channel::Sender<SendRequest>,
    ) -> Self {
        PrivateXoNetworkSender {
            config,
            service_sender,
        }
    }
}

impl ConsensusNetworkSender for PrivateXoNetworkSender {
    fn send_to(&self, peer_id: &PeerId, message: Vec<u8>) -> Result<(), ConsensusSendError> {
        let consensus_message =
            ConsensusMessage::new(message, self.config.service_id().as_bytes().into());
        let mut msg = PrivateXoMessage::new();
        msg.set_message_type(PrivateXoMessage_Type::CONSENSUS_MESSAGE);
        msg.set_consensus_message(consensus_message.try_into()?);

        self.service_sender
            .send(SendRequest::new(
                self.config.peer_id().into(),
                create_circuit_direct_msg(
                    self.config.circuit().into(),
                    self.config.service_id().into(),
                    String::from_utf8(peer_id.clone().into())
                        .map_err(|err| ConsensusSendError::Internal(Box::new(err)))?,
                    &msg,
                    Uuid::new_v4().to_string(),
                )
                .map_err(|err| ConsensusSendError::Internal(Box::new(err)))?,
            ))
            .map_err(|err| ConsensusSendError::Internal(Box::new(ServiceError::from(err))))?;

        Ok(())
    }

    fn broadcast(&self, message: Vec<u8>) -> Result<(), ConsensusSendError> {
        let consensus_message =
            ConsensusMessage::new(message, self.config.service_id().as_bytes().into());
        let mut msg = PrivateXoMessage::new();
        msg.set_message_type(PrivateXoMessage_Type::CONSENSUS_MESSAGE);
        msg.set_consensus_message(consensus_message.try_into()?);

        for verifier in self.config.verifiers() {
            self.service_sender
                .send(SendRequest::new(
                    self.config.peer_id().into(),
                    create_circuit_direct_msg(
                        self.config.circuit().into(),
                        self.config.service_id().into(),
                        verifier.clone(),
                        &msg,
                        Uuid::new_v4().to_string(),
                    )
                    .map_err(|err| ConsensusSendError::Internal(Box::new(err)))?,
                ))
                .map_err(|err| ConsensusSendError::Internal(Box::new(ServiceError::from(err))))?;
        }

        Ok(())
    }
}
