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

//! A simple n-party, two-phase commit (2PC) consensus algorithm implemented as a
//! `ConsensusEngine`. This is a bully algorithm where the coordinator for a proposal is determined
//! as the node with the lowest ID in the set of verifiers. Only one proposal is considered at a
//! time. A proposal manager can define its own set of required verifiers by setting this
//! information in the consensus data.
//!
//! # Known limitations of this 2PC implementation
//!
//! There is a potential race condition in two-phase commit where two different proposals are in
//! flight:
//!
//! - The two proposals have different coordinators
//! - Both proposals have two or more verifiers in common
//! - One of the common verifiers evaluates the 1st proposal; the other evaluates the 2nd proposal
//! - Neither proposal will be completed, since only a single proposal can be evaluated by a
//!   verifier at a time
//!
//! The solution to this limitation would require 2PC to have more sophisticated knowledge about
//! the proposals available to it, and be able to process multiple non-overlapping proposals at the
//! same time.
//!
//! Another limitation of this implementation is that it is not fully resilient to crashes; for
//! instance, if the coordinator commits a proposal but crashes before it is able to send the
//! `APPLY` message to the other nodes, the network will be out of sync because the coordinator
//! does not know to send the message when it restarts. This limitation will be solved by
//! re-implementing 2PC as a stateless algorithm.

mod timing;

use std::collections::{HashSet, VecDeque};
use std::iter::FromIterator;
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::Duration;

use protobuf::Message;

use crate::consensus::{
    ConsensusEngine, ConsensusEngineError, ConsensusMessage, ConsensusNetworkSender, PeerId,
    Proposal, ProposalId, ProposalManager, ProposalUpdate, StartupState,
};
use crate::protos::two_phase::{
    RequiredVerifiers, TwoPhaseMessage, TwoPhaseMessage_ProposalResult,
    TwoPhaseMessage_ProposalVerificationResponse, TwoPhaseMessage_Type,
};

use self::timing::Timeout;

const MESSAGE_RECV_TIMEOUT_MILLIS: u64 = 100;
const PROPOSAL_RECV_TIMEOUT_MILLIS: u64 = 100;

#[derive(Debug)]
enum State {
    Idle,
    AwaitingProposal,
    EvaluatingProposal(TwoPhaseProposal),
}

/// Contains information about a proposal that two phase consensus needs to keep track of
#[derive(Debug)]
struct TwoPhaseProposal {
    proposal_id: ProposalId,
    coordinator_id: PeerId,
    peers_verified: HashSet<PeerId>,
    required_verifiers: HashSet<PeerId>,
}

impl TwoPhaseProposal {
    fn new(
        proposal_id: ProposalId,
        coordinator_id: PeerId,
        required_verifiers: HashSet<PeerId>,
    ) -> Self {
        TwoPhaseProposal {
            proposal_id,
            coordinator_id,
            peers_verified: HashSet::new(),
            required_verifiers,
        }
    }

    fn proposal_id(&self) -> &ProposalId {
        &self.proposal_id
    }

