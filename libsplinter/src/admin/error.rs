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
use std::fmt;

use crate::consensus::error::ProposalManagerError;
use crate::service::error::ServiceError;

impl From<ServiceError> for ProposalManagerError {
    fn from(err: ServiceError) -> Self {
        ProposalManagerError::Internal(Box::new(err))
    }
}

#[derive(Debug)]
pub enum AdminSharedError {
    HashError(Sha256Error),
    InvalidMessageFormat(MarshallingError),
    NoPendingChanges,
    UnknownAction(String),
    ValidationFailed(String),
}

impl Error for AdminSharedError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            AdminSharedError::HashError(err) => Some(err),
            AdminSharedError::InvalidMessageFormat(err) => Some(err),
            AdminSharedError::NoPendingChanges => None,
            AdminSharedError::UnknownAction(_) => None,
            AdminSharedError::ValidationFailed(_) => None,
        }
    }
}

impl fmt::Display for AdminSharedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AdminSharedError::HashError(err) => write!(f, "received error while hashing: {}", err),
            AdminSharedError::InvalidMessageFormat(err) => {
                write!(f, "invalid message format: {}", err)
            }
            AdminSharedError::NoPendingChanges => {
                write!(f, "tried to commit without pending changes")
            }
            AdminSharedError::UnknownAction(msg) => {
                write!(f, "received message with unknown action: {}", msg)
            }
            AdminSharedError::ValidationFailed(msg) => write!(f, "validation failed: {}", msg),
        }
    }
}

#[derive(Debug)]
pub struct AdminConsensusManagerError(pub Box<dyn Error + Send>);

impl Error for AdminConsensusManagerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&*self.0)
    }
}

impl std::fmt::Display for AdminConsensusManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "admin consensus manager failed: {}", self.0)
    }
}

#[derive(Debug)]
pub enum AdminError {
    ConsensusFailed(AdminConsensusManagerError),
    MessageTypeUnset,
}

impl Error for AdminError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            AdminError::ConsensusFailed(err) => Some(err),
            AdminError::MessageTypeUnset => None,
        }
    }
}

impl std::fmt::Display for AdminError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AdminError::ConsensusFailed(err) => write!(f, "admin consensus failed: {}", err),
            AdminError::MessageTypeUnset => write!(f, "received message with unset type"),
        }
    }
}

impl From<AdminConsensusManagerError> for AdminError {
    fn from(err: AdminConsensusManagerError) -> Self {
        AdminError::ConsensusFailed(err)
    }
}

#[derive(Debug)]
pub struct Sha256Error(pub Box<dyn Error + Send>);

impl Error for Sha256Error {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&*self.0)
    }
}

impl std::fmt::Display for Sha256Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "unable to get sha256 hash: {}", self.0)
    }
}

impl From<Sha256Error> for AdminSharedError {
    fn from(err: Sha256Error) -> Self {
        AdminSharedError::HashError(err)
    }
}

#[derive(Debug)]
pub enum MarshallingError {
    UnsetField(String),
}

impl std::error::Error for MarshallingError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            MarshallingError::UnsetField(_) => None,
        }
    }
}

impl std::fmt::Display for MarshallingError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MarshallingError::UnsetField(_) => write!(f, "Invalid enumerated type"),
        }
    }
}
