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

use std::convert::{TryFrom, TryInto};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{Builder, JoinHandle};

use protobuf::Message;
use transact::protos::IntoBytes;

use crate::consensus::two_phase::TwoPhaseEngine;
use crate::consensus::{
    error::{ConsensusSendError, ProposalManagerError},
    ConsensusEngine, ConsensusMessage, ConsensusNetworkSender, PeerId, Proposal, ProposalId,
    ProposalManager, ProposalUpdate, StartupState,
};
use crate::protos::scabbard::{ProposedBatch, ScabbardMessage, ScabbardMessage_Type};

use super::error::{ScabbardConsensusManagerError, ScabbardError};
use super::shared::ScabbardShared;

/// Component used by the service to manage and interact with consenus
pub struct ScabbardConsensusManager {
    consensus_msg_tx: Sender<ConsensusMessage>,
    proposal_update_tx: Sender<ProposalUpdate>,
    thread_handle: JoinHandle<()>,
}

impl ScabbardConsensusManager {
    /// Create the proposal manager, network sender, and channels used to communicate with
    /// consensus, and start consensus in a separate thread.
    pub fn new(
        service_id: String,
        shared: Arc<Mutex<ScabbardShared>>,
    ) -> Result<Self, ScabbardConsensusManagerError> {
        let peer_ids = shared
            .lock()
            .map_err(|_| ScabbardConsensusManagerError(Box::new(ScabbardError::LockPoisoned)))?
            .peer_services()
            .iter()
            .map(|id| id.as_bytes().into())
            .collect();

        let (consensus_msg_tx, consensus_msg_rx) = channel();
        let (proposal_update_tx, proposal_update_rx) = channel();

        let proposal_manager = ScabbardProposalManager::new(
            service_id.clone(),
            proposal_update_tx.clone(),
            shared.clone(),
        );
        let consensus_network_sender =
            ScabbardConsensusNetworkSender::new(service_id.clone(), shared);
        let startup_state = StartupState {
            id: service_id.as_bytes().into(),
            peer_ids,
            last_proposal: None,
        };

        let thread_handle = Builder::new()
            .name(format!("consensus-{}", service_id))
            .spawn(move || {
                let mut two_phase_engine = TwoPhaseEngine::default();
                if let Err(err) = two_phase_engine.run(
                    consensus_msg_rx,
                    proposal_update_rx,
                    Box::new(consensus_network_sender),
                    Box::new(proposal_manager),
                    startup_state,
                ) {
                    error!("two phase consensus exited with an error: {}", err)
                }
            })
            .map_err(|err| ScabbardConsensusManagerError(Box::new(err)))?;

        Ok(ScabbardConsensusManager {
            consensus_msg_tx,
            proposal_update_tx,
            thread_handle,
        })
    }

    /// Consumes self and shuts down the consensus thread.
    pub fn shutdown(self) -> Result<(), ScabbardConsensusManagerError> {
        self.send_update(ProposalUpdate::Shutdown)?;

        self.thread_handle
            .join()
            .unwrap_or_else(|err| error!("consensus thread failed: {:?}", err));

        Ok(())
    }

    pub fn handle_message(
        &self,
        message_bytes: &[u8],
    ) -> Result<(), ScabbardConsensusManagerError> {
        let consensus_message = ConsensusMessage::try_from(message_bytes)
            .map_err(|err| ScabbardConsensusManagerError(Box::new(err)))?;

        self.consensus_msg_tx
            .send(consensus_message)
            .map_err(|err| ScabbardConsensusManagerError(Box::new(err)))?;

        Ok(())
    }

    pub fn send_update(&self, update: ProposalUpdate) -> Result<(), ScabbardConsensusManagerError> {
        self.proposal_update_tx
            .send(update)
            .map_err(|err| ScabbardConsensusManagerError(Box::new(err)))
    }
}

pub struct ScabbardProposalManager {
    service_id: String,
    proposal_update_sender: Sender<ProposalUpdate>,
    shared: Arc<Mutex<ScabbardShared>>,
}

impl ScabbardProposalManager {
    pub fn new(
        service_id: String,
        proposal_update_sender: Sender<ProposalUpdate>,
        shared: Arc<Mutex<ScabbardShared>>,
    ) -> Self {
        ScabbardProposalManager {
            service_id,
            proposal_update_sender,
            shared,
        }
    }
}

