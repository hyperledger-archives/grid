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

//! The API that defines interactions between consensus and a Splinter service.

pub mod error;
pub mod two_phase;

use std::convert::{TryFrom, TryInto};
use std::sync::mpsc::Receiver;

use protobuf::error::ProtobufError;
use protobuf::Message;

use crate::protos::consensus::{
    ConsensusMessage as ConsensusMessageProto, Proposal as ProposalProto,
};

pub use error::{ConsensusEngineError, ConsensusSendError, ProposalManagerError};

macro_rules! id_type {
    ($type:ident) => {
        #[derive(Clone, Default, Eq, Hash, PartialEq)]
        pub struct $type(Vec<u8>);

        impl AsRef<[u8]> for $type {
            fn as_ref(&self) -> &[u8] {
                &self.0
            }
        }

        impl std::fmt::Debug for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{:?}", self.0)
            }
        }

        impl std::fmt::Display for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                for b in &self.0 {
                    write!(f, "{:02x}", b)?;
                }
                Ok(())
            }
        }

        impl From<&[u8]> for $type {
            fn from(bytes: &[u8]) -> Self {
                $type(bytes.into())
            }
        }

        impl From<Vec<u8>> for $type {
            fn from(vec: Vec<u8>) -> Self {
                $type(vec)
            }
        }

        impl Into<Vec<u8>> for $type {
            fn into(self) -> Vec<u8> {
                self.0
            }
        }
    };
}

id_type!(PeerId);
id_type!(ProposalId);

impl Ord for PeerId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_ref().cmp(other.as_ref())
    }
}

impl PartialOrd for PeerId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.as_ref().partial_cmp(other.as_ref())
    }
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct Proposal {
    pub id: ProposalId,
    pub previous_id: ProposalId,
    pub proposal_height: u64,
    pub summary: Vec<u8>,
    pub consensus_data: Vec<u8>,
}

impl From<ProposalProto> for Proposal {
    fn from(proposal: ProposalProto) -> Self {
        Proposal {
            id: proposal.id.into(),
            previous_id: proposal.previous_id.into(),
            proposal_height: proposal.proposal_height,
            summary: proposal.summary,
            consensus_data: proposal.consensus_data,
        }
    }
}

impl Into<ProposalProto> for Proposal {
    fn into(self) -> ProposalProto {
        let mut msg = ProposalProto::new();
        msg.set_id(self.id.into());
        msg.set_previous_id(self.previous_id.into());
        msg.set_proposal_height(self.proposal_height);
        msg.set_summary(self.summary);
        msg.set_consensus_data(self.consensus_data);
        msg
    }
}

impl TryFrom<&[u8]> for Proposal {
    type Error = ProtobufError;
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let proto: ProposalProto = protobuf::parse_from_bytes(bytes)?;
        Ok(Proposal::from(proto))
    }
}

impl TryInto<Vec<u8>> for Proposal {
    type Error = ProtobufError;
    fn try_into(self) -> Result<Vec<u8>, Self::Error> {
        let proto: ProposalProto = self.into();
        proto.write_to_bytes()
    }
}

/// Interface used by consensus to create, check, accept, and reject proposals
pub trait ProposalManager: Send {
    /// Informs the manager if consensus will ask for proposals; if not, the manager does not need
    /// to build them.
    ///
    /// Some managers may take arbitrarily long to assemble/process proposals, and some consensus
    /// algorithms designate nodes that do not create proposals at all. This allows for optimizing
    /// performance in some cases.
    ///
    /// The default implementation does nothing, since this is only useful for some managers.
    fn should_build_proposals(&self, _should_build: bool) -> Result<(), ProposalManagerError> {
        Ok(())
    }

    /// Generate a new Proposal with the given consensus bytes that’s based on the previous
    /// proposal if Some, otherwise the manager will use the last applied proposal.
    fn create_proposal(
        &self,
        previous_proposal_id: Option<ProposalId>,
        consensus_data: Vec<u8>,
    ) -> Result<(), ProposalManagerError>;

    /// Verify that the data corresponding to specified proposal ID is valid from the perspective
    /// of the proposal manager (only necessary for `Proposal`s received from peers).
    fn check_proposal(&self, id: &ProposalId) -> Result<(), ProposalManagerError>;

    /// Consensus has approved the given proposal. New consensus data may be provided to replace
    /// the existing data.
    fn accept_proposal(
        &self,
        id: &ProposalId,
        consensus_data: Option<Vec<u8>>,
    ) -> Result<(), ProposalManagerError>;

    /// Consensus has rejected the given proposal.
    fn reject_proposal(&self, id: &ProposalId) -> Result<(), ProposalManagerError>;
}

/// Messages the `ProposalManager` sends to consensus
#[derive(Debug)]
pub enum ProposalUpdate {
    /// The manager generated a `Proposal`; if `None`, no proposal is currently available.
    ProposalCreated(Option<Proposal>),

    /// A `Proposal` has been received from a peer.
    ProposalReceived(Proposal, PeerId),

