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

//! A simple n-party, two-phase consensus algorithm implemented as a `ConsensusEngine`. This is a
//! bully algorithm where there is no established coordinator; instead, whichever peer makes a
//! proposal first is considered the coordinator for the life of that proposal. Only one proposal
//! is considered at a time. A proposal manager can define its own set of required verifiers by
//! setting this information in the consensus data.

use std::collections::HashSet;
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::Duration;

use protobuf::Message;

use crate::consensus::{
    ConsensusEngine, ConsensusEngineError, ConsensusMessage, ConsensusNetworkSender, PeerId,
    ProposalId, ProposalManager, ProposalUpdate, StartupState,
};
use crate::protos::two_phase::{
    RequiredVerifiers, TwoPhaseMessage, TwoPhaseMessage_ProposalResult,
    TwoPhaseMessage_ProposalVerificationResponse, TwoPhaseMessage_Type,
};

#[derive(Debug)]
enum State {
    Idle,
    AwaitingProposal,
    EvaluatingProposal(ProposalStatus),
}

#[derive(Debug)]
struct ProposalStatus {
    proposal_id: ProposalId,
    proposer_id: PeerId,
    peers_verified: HashSet<PeerId>,
    required_verifiers: HashSet<PeerId>,
}

impl ProposalStatus {
    fn new(
        proposal_id: ProposalId,
        proposer_id: PeerId,
        required_verifiers: HashSet<PeerId>,
    ) -> Self {
        ProposalStatus {
            proposal_id,
            proposer_id,
            peers_verified: HashSet::new(),
            required_verifiers,
        }
    }

    fn proposal_id(&self) -> &ProposalId {
        &self.proposal_id
    }

    fn proposer_id(&self) -> &PeerId {
        &self.proposer_id
    }

    fn peers_verified(&self) -> &HashSet<PeerId> {
        &self.peers_verified
    }

    fn required_verifiers(&self) -> &HashSet<PeerId> {
        &self.required_verifiers
    }

    fn add_verified_peer(&mut self, id: PeerId) {
        self.peers_verified.insert(id);
    }
}

pub struct TwoPhaseEngine {
    id: PeerId,
    peers: HashSet<PeerId>,
    state: State,
    verification_request_backlog: HashSet<ProposalId>,
}

impl Default for TwoPhaseEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl TwoPhaseEngine {
    pub fn new() -> Self {
        TwoPhaseEngine {
            id: PeerId::default(),
            peers: HashSet::new(),
            state: State::Idle,
            verification_request_backlog: HashSet::new(),
        }
    }

