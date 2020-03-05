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

mod consensus;
pub(crate) mod error;
mod mailbox;
pub(crate) mod messages;
pub(super) mod open_proposals;
#[cfg(feature = "proposal-read")]
pub(super) mod proposal_store;
mod shared;

use std::any::Any;
#[cfg(feature = "service-arg-validation")]
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use openssl::hash::{hash, MessageDigest};
use protobuf::{self, Message};

use crate::circuit::SplinterState;
use crate::consensus::Proposal;
use crate::hex::to_hex;
use crate::keys::{KeyPermissionManager, KeyRegistry};
use crate::network::{
    auth::{AuthorizationCallbackError, AuthorizationInquisitor, PeerAuthorizationState},
    peer::PeerConnector,
};
use crate::orchestrator::ServiceOrchestrator;
use crate::protos::admin::{AdminMessage, AdminMessage_Type, CircuitManagementPayload};
#[cfg(feature = "service-arg-validation")]
use crate::service::validation::ServiceArgValidator;
use crate::service::{
    error::{ServiceDestroyError, ServiceError, ServiceStartError, ServiceStopError},
    Service, ServiceMessageContext, ServiceNetworkRegistry,
};
use crate::signing::SignatureVerifier;

use self::consensus::AdminConsensusManager;
use self::error::{AdminError, Sha256Error};
#[cfg(feature = "proposal-read")]
use self::proposal_store::{AdminServiceProposals, ProposalStore};
use self::shared::AdminServiceShared;

pub use self::error::AdminServiceError;
pub use self::error::AdminSubscriberError;

const DEFAULT_COORDINATOR_TIMEOUT_MILLIS: u64 = 30000; // 30 seconds

pub trait AdminServiceEventSubscriber: Send {
    fn handle_event(
        &self,
        admin_service_event: &messages::AdminServiceEvent,
        timestamp: &SystemTime,
    ) -> Result<(), AdminSubscriberError>;
}

pub trait AdminCommands: Send + Sync {
    fn submit_circuit_change(
        &self,
        circuit_change: CircuitManagementPayload,
    ) -> Result<(), AdminServiceError>;

    fn add_event_subscriber(
        &self,
        event_type: &str,
        subscriber: Box<dyn AdminServiceEventSubscriber>,
    ) -> Result<(), AdminServiceError>;

    fn get_events_since(
        &self,
        since_timestamp: &SystemTime,
        event_type: &str,
    ) -> Result<Events, AdminServiceError>;

    fn clone_boxed(&self) -> Box<dyn AdminCommands>;
}

impl Clone for Box<dyn AdminCommands> {
    fn clone(&self) -> Self {
        self.clone_boxed()
    }
}

/// An iterator over AdminServiceEvents and the time that each occurred.
pub struct Events {
    inner: Box<dyn Iterator<Item = (SystemTime, messages::AdminServiceEvent)> + Send>,
}