    /// The `Proposal` with the given ID was checked and found to be valid.
    ProposalValid(ProposalId),

    /// The `Proposal` with the given ID was checked and found to be invalid.
    ProposalInvalid(ProposalId),

    /// The `Proposal` with the given ID was accepted.
    ProposalAccepted(ProposalId),

    /// The `Proposal` with the given ID could not be accepted due to the specified error.
    ProposalAcceptFailed(ProposalId, String),

    /// Signal consensus to shutdown gracefully.
    Shutdown,
}

#[derive(Debug, Default)]
pub struct ConsensusMessage {
    pub message: Vec<u8>,
    pub origin_id: PeerId,
}

impl ConsensusMessage {
    pub fn new(message: Vec<u8>, origin_id: PeerId) -> Self {
        ConsensusMessage { message, origin_id }
    }
}

impl From<ConsensusMessageProto> for ConsensusMessage {
    fn from(msg: ConsensusMessageProto) -> Self {
        ConsensusMessage {
            message: msg.message,
            origin_id: msg.origin_id.into(),
        }
    }
}

impl Into<ConsensusMessageProto> for ConsensusMessage {
    fn into(self) -> ConsensusMessageProto {
        let mut msg = ConsensusMessageProto::new();
        msg.set_message(self.message);
        msg.set_origin_id(self.origin_id.into());
        msg
    }
}

impl TryFrom<&[u8]> for ConsensusMessage {
    type Error = ProtobufError;
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let proto: ConsensusMessageProto = protobuf::parse_from_bytes(bytes)?;
        Ok(ConsensusMessage::from(proto))
    }
}

impl TryInto<Vec<u8>> for ConsensusMessage {
    type Error = ProtobufError;
    fn try_into(self) -> Result<Vec<u8>, Self::Error> {
        let proto: ConsensusMessageProto = self.into();
        proto.write_to_bytes()
    }
}

/// Interface used by consensus to send messages to other nodes
pub trait ConsensusNetworkSender: Send {
    /// Send the message to the given peer.
    fn send_to(&self, peer_id: &PeerId, message: Vec<u8>) -> Result<(), ConsensusSendError>;

    /// Send the message to all peers.
    fn broadcast(&self, message: Vec<u8>) -> Result<(), ConsensusSendError>;
}

/// Consensus algorithms are implemented as consensus engines. The ConsensusEngine interface
/// defines how consensus algorithms are identified (name, version, and supported protocols), as
/// well as how they are run and what values are required for running.
pub trait ConsensusEngine: Send {
    /// The name of the consensus engine
    fn name(&self) -> &str;

    /// The version of the consensus engine
    fn version(&self) -> &str;

    /// Any additional name/version pairs this engine supports
    fn additional_protocols(&self) -> Vec<(String, String)>;

    /// Run the consensus engine.
    fn run(
        &mut self,
        consensus_messages: Receiver<ConsensusMessage>,
        proposal_updates: Receiver<ProposalUpdate>,
        network_sender: Box<dyn ConsensusNetworkSender>,
        proposal_manager: Box<dyn ProposalManager>,
        startup_state: StartupState,
    ) -> Result<(), ConsensusEngineError>;
}

pub struct StartupState {
    /// The identifier of this consensus engine within the consensus network
    pub id: PeerId,
    /// List of all consensus engines that the network sender currently has a connection to
    pub peer_ids: Vec<PeerId>,
    /// The last `Proposal` that was accepted
    pub last_proposal: Option<Proposal>,
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use std::cell::RefCell;
    use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
    use std::sync::mpsc::Sender;
    use std::sync::{Arc, Mutex, MutexGuard};

    pub struct MockProposalManager {
        update_sender: Sender<ProposalUpdate>,
        last_proposal_height: AtomicU8,
        last_proposal_id: RefCell<ProposalId>,
        accepted_proposals: Arc<Mutex<Vec<(ProposalId, Vec<u8>)>>>,
        rejected_proposals: Arc<Mutex<Vec<ProposalId>>>,
        next_proposal_valid: Arc<AtomicBool>,
        return_proposal: Arc<AtomicBool>,
        consensus_data: Option<Vec<u8>>,
    }

    impl Clone for MockProposalManager {
        fn clone(&self) -> Self {
            MockProposalManager {
                update_sender: self.update_sender.clone(),
                last_proposal_height: AtomicU8::new(
                    self.last_proposal_height.load(Ordering::Relaxed),
                ),
                last_proposal_id: self.last_proposal_id.clone(),
                accepted_proposals: self.accepted_proposals.clone(),
                rejected_proposals: self.rejected_proposals.clone(),
                next_proposal_valid: self.next_proposal_valid.clone(),
                return_proposal: self.return_proposal.clone(),
                consensus_data: self.consensus_data.clone(),
            }
        }
    }