impl ProposalManager for ScabbardProposalManager {
    fn create_proposal(
        &self,
        // Ignoring previous proposal ID and consensus data, because this service and two phase
        // consensus don't care about it.
        _previous_proposal_id: Option<ProposalId>,
        _consensus_data: Vec<u8>,
    ) -> Result<(), ProposalManagerError> {
        let mut shared = self
            .shared
            .lock()
            .map_err(|_| ProposalManagerError::Internal(Box::new(ScabbardError::LockPoisoned)))?;

        if let Some(batch) = shared.pop_batch_from_queue() {
            let expected_hash = shared
                .state_mut()
                .prepare_change(batch.clone())
                .map_err(|err| ProposalManagerError::Internal(Box::new(err)))?;

            // Intentionally leaving out the previous_id and proposal_height fields, since this
            // service and two phase consensus don't use them. This means the proposal ID can just
            // be the summary.
            let mut proposal = Proposal::default();
            proposal.id = expected_hash.as_bytes().into();
            proposal.summary = expected_hash.as_bytes().into();

            shared.add_proposed_batch(proposal.id.clone(), batch.clone());

            // Send the proposal to the other services
            let mut proposed_batch = ProposedBatch::new();
            proposed_batch.set_proposal(
                proposal
                    .clone()
                    .try_into()
                    .map_err(|err| ProposalManagerError::Internal(Box::new(err)))?,
            );
            proposed_batch.set_batch(
                batch
                    .into_bytes()
                    .map_err(|err| ProposalManagerError::Internal(Box::new(err)))?,
            );
            proposed_batch.set_service_id(self.service_id.clone());

            let mut msg = ScabbardMessage::new();
            msg.set_message_type(ScabbardMessage_Type::PROPOSED_BATCH);
            msg.set_proposed_batch(proposed_batch);
            let msg_bytes = msg
                .write_to_bytes()
                .map_err(|err| ProposalManagerError::Internal(Box::new(err)))?;

            let sender = shared
                .network_sender()
                .ok_or(ProposalManagerError::NotReady)?;

            for service in shared.peer_services() {
                sender
                    .send(service, msg_bytes.as_slice())
                    .map_err(|err| ProposalManagerError::Internal(Box::new(err)))?;
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
        let mut shared = self
            .shared
            .lock()
            .map_err(|_| ProposalManagerError::Internal(Box::new(ScabbardError::LockPoisoned)))?;

        let batch = shared
            .get_proposed_batch(id)
            .ok_or_else(|| ProposalManagerError::UnknownProposal(id.clone()))?
            .clone();

        let hash = shared
            .state_mut()
            .prepare_change(batch)
            .map_err(|err| ProposalManagerError::Internal(Box::new(err)))?;
        if hash.as_bytes() != id.as_ref() {
            warn!("Hash mismatch: expected {} but was {}", id, hash);

            self.proposal_update_sender
                .send(ProposalUpdate::ProposalInvalid(id.clone()))?;
        } else {
            self.proposal_update_sender
                .send(ProposalUpdate::ProposalValid(id.clone()))?;
        }

        Ok(())
    }

    fn accept_proposal(
        &self,
        id: &ProposalId,
        // Ignoring consensus data, because this service and two phase consensus don't care about
        // it.
        _consensus_data: Option<Vec<u8>>,
    ) -> Result<(), ProposalManagerError> {
        let mut shared = self
            .shared
            .lock()
            .map_err(|_| ProposalManagerError::Internal(Box::new(ScabbardError::LockPoisoned)))?;

        shared
            .remove_proposed_batch(id)
            .ok_or_else(|| ProposalManagerError::UnknownProposal(id.clone()))?;

        shared
            .state_mut()
            .commit()
            .map_err(|err| ProposalManagerError::Internal(Box::new(err)))?;

        self.proposal_update_sender
            .send(ProposalUpdate::ProposalAccepted(id.clone()))?;

        info!("Committed proposal {}", id);

        Ok(())
    }

    fn reject_proposal(&self, id: &ProposalId) -> Result<(), ProposalManagerError> {
        let mut shared = self
            .shared
            .lock()
            .map_err(|_| ProposalManagerError::Internal(Box::new(ScabbardError::LockPoisoned)))?;

        shared
            .remove_proposed_batch(id)
            .ok_or_else(|| ProposalManagerError::UnknownProposal(id.clone()))?;

        shared
            .state_mut()
            .rollback()
            .map_err(|err| ProposalManagerError::Internal(Box::new(err)))?;

        info!("Rolled back proposal {}", id);

        Ok(())
    }
}

pub struct ScabbardConsensusNetworkSender {
    service_id: String,
    shared: Arc<Mutex<ScabbardShared>>,
}

impl ScabbardConsensusNetworkSender {
    pub fn new(service_id: String, shared: Arc<Mutex<ScabbardShared>>) -> Self {
        ScabbardConsensusNetworkSender { service_id, shared }
    }
}

impl ConsensusNetworkSender for ScabbardConsensusNetworkSender {
    fn send_to(&self, peer_id: &PeerId, message: Vec<u8>) -> Result<(), ConsensusSendError> {
        let peer_id_string = String::from_utf8(peer_id.clone().into())
            .map_err(|err| ConsensusSendError::Internal(Box::new(err)))?;

        let consensus_message = ConsensusMessage::new(message, self.service_id.as_bytes().into());
        let mut msg = ScabbardMessage::new();
        msg.set_message_type(ScabbardMessage_Type::CONSENSUS_MESSAGE);
        msg.set_consensus_message(consensus_message.try_into()?);

        let shared = self
            .shared
            .lock()
            .map_err(|_| ConsensusSendError::Internal(Box::new(ScabbardError::LockPoisoned)))?;

        if !shared.peer_services().contains(&peer_id_string) {
            return Err(ConsensusSendError::UnknownPeer(peer_id.clone()));
        }

        let network_sender = shared
            .network_sender()
            .ok_or(ConsensusSendError::NotReady)?;

        network_sender
            .send(&peer_id_string, msg.write_to_bytes()?.as_slice())
            .map_err(|err| ConsensusSendError::Internal(Box::new(err)))?;

        Ok(())
    }

    fn broadcast(&self, message: Vec<u8>) -> Result<(), ConsensusSendError> {
        let consensus_message = ConsensusMessage::new(message, self.service_id.as_bytes().into());
        let mut msg = ScabbardMessage::new();
        msg.set_message_type(ScabbardMessage_Type::CONSENSUS_MESSAGE);
        msg.set_consensus_message(consensus_message.try_into()?);

        let shared = self
            .shared
            .lock()
            .map_err(|_| ConsensusSendError::Internal(Box::new(ScabbardError::LockPoisoned)))?;

        let network_sender = shared
            .network_sender()
            .ok_or(ConsensusSendError::NotReady)?;

        for service in shared.peer_services() {
            network_sender
                .send(service, msg.write_to_bytes()?.as_slice())
                .map_err(|err| ConsensusSendError::Internal(Box::new(err)))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::{HashSet, VecDeque};
    use std::path::Path;

    use crate::service::scabbard::state::ScabbardState;
    use crate::service::tests::*;
    use crate::signing::hash::HashVerifier;

    /// Tests that the network sender properly creates messages and sends them using the
    /// `ServiceNetworkSender`.
    #[test]
    fn network_sender() {
        let service_sender = MockServiceNetworkSender::new();
        let mut peer_services = HashSet::new();
        peer_services.insert("1".to_string());
        peer_services.insert("2".to_string());

        let shared = Arc::new(Mutex::new(ScabbardShared::new(
            VecDeque::new(),
            Some(Box::new(service_sender.clone())),
            peer_services.clone(),
            Box::new(HashVerifier),
            ScabbardState::new(Path::new("/tmp/network_sender.lmdb"), 1024 * 1024, vec![])
                .expect("failed to create state"),
        )));
        let consensus_sender = ScabbardConsensusNetworkSender::new("0".into(), shared);

        // Test send_to
        consensus_sender
            .send_to(&"1".as_bytes().into(), vec![0])
            .expect("failed to send");

        let (recipient, message) = service_sender
            .sent
            .lock()
            .expect("sent lock poisoned")
            .get(0)
            .expect("1st message not sent")
            .clone();
        assert_eq!(recipient, "1".to_string());

        let scabbard_message: ScabbardMessage =
            protobuf::parse_from_bytes(&message).expect("failed to parse 1st scabbard message");
        assert_eq!(
            scabbard_message.get_message_type(),
            ScabbardMessage_Type::CONSENSUS_MESSAGE
        );

        let consensus_message =
            ConsensusMessage::try_from(scabbard_message.get_consensus_message())
                .expect("failed to parse 1st consensus message");
        assert_eq!(consensus_message.message, vec![0]);
        assert_eq!(consensus_message.origin_id, "0".as_bytes().into());

        // Test broadcast
        consensus_sender.broadcast(vec![1]).expect("failed to send");

        // First broadcast message
        let (recipient, message) = service_sender
            .sent
            .lock()
            .expect("sent lock poisoned")
            .get(1)
            .expect("2nd message not sent")
            .clone();
        assert!(peer_services.remove(&recipient));

        let scabbard_message: ScabbardMessage =
            protobuf::parse_from_bytes(&message).expect("failed to parse 2nd scabbard message");
        assert_eq!(
            scabbard_message.get_message_type(),
            ScabbardMessage_Type::CONSENSUS_MESSAGE
        );

        let consensus_message =
            ConsensusMessage::try_from(scabbard_message.get_consensus_message())
                .expect("failed to parse 2nd consensus message");
        assert_eq!(consensus_message.message, vec![1]);
        assert_eq!(consensus_message.origin_id, "0".as_bytes().into());

        // Second broadcast message
        let (recipient, message) = service_sender
            .sent
            .lock()
            .expect("sent lock poisoned")
            .get(2)
            .expect("3rd message not sent")
            .clone();
        assert!(peer_services.remove(&recipient));

        let scabbard_message: ScabbardMessage =
            protobuf::parse_from_bytes(&message).expect("failed to parse 3rd scabbard message");
        assert_eq!(
            scabbard_message.get_message_type(),
            ScabbardMessage_Type::CONSENSUS_MESSAGE
        );

        let consensus_message =
            ConsensusMessage::try_from(scabbard_message.get_consensus_message())
                .expect("failed to parse 3rd consensus message");
        assert_eq!(consensus_message.message, vec![1]);
        assert_eq!(consensus_message.origin_id, "0".as_bytes().into());
    }
}