impl Iterator for Events {
    type Item = (SystemTime, messages::AdminServiceEvent);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

pub struct AdminService {
    service_id: String,
    admin_service_shared: Arc<Mutex<AdminServiceShared>>,
    /// The coordinator timeout for the two-phase commit consensus engine
    coordinator_timeout: Duration,
    consensus: Option<AdminConsensusManager>,
}

impl AdminService {
    #![allow(clippy::too_many_arguments)]
    pub fn new(
        node_id: &str,
        orchestrator: ServiceOrchestrator,
        #[cfg(feature = "service-arg-validation")] service_arg_validators: HashMap<
            String,
            Box<dyn ServiceArgValidator + Send>,
        >,
        peer_connector: PeerConnector,
        authorization_inquistor: Box<dyn AuthorizationInquisitor>,
        splinter_state: SplinterState,
        signature_verifier: Box<dyn SignatureVerifier + Send>,
        key_registry: Box<dyn KeyRegistry>,
        key_permission_manager: Box<dyn KeyPermissionManager>,
        storage_type: &str,
        // The coordinator timeout for the two-phase commit consensus engine; if `None`, the
        // default value will be used (30 seconds).
        coordinator_timeout: Option<Duration>,
    ) -> Result<Self, ServiceError> {
        let coordinator_timeout = coordinator_timeout
            .unwrap_or_else(|| Duration::from_millis(DEFAULT_COORDINATOR_TIMEOUT_MILLIS));

        let new_service = Self {
            service_id: admin_service_id(node_id),
            admin_service_shared: Arc::new(Mutex::new(AdminServiceShared::new(
                node_id.to_string(),
                orchestrator,
                #[cfg(feature = "service-arg-validation")]
                service_arg_validators,
                peer_connector,
                authorization_inquistor,
                splinter_state,
                signature_verifier,
                key_registry,
                key_permission_manager,
                storage_type,
            )?)),
            coordinator_timeout,
            consensus: None,
        };

        let auth_callback_shared = Arc::clone(&new_service.admin_service_shared);

        new_service
            .admin_service_shared
            .lock()
            .map_err(|_| {
                ServiceError::PoisonedLock(
                    "The lock was poisoned while creating the service".into(),
                )
            })?
            .auth_inquisitor()
            .register_callback(Box::new(
                move |peer_id: &str, state: PeerAuthorizationState| {
                    let result = auth_callback_shared
                        .lock()
                        .map_err(|_| {
                            AuthorizationCallbackError(
                                "admin service shared lock was poisoned".into(),
                            )
                        })?
                        .on_authorization_change(peer_id, state);
                    if let Err(err) = result {
                        error!("{}", err);
                    }

                    Ok(())
                },
            ))
            .map_err(|err| ServiceError::UnableToCreate(Box::new(err)))?;

        Ok(new_service)
    }

    pub fn commands(&self) -> impl AdminCommands + Clone {
        AdminServiceCommands {
            shared: Arc::clone(&self.admin_service_shared),
        }
    }

    #[cfg(feature = "proposal-read")]
    pub fn proposals(&self) -> impl ProposalStore {
        AdminServiceProposals::new(&self.admin_service_shared)
    }
}

impl Service for AdminService {
    fn service_id(&self) -> &str {
        &self.service_id
    }

    fn service_type(&self) -> &str {
        "admin"
    }

    fn start(
        &mut self,
        service_registry: &dyn ServiceNetworkRegistry,
    ) -> Result<(), ServiceStartError> {
        if self.consensus.is_some() {
            return Err(ServiceStartError::AlreadyStarted);
        }

        let network_sender = service_registry.connect(&self.service_id)?;

        {
            let mut admin_service_shared = self.admin_service_shared.lock().map_err(|_| {
                ServiceStartError::PoisonedLock("the admin shared lock was poisoned".into())
            })?;

            admin_service_shared.set_network_sender(Some(network_sender));
        }

        // Setup consensus
        let consensus = AdminConsensusManager::new(
            self.service_id().into(),
            self.admin_service_shared.clone(),
            self.coordinator_timeout,
        )
        .map_err(|err| ServiceStartError::Internal(Box::new(err)))?;
        let proposal_sender = consensus.proposal_update_sender();

        self.consensus = Some(consensus);

        self.admin_service_shared
            .lock()
            .map_err(|_| {
                ServiceStartError::PoisonedLock("the admin shared lock was poisoned".into())
            })?
            .set_proposal_sender(Some(proposal_sender));

        self.admin_service_shared
            .lock()
            .map_err(|_| {
                ServiceStartError::PoisonedLock("the admin shared lock was poisoned".into())
            })?
            .restart_services()
            .map_err(|err| ServiceStartError::Internal(Box::new(err)))?;

        self.admin_service_shared
            .lock()
            .map_err(|_| {
                ServiceStartError::PoisonedLock("the admin shared lock was poisoned".into())
            })?
            .add_services_to_directory()
            .map_err(|err| ServiceStartError::Internal(Box::new(err)))?;

        Ok(())
    }

