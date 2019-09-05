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

//! Scabbard is a Splinter `Service` that runs the Sawtooth Sabre smart contract engine using the
//! `transact` library for state. Scabbard uses two-phase consensus to reach agreement on
//! transactions.

mod consensus;
mod error;
mod factory;
mod shared;
mod state;

use std::any::Any;
use std::collections::{HashSet, VecDeque};
use std::convert::TryFrom;
use std::path::Path;
use std::sync::{Arc, Mutex};

use transact::protocol::batch::BatchPair;
use transact::protos::FromBytes;
use uuid::Uuid;

use crate::consensus::{Proposal, ProposalUpdate};
use crate::protos::scabbard::{ScabbardMessage, ScabbardMessage_Type};
use crate::rest_api::{Request, Response, ResponseError};

use super::{
    Service, ServiceDestroyError, ServiceError, ServiceMessageContext, ServiceNetworkRegistry,
    ServiceStartError, ServiceStopError,
};

use consensus::ScabbardConsensusManager;
use error::ScabbardError;
pub use factory::ScabbardFactory;
use shared::ScabbardShared;
use state::ScabbardState;

const SERVICE_TYPE: &str = "scabbard";

/// A service for running Sawtooth Sabre smart contracts with two-phase commit consensus.
pub struct Scabbard {
    service_id: String,
    shared: Arc<Mutex<ScabbardShared>>,
    consensus: Option<ScabbardConsensusManager>,
}

impl Scabbard {
    /// Generate a new Scabbard service.
    pub fn new(
        service_id: String,
        // List of other scabbard services on the same circuit that this service shares state with
        peer_services: HashSet<String>,
        // The directory in which to create sabre's LMDB database
        db_dir: &Path,
        // The size of sabre's LMDB database
        db_size: usize,
        // The public keys that are authorized to create and manage sabre contracts
        admin_keys: Vec<String>,
    ) -> Result<Self, ScabbardError> {
        let db_path = db_dir.join(format!("{}.lmdb", Uuid::new_v4()));
        let state = ScabbardState::new(db_path.as_path(), db_size, admin_keys)
            .map_err(|err| ScabbardError::InitializationFailed(Box::new(err)))?;
        let shared = ScabbardShared::new(VecDeque::new(), None, peer_services, state);

        Ok(Scabbard {
            service_id,
            shared: Arc::new(Mutex::new(shared)),
            consensus: None,
        })
    }

    pub fn add_batches(&self, batches: Vec<BatchPair>) -> Result<(), ServiceError> {
        let mut shared = self
            .shared
            .lock()
            .map_err(|_| ServiceError::PoisonedLock("shared lock poisoned".into()))?;

        for batch in batches {
            shared.add_batch_to_queue(batch);
        }

        Ok(())
    }

    pub fn subscribe_to_state(
        &self,
        request: Request,
    ) -> Result<Result<Response, ResponseError>, ServiceError> {
        Ok(self
            .shared
            .lock()
            .map_err(|_| ServiceError::PoisonedLock("shared lock poisoned".into()))?
            .state_mut()
            .subscribe_to_state(request))
    }
}

impl Service for Scabbard {
    fn service_id(&self) -> &str {
        &self.service_id
    }

    fn service_type(&self) -> &str {
        SERVICE_TYPE
    }

    fn start(
        &mut self,
        service_registry: &dyn ServiceNetworkRegistry,
    ) -> Result<(), ServiceStartError> {
        if self.consensus.is_some() {
            return Err(ServiceStartError::AlreadyStarted);
        }

        self.shared
            .lock()
            .map_err(|_| ServiceStartError::PoisonedLock("shared lock poisoned".into()))?
            .set_network_sender(service_registry.connect(self.service_id())?);

        // Setup consensus
        self.consensus = Some(
            ScabbardConsensusManager::new(self.service_id().into(), self.shared.clone())
                .map_err(|err| ServiceStartError::Internal(Box::new(ScabbardError::from(err))))?,
        );

        Ok(())
    }