    fn handle_consensus_msg(
        &mut self,
        consensus_msg: ConsensusMessage,
        network_sender: &dyn ConsensusNetworkSender,
        proposal_manager: &dyn ProposalManager,
    ) -> Result<(), ConsensusEngineError> {
        let two_phase_msg: TwoPhaseMessage = protobuf::parse_from_bytes(&consensus_msg.message)?;
        let proposal_id = ProposalId::from(two_phase_msg.get_proposal_id());

        // Ignore any messages that aren't for the current proposal (except for verification
        // requests, which are backlogged)
        if !self.evaluating_proposal(&proposal_id) {
            match two_phase_msg.get_message_type() {
                TwoPhaseMessage_Type::PROPOSAL_VERIFICATION_REQUEST => {
                    debug!(
                        "backlogging verification request for unknown proposal: {}",
                        proposal_id,
                    );
                    // Note: this is a potential leak, because requests don't get removed unless
                    // the proposal is actually evaluated at some point in the future.
                    self.verification_request_backlog.insert(proposal_id);
                }
                _ => warn!(
                    "ignoring message for proposal that is not being evaluated: {}",
                    proposal_id
                ),
            }

            return Ok(());
        }

        match two_phase_msg.get_message_type() {
            TwoPhaseMessage_Type::PROPOSAL_VERIFICATION_REQUEST => {
                debug!("Verifying proposal {}", proposal_id);
                proposal_manager.check_proposal(&proposal_id)?;
            }
            TwoPhaseMessage_Type::PROPOSAL_VERIFICATION_RESPONSE => {
                match two_phase_msg.get_proposal_verification_response() {
                    TwoPhaseMessage_ProposalVerificationResponse::VERIFIED => {
                        debug!(
                            "Proposal {} verified by peer {}",
                            proposal_id, consensus_msg.origin_id
                        );
                        if let State::EvaluatingProposal(proposal_status) = &mut self.state {
                            proposal_status.add_verified_peer(consensus_msg.origin_id);

                            if proposal_status.peers_verified()
                                == proposal_status.required_verifiers()
                            {
                                let mut result = TwoPhaseMessage::new();
                                result.set_message_type(TwoPhaseMessage_Type::PROPOSAL_RESULT);
                                result.set_proposal_id(proposal_id.clone().into());
                                result.set_proposal_result(TwoPhaseMessage_ProposalResult::APPLY);

                                network_sender.broadcast(result.write_to_bytes()?)?;

                                proposal_manager.accept_proposal(&proposal_id, None)?;
                                self.state = State::Idle;
                            }
                        } else {
                            // self.evaluating_proposal(), which is called above, checks that the
                            // state is EvaluatingProposal and the current proposal matches the one
                            // this message is for.
                            panic!("Already checked proposal being verified");
                        }
                    }
                    TwoPhaseMessage_ProposalVerificationResponse::FAILED => {
                        debug!(
                            "Proposal {} failed by peer {}",
                            proposal_id, consensus_msg.origin_id
                        );
                        let mut result = TwoPhaseMessage::new();
                        result.set_message_type(TwoPhaseMessage_Type::PROPOSAL_RESULT);
                        result.set_proposal_id(proposal_id.clone().into());
                        result.set_proposal_result(TwoPhaseMessage_ProposalResult::REJECT);

                        network_sender.broadcast(result.write_to_bytes()?)?;

                        proposal_manager.reject_proposal(&proposal_id)?;
                        self.state = State::Idle;
                    }
                    TwoPhaseMessage_ProposalVerificationResponse::UNSET_VERIFICATION_RESPONSE => {
                        warn!(
                            "ignoring improperly specified proposal verification response from {}",
                            consensus_msg.origin_id
                        )
                    }
                }
            }
            TwoPhaseMessage_Type::PROPOSAL_RESULT => match two_phase_msg.get_proposal_result() {
                TwoPhaseMessage_ProposalResult::APPLY => {
                    debug!("accepting proposal {}", proposal_id);
                    proposal_manager.accept_proposal(&proposal_id, None)?;
                    self.state = State::Idle;
                }
                TwoPhaseMessage_ProposalResult::REJECT => {
                    debug!("rejecting proposal {}", proposal_id);
                    proposal_manager.reject_proposal(&proposal_id)?;
                    self.state = State::Idle;
                }
                TwoPhaseMessage_ProposalResult::UNSET_RESULT => warn!(
                    "ignoring improperly specified proposal result from {}",
                    consensus_msg.origin_id
                ),
            },
            TwoPhaseMessage_Type::UNSET_TYPE => warn!(
                "ignoring improperly specified two-phase message from {}",
                consensus_msg.origin_id
            ),
        }

        Ok(())
    }

