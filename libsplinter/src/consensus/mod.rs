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

//! The API that defines interactions between consensus and a Splinter service.

pub mod error;

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

#[derive(Debug, Default)]
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
        network_sender: Box<ConsensusNetworkSender>,
        proposal_manager: Box<ProposalManager>,
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
