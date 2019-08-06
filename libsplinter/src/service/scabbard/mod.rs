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

mod consensus;
mod error;
mod rest_api;
mod shared;
mod state;

use std::collections::{HashSet, VecDeque};
use std::convert::TryFrom;
use std::path::Path;
use std::sync::{Arc, Mutex};

use protobuf::Message;
use transact::protocol::batch::BatchPair;
use transact::protos::FromBytes;
use uuid::Uuid;

use crate::consensus::{Proposal, ProposalUpdate};
use crate::protos::scabbard::{ScabbardMessage, ScabbardMessage_Type};

use super::{
    Service, ServiceDestroyError, ServiceError, ServiceMessageContext, ServiceNetworkRegistry,
    ServiceStartError, ServiceStopError,
};

use consensus::ScabbardConsensusManager;
use error::ScabbardError;
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
        node_id: String,
        initial_peers: HashSet<String>,
        db_dir: &Path,
        db_size: usize,
    ) -> Result<Self, ScabbardError> {
        let db_path = db_dir.join(format!("{}.lmdb", Uuid::new_v4()));
        let state = ScabbardState::new(db_path.as_path(), db_size)
            .map_err(|err| ScabbardError::InitializationFailed(Box::new(err)))?;
        let shared = ScabbardShared::new(VecDeque::new(), None, initial_peers, state);

        Ok(Scabbard {
            service_id: format!("{}::{}", SERVICE_TYPE, node_id),
            shared: Arc::new(Mutex::new(shared)),
            consensus: None,
        })
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

        // Send connected messages to all other services
        {
            let mut shared = self
                .shared
                .lock()
                .map_err(|_| ServiceStartError::PoisonedLock("shared lock poisoned".into()))?;

            shared.set_network_sender(service_registry.connect(self.service_id())?);

            let mut connection_msg = ScabbardMessage::new();
            connection_msg.set_message_type(ScabbardMessage_Type::SERVICE_CONNECTED);
            connection_msg.set_service_id(self.service_id().to_string());

            for service in shared.peer_services() {
                shared
                    .network_sender()
                    // This unwrap is safe because the network sender was just set
                    .unwrap()
                    .send(
                        service,
                        connection_msg
                            .write_to_bytes()
                            .map_err(|err| ServiceStartError::Internal(Box::new(err)))?
                            .as_slice(),
                    )
                    .map_err(|err| ServiceStartError::Internal(Box::new(err)))?;
            }
        }

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

        // Send disconnected messages to all other services
        let mut disconnection_msg = ScabbardMessage::new();
        disconnection_msg.set_message_type(ScabbardMessage_Type::SERVICE_DISCONNECTED);
        disconnection_msg.set_service_id(self.service_id().to_string());

        let mut shared = self
            .shared
            .lock()
            .map_err(|_| ServiceStopError::PoisonedLock("shared lock poisoned".into()))?;
        let network_sender = shared
            .take_network_sender()
            .ok_or_else(|| ServiceStopError::Internal(Box::new(ScabbardError::NotConnected)))?;

        for service in shared.peer_services() {
            network_sender
                .send(
                    service,
                    disconnection_msg
                        .write_to_bytes()
                        .map_err(|err| ServiceStopError::Internal(Box::new(err)))?
                        .as_slice(),
                )
                .map_err(|err| ServiceStopError::Internal(Box::new(err)))?;
        }

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
            ScabbardMessage_Type::SERVICE_CONNECTED => {
                let mut shared = self
                    .shared
                    .lock()
                    .map_err(|_| ServiceError::PoisonedLock("shared lock poisoned".into()))?;

                if !shared.add_peer_service(message.get_service_id().to_string()) {
                    debug!(
                        "received connect from service that is already connected: {}",
                        message.get_service_id(),
                    );
                }

                Ok(())
            }
            ScabbardMessage_Type::SERVICE_DISCONNECTED => {
                let mut shared = self
                    .shared
                    .lock()
                    .map_err(|_| ServiceError::PoisonedLock("shared lock poisoned".into()))?;

                if !shared.remove_peer_service(message.get_service_id()) {
                    warn!(
                        "received disconnect from service that is not connected: {}",
                        message.get_service_id(),
                    );
                }

                Ok(())
            }
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
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use crate::service::tests::*;

    /// Tests that a new scabbard service is properly instantiated.
    #[test]
    fn new_scabbard() {
        let service = Scabbard::new("0".into(), HashSet::new(), Path::new("/tmp"), 1024 * 1024)
            .expect("failed to create service");
        assert_eq!(service.service_id(), &format!("{}::0", SERVICE_TYPE));
        assert_eq!(service.service_type(), SERVICE_TYPE);
    }

    /// Tests that the scabbard service properly shuts down its internal thread on stop. This test
    /// will hang if the thread does not get shutdown correctly.
    #[test]
    fn thread_cleanup() {
        let mut service = Scabbard::new(
            "scabbard".into(),
            HashSet::new(),
            Path::new("/tmp"),
            1024 * 1024,
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
            "scabbard".into(),
            HashSet::new(),
            Path::new("/tmp"),
            1024 * 1024,
        )
        .expect("failed to create service");
        test_connect_and_disconnect(&mut service);
    }

    /// Tests that the service properly sends `SERVICE_CONNECTED` and `SERVICE_DISCONNECTED`
    /// messages to peers on start and stop, respectively.
    #[test]
    fn service_connected_and_disconnected() {
        let mut peer_services = HashSet::new();
        peer_services.insert("0".into());
        peer_services.insert("1".into());
        let mut service = Scabbard::new(
            "scabbard".into(),
            peer_services.clone(),
            Path::new("/tmp"),
            1024 * 1024,
        )
        .expect("failed to create service");
        let registry = MockServiceNetworkRegistry::new();

        service.start(&registry).expect("failed to start engine");

        {
            let sent_messages = registry
                .network_sender()
                .sent
                .lock()
                .expect("sent lock poisoned");
            assert_eq!(2, sent_messages.len());
            let mut peer_services_to_verify = peer_services.clone();
            for (recipient, msg) in sent_messages.iter() {
                assert!(peer_services_to_verify.remove(recipient));
                let scabbard_message: ScabbardMessage =
                    protobuf::parse_from_bytes(&msg).expect("failed to parse scabbard message");
                assert_eq!(
                    scabbard_message.get_message_type(),
                    ScabbardMessage_Type::SERVICE_CONNECTED
                );
                assert_eq!(scabbard_message.get_service_id(), service.service_id());
            }
        }

        service.stop(&registry).expect("failed to stop engine");

        {
            let sent_messages = registry
                .network_sender()
                .sent
                .lock()
                .expect("sent lock poisoned")
                .split_off(2);
            assert_eq!(2, sent_messages.len());
            for (recipient, msg) in sent_messages.iter() {
                assert!(peer_services.remove(recipient));
                let scabbard_message: ScabbardMessage =
                    protobuf::parse_from_bytes(&msg).expect("failed to parse scabbard message");
                assert_eq!(
                    scabbard_message.get_message_type(),
                    ScabbardMessage_Type::SERVICE_DISCONNECTED,
                );
                assert_eq!(scabbard_message.get_service_id(), service.service_id());
            }
        }
    }

    /// Tests that the service properly adds and removes peers when it receives `SERVICE_CONNECTED`
    /// and `SERVICE_DISCONNECTED` messages, respectively.
    #[test]
    fn add_and_remove_peers() {
        let service = Scabbard::new(
            "scabbard".into(),
            HashSet::new(),
            Path::new("/tmp"),
            1024 * 1024,
        )
        .expect("failed to create service");

        let msg_context = ServiceMessageContext {
            sender: "0".into(),
            circuit: "alpha".into(),
            correlation_id: "123".into(),
        };

        let mut connect_msg = ScabbardMessage::new();
        connect_msg.set_message_type(ScabbardMessage_Type::SERVICE_CONNECTED);
        connect_msg.set_service_id("0".into());

        service
            .handle_message(
                connect_msg
                    .write_to_bytes()
                    .expect("failed to serialize connect msg")
                    .as_slice(),
                &msg_context,
            )
            .expect("failed to handle connect message");
        assert!(service
            .shared
            .lock()
            .expect("shared lock poisoned")
            .peer_services()
            .contains("0"));

        let mut disconnect_msg = ScabbardMessage::new();
        disconnect_msg.set_message_type(ScabbardMessage_Type::SERVICE_DISCONNECTED);
        disconnect_msg.set_service_id("0".into());

        service
            .handle_message(
                disconnect_msg
                    .write_to_bytes()
                    .expect("failed to serialize disconnect msg")
                    .as_slice(),
                &msg_context,
            )
            .expect("failed to handle disconnect message");
        assert!(!service
            .shared
            .lock()
            .expect("shared lock poisoned")
            .peer_services()
            .contains("0"));
    }
}