    fn handle_proposal_update(
        &mut self,
        update: ProposalUpdate,
        network_sender: &dyn ConsensusNetworkSender,
        proposal_manager: &dyn ProposalManager,
    ) -> Result<(), ConsensusEngineError> {
        match update {
            ProposalUpdate::ProposalCreated(Some(proposal)) => {
                if let State::AwaitingProposal = self.state {
                    debug!("Proposal created: {}", proposal.id);
                    let mut verifiers: HashSet<PeerId> = HashSet::new();

                    if !proposal.consensus_data.is_empty() {
                        let mut required_verifiers: RequiredVerifiers =
                            protobuf::parse_from_bytes(&proposal.consensus_data)?;

                        for id in required_verifiers.take_verifiers().to_vec() {
                            verifiers.insert(id.into());
                        }
                    } else {
                        verifiers = self.peers.clone();
                    }

                    self.state = State::EvaluatingProposal(ProposalStatus::new(
                        proposal.id.clone(),
                        self.id.clone(),
                        verifiers,
                    ));

                    let mut request = TwoPhaseMessage::new();
                    request.set_message_type(TwoPhaseMessage_Type::PROPOSAL_VERIFICATION_REQUEST);
                    request.set_proposal_id(proposal.id.into());

                    network_sender.broadcast(request.write_to_bytes()?)?;
                } else if !self.evaluating_proposal(&proposal.id) {
                    warn!("Received proposal creation, but not awaiting one");
                    proposal_manager.reject_proposal(&proposal.id)?;
                }
            }
            ProposalUpdate::ProposalCreated(None) => {
                if let State::AwaitingProposal = self.state {
                    self.state = State::Idle;
                }
            }
            ProposalUpdate::ProposalReceived(proposal, peer_id) => match self.state {
                State::EvaluatingProposal(ref proposal_status) => {
                    if proposal_status.proposal_id() != &proposal.id {
                        warn!(
                            "Rejecting proposal {} because another ({}) is currently being \
                             evaluated",
                            proposal.id,
                            proposal_status.proposal_id(),
                        );
                        proposal_manager.reject_proposal(&proposal.id)?;
                    }
                }
                _ => {
                    debug!("Proposal received: {}", proposal.id);
                    let mut verifiers: HashSet<PeerId> = HashSet::new();
                    if !proposal.consensus_data.is_empty() {
                        let mut required_verifiers: RequiredVerifiers =
                            protobuf::parse_from_bytes(&proposal.consensus_data)?;

                        for id in required_verifiers.take_verifiers().to_vec() {
                            verifiers.insert(id.into());
                        }
                    } else {
                        verifiers = self.peers.clone();
                    }
                    self.state = State::EvaluatingProposal(ProposalStatus::new(
                        proposal.id,
                        peer_id,
                        verifiers,
                    ));
                }
            },
            ProposalUpdate::ProposalValid(proposal_id) => match self.state {
                State::EvaluatingProposal(ref proposal_status)
                    if proposal_status.proposal_id() == &proposal_id =>
                {
                    debug!(
                        "Received valid proposal message for proposal {}",
                        proposal_id
                    );
                    let mut response = TwoPhaseMessage::new();
                    response.set_message_type(TwoPhaseMessage_Type::PROPOSAL_VERIFICATION_RESPONSE);
                    response.set_proposal_id(proposal_id.into());
                    response.set_proposal_verification_response(
                        TwoPhaseMessage_ProposalVerificationResponse::VERIFIED,
                    );

                    network_sender
                        .send_to(proposal_status.proposer_id(), response.write_to_bytes()?)?;
                }
                _ => warn!("Got valid message for unknown proposal: {}", proposal_id),
            },
            ProposalUpdate::ProposalInvalid(proposal_id) => match self.state {
                State::EvaluatingProposal(ref proposal_status)
                    if proposal_status.proposal_id() == &proposal_id =>
                {
                    debug!(
                        "Received invalid proposal message for proposal {}",
                        proposal_id
                    );
                    let mut response = TwoPhaseMessage::new();
                    response.set_message_type(TwoPhaseMessage_Type::PROPOSAL_VERIFICATION_RESPONSE);
                    response.set_proposal_id(proposal_id.into());
                    response.set_proposal_verification_response(
                        TwoPhaseMessage_ProposalVerificationResponse::FAILED,
                    );

                    network_sender
                        .send_to(proposal_status.proposer_id(), response.write_to_bytes()?)?;
                }
                _ => warn!("Got invalid message for unknown proposal: {}", proposal_id),
            },
            ProposalUpdate::ProposalAccepted(proposal_id) => {
                info!("proposal accepted: {}", proposal_id);
            }
            ProposalUpdate::ProposalAcceptFailed(proposal_id, err) => {
                error!(
                    "failed to accept proposal {} due to error: {}",
                    proposal_id, err
                );
            }
            other => {
                debug!("ignoring update: {:?}", other);
            }
        }

        Ok(())
    }

    fn evaluating_proposal(&self, proposal_id: &ProposalId) -> bool {
        match self.state {
            State::EvaluatingProposal(ref proposal_status)
                if proposal_status.proposal_id() == proposal_id =>
            {
                true
            }
            _ => false,
        }
    }
}

impl ConsensusEngine for TwoPhaseEngine {
    fn name(&self) -> &str {
        "two-phase"
    }