    impl MockProposalManager {
        pub fn new(update_sender: Sender<ProposalUpdate>) -> Self {
            MockProposalManager {
                update_sender,
                last_proposal_height: AtomicU8::new(0),
                last_proposal_id: RefCell::new(ProposalId::default()),
                accepted_proposals: Arc::new(Mutex::new(vec![])),
                rejected_proposals: Arc::new(Mutex::new(vec![])),
                next_proposal_valid: Arc::new(AtomicBool::new(true)),
                return_proposal: Arc::new(AtomicBool::new(true)),
                consensus_data: None,
            }
        }

        pub fn set_next_proposal_valid(&self, valid: bool) {
            self.next_proposal_valid.store(valid, Ordering::Relaxed);
        }

        pub fn set_return_proposal(&self, return_proposal: bool) {
            self.return_proposal
                .store(return_proposal, Ordering::Relaxed);
        }

        pub fn set_consensus_data(&mut self, data: Option<Vec<u8>>) {
            self.consensus_data = data;
        }

        pub fn accepted_proposals(&self) -> MutexGuard<Vec<(ProposalId, Vec<u8>)>> {
            self.accepted_proposals
                .lock()
                .expect("failed to get accepted proposals")
        }

        pub fn rejected_proposals(&self) -> MutexGuard<Vec<ProposalId>> {
            self.rejected_proposals
                .lock()
                .expect("failed to get rejected proposals")
        }
    }

    impl ProposalManager for MockProposalManager {
        fn create_proposal(
            &self,
            previous_proposal_id: Option<ProposalId>,
            consensus_data: Vec<u8>,
        ) -> Result<(), ProposalManagerError> {
            if self.return_proposal.load(Ordering::Relaxed) {
                let height = self.last_proposal_height.load(Ordering::Relaxed) + 1;
                let id = vec![height];

                let mut proposal = Proposal::default();
                proposal.id = id.clone().into();
                proposal.previous_id =
                    previous_proposal_id.unwrap_or((*self.last_proposal_id.borrow_mut()).clone());
                proposal.proposal_height = height as u64;
                proposal.summary = id.clone();

                if let Some(data) = &self.consensus_data {
                    proposal.consensus_data = data.clone();
                } else {
                    proposal.consensus_data = consensus_data;
                }

                self.last_proposal_id.replace(id.into());
                self.last_proposal_height.store(height, Ordering::Relaxed);

                self.update_sender
                    .send(ProposalUpdate::ProposalCreated(Some(proposal)))
                    .expect("failed to send proposal");
            } else {
                self.update_sender
                    .send(ProposalUpdate::ProposalCreated(None))
                    .expect("failed to send proposal");
            }

            Ok(())
        }

        fn check_proposal(&self, id: &ProposalId) -> Result<(), ProposalManagerError> {
            if self.next_proposal_valid.load(Ordering::Relaxed) {
                self.update_sender
                    .send(ProposalUpdate::ProposalValid(id.clone()))
                    .expect("failed to send valid message");
            } else {
                self.update_sender
                    .send(ProposalUpdate::ProposalInvalid(id.clone()))
                    .expect("failed to send invalid message");
            }

            Ok(())
        }

        fn accept_proposal(
            &self,
            id: &ProposalId,
            consensus_data: Option<Vec<u8>>,
        ) -> Result<(), ProposalManagerError> {
            self.accepted_proposals
                .lock()
                .expect("failed to get accepted proposals lock")
                .push((id.clone(), consensus_data.unwrap_or(vec![])));
            Ok(())
        }

        fn reject_proposal(&self, id: &ProposalId) -> Result<(), ProposalManagerError> {
            self.rejected_proposals
                .lock()
                .expect("failed to get rejected proposals lock")
                .push(id.clone());
            Ok(())
        }
    }

    #[derive(Clone)]
    pub struct MockConsensusNetworkSender {
        sent_messages: Arc<Mutex<Vec<(Vec<u8>, PeerId)>>>,
        broadcast_messages: Arc<Mutex<Vec<Vec<u8>>>>,
    }

    impl MockConsensusNetworkSender {
        pub fn new() -> Self {
            MockConsensusNetworkSender {
                sent_messages: Arc::new(Mutex::new(vec![])),
                broadcast_messages: Arc::new(Mutex::new(vec![])),
            }
        }

        pub fn sent_messages(&self) -> MutexGuard<Vec<(Vec<u8>, PeerId)>> {
            self.sent_messages
                .lock()
                .expect("failed to get sent messages")
        }

        pub fn broadcast_messages(&self) -> MutexGuard<Vec<Vec<u8>>> {
            self.broadcast_messages
                .lock()
                .expect("failed to get broadcast messages")
        }
    }

    impl ConsensusNetworkSender for MockConsensusNetworkSender {
        fn send_to(&self, peer_id: &PeerId, message: Vec<u8>) -> Result<(), ConsensusSendError> {
            self.sent_messages
                .lock()
                .expect("failed to get sent messages")
                .push((message, peer_id.clone()));
            Ok(())
        }

        fn broadcast(&self, message: Vec<u8>) -> Result<(), ConsensusSendError> {
            self.broadcast_messages
                .lock()
                .expect("failed to get broadcast messages")
                .push(message);
            Ok(())
        }
    }
}
