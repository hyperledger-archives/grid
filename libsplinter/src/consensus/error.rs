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

use std::borrow::Borrow;
use std::error::Error;
use std::sync::mpsc::SendError;

use protobuf::error::ProtobufError;

use super::ProposalUpdate;

#[derive(Debug)]
pub enum ProposalManagerError {
    Internal(Box<dyn Error>),
    UpdateSendFailed(SendError<ProposalUpdate>),
}

impl Error for ProposalManagerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ProposalManagerError::Internal(err) => Some(err.borrow()),
            ProposalManagerError::UpdateSendFailed(err) => Some(err),
        }
    }
}

impl std::fmt::Display for ProposalManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let msg = match self {
            ProposalManagerError::Internal(err) => err.to_string(),
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
pub struct ConsensusSendError(pub Box<dyn Error>);

impl Error for ConsensusSendError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.0.borrow())
    }
}

impl std::fmt::Display for ConsensusSendError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "unable to send consensus message: {}", self.0)
    }
}

impl From<ProtobufError> for ConsensusSendError {
    fn from(err: ProtobufError) -> Self {
        ConsensusSendError(Box::new(err))
    }
}

#[derive(Debug)]
pub struct ConsensusEngineError(pub Box<dyn Error>);

impl Error for ConsensusEngineError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.0.borrow())
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
