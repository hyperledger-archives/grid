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
use crate::orchestrator::InitializeServiceError;
use crate::service::error::{ServiceError, ServiceSendError};
use crate::signing;

use protobuf::error;

impl From<ServiceError> for ProposalManagerError {
    fn from(err: ServiceError) -> Self {
        ProposalManagerError::Internal(Box::new(err))
    }
}

#[derive(Debug)]
pub enum AdminSharedError {
    PoisonedLock(String),
    HashError(Sha256Error),
    InvalidMessageFormat(MarshallingError),
    NoPendingChanges,
    ServiceInitializationFailed(InitializeServiceError),
    ServiceSendError(ServiceSendError),
    UnknownAction(String),
    ValidationFailed(String),

    /// An error occured while attempting to verify a payload's signature
    SignerError(signing::error::Error),

    /// Returned if a signer is not specified currently only usra is supported
    /// and the feature flag "ursa-compat" must be supplied.
    UndefinedSigner,
    // Returned if a circuit cannot be added to splinter state
    CommitError(String),
}

impl Error for AdminSharedError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            AdminSharedError::PoisonedLock(_) => None,
            AdminSharedError::HashError(err) => Some(err),
            AdminSharedError::InvalidMessageFormat(err) => Some(err),
            AdminSharedError::NoPendingChanges => None,
            AdminSharedError::ServiceInitializationFailed(err) => Some(err),
            AdminSharedError::ServiceSendError(err) => Some(err),
            AdminSharedError::UnknownAction(_) => None,
            AdminSharedError::ValidationFailed(_) => None,
            AdminSharedError::SignerError(_) => None,
            AdminSharedError::UndefinedSigner => None,
            AdminSharedError::CommitError(_) => None,
        }
    }
}

impl fmt::Display for AdminSharedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AdminSharedError::PoisonedLock(details) => write!(f, "lock was poisoned: {}", details),
            AdminSharedError::HashError(err) => write!(f, "received error while hashing: {}", err),
            AdminSharedError::InvalidMessageFormat(err) => {
                write!(f, "invalid message format: {}", err)
            }
            AdminSharedError::NoPendingChanges => {
                write!(f, "tried to commit without pending changes")
            }
            AdminSharedError::ServiceInitializationFailed(err) => {
                write!(f, "failed to initialize service: {}", err)
            }
            AdminSharedError::ServiceSendError(err) => {
                write!(f, "failed to send service message: {}", err)
            }
            AdminSharedError::UnknownAction(msg) => {
                write!(f, "received message with unknown action: {}", msg)
            }
            AdminSharedError::ValidationFailed(msg) => write!(f, "validation failed: {}", msg),
            AdminSharedError::SignerError(ref msg) => write!(f, "Signing error: {}", msg),
            AdminSharedError::UndefinedSigner => f.write_str("Signing method was not defined"),
            AdminSharedError::CommitError(msg) => write!(f, "unable to commit circuit: {}", msg),
        }
    }
}

impl From<InitializeServiceError> for AdminSharedError {
    fn from(err: InitializeServiceError) -> Self {
        AdminSharedError::ServiceInitializationFailed(err)
    }
}
impl From<ServiceSendError> for AdminSharedError {
    fn from(err: ServiceSendError) -> Self {
        AdminSharedError::ServiceSendError(err)
    }
}

impl From<signing::error::Error> for AdminSharedError {
    fn from(err: signing::error::Error) -> Self {
        AdminSharedError::SignerError(err)
    }
}

impl From<MarshallingError> for AdminSharedError {
    fn from(err: MarshallingError) -> Self {
        AdminSharedError::InvalidMessageFormat(err)
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
    ProtobufError(error::ProtobufError),
}

impl std::error::Error for MarshallingError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            MarshallingError::UnsetField(_) => None,
            MarshallingError::ProtobufError(err) => Some(err),
        }
    }
}

impl std::fmt::Display for MarshallingError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MarshallingError::UnsetField(_) => write!(f, "Invalid enumerated type"),
            MarshallingError::ProtobufError(err) => write!(f, "Protobuf Error: {}", err),
        }
    }
}

impl From<error::ProtobufError> for MarshallingError {
    fn from(err: error::ProtobufError) -> Self {
        MarshallingError::ProtobufError(err)
    }
}