    fn stop(
        &mut self,
        service_registry: &dyn ServiceNetworkRegistry,
    ) -> Result<(), ServiceStopError> {
        // Shutdown consensus
        self.consensus
            .take()
            .ok_or_else(|| ServiceStopError::NotStarted)?
            .shutdown()
            .map_err(|err| ServiceStopError::Internal(Box::new(ScabbardError::from(err))))?;

        self.shared
            .lock()
            .map_err(|_| ServiceStopError::PoisonedLock("shared lock poisoned".into()))?
            .take_network_sender()
            .ok_or_else(|| ServiceStopError::Internal(Box::new(ScabbardError::NotConnected)))?;

        service_registry.disconnect(self.service_id())?;

        Ok(())
    }

    fn destroy(self: Box<Self>) -> Result<(), ServiceDestroyError> {
        if self.consensus.is_some() {
            Err(ServiceDestroyError::NotStopped)
        } else {
            Ok(())
        }
    }

    fn handle_message(
        &self,
        message_bytes: &[u8],
        _message_context: &ServiceMessageContext,
    ) -> Result<(), ServiceError> {
        let message: ScabbardMessage = protobuf::parse_from_bytes(message_bytes)?;

        match message.get_message_type() {
            ScabbardMessage_Type::CONSENSUS_MESSAGE => self
                .consensus
                .as_ref()
                .ok_or_else(|| ServiceError::NotStarted)?
                .handle_message(message.get_consensus_message())
                .map_err(|err| ServiceError::UnableToHandleMessage(Box::new(err))),
            ScabbardMessage_Type::PROPOSED_BATCH => {
                let proposed_batch = message.get_proposed_batch();

                let proposal = Proposal::try_from(proposed_batch.get_proposal())?;
                let batch = BatchPair::from_bytes(proposed_batch.get_batch())
                    .map_err(|err| ServiceError::UnableToHandleMessage(Box::new(err)))?;

                self.shared
                    .lock()
                    .map_err(|_| ServiceError::PoisonedLock("shared lock poisoned".into()))?
                    .add_proposed_batch(proposal.id.clone(), batch);

                self.consensus
                    .as_ref()
                    .ok_or_else(|| ServiceError::NotStarted)?
                    .send_update(ProposalUpdate::ProposalReceived(
                        proposal,
                        proposed_batch.get_service_id().as_bytes().into(),
                    ))
                    .map_err(|err| ServiceError::UnableToHandleMessage(Box::new(err)))
            }
            ScabbardMessage_Type::UNSET => Err(ServiceError::InvalidMessageFormat(Box::new(
                ScabbardError::MessageTypeUnset,
            ))),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use crate::service::tests::*;

    /// Tests that a new scabbard service is properly instantiated.
    #[test]
    fn new_scabbard() {
        let service = Scabbard::new(
            "new_scabbard".into(),
            HashSet::new(),
            Path::new("/tmp"),
            1024 * 1024,
            vec![],
        )
        .expect("failed to create service");
        assert_eq!(service.service_id(), "new_scabbard");
        assert_eq!(service.service_type(), SERVICE_TYPE);
    }

    /// Tests that the scabbard service properly shuts down its internal thread on stop. This test
    /// will hang if the thread does not get shutdown correctly.
    #[test]
    fn thread_cleanup() {
        let mut service = Scabbard::new(
            "thread_cleanup".into(),
            HashSet::new(),
            Path::new("/tmp"),
            1024 * 1024,
            vec![],
        )
        .expect("failed to create service");
        let registry = MockServiceNetworkRegistry::new();
        service.start(&registry).expect("failed to start service");
        service.stop(&registry).expect("failed to stop service");
    }

    /// Tests that the service properly connects and disconnects using the network registry.
    #[test]
    fn connect_and_disconnect() {
        let mut service = Scabbard::new(
            "connect_and_disconnect".into(),
            HashSet::new(),
            Path::new("/tmp"),
            1024 * 1024,
            vec![],
        )
        .expect("failed to create service");
        test_connect_and_disconnect(&mut service);
    }
}