    fn stop(
        &mut self,
        service_registry: &dyn ServiceNetworkRegistry,
    ) -> Result<(), ServiceStopError> {
        service_registry.disconnect(&self.service_id)?;

        // Shutdown consensus
        self.consensus
            .take()
            .ok_or_else(|| ServiceStopError::NotStarted)?
            .shutdown()
            .map_err(|err| ServiceStopError::Internal(Box::new(err)))?;

        let mut admin_service_shared = self.admin_service_shared.lock().map_err(|_| {
            ServiceStopError::PoisonedLock("the admin shared lock was poisoned".into())
        })?;

        admin_service_shared.set_network_sender(None);

        admin_service_shared.remove_all_event_subscribers();

        admin_service_shared
            .stop_services()
            .map_err(|err| ServiceStopError::Internal(Box::new(err)))?;

        info!("Admin service stopped and disconnected");

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
        message_context: &ServiceMessageContext,
    ) -> Result<(), ServiceError> {
        let admin_message: AdminMessage = protobuf::parse_from_bytes(message_bytes)
            .map_err(|err| ServiceError::InvalidMessageFormat(Box::new(err)))?;
        debug!("received admin message {:?}", admin_message);
        match admin_message.get_message_type() {
            AdminMessage_Type::CONSENSUS_MESSAGE => self
                .consensus
                .as_ref()
                .ok_or_else(|| ServiceError::NotStarted)?
                .handle_message(admin_message.get_consensus_message())
                .map_err(|err| ServiceError::UnableToHandleMessage(Box::new(err))),
            AdminMessage_Type::PROPOSED_CIRCUIT => {
                let proposed_circuit = admin_message.get_proposed_circuit();

                let expected_hash = proposed_circuit.get_expected_hash().to_vec();
                let circuit_payload = proposed_circuit.get_circuit_payload();
                let required_verifiers = proposed_circuit.get_required_verifiers();
                let mut proposal = Proposal::default();

                proposal.id = sha256(circuit_payload)
                    .map_err(|err| ServiceError::UnableToHandleMessage(Box::new(err)))?
                    .as_bytes()
                    .into();
                proposal.summary = expected_hash;
                proposal.consensus_data = required_verifiers.to_vec();
                let mut admin_service_shared = self.admin_service_shared.lock().map_err(|_| {
                    ServiceError::PoisonedLock("the admin shared lock was poisoned".into())
                })?;

                admin_service_shared.handle_proposed_circuit(
                    proposal,
                    circuit_payload.clone(),
                    message_context.sender.to_string(),
                )
            }
            AdminMessage_Type::MEMBER_READY => {
                let member_ready = admin_message.get_member_ready();
                let circuit_id = member_ready.get_circuit_id();
                let member_node_id = member_ready.get_member_node_id();

                let mut shared = self.admin_service_shared.lock().map_err(|_| {
                    ServiceError::PoisonedLock("the admin shared lock was poisoned".into())
                })?;

                shared
                    .add_ready_member(circuit_id, member_node_id.into())
                    .map_err(|err| ServiceError::UnableToHandleMessage(Box::new(err)))
            }
            AdminMessage_Type::UNSET => Err(ServiceError::InvalidMessageFormat(Box::new(
                AdminError::MessageTypeUnset,
            ))),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone)]
struct AdminServiceCommands {
    shared: Arc<Mutex<AdminServiceShared>>,
}

impl AdminCommands for AdminServiceCommands {
    fn submit_circuit_change(
        &self,
        circuit_change: CircuitManagementPayload,
    ) -> Result<(), AdminServiceError> {
        self.shared
            .lock()
            .map_err(|_| AdminServiceError::general_error("Admin shared lock was lock poisoned"))?
            .submit(circuit_change)?;

        Ok(())
    }

    fn add_event_subscriber(
        &self,
        event_type: &str,
        subscriber: Box<dyn AdminServiceEventSubscriber>,
    ) -> Result<(), AdminServiceError> {
        self.shared
            .lock()
            .map_err(|_| AdminServiceError::general_error("Admin shared lock was lock poisoned"))?
            .add_subscriber(event_type.into(), subscriber)
            .map_err(|err| {
                AdminServiceError::general_error_with_source(
                    "Unable to add event subscriber",
                    Box::new(err),
                )
            })
    }

    fn get_events_since(
        &self,
        since_timestamp: &SystemTime,
        event_type: &str,
    ) -> Result<Events, AdminServiceError> {
        self.shared
            .lock()
            .map_err(|_| AdminServiceError::general_error("Admin shared lock was lock poisoned"))?
            .get_events_since(since_timestamp, event_type)
            .map_err(|err| {
                AdminServiceError::general_error_with_source("Unable to get events", Box::new(err))
            })
    }