    fn coordinator_id(&self) -> &PeerId {
        &self.coordinator_id
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
    coordinator_timeout: Timeout,
    proposal_backlog: VecDeque<TwoPhaseProposal>,
    verification_request_backlog: VecDeque<ProposalId>,
}

impl TwoPhaseEngine {
    pub fn new(coordinator_timeout_duration: Duration) -> Self {
        TwoPhaseEngine {
            id: PeerId::default(),
            peers: HashSet::new(),
            state: State::Idle,
            coordinator_timeout: Timeout::new(coordinator_timeout_duration),
            proposal_backlog: VecDeque::new(),
            verification_request_backlog: VecDeque::new(),
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

        match two_phase_msg.get_message_type() {
            TwoPhaseMessage_Type::PROPOSAL_VERIFICATION_REQUEST => {
                debug!("Proposal verification request received: {}", proposal_id);

                match self.state {
                    State::EvaluatingProposal(ref tpc_proposal)
                        if tpc_proposal.proposal_id() != &proposal_id =>
                    {
                        debug!(
                            "Proposal already in progress, backlogging verification request: {}",
                            proposal_id
                        );
                        self.verification_request_backlog.push_back(proposal_id);
                    }
                    _ => {
                        // Try to find the proposal in the backlog
                        match self
                            .proposal_backlog
                            .iter()
                            .position(|tpc_proposal| tpc_proposal.proposal_id() == &proposal_id)
                        {
                            Some(idx) => {
                                debug!("Checking proposal {}", proposal_id);
                                proposal_manager.check_proposal(&proposal_id)?;
                                self.state = State::EvaluatingProposal(
                                    self.proposal_backlog.remove(idx).unwrap(),
                                );
                            }
                            None => {
                                debug!(
                                    "Proposal not yet received, backlogging verification request: \
                                     {}",
                                    proposal_id
                                );
                                self.verification_request_backlog.push_back(proposal_id);
                            }
                        }
                    }
                }
            }
            TwoPhaseMessage_Type::PROPOSAL_VERIFICATION_RESPONSE => {
                if !self.evaluating_proposal(&proposal_id) {
                    warn!(
                        "Received unexpected verification response for proposal {}",
                        proposal_id
                    );
                    return Ok(());
                }

                match two_phase_msg.get_proposal_verification_response() {
                    TwoPhaseMessage_ProposalVerificationResponse::VERIFIED => {
                        debug!(
                            "Proposal {} verified by peer {}",
                            proposal_id, consensus_msg.origin_id
                        );
                        // Already checked state above in self.evaluating_proposal()
                        if let State::EvaluatingProposal(tpc_proposal) = &mut self.state {
                            tpc_proposal.add_verified_peer(consensus_msg.origin_id);

                            if tpc_proposal.peers_verified() == tpc_proposal.required_verifiers() {
                                debug!(
                                    "All verifiers have approved; accepting proposal {}",
                                    proposal_id
                                );
                                self.complete_coordination(
                                    proposal_id,
                                    TwoPhaseMessage_ProposalResult::APPLY,
                                    network_sender,
                                    proposal_manager,
                                )?;
                            }
                        }
                    }
                    TwoPhaseMessage_ProposalVerificationResponse::FAILED => {
                        debug!(
                            "Proposal failed by peer {}; rejecting proposal {}",
                            consensus_msg.origin_id, proposal_id
                        );
                        self.complete_coordination(
                            proposal_id,
                            TwoPhaseMessage_ProposalResult::REJECT,
                            network_sender,
                            proposal_manager,
                        )?;
                    }
                    TwoPhaseMessage_ProposalVerificationResponse::UNSET_VERIFICATION_RESPONSE => {
                        warn!(
                            "Ignoring improperly specified proposal verification response from {}",
                            consensus_msg.origin_id
                        )
                    }
                }
            }
            TwoPhaseMessage_Type::PROPOSAL_RESULT => match two_phase_msg.get_proposal_result() {
                TwoPhaseMessage_ProposalResult::APPLY => {
                    if self.evaluating_proposal(&proposal_id) {
                        debug!("Accepting proposal {}", proposal_id);
                        proposal_manager.accept_proposal(&proposal_id, None)?;
                        self.state = State::Idle;
                    } else {
                        warn!(
                            "Received unexpected apply result for proposal {}",
                            proposal_id
                        );
                    }
                }
                TwoPhaseMessage_ProposalResult::REJECT => {
                    debug!("Rejecting proposal {}", proposal_id);
                    proposal_manager.reject_proposal(&proposal_id)?;

                    // Only update state if this was the currently evaluating proposal
                    if self.evaluating_proposal(&proposal_id) {
                        self.state = State::Idle;
                    }
                }
                TwoPhaseMessage_ProposalResult::UNSET_RESULT => warn!(
                    "Ignoring improperly specified proposal result from {}",
                    consensus_msg.origin_id
                ),
            },
            TwoPhaseMessage_Type::UNSET_TYPE => warn!(
                "Ignoring improperly specified two-phase message from {}",
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
            ProposalUpdate::ProposalCreated(None) => {
                if let State::AwaitingProposal = self.state {
                    self.state = State::Idle;
                }
            }
            ProposalUpdate::ProposalCreated(Some(proposal)) => {
                debug!("Proposal created: {}", proposal.id);
                self.handle_proposal(proposal, network_sender, proposal_manager)?;
            }
            ProposalUpdate::ProposalReceived(proposal, _) => {
                debug!("Proposal received: {}", proposal.id);
                self.handle_proposal(proposal, network_sender, proposal_manager)?;
            }
            ProposalUpdate::ProposalValid(proposal_id) => match &mut self.state {
                State::EvaluatingProposal(tpc_proposal)
                    if tpc_proposal.proposal_id() == &proposal_id =>
                {
                    debug!("Proposal valid: {}", proposal_id);

                    if &self.id == tpc_proposal.coordinator_id() {
                        tpc_proposal.add_verified_peer(self.id.clone());

                        debug!("Requesting verification of proposal {}", proposal_id);

                        let mut request = TwoPhaseMessage::new();
                        request
                            .set_message_type(TwoPhaseMessage_Type::PROPOSAL_VERIFICATION_REQUEST);
                        request.set_proposal_id(proposal_id.into());

                        network_sender.broadcast(request.write_to_bytes()?)?;
                    } else {
                        debug!("Sending verified response for proposal {}", proposal_id);

                        let mut response = TwoPhaseMessage::new();
                        response
                            .set_message_type(TwoPhaseMessage_Type::PROPOSAL_VERIFICATION_RESPONSE);
                        response.set_proposal_id(proposal_id.into());
                        response.set_proposal_verification_response(
                            TwoPhaseMessage_ProposalVerificationResponse::VERIFIED,
                        );

                        network_sender
                            .send_to(tpc_proposal.coordinator_id(), response.write_to_bytes()?)?;
                    }
                }
                _ => warn!("Got valid message for unknown proposal: {}", proposal_id),
            },
            ProposalUpdate::ProposalInvalid(proposal_id) => match self.state {
                State::EvaluatingProposal(ref tpc_proposal)
                    if tpc_proposal.proposal_id() == &proposal_id =>
                {
                    debug!("Proposal invalid: {}", proposal_id);

                    if &self.id == tpc_proposal.coordinator_id() {
                        debug!("Rejecting proposal {}", proposal_id);
                        self.complete_coordination(
                            proposal_id,
                            TwoPhaseMessage_ProposalResult::REJECT,
                            network_sender,
                            proposal_manager,
                        )?;
                    } else {
                        debug!("Sending failed response for proposal {}", proposal_id);

                        let mut response = TwoPhaseMessage::new();
                        response
                            .set_message_type(TwoPhaseMessage_Type::PROPOSAL_VERIFICATION_RESPONSE);
                        response.set_proposal_id(proposal_id.into());
                        response.set_proposal_verification_response(
                            TwoPhaseMessage_ProposalVerificationResponse::FAILED,
                        );

                        network_sender
                            .send_to(tpc_proposal.coordinator_id(), response.write_to_bytes()?)?;
                    }
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

    fn start_coordination(
        &mut self,
        tpc_proposal: TwoPhaseProposal,
        network_sender: &dyn ConsensusNetworkSender,
        proposal_manager: &dyn ProposalManager,
    ) -> Result<(), ConsensusEngineError> {
        debug!("Checking proposal {}", tpc_proposal.proposal_id());
        match proposal_manager.check_proposal(tpc_proposal.proposal_id()) {
            Ok(_) => {
                self.state = State::EvaluatingProposal(tpc_proposal);
                self.coordinator_timeout.start();
            }
            Err(err) => {
                debug!(
                    "Rejecting proposal {}; failed to check proposal due to err: {}",
                    tpc_proposal.proposal_id(),
                    err
                );
                self.complete_coordination(
                    tpc_proposal.proposal_id().clone(),
                    TwoPhaseMessage_ProposalResult::REJECT,
                    network_sender,
                    proposal_manager,
                )?;
            }
        }
        Ok(())
    }

    fn complete_coordination(
        &mut self,
        proposal_id: ProposalId,
        proposal_result: TwoPhaseMessage_ProposalResult,
        network_sender: &dyn ConsensusNetworkSender,
        proposal_manager: &dyn ProposalManager,
    ) -> Result<(), ConsensusEngineError> {
        match proposal_result {
            TwoPhaseMessage_ProposalResult::APPLY => {
                proposal_manager.accept_proposal(&proposal_id, None)?;
            }
            TwoPhaseMessage_ProposalResult::REJECT => {
                proposal_manager.reject_proposal(&proposal_id)?;
            }
            TwoPhaseMessage_ProposalResult::UNSET_RESULT => {
                warn!(
                    "Unset proposal result when completing proposal {}",
                    proposal_id
                );
                return Ok(());
            }
        }

        self.state = State::Idle;
        self.coordinator_timeout.stop();

        let mut result = TwoPhaseMessage::new();
        result.set_message_type(TwoPhaseMessage_Type::PROPOSAL_RESULT);
        result.set_proposal_id(proposal_id.into());
        result.set_proposal_result(proposal_result);

        network_sender.broadcast(result.write_to_bytes()?)?;

        Ok(())
    }

    fn handle_proposal(
        &mut self,
        proposal: Proposal,
        network_sender: &dyn ConsensusNetworkSender,
        proposal_manager: &dyn ProposalManager,
    ) -> Result<(), ConsensusEngineError> {
        let proposal_in_backlog = self
            .proposal_backlog
            .iter()
            .any(|tpc_proposal| tpc_proposal.proposal_id() == &proposal.id);
        if proposal_in_backlog {
            debug!(
                "Proposal already received and backlogged; ignoring: {}",
                proposal.id
            );
            return Ok(());
        }

        // Determine which peers must verify the proposal for it to be committed. If the proposal
        // manager provides a list in the consensus data field, those peers are used; otherwise,
        // the list will be all peers.
        let verifiers = if !proposal.consensus_data.is_empty() {
            let required_verifiers: RequiredVerifiers =
                protobuf::parse_from_bytes(&proposal.consensus_data)?;
            HashSet::from_iter(required_verifiers.verifiers.into_iter().map(PeerId::from))
        } else {
            let mut verifiers = self.peers.clone();
            verifiers.insert(self.id.clone());
            verifiers
        };

        // Determines which verifier is the coordinator; the coordinator is the verifier with the
        // lowest peer ID (bully algorithm).
        let coordinator = match verifiers.iter().min() {
            Some(coordinator) => coordinator.clone(),
            None => {
                error!(
                    "Rejecting proposal; no verifiers specified: {}",
                    proposal.id
                );
                proposal_manager.reject_proposal(&proposal.id)?;
                self.state = State::Idle;
                return Ok(());
            }
        };

        let tpc_proposal = TwoPhaseProposal::new(proposal.id, coordinator, verifiers);

        if let State::EvaluatingProposal(ref current_proposal) = self.state {
            if tpc_proposal.proposal_id() == current_proposal.proposal_id() {
                debug!(
                    "This proposal is already being evaluated; ignoring: {}",
                    tpc_proposal.proposal_id()
                );
            } else {
                debug!(
                    "Another proposal is already in progress; backlogging proposal {}",
                    tpc_proposal.proposal_id()
                );
                self.proposal_backlog.push_back(tpc_proposal);
            }
        } else if &self.id == tpc_proposal.coordinator_id() {
            debug!(
                "Starting coordination for proposal {}",
                tpc_proposal.proposal_id()
            );
            self.start_coordination(tpc_proposal, network_sender, proposal_manager)?;
        } else {
            debug!(
                "Not coordinator, backlogging proposal {}",
                tpc_proposal.proposal_id()
            );
            self.proposal_backlog.push_back(tpc_proposal);
        }

        Ok(())
    }

    /// If the coordinator timeout has expired, abort the current proposal.
    fn abort_proposal_if_timed_out(
        &mut self,
        network_sender: &dyn ConsensusNetworkSender,
        proposal_manager: &dyn ProposalManager,
    ) -> Result<(), ConsensusEngineError> {
        if let State::EvaluatingProposal(ref tpc_proposal) = self.state {
            if self.coordinator_timeout.check_expired() {
                warn!(
                    "Proposal timed out; rejecting: {}",
                    tpc_proposal.proposal_id()
                );
                let proposal_id = tpc_proposal.proposal_id().clone();
                self.complete_coordination(
                    proposal_id,
                    TwoPhaseMessage_ProposalResult::REJECT,
                    network_sender,
                    proposal_manager,
                )?;
            }
        }

        Ok(())
    }

    /// If not doing anything, see if there are any backlogged verification requests that this node
    /// has received a proposal for, and evaluate that proposal.
    fn handle_backlogged_verification_request(
        &mut self,
        proposal_manager: &dyn ProposalManager,
    ) -> Result<(), ConsensusEngineError> {
        if let State::Idle = self.state {
            if let Some(idx) = self
                .verification_request_backlog
                .iter()
                .position(|proposal_id| {
                    self.proposal_backlog
                        .iter()
                        .any(|tpc_proposal| tpc_proposal.proposal_id() == proposal_id)
                })
            {
                let proposal_id = self.verification_request_backlog.remove(idx).unwrap();
                let proposal_idx = self
                    .proposal_backlog
                    .iter()
                    .position(|tpc_proposal| tpc_proposal.proposal_id() == &proposal_id)
                    .unwrap();
                let tpc_proposal = self.proposal_backlog.remove(proposal_idx).unwrap();

                debug!("Checking proposal from backlog: {}", proposal_id);
                proposal_manager.check_proposal(&proposal_id)?;
                self.state = State::EvaluatingProposal(tpc_proposal);
            }
        }

        Ok(())
    }

    /// If not doing anything, try to get the next proposal. First check if there's one that this
    /// node is the coordinator for in the local backlog; if not, ask the proposal manager.
    fn get_next_proposal(
        &mut self,
        network_sender: &dyn ConsensusNetworkSender,
        proposal_manager: &dyn ProposalManager,
    ) -> Result<(), ConsensusEngineError> {
        if let State::Idle = self.state {
            if let Some(idx) = self
                .proposal_backlog
                .iter()
                .position(|tpc_proposal| tpc_proposal.coordinator_id() == &self.id)
            {
                let tpc_proposal = self.proposal_backlog.remove(idx).unwrap();
                debug!(
                    "Starting coordination for backlogged proposal {}",
                    tpc_proposal.proposal_id()
                );
                if let Err(err) =
                    self.start_coordination(tpc_proposal, network_sender, proposal_manager)
                {
                    error!("Failed to start coordination for proposal: {}", err);
                }
            } else {
                match proposal_manager.create_proposal(None, vec![]) {
                    Ok(()) => self.state = State::AwaitingProposal,
                    Err(err) => error!("Error while creating proposal: {}", err),
                }
            }
        }

        Ok(())
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
        network_sender: Box<dyn ConsensusNetworkSender>,
        proposal_manager: Box<dyn ProposalManager>,
        startup_state: StartupState,
    ) -> Result<(), ConsensusEngineError> {
        let message_timeout = Duration::from_millis(MESSAGE_RECV_TIMEOUT_MILLIS);
        let proposal_timeout = Duration::from_millis(PROPOSAL_RECV_TIMEOUT_MILLIS);

        self.id = startup_state.id;

        for id in startup_state.peer_ids {
            self.peers.insert(id);
        }

        loop {
            if let Err(err) = self.abort_proposal_if_timed_out(&*network_sender, &*proposal_manager)
            {
                error!("Failed to abort timed-out proposal: {}", err);
            }

            if let Err(err) = self.handle_backlogged_verification_request(&*proposal_manager) {
                error!("Failed to handle backlogged verification request: {}", err);
            }

            if let Err(err) = self.get_next_proposal(&*network_sender, &*proposal_manager) {
                error!("Failed to get next proposal: {}", err);
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

    const COORDINATOR_TIMEOUT_MILLIS: u64 = 5000;

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

        let mut engine = TwoPhaseEngine::new(Duration::from_millis(COORDINATOR_TIMEOUT_MILLIS));
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

        let mut engine = TwoPhaseEngine::new(Duration::from_millis(COORDINATOR_TIMEOUT_MILLIS));
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
            vec![0].into(),
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

        let mut engine = TwoPhaseEngine::new(Duration::from_millis(COORDINATOR_TIMEOUT_MILLIS));
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

        let mut engine = TwoPhaseEngine::new(Duration::from_millis(COORDINATOR_TIMEOUT_MILLIS));
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

    /// Test that the coordinator will abort a commit if the coordinator timeout expires while
    /// evaluating the commit.
    #[test]
    fn test_coordinator_timeout() {
        let (update_tx, update_rx) = channel();
        let (_consensus_msg_tx, consensus_msg_rx) = channel();

        let manager = MockProposalManager::new(update_tx.clone());
        let network = MockConsensusNetworkSender::new();
        let startup_state = StartupState {
            id: vec![0].into(),
            peer_ids: vec![vec![1].into(), vec![2].into()],
            last_proposal: None,
        };

        // Start engine with a very short coordinator timeout
        let mut engine = TwoPhaseEngine::new(Duration::from_millis(10));
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

        // Check that a proposal verification request is sent
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

        // Verify the Reject message is sent for the proposal (due to the timeout)
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
                    TwoPhaseMessage_ProposalResult::REJECT
                );
                assert_eq!(msg.get_proposal_id(), vec![1].as_slice());
                break;
            }
        }

        // Verify the proposal was rejected
        loop {
            if let Some(id) = manager.rejected_proposals().get(0) {
                assert_eq!(id, &vec![1].into());
                break;
            }
        }

        update_tx
            .send(ProposalUpdate::Shutdown)
            .expect("failed to send shutdown");
        thread.join().expect("failed to join engine thread");
    }
}
