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

use openssl::hash::{hash, MessageDigest};
use transact::protocol::batch::BatchPair;
use transact::protos::FromBytes;

use crate::consensus::{Proposal, ProposalUpdate};
use crate::hex::to_hex;
use crate::protos::scabbard::{ScabbardMessage, ScabbardMessage_Type};
use crate::rest_api::{Request, Response, ResponseError};
use crate::signing::SignatureVerifier;

use super::{
    Service, ServiceDestroyError, ServiceError, ServiceMessageContext, ServiceNetworkRegistry,
    ServiceStartError, ServiceStopError,
};

use consensus::ScabbardConsensusManager;
use error::ScabbardError;
pub use factory::ScabbardFactory;
use shared::ScabbardShared;
use state::ScabbardState;
pub use state::{BatchInfo, BatchStatus, StateChange, StateChangeEvent};

const SERVICE_TYPE: &str = "scabbard";

/// A service for running Sawtooth Sabre smart contracts with two-phase commit consensus.
#[derive(Clone)]
pub struct Scabbard {
    circuit_id: String,
    service_id: String,
    shared: Arc<Mutex<ScabbardShared>>,
    state: Arc<Mutex<ScabbardState>>,
    consensus: Arc<Mutex<Option<ScabbardConsensusManager>>>,
}

impl Scabbard {
    #[allow(clippy::too_many_arguments)]
    /// Generate a new Scabbard service.
    pub fn new(
        service_id: String,
        circuit_id: &str,
        // List of other scabbard services on the same circuit that this service shares state with
        peer_services: HashSet<String>,
        // The directory in which to create sabre's LMDB database
        state_db_dir: &Path,
        // The size of sabre's LMDB database
        state_db_size: usize,
        // The directory in which to create the transaction receipt store's LMDB database
        receipt_db_dir: &Path,
        // The size of the transaction receipt store's LMDB database
        receipt_db_size: usize,
        signature_verifier: Box<dyn SignatureVerifier>,
        // The public keys that are authorized to create and manage sabre contracts
        admin_keys: Vec<String>,
    ) -> Result<Self, ScabbardError> {
        let shared = ScabbardShared::new(VecDeque::new(), None, peer_services, signature_verifier);

        let hash = hash(
            MessageDigest::sha256(),
            format!("{}::{}", service_id, circuit_id).as_bytes(),
        )
        .map(|digest| to_hex(&*digest))
        .map_err(|err| ScabbardError::InitializationFailed(Box::new(err)))?;
        let state_db_path = state_db_dir.join(format!("{}-state.lmdb", hash));
        let receipt_db_path = receipt_db_dir.join(format!("{}-receipts.lmdb", hash));
        let state = ScabbardState::new(
            state_db_path.as_path(),
            state_db_size,
            receipt_db_path.as_path(),
            receipt_db_size,
            admin_keys,
        )
        .map_err(|err| ScabbardError::InitializationFailed(Box::new(err)))?;

        Ok(Scabbard {
            circuit_id: circuit_id.to_string(),
            service_id,
            shared: Arc::new(Mutex::new(shared)),
            state: Arc::new(Mutex::new(state)),
            consensus: Arc::new(Mutex::new(None)),
        })
    }

    pub fn add_batches(
        &self,
        batches: Vec<BatchPair>,
    ) -> Result<Option<BatchListPath>, ServiceError> {
        let mut shared = self
            .shared
            .lock()
            .map_err(|_| ServiceError::PoisonedLock("shared lock poisoned".into()))?;

        if shared.verify_batches(&batches)? {
            let mut link = format!(
                "/scabbard/{}/{}/batch_statuses?ids=",
                self.circuit_id, self.service_id
            );

            for batch in batches {
                self.state
                    .lock()
                    .map_err(|_| ServiceError::PoisonedLock("state lock poisoned".into()))?
                    .batch_history()
                    .add_batch(&batch.batch().header_signature())
                    .map_err(|err| ServiceError::UnableToHandleMessage(Box::new(err)))?;

                link.push_str(&format!("{},", batch.batch().header_signature()));
                shared.add_batch_to_queue(batch);
            }

            // Remove trailing comma
            link.pop();

            debug!("Batch Status Link Created: {}", link);
            Ok(Some(BatchListPath { link }))
        } else {
            Ok(None)
        }
    }