    fn version(&self) -> &str {
        "0.1"
    }

    fn additional_protocols(&self) -> Vec<(String, String)> {
        vec![]
    }

    fn run(
        &mut self,
        consensus_messages: Receiver<ConsensusMessage>,
        proposal_updates: Receiver<ProposalUpdate>,
        network_sender: Box<ConsensusNetworkSender>,
        proposal_manager: Box<ProposalManager>,
        startup_state: StartupState,
    ) -> Result<(), ConsensusEngineError> {
        let message_timeout = Duration::from_millis(100);
        let proposal_timeout = Duration::from_millis(100);

        self.id = startup_state.id;

        for id in startup_state.peer_ids {
            self.peers.insert(id);
        }

        loop {
            // If not doing anything, try to get the next proposal
            if let State::Idle = self.state {
                match proposal_manager.create_proposal(None, vec![]) {
                    Ok(()) => self.state = State::AwaitingProposal,
                    Err(err) => error!("error while creating proposal: {}", err),
                }
            }

            // If evaluating a proposal whose verification request has already been received, check
            // the validity of the proposal
            if let State::EvaluatingProposal(ref proposal_status) = self.state {
                if self
                    .verification_request_backlog
                    .remove(proposal_status.proposal_id())
                {
                    debug!(
                        "verifying proposal from backlog: {}",
                        proposal_status.proposal_id()
                    );
                    if let Err(err) = proposal_manager.check_proposal(proposal_status.proposal_id())
                    {
                        error!("failed to check backlogged proposal: {}", err);
                    }
                }
            }

            // Get and handle a consensus message if there is one
            match consensus_messages.recv_timeout(message_timeout) {
                Ok(consensus_message) => {
                    if let Err(err) = self.handle_consensus_msg(
                        consensus_message,
                        &*network_sender,
                        &*proposal_manager,
                    ) {
                        error!("error while handling consensus message: {}", err);
                    }
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => {
                    info!("consensus message receiver disconnected");
                    break;
                }
            }

            // Get and handle a proposal update if there is one
            match proposal_updates.recv_timeout(proposal_timeout) {
                Ok(ProposalUpdate::Shutdown) => {
                    info!("received shutdown");
                    break;
                }
                Ok(update) => {
                    if let Err(err) =
                        self.handle_proposal_update(update, &*network_sender, &*proposal_manager)
                    {
                        error!("error while handling proposal update: {}", err);
                    }
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => {
                    info!("proposal update receiver disconnected");
                    break;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use std::sync::mpsc::channel;

    use protobuf::RepeatedField;

    use crate::consensus::tests::{MockConsensusNetworkSender, MockProposalManager};
    use crate::consensus::Proposal;

    /// Verify that the engine properly shuts down when it receives the Shutdown update.
    #[test]
    fn test_shutdown() {
        let (update_tx, update_rx) = channel();
        let (_, consensus_msg_rx) = channel();

        let manager = MockProposalManager::new(update_tx.clone());
        let network = MockConsensusNetworkSender::new();
        let startup_state = StartupState {
            id: vec![0].into(),
            peer_ids: vec![vec![1].into()],
            last_proposal: None,
        };

        let mut engine = TwoPhaseEngine::new();
        let thread = std::thread::spawn(move || {
            engine
                .run(
                    consensus_msg_rx,
                    update_rx,
                    Box::new(network),
                    Box::new(manager),
                    startup_state,
                )
                .expect("engine failed")
        });

        update_tx
            .send(ProposalUpdate::Shutdown)
            .expect("failed to send shutdown");
        thread.join().expect("failed to join engine thread");
    }

    /// Test the coordinator (leader) of a 3 node network by simulating the flow of a valid
    /// proposal (both participants verify the proposal) and a failed proposal (one participant
    /// fails the proposal). This test uses default peers for which nodes need to verify the
    /// proposal. The peers are defined in the startup state.
    #[test]
    fn test_coordinator_default_peers() {
        let (update_tx, update_rx) = channel();
        let (consensus_msg_tx, consensus_msg_rx) = channel();

        let manager = MockProposalManager::new(update_tx.clone());
        let network = MockConsensusNetworkSender::new();
        let startup_state = StartupState {
            id: vec![0].into(),
            peer_ids: vec![vec![1].into(), vec![2].into()],
            last_proposal: None,
        };

        let mut engine = TwoPhaseEngine::new();
        let network_clone = network.clone();
        let manager_clone = manager.clone();
        let thread = std::thread::spawn(move || {
            engine
                .run(
                    consensus_msg_rx,
                    update_rx,
                    Box::new(network_clone),
                    Box::new(manager_clone),
                    startup_state,
                )
                .expect("engine failed")
        });

        // Check that verification request is sent for the first proposal
        loop {
            if let Some(msg) = network.broadcast_messages().get(0) {
                let msg: TwoPhaseMessage =
                    protobuf::parse_from_bytes(msg).expect("failed to parse message");
                assert_eq!(
                    msg.get_message_type(),
                    TwoPhaseMessage_Type::PROPOSAL_VERIFICATION_REQUEST
                );
                assert_eq!(msg.get_proposal_id(), vec![1].as_slice());
                break;
            }
        }

        // Receive the verification responses
        let mut response = TwoPhaseMessage::new();
        response.set_message_type(TwoPhaseMessage_Type::PROPOSAL_VERIFICATION_RESPONSE);
        response.set_proposal_id(vec![1]);
        response.set_proposal_verification_response(
            TwoPhaseMessage_ProposalVerificationResponse::VERIFIED,
        );
        let message_bytes = response
            .write_to_bytes()
            .expect("failed to write failed response to bytes");

        consensus_msg_tx
            .send(ConsensusMessage::new(message_bytes.clone(), vec![1].into()))
            .expect("failed to send 1st response");
        consensus_msg_tx
            .send(ConsensusMessage::new(message_bytes, vec![2].into()))
            .expect("failed to send 2nd response");

        // Verify the Apply message is sent for the proposal
        loop {
            if let Some(msg) = network.broadcast_messages().get(1) {
                let msg: TwoPhaseMessage =
                    protobuf::parse_from_bytes(msg).expect("failed to parse message");
                assert_eq!(
                    msg.get_message_type(),
                    TwoPhaseMessage_Type::PROPOSAL_RESULT
                );
                assert_eq!(
                    msg.get_proposal_result(),
                    TwoPhaseMessage_ProposalResult::APPLY
                );
                assert_eq!(msg.get_proposal_id(), vec![1].as_slice());
                break;
            }
        }

        // Verify the proposal was accepted
        loop {
            if let Some((id, _)) = manager.accepted_proposals().get(0) {
                assert_eq!(id, &vec![1].into());
                break;
            }
        }

        // Check that verification request is sent for the second proposal
        loop {
            if let Some(msg) = network.broadcast_messages().get(2) {
                let msg: TwoPhaseMessage =
                    protobuf::parse_from_bytes(msg).expect("failed to parse message");
                assert_eq!(
                    msg.get_message_type(),
                    TwoPhaseMessage_Type::PROPOSAL_VERIFICATION_REQUEST
                );
                assert_eq!(msg.get_proposal_id(), vec![2].as_slice());
                break;
            }
        }

        // Receive the verification responses
        let mut response = TwoPhaseMessage::new();
        response.set_message_type(TwoPhaseMessage_Type::PROPOSAL_VERIFICATION_RESPONSE);
        response.set_proposal_id(vec![2]);
        response.set_proposal_verification_response(
            TwoPhaseMessage_ProposalVerificationResponse::VERIFIED,
        );
        let message_bytes = response
            .write_to_bytes()
            .expect("failed to write failed response to bytes");

        consensus_msg_tx
            .send(ConsensusMessage::new(message_bytes, vec![1].into()))
            .expect("failed to send 1st response");

        let mut response = TwoPhaseMessage::new();
        response.set_message_type(TwoPhaseMessage_Type::PROPOSAL_VERIFICATION_RESPONSE);
        response.set_proposal_id(vec![2]);
        response.set_proposal_verification_response(
            TwoPhaseMessage_ProposalVerificationResponse::FAILED,
        );
        let message_bytes = response
            .write_to_bytes()
            .expect("failed to write failed response to bytes");

        consensus_msg_tx
            .send(ConsensusMessage::new(message_bytes, vec![2].into()))
            .expect("failed to send 2nd response");

        // Verify the Reject message is sent for the proposal
        loop {
            if let Some(msg) = network.broadcast_messages().get(3) {
                let msg: TwoPhaseMessage =
                    protobuf::parse_from_bytes(msg).expect("failed to parse message");
                assert_eq!(
                    msg.get_message_type(),
                    TwoPhaseMessage_Type::PROPOSAL_RESULT
                );
                assert_eq!(
                    msg.get_proposal_result(),
                    TwoPhaseMessage_ProposalResult::REJECT
                );
                assert_eq!(msg.get_proposal_id(), vec![2].as_slice());
                break;
            }
        }

        // Verify the proposal was rejected
        loop {
            if let Some(id) = manager.rejected_proposals().get(0) {
                assert_eq!(id, &vec![2].into());
                break;
            }
        }

        update_tx
            .send(ProposalUpdate::Shutdown)
            .expect("failed to send shutdown");
        thread.join().expect("failed to join engine thread");
    }

    /// Test the coordinator (leader) of a 3 node network by simulating the flow of a valid
    /// proposal (both participants verify the proposal) and a failed proposal (one participant
    /// fails the proposal). This test uses dynamic peers for which nodes need to verify the
    /// proposal. The peers are defined in the consensus data on the proposal.
    #[test]
    fn test_coordinator_dynamic_peers() {
        let (update_tx, update_rx) = channel();
        let (consensus_msg_tx, consensus_msg_rx) = channel();

        let mut manager = MockProposalManager::new(update_tx.clone());
        let network = MockConsensusNetworkSender::new();

        let mut required_verifiers = RequiredVerifiers::new();
        required_verifiers.set_verifiers(RepeatedField::from_vec(vec![
            vec![1].into(),
            vec![2].into(),
        ]));
        let data = required_verifiers.write_to_bytes().unwrap();
        manager.set_consensus_data(Some(data));

        let startup_state = StartupState {
            id: vec![0].into(),
            peer_ids: vec![],
            last_proposal: None,
        };

        let mut engine = TwoPhaseEngine::new();
        let network_clone = network.clone();
        let manager_clone = manager.clone();
        let thread = std::thread::spawn(move || {
            engine
                .run(
                    consensus_msg_rx,
                    update_rx,
                    Box::new(network_clone),
                    Box::new(manager_clone),
                    startup_state,
                )
                .expect("engine failed")
        });

        // Check that verification request is sent for the first proposal
        loop {
            if let Some(msg) = network.broadcast_messages().get(0) {
                let msg: TwoPhaseMessage =
                    protobuf::parse_from_bytes(msg).expect("failed to parse message");
                assert_eq!(
                    msg.get_message_type(),
                    TwoPhaseMessage_Type::PROPOSAL_VERIFICATION_REQUEST
                );
                assert_eq!(msg.get_proposal_id(), vec![1].as_slice());
                break;
            }
        }

        // Receive the verification responses
        let mut response = TwoPhaseMessage::new();
        response.set_message_type(TwoPhaseMessage_Type::PROPOSAL_VERIFICATION_RESPONSE);
        response.set_proposal_id(vec![1]);
        response.set_proposal_verification_response(
            TwoPhaseMessage_ProposalVerificationResponse::VERIFIED,
        );
        let message_bytes = response
            .write_to_bytes()
            .expect("failed to write failed response to bytes");

        consensus_msg_tx
            .send(ConsensusMessage::new(message_bytes.clone(), vec![1].into()))
            .expect("failed to send 1st response");
        consensus_msg_tx
            .send(ConsensusMessage::new(message_bytes, vec![2].into()))
            .expect("failed to send 2nd response");

        // Verify the Apply message is sent for the proposal
        loop {
            if let Some(msg) = network.broadcast_messages().get(1) {
                let msg: TwoPhaseMessage =
                    protobuf::parse_from_bytes(msg).expect("failed to parse message");
                assert_eq!(
                    msg.get_message_type(),
                    TwoPhaseMessage_Type::PROPOSAL_RESULT
                );
                assert_eq!(
                    msg.get_proposal_result(),
                    TwoPhaseMessage_ProposalResult::APPLY
                );
                assert_eq!(msg.get_proposal_id(), vec![1].as_slice());
                break;
            }
        }

        // Verify the proposal was accepted
        loop {
            if let Some((id, _)) = manager.accepted_proposals().get(0) {
                assert_eq!(id, &vec![1].into());
                break;
            }
        }

        // Check that verification request is sent for the second proposal
        loop {
            if let Some(msg) = network.broadcast_messages().get(2) {
                let msg: TwoPhaseMessage =
                    protobuf::parse_from_bytes(msg).expect("failed to parse message");
                assert_eq!(
                    msg.get_message_type(),
                    TwoPhaseMessage_Type::PROPOSAL_VERIFICATION_REQUEST
                );
                assert_eq!(msg.get_proposal_id(), vec![2].as_slice());
                break;
            }
        }

        // Receive the verification responses
        let mut response = TwoPhaseMessage::new();
        response.set_message_type(TwoPhaseMessage_Type::PROPOSAL_VERIFICATION_RESPONSE);
        response.set_proposal_id(vec![2]);
        response.set_proposal_verification_response(
            TwoPhaseMessage_ProposalVerificationResponse::VERIFIED,
        );
        let message_bytes = response
            .write_to_bytes()
            .expect("failed to write failed response to bytes");

        consensus_msg_tx
            .send(ConsensusMessage::new(message_bytes, vec![1].into()))
            .expect("failed to send 1st response");

        let mut response = TwoPhaseMessage::new();
        response.set_message_type(TwoPhaseMessage_Type::PROPOSAL_VERIFICATION_RESPONSE);
        response.set_proposal_id(vec![2]);
        response.set_proposal_verification_response(
            TwoPhaseMessage_ProposalVerificationResponse::FAILED,
        );
        let message_bytes = response
            .write_to_bytes()
            .expect("failed to write failed response to bytes");

        consensus_msg_tx
            .send(ConsensusMessage::new(message_bytes, vec![2].into()))
            .expect("failed to send 2nd response");

        // Verify the Reject message is sent for the proposal
        loop {
            if let Some(msg) = network.broadcast_messages().get(3) {
                let msg: TwoPhaseMessage =
                    protobuf::parse_from_bytes(msg).expect("failed to parse message");
                assert_eq!(
                    msg.get_message_type(),
                    TwoPhaseMessage_Type::PROPOSAL_RESULT
                );
                assert_eq!(
                    msg.get_proposal_result(),
                    TwoPhaseMessage_ProposalResult::REJECT
                );
                assert_eq!(msg.get_proposal_id(), vec![2].as_slice());
                break;
            }
        }

        // Verify the proposal was rejected
        loop {
            if let Some(id) = manager.rejected_proposals().get(0) {
                assert_eq!(id, &vec![2].into());
                break;
            }
        }

        update_tx
            .send(ProposalUpdate::Shutdown)
            .expect("failed to send shutdown");
        thread.join().expect("failed to join engine thread");
    }

    /// Test a participant (follower) by simulating the flow of a valid and a failed proposal.
    #[test]
    fn test_participant() {
        let (update_tx, update_rx) = channel();
        let (consensus_msg_tx, consensus_msg_rx) = channel();

        let manager = MockProposalManager::new(update_tx.clone());
        manager.set_return_proposal(false);
        let network = MockConsensusNetworkSender::new();
        let startup_state = StartupState {
            id: vec![1].into(),
            peer_ids: vec![vec![0].into()],
            last_proposal: None,
        };

        let mut engine = TwoPhaseEngine::new();
        let network_clone = network.clone();
        let manager_clone = manager.clone();
        let thread = std::thread::spawn(move || {
            engine
                .run(
                    consensus_msg_rx,
                    update_rx,
                    Box::new(network_clone),
                    Box::new(manager_clone),
                    startup_state,
                )
                .expect("engine failed")
        });

        // Receive the first proposal
        let mut proposal = Proposal::default();
        proposal.id = vec![1].into();
        update_tx
            .send(ProposalUpdate::ProposalReceived(proposal, vec![0].into()))
            .expect("failed to send 1st proposal");

        // Receive the first verification request
        let mut request = TwoPhaseMessage::new();
        request.set_message_type(TwoPhaseMessage_Type::PROPOSAL_VERIFICATION_REQUEST);
        request.set_proposal_id(vec![1]);
        let message_bytes = request
            .write_to_bytes()
            .expect("failed to write request to bytes");

        consensus_msg_tx
            .send(ConsensusMessage::new(message_bytes, vec![0].into()))
            .expect("failed to send 1st verification request");

        // Check that the Verified verification response is sent
        loop {
            if let Some((msg, peer_id)) = network.sent_messages().get(0) {
                let msg: TwoPhaseMessage =
                    protobuf::parse_from_bytes(msg).expect("failed to parse message");
                assert_eq!(peer_id, &vec![0].into());
                assert_eq!(
                    msg.get_message_type(),
                    TwoPhaseMessage_Type::PROPOSAL_VERIFICATION_RESPONSE
                );
                assert_eq!(
                    msg.get_proposal_verification_response(),
                    TwoPhaseMessage_ProposalVerificationResponse::VERIFIED
                );
                assert_eq!(msg.get_proposal_id(), vec![1].as_slice());
                break;
            }
        }

        // Receive the Apply result
        let mut result = TwoPhaseMessage::new();
        result.set_message_type(TwoPhaseMessage_Type::PROPOSAL_RESULT);
        result.set_proposal_id(vec![1]);
        result.set_proposal_result(TwoPhaseMessage_ProposalResult::APPLY);
        let message_bytes = result
            .write_to_bytes()
            .expect("failed to write apply result to bytes");

        consensus_msg_tx
            .send(ConsensusMessage::new(message_bytes, vec![0].into()))
            .expect("failed to send apply result");

        // Verify the proposal was accepted
        loop {
            if let Some((id, _)) = manager.accepted_proposals().get(0) {
                assert_eq!(id, &vec![1].into());
                break;
            }
        }

        // Receive the second proposal
        let mut proposal = Proposal::default();
        proposal.id = vec![2].into();
        update_tx
            .send(ProposalUpdate::ProposalReceived(proposal, vec![0].into()))
            .expect("failed to send 2nd proposal");

        // Receive the second verification request (the manager will say this proposal is invalid)
        manager.set_next_proposal_valid(false);

        let mut request = TwoPhaseMessage::new();
        request.set_message_type(TwoPhaseMessage_Type::PROPOSAL_VERIFICATION_REQUEST);
        request.set_proposal_id(vec![2]);
        let message_bytes = request
            .write_to_bytes()
            .expect("failed to write request to bytes");

        consensus_msg_tx
            .send(ConsensusMessage::new(message_bytes, vec![0].into()))
            .expect("failed to send 2nd verification request");

        // Check that the Failed verification response is sent
        loop {
            if let Some((msg, peer_id)) = network.sent_messages().get(1) {
                let msg: TwoPhaseMessage =
                    protobuf::parse_from_bytes(msg).expect("failed to parse message");
                assert_eq!(peer_id, &vec![0].into());
                assert_eq!(
                    msg.get_message_type(),
                    TwoPhaseMessage_Type::PROPOSAL_VERIFICATION_RESPONSE
                );
                assert_eq!(
                    msg.get_proposal_verification_response(),
                    TwoPhaseMessage_ProposalVerificationResponse::FAILED
                );
                assert_eq!(msg.get_proposal_id(), vec![2].as_slice());
                break;
            }
        }

        // Receive the Reject result
        let mut result = TwoPhaseMessage::new();
        result.set_message_type(TwoPhaseMessage_Type::PROPOSAL_RESULT);
        result.set_proposal_id(vec![2]);
        result.set_proposal_result(TwoPhaseMessage_ProposalResult::REJECT);
        let message_bytes = result
            .write_to_bytes()
            .expect("failed to write reject result to bytes");

        consensus_msg_tx
            .send(ConsensusMessage::new(message_bytes, vec![0].into()))
            .expect("failed to send reject result");

        // Verify the proposal was rejected
        loop {
            if let Some(id) = manager.rejected_proposals().get(0) {
                assert_eq!(id, &vec![2].into());
                break;
            }
        }

        update_tx
            .send(ProposalUpdate::Shutdown)
            .expect("failed to send shutdown");
        thread.join().expect("failed to join engine thread");
    }
}
