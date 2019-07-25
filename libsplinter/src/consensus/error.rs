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

use std::error::Error;
use std::sync::mpsc::SendError;

use protobuf::error::ProtobufError;

use super::{PeerId, ProposalId, ProposalUpdate};

#[derive(Debug)]
pub enum ProposalManagerError {
    /// `ProposalManager` encountered an internal error while attempting to fulfill a request.
    Internal(Box<dyn Error + Send>),
    /// `ProposalManager` is not yet ready to process requests.
    NotReady,
    /// `ProposalManager` does not know about the specified proposal.
    UnknownProposal(ProposalId),
    /// `ProposalManager` failed to send send an update back to consensus.
    UpdateSendFailed(SendError<ProposalUpdate>),
}

impl Error for ProposalManagerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ProposalManagerError::Internal(err) => Some(&**err),
            ProposalManagerError::NotReady => None,
            ProposalManagerError::UnknownProposal(_) => None,
            ProposalManagerError::UpdateSendFailed(err) => Some(err),
        }
    }
}

impl std::fmt::Display for ProposalManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let msg = match self {
            ProposalManagerError::Internal(err) => err.to_string(),
            ProposalManagerError::NotReady => "not ready to process requests".to_string(),
            ProposalManagerError::UnknownProposal(id) => {
                format!("unknown proposal was specified: {}", id)
            }
            ProposalManagerError::UpdateSendFailed(err) => err.to_string(),
        };
        write!(f, "proposal manager error occurred: {}", msg)
    }
}

impl From<SendError<ProposalUpdate>> for ProposalManagerError {
    fn from(err: SendError<ProposalUpdate>) -> Self {
        ProposalManagerError::UpdateSendFailed(err)
    }
}

#[derive(Debug)]
pub enum ConsensusSendError {
    /// `ConsensusNetworkSender` encountered an internal error while attempting to send a message.
    Internal(Box<dyn Error + Send>),
    /// `ConsensusNetworkSender` is not yet ready to send messages.
    NotReady,
    /// `ConsensusNetworkSender` doesn't know about the peer the message was directed to.
    UnknownPeer(PeerId),
}

impl Error for ConsensusSendError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ConsensusSendError::Internal(err) => Some(&**err),
            ConsensusSendError::NotReady => None,
            ConsensusSendError::UnknownPeer(_) => None,
        }
    }
}

impl std::fmt::Display for ConsensusSendError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ConsensusSendError::Internal(err) => {
                write!(f, "internal error while sending consensus message: {}", err)
            }
            ConsensusSendError::NotReady => write!(f, "not ready to send messages"),
            ConsensusSendError::UnknownPeer(peer_id) => {
                write!(f, "attempted to send message to unknown peer: {}", peer_id)
            }
        }
    }
}

impl From<ProtobufError> for ConsensusSendError {
    fn from(err: ProtobufError) -> Self {
        ConsensusSendError::Internal(Box::new(err))
    }
}

#[derive(Debug)]
pub struct ConsensusEngineError(pub Box<dyn Error + Send>);

impl Error for ConsensusEngineError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&*self.0)
    }
}

impl std::fmt::Display for ConsensusEngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "consensus engine unable to send consensus message: {}",
            self.0
        )
    }
}

impl From<ProtobufError> for ConsensusEngineError {
    fn from(err: ProtobufError) -> Self {
        ConsensusEngineError(Box::new(err))
    }
}

impl From<ProposalManagerError> for ConsensusEngineError {
    fn from(err: ProposalManagerError) -> Self {
        ConsensusEngineError(Box::new(err))
    }
}

impl From<ConsensusSendError> for ConsensusEngineError {
    fn from(err: ConsensusSendError) -> Self {
        ConsensusEngineError(Box::new(err))
    }
}