    pub fn get_batch_info(&self, ids: Vec<String>) -> Result<Vec<BatchInfo>, ServiceError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| ServiceError::PoisonedLock("state lock poisoned".into()))?;

        ids.iter()
            .map(|signature| state.batch_history().get_batch_info(signature))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| ServiceError::InvalidMessageFormat(Box::new(err)))
    }

    pub fn subscribe_to_state(
        &self,
        request: Request,
    ) -> Result<Result<Response, ResponseError>, ServiceError> {
        Ok(self
            .state
            .lock()
            .map_err(|_| ServiceError::PoisonedLock("state lock poisoned".into()))?
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
        let mut consensus = self
            .consensus
            .lock()
            .map_err(|_| ServiceStartError::PoisonedLock("consensus lock poisoned".into()))?;

        if consensus.is_some() {
            return Err(ServiceStartError::AlreadyStarted);
        }

        self.shared
            .lock()
            .map_err(|_| ServiceStartError::PoisonedLock("shared lock poisoned".into()))?
            .set_network_sender(service_registry.connect(self.service_id())?);

        // Setup consensus
        consensus.replace(
            ScabbardConsensusManager::new(
                self.service_id().into(),
                self.shared.clone(),
                self.state.clone(),
            )
            .map_err(|err| ServiceStartError::Internal(Box::new(ScabbardError::from(err))))?,
        );

        Ok(())
    }

    fn stop(
        &mut self,
        service_registry: &dyn ServiceNetworkRegistry,
    ) -> Result<(), ServiceStopError> {
        debug!("Stopping scabbard service with id {}", self.service_id);

        // Shutdown consensus
        self.consensus
            .lock()
            .map_err(|_| ServiceStopError::PoisonedLock("consensus lock poisoned".into()))?
            .take()
            .ok_or_else(|| ServiceStopError::NotStarted)?
            .shutdown()
            .map_err(|err| ServiceStopError::Internal(Box::new(ScabbardError::from(err))))?;

        self.shared
            .lock()
            .map_err(|_| ServiceStopError::PoisonedLock("shared lock poisoned".into()))?
            .take_network_sender()
            .ok_or_else(|| ServiceStopError::Internal(Box::new(ScabbardError::NotConnected)))?;

        // Shutdown event senders and disconnect websockets
        self.state
            .lock()
            .map_err(|_| ServiceStopError::PoisonedLock("state lock poisoned".into()))?
            .shutdown_event_senders();

        service_registry.disconnect(self.service_id())?;

        Ok(())
    }

    fn destroy(self: Box<Self>) -> Result<(), ServiceDestroyError> {
        if self
            .consensus
            .lock()
            .map_err(|_| ServiceDestroyError::PoisonedLock("consensus lock poisoned".into()))?
            .is_some()
        {
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
                .lock()
                .map_err(|_| ServiceError::PoisonedLock("consensus lock poisoned".into()))?
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
                    .lock()
                    .map_err(|_| ServiceError::PoisonedLock("consensus lock poisoned".into()))?
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
#[derive(Serialize, Deserialize, Clone)]
pub struct BatchListPath {
    link: String,
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use crate::service::tests::*;
    use crate::signing::hash::HashVerifier;

    /// Tests that a new scabbard service is properly instantiated.
    #[test]
    fn new_scabbard() {
        let service = Scabbard::new(
            "new_scabbard".into(),
            "test_circuit",
            HashSet::new(),
            Path::new("/tmp"),
            1024 * 1024,
            Path::new("/tmp"),
            1024 * 1024,
            Box::new(HashVerifier),
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
            "test_circuit",
            HashSet::new(),
            Path::new("/tmp"),
            1024 * 1024,
            Path::new("/tmp"),
            1024 * 1024,
            Box::new(HashVerifier),
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
            "test_circuit",
            HashSet::new(),
            Path::new("/tmp"),
            1024 * 1024,
            Path::new("/tmp"),
            1024 * 1024,
            Box::new(HashVerifier),
            vec![],
        )
        .expect("failed to create service");
        test_connect_and_disconnect(&mut service);
    }
}