    fn clone_boxed(&self) -> Box<dyn AdminCommands> {
        Box::new(self.clone())
    }
}

pub fn admin_service_id(node_id: &str) -> String {
    format!("admin::{}", node_id)
}

fn sha256<T>(message: &T) -> Result<String, Sha256Error>
where
    T: Message,
{
    let bytes = message
        .write_to_bytes()
        .map_err(|err| Sha256Error(Box::new(err)))?;
    hash(MessageDigest::sha256(), &bytes)
        .map(|digest| to_hex(&*digest))
        .map_err(|err| Sha256Error(Box::new(err)))
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::VecDeque;
    use std::sync::mpsc::{channel, Sender};
    use std::time::{Duration, Instant};

    use crate::circuit::{directory::CircuitDirectory, SplinterState};
    use crate::keys::{
        insecure::AllowAllKeyPermissionManager, storage::StorageKeyRegistry, KeyInfo,
    };
    use crate::mesh::Mesh;
    use crate::network::{auth::AuthorizationCallback, Network};
    use crate::protos::{
        admin,
        authorization::{AuthorizationMessage, AuthorizationMessageType, AuthorizedMessage},
        network::{NetworkMessage, NetworkMessageType},
    };
    use crate::service::{error, ServiceNetworkRegistry, ServiceNetworkSender};
    use crate::signing::{
        hash::{HashSigner, HashVerifier},
        Signer,
    };
    use crate::storage::get_storage;
    use crate::transport::{
        ConnectError, Connection, DisconnectError, RecvError, SendError, Transport,
    };

    /// Test that a circuit creation creates the correct connections and sends the appropriate
    /// messages.
    #[test]
    fn test_propose_circuit() {
        let mesh = Mesh::new(4, 16);
        let network = Network::new(mesh.clone(), 0).unwrap();
        let mut transport = MockConnectingTransport::expect_connections(vec![
            Ok(Box::new(MockConnection::new())),
            Ok(Box::new(MockConnection::new())),
        ]);

        let mut storage = get_storage("memory", CircuitDirectory::new).unwrap();

        // set up key registry
        let pub_key = (0u8..33).collect::<Vec<_>>();
        let mut key_registry = StorageKeyRegistry::new("memory".to_string()).unwrap();
        let key_info = KeyInfo::builder(pub_key.clone(), "test_node".to_string()).build();
        key_registry.save_key(key_info).unwrap();

        let circuit_directory = storage.write().clone();
        let state = SplinterState::new("memory".to_string(), circuit_directory);

        let orchestrator_connection = transport
            .connect("inproc://admin-service")
            .expect("failed to create connection");
        let orchestrator = ServiceOrchestrator::new(vec![], orchestrator_connection, 1, 1, 1)
            .expect("failed to create orchestrator");

        let peer_connector = PeerConnector::new(network.clone(), Box::new(transport));
        let mut admin_service = AdminService::new(
            "test-node".into(),
            orchestrator,
            #[cfg(feature = "service-arg-validation")]
            HashMap::new(),
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
            Box::new(HashVerifier),
            Box::new(key_registry),
            Box::new(AllowAllKeyPermissionManager),
            "memory",
            None,
        )
        .expect("Service should have been created correctly");

        let (tx, rx) = channel();
        admin_service
            .start(&MockNetworkRegistry { tx })
            .expect("Service should have started correctly");

        let mut proposed_circuit = admin::Circuit::new();
        proposed_circuit.set_circuit_id("test_propose_circuit".into());
        proposed_circuit
            .set_authorization_type(admin::Circuit_AuthorizationType::TRUST_AUTHORIZATION);
        proposed_circuit.set_persistence(admin::Circuit_PersistenceType::ANY_PERSISTENCE);
        proposed_circuit.set_routes(admin::Circuit_RouteType::ANY_ROUTE);
        proposed_circuit.set_durability(admin::Circuit_DurabilityType::NO_DURABILITY);
        proposed_circuit.set_circuit_management_type("test app auth handler".into());

        proposed_circuit.set_members(protobuf::RepeatedField::from_vec(vec![
            splinter_node("test-node", "tcp://someplace:8000"),
            splinter_node("other-node", "tcp://otherplace:8000"),
        ]));
        proposed_circuit.set_roster(protobuf::RepeatedField::from_vec(vec![
            splinter_service("service-a", "sabre", "test-node"),
            splinter_service("service-b", "sabre", "other-node"),
        ]));

        let mut request = admin::CircuitCreateRequest::new();
        request.set_circuit(proposed_circuit.clone());

        let mut header = admin::CircuitManagementPayload_Header::new();
        header.set_action(admin::CircuitManagementPayload_Action::CIRCUIT_CREATE_REQUEST);
        header.set_requester(pub_key);
        header.set_requester_node_id("test_node".to_string());

        let mut payload = admin::CircuitManagementPayload::new();

        payload.set_signature(HashSigner.sign(&payload.header).unwrap());
        payload.set_header(protobuf::Message::write_to_bytes(&header).unwrap());
        payload.set_circuit_create_request(request);

        admin_service
            .admin_service_shared
            .lock()
            .unwrap()
            .propose_circuit(payload, "test".to_string())
            .expect("The proposal was not handled correctly");

        // wait up to 1 second for the proposed circuit message
        let recipient;
        let message;
        let start = Instant::now();
        loop {
            if Instant::now().duration_since(start) > Duration::from_secs(1) {
                panic!("Failed to receive proposed circuit message in time");
            }
            if let Ok((r, m)) = rx.recv_timeout(Duration::from_millis(100)) {
                recipient = r;
                message = m;
                break;
            }
        }

        assert_eq!("admin::other-node".to_string(), recipient);

        let mut admin_envelope: admin::AdminMessage =
            protobuf::parse_from_bytes(&message).expect("The message could not be parsed");

        assert_eq!(
            admin::AdminMessage_Type::PROPOSED_CIRCUIT,
            admin_envelope.get_message_type()
        );

        let mut envelope = admin_envelope
            .take_proposed_circuit()
            .take_circuit_payload();

        let header: admin::CircuitManagementPayload_Header =
            protobuf::parse_from_bytes(envelope.get_header()).unwrap();
        assert_eq!(
            admin::CircuitManagementPayload_Action::CIRCUIT_CREATE_REQUEST,
            header.get_action()
        );
        assert_eq!(
            proposed_circuit,
            envelope.take_circuit_create_request().take_circuit()
        );
    }

    fn splinter_node(node_id: &str, endpoint: &str) -> admin::SplinterNode {
        let mut node = admin::SplinterNode::new();
        node.set_node_id(node_id.into());
        node.set_endpoint(endpoint.into());
        node
    }

    fn splinter_service(
        service_id: &str,
        service_type: &str,
        allowed_node: &str,
    ) -> admin::SplinterService {
        let mut service = admin::SplinterService::new();
        service.set_service_id(service_id.into());
        service.set_service_type(service_type.into());
        service.set_allowed_nodes(vec![allowed_node.into()].into());
        service
    }

    struct MockNetworkRegistry {
        tx: Sender<(String, Vec<u8>)>,
    }

    impl ServiceNetworkRegistry for MockNetworkRegistry {
        fn connect(
            &self,
            _service_id: &str,
        ) -> Result<Box<dyn ServiceNetworkSender>, error::ServiceConnectionError> {
            Ok(Box::new(MockNetworkSender {
                tx: self.tx.clone(),
            }))
        }

        fn disconnect(&self, _service_id: &str) -> Result<(), error::ServiceDisconnectionError> {
            Ok(())
        }
    }

    #[derive(Clone)]
    struct MockNetworkSender {
        tx: Sender<(String, Vec<u8>)>,
    }

    impl ServiceNetworkSender for MockNetworkSender {
        fn send(&self, recipient: &str, message: &[u8]) -> Result<(), error::ServiceSendError> {
            self.tx
                .send((recipient.to_string(), message.to_vec()))
                .expect("Unable to send test message");

            Ok(())
        }

        fn send_and_await(
            &self,
            _recipient: &str,
            _message: &[u8],
        ) -> Result<Vec<u8>, error::ServiceSendError> {
            panic!("MockNetworkSender.send_and_await unexpectedly called")
        }

        fn reply(
            &self,
            _message_origin: &ServiceMessageContext,
            _message: &[u8],
        ) -> Result<(), error::ServiceSendError> {
            panic!("MockNetworkSender.reply unexpectedly called")
        }

        fn clone_box(&self) -> Box<dyn ServiceNetworkSender> {
            Box::new(self.clone())
        }
    }

    struct MockConnectingTransport {
        connection_results: VecDeque<Result<Box<dyn Connection>, ConnectError>>,
    }

    impl MockConnectingTransport {
        fn expect_connections(results: Vec<Result<Box<dyn Connection>, ConnectError>>) -> Self {
            Self {
                connection_results: results.into_iter().collect(),
            }
        }
    }

    impl Transport for MockConnectingTransport {
        fn accepts(&self, _: &str) -> bool {
            true
        }

        fn connect(&mut self, _: &str) -> Result<Box<dyn Connection>, ConnectError> {
            self.connection_results
                .pop_front()
                .expect("No test result added to mock")
        }

        fn listen(
            &mut self,
            _: &str,
        ) -> Result<Box<dyn crate::transport::Listener>, crate::transport::ListenError> {
            panic!("MockConnectingTransport.listen unexpectedly called")
        }
    }

    struct MockConnection {
        auth_response: Option<Vec<u8>>,
        evented: MockEvented,
    }

    impl MockConnection {
        fn new() -> Self {
            Self {
                auth_response: Some(authorized_response()),
                evented: MockEvented::new(),
            }
        }
    }

    impl Connection for MockConnection {
        fn send(&mut self, _message: &[u8]) -> Result<(), SendError> {
            Ok(())
        }

        fn recv(&mut self) -> Result<Vec<u8>, RecvError> {
            Ok(self.auth_response.take().unwrap_or_else(|| vec![]))
        }

        fn remote_endpoint(&self) -> String {
            String::from("MockConnection")
        }

        fn local_endpoint(&self) -> String {
            String::from("MockConnection")
        }

        fn disconnect(&mut self) -> Result<(), DisconnectError> {
            Ok(())
        }

        fn evented(&self) -> &dyn mio::Evented {
            &self.evented
        }
    }

    struct MockEvented {
        registration: mio::Registration,
        set_readiness: mio::SetReadiness,
    }

    impl MockEvented {
        fn new() -> Self {
            let (registration, set_readiness) = mio::Registration::new2();

            Self {
                registration,
                set_readiness,
            }
        }
    }

    impl mio::Evented for MockEvented {
        fn register(
            &self,
            poll: &mio::Poll,
            token: mio::Token,
            interest: mio::Ready,
            opts: mio::PollOpt,
        ) -> std::io::Result<()> {
            self.registration.register(poll, token, interest, opts)?;
            self.set_readiness.set_readiness(mio::Ready::readable())?;

            Ok(())
        }

        fn reregister(
            &self,
            poll: &mio::Poll,
            token: mio::Token,
            interest: mio::Ready,
            opts: mio::PollOpt,
        ) -> std::io::Result<()> {
            self.registration.reregister(poll, token, interest, opts)
        }

        fn deregister(&self, poll: &mio::Poll) -> std::io::Result<()> {
            poll.deregister(&self.registration)
        }
    }

    fn authorized_response() -> Vec<u8> {
        let msg_type = AuthorizationMessageType::AUTHORIZE;
        let auth_msg = AuthorizedMessage::new();
        let mut auth_msg_env = AuthorizationMessage::new();
        auth_msg_env.set_message_type(msg_type);
        auth_msg_env.set_payload(auth_msg.write_to_bytes().expect("unable to write to bytes"));

        let mut network_msg = NetworkMessage::new();
        network_msg.set_message_type(NetworkMessageType::AUTHORIZATION);
        network_msg.set_payload(
            auth_msg_env
                .write_to_bytes()
                .expect("unable to write to bytes"),
        );

        network_msg
            .write_to_bytes()
            .expect("unable to write to bytes")
    }

    struct MockAuthInquisitor;

    impl AuthorizationInquisitor for MockAuthInquisitor {
        fn is_authorized(&self, _: &str) -> bool {
            true
        }

        fn register_callback(
            &self,
            _: Box<dyn AuthorizationCallback>,
        ) -> Result<(), AuthorizationCallbackError> {
            // The callback won't be called, as this test implementation indicates that all nodes
            // are peered.
            Ok(())
        }
    }
}
