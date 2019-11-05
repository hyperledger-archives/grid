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

mod error;
mod rest_api;

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crossbeam_channel::{Receiver, Sender};
use protobuf::Message;

use crate::channel;
use crate::mesh::{Envelope, Mesh, RecvTimeoutError as MeshRecvTimeoutError};
use crate::network::reply::InboundRouter;
use crate::protos::authorization::{
    AuthorizationMessage, AuthorizationMessageType, ConnectRequest, ConnectRequest_HandshakeMode,
};
use crate::protos::circuit::{
    AdminDirectMessage, CircuitDirectMessage, CircuitError, CircuitMessage, CircuitMessageType,
    ServiceConnectResponse, ServiceDisconnectResponse,
};
use crate::protos::network::{NetworkMessage, NetworkMessageType};
use crate::service::{
    Service, ServiceFactory, ServiceMessageContext, StandardServiceNetworkRegistry,
};
use crate::transport::Connection;

pub use self::error::{
    InitializeServiceError, ListServicesError, NewOrchestratorError, OrchestratorError,
    ShutdownServiceError,
};

// Recv timeout in secs
const TIMEOUT_SEC: u64 = 2;

/// Identifies a unique service instance from the perspective of the orchestrator
#[derive(Clone, Eq, Hash, PartialEq, Debug)]
pub struct ServiceDefinition {
    pub circuit: String,
    pub service_id: String,
    pub service_type: String,
}

/// Stores a service and other structures that are used to manage it
struct ManagedService {
    pub service: Box<dyn Service>,
    pub registry: StandardServiceNetworkRegistry,
}

/// The `ServiceOrchestrator` manages initialization and shutdown of services.
pub struct ServiceOrchestrator {
    /// A (ServiceDefinition, ManagedService) map
    services: Arc<Mutex<HashMap<ServiceDefinition, ManagedService>>>,
    /// Factories used to create new services.
    service_factories: Vec<Box<dyn ServiceFactory>>,
    supported_service_types: Vec<String>,
    /// `network_sender` and `inbound_router` are used to create services' senders.
    network_sender: Sender<Vec<u8>>,
    inbound_router: InboundRouter<CircuitMessageType>,
    /// `running` and `join_handles` are used to shutdown the orchestrator's background threads
    running: Arc<AtomicBool>,
    join_handles: JoinHandles<Result<(), OrchestratorError>>,
}

impl ServiceOrchestrator {
    /// Create a new `ServiceOrchestrator`. This starts up 3 threads for relaying messages to and
    /// from services.
    pub fn new(
        service_factories: Vec<Box<dyn ServiceFactory>>,
        connection: Box<dyn Connection>,
        incoming_capacity: usize,
        outgoing_capacity: usize,
        channel_capacity: usize,
    ) -> Result<Self, NewOrchestratorError> {
        let services = Arc::new(Mutex::new(HashMap::new()));
        let mesh = Mesh::new(incoming_capacity, outgoing_capacity);
        let mesh_id = mesh
            .add(connection)
            .map_err(|err| NewOrchestratorError(Box::new(err)))?;
        let (network_sender, network_receiver) = crossbeam_channel::bounded(channel_capacity);
        let (inbound_sender, inbound_receiver) = crossbeam_channel::bounded(channel_capacity);
        let inbound_router = InboundRouter::new(Box::new(inbound_sender));
        let running = Arc::new(AtomicBool::new(true));

        // Start the authorization process with the splinter node.
        // If running over inproc connection, this is the only authorization message required.
        let connect_request =
            create_connect_request().map_err(|err| NewOrchestratorError(Box::new(err)))?;
        mesh.send(Envelope::new(mesh_id, connect_request))
            .map_err(|err| NewOrchestratorError(Box::new(err)))?;

        // Wait for the auth response.  Currently, this is on an inproc transport, so this will be
        // an "ok" response
        let _authed_response = mesh
            .recv()
            .map_err(|err| NewOrchestratorError(Box::new(err)))?;

        debug!("Orchestrator authorized");

        // Start thread that handles incoming messages from a splinter node.
        let incoming_mesh = mesh.clone();
        let incoming_running = running.clone();
        let incoming_router = inbound_router.clone();
        let incoming_join_handle = thread::Builder::new()
            .name("Orchestrator Incoming".into())
            .spawn(move || {
                if let Err(err) =
                    run_incoming_loop(incoming_mesh, incoming_running, incoming_router)
                {
                    error!(
                        "Terminating orchestrator incoming thread due to error: {}",
                        err
                    );
                    Err(err)
                } else {
                    Ok(())
                }
            })
            .map_err(|err| NewOrchestratorError(Box::new(err)))?;

        // Start thread that handles messages that do not have a matching correlation id.
        let inbound_services = services.clone();
        let inbound_running = running.clone();
        let inbound_join_handle = thread::Builder::new()
            .name("Orchestrator Inbound".into())
            .spawn(move || {
                if let Err(err) =
                    run_inbound_loop(inbound_services, inbound_receiver, inbound_running)
                {
                    error!(
                        "Terminating orchestrator inbound thread due to error: {}",
                        err
                    );
                    Err(err)
                } else {
                    Ok(())
                }
            })
            .map_err(|err| NewOrchestratorError(Box::new(err)))?;

        // Start thread that handles outgoing messages that need to be sent to the splinter node.
        let outgoing_mesh = mesh.clone();
        let outgoing_running = running.clone();
        let outgoing_join_handle = thread::Builder::new()
            .name("Orchestrator Outgoing".into())
            .spawn(move || {
                if let Err(err) =
                    run_outgoing_loop(outgoing_mesh, outgoing_running, network_receiver, mesh_id)
                {
                    error!(
                        "Terminating orchestrator outgoing thread due to error: {}",
                        err
                    );
                    Err(err)
                } else {
                    Ok(())
                }
            })
            .map_err(|err| NewOrchestratorError(Box::new(err)))?;

        let supported_service_types_vec = service_factories
            .iter()
            .map(|factory| factory.available_service_types().to_vec())
            .collect::<Vec<Vec<String>>>();

        let mut supported_service_types = vec![];
        for mut service_types in supported_service_types_vec {
            supported_service_types.append(&mut service_types);
        }

        Ok(Self {
            services,
            service_factories,
            supported_service_types,
            network_sender,
            inbound_router,
            running,
            join_handles: JoinHandles::new(vec![
                incoming_join_handle,
                inbound_join_handle,
                outgoing_join_handle,
            ]),
        })
    }

    /// Initialize (create and start) a service according to the specified definition. The
    /// arguments provided must match those required to create the service.
    pub fn initialize_service(
        &self,
        service_definition: ServiceDefinition,
        args: HashMap<String, String>,
    ) -> Result<(), InitializeServiceError> {
        // Get the factory that can create this service.
        let factory = self
            .service_factories
            .iter()
            .find(|factory| {
                factory
                    .available_service_types()
                    .contains(&service_definition.service_type)
            })
            .ok_or(InitializeServiceError::UnknownType)?;

        // Create the service.
        let mut service = factory.create(
            service_definition.service_id.clone(),
            service_definition.service_type.as_str(),
            service_definition.circuit.as_str(),
            args,
        )?;

        // Start the service.
        let registry = StandardServiceNetworkRegistry::new(
            service_definition.circuit.clone(),
            self.network_sender.clone(),
            self.inbound_router.clone(),
        );

        service
            .start(&registry)
            .map_err(|err| InitializeServiceError::InitializationFailed(Box::new(err)))?;

        // Save the service.
        self.services
            .lock()
            .map_err(|_| InitializeServiceError::LockPoisoned)?
            .insert(service_definition, ManagedService { service, registry });

        Ok(())
    }

    /// Shut down (stop and destroy) the specified service.
    pub fn shutdown_service(
        &self,
        service_definition: &ServiceDefinition,
    ) -> Result<(), ShutdownServiceError> {
        let ManagedService {
            mut service,
            registry,
        } = self
            .services
            .lock()
            .map_err(|_| ShutdownServiceError::LockPoisoned)?
            .remove(service_definition)
            .ok_or(ShutdownServiceError::UnknownService)?;

        service.stop(&registry).map_err(|err| {
            ShutdownServiceError::ShutdownFailed((service_definition.clone(), Box::new(err)))
        })?;
        service.destroy().map_err(|err| {
            ShutdownServiceError::ShutdownFailed((service_definition.clone(), Box::new(err)))
        })?;

        Ok(())
    }

    /// List services managed by this `ServiceOrchestrator`; filters may be provided to only show
    /// services on specified circuit(s) and of given service type(s).
    pub fn list_services(
        &self,
        circuits: Vec<String>,
        service_types: Vec<String>,
    ) -> Result<Vec<ServiceDefinition>, ListServicesError> {
        Ok(self
            .services
            .lock()
            .map_err(|_| ListServicesError::LockPoisoned)?
            .iter()
            .filter_map(|(service, _)| {
                if (circuits.is_empty() || circuits.contains(&service.circuit))
                    && (service_types.is_empty() || service_types.contains(&service.service_type))
                {
                    Some(service)
                } else {
                    None
                }
            })
            .cloned()
            .collect())
    }

    pub fn supported_service_types(&self) -> &[String] {
        &self.supported_service_types
    }

    pub fn destroy(self) -> Result<(), OrchestratorError> {
        let mut services = self
            .services
            .lock()
            .map_err(|_| OrchestratorError::LockPoisoned)?;

        for (_, managed_service) in services.drain() {
            let ManagedService {
                mut service,
                registry,
            } = managed_service;
            service
                .stop(&registry)
                .map_err(|err| OrchestratorError::Internal(Box::new(err)))?;
            service
                .destroy()
                .map_err(|err| OrchestratorError::Internal(Box::new(err)))?;
        }

        self.running.store(false, Ordering::SeqCst);

        match self.join_handles.join_all() {
            Ok(results) => results.iter().for_each(|res| {
                if let Err(err) = res {
                    error!(
                        "Orchestrator background thread failed while running: {}",
                        err
                    )
                }
            }),
            Err(err) => error!(
                "Orchestrator failed to join background thread(s) cleanly: {:?}",
                err
            ),
        };
        Ok(())
    }
}

pub struct JoinHandles<T> {
    join_handles: Vec<JoinHandle<T>>,
}

impl<T> JoinHandles<T> {
    fn new(join_handles: Vec<JoinHandle<T>>) -> Self {
        Self { join_handles }
    }

    pub fn join_all(self) -> thread::Result<Vec<T>> {
        let mut res = Vec::with_capacity(self.join_handles.len());

        for jh in self.join_handles.into_iter() {
            res.push(jh.join()?);
        }

        Ok(res)
    }
}

/// Helper function to build a ConnectRequest
fn create_connect_request() -> Result<Vec<u8>, protobuf::ProtobufError> {
    let mut connect_request = ConnectRequest::new();
    connect_request.set_handshake_mode(ConnectRequest_HandshakeMode::UNIDIRECTIONAL);

    let mut auth_msg_env = AuthorizationMessage::new();
    auth_msg_env.set_message_type(AuthorizationMessageType::CONNECT_REQUEST);
    auth_msg_env.set_payload(connect_request.write_to_bytes()?);

    let mut network_msg = NetworkMessage::new();
    network_msg.set_message_type(NetworkMessageType::AUTHORIZATION);
    network_msg.set_payload(auth_msg_env.write_to_bytes()?);

    network_msg.write_to_bytes()
}

pub fn run_incoming_loop(
    incoming_mesh: Mesh,
    incoming_running: Arc<AtomicBool>,
    mut inbound_router: InboundRouter<CircuitMessageType>,
) -> Result<(), OrchestratorError> {
    while incoming_running.load(Ordering::SeqCst) {
        let timeout = Duration::from_secs(TIMEOUT_SEC);
        let message_bytes = match incoming_mesh.recv_timeout(timeout) {
            Ok(envelope) => envelope.take_payload(),
            Err(MeshRecvTimeoutError::Timeout) => continue,
            Err(MeshRecvTimeoutError::Disconnected) => {
                error!("Mesh Disconnected");
                break;
            }
        };

        let msg: NetworkMessage = protobuf::parse_from_bytes(&message_bytes)
            .map_err(|err| OrchestratorError::Internal(Box::new(err)))?;

        // if a service is waiting on a reply the inbound router will
        // route back the reponse to the service based on the correlation id in
        // the message, otherwise it will be sent to the inbound thread
        match msg.get_message_type() {
            NetworkMessageType::CIRCUIT => {
                let mut circuit_msg: CircuitMessage =
                    protobuf::parse_from_bytes(&msg.get_payload())
                        .map_err(|err| OrchestratorError::Internal(Box::new(err)))?;

                match circuit_msg.get_message_type() {
                    CircuitMessageType::ADMIN_DIRECT_MESSAGE => {
                        let admin_direct_message: AdminDirectMessage =
                            protobuf::parse_from_bytes(circuit_msg.get_payload())
                                .map_err(|err| OrchestratorError::Internal(Box::new(err)))?;
                        inbound_router
                            .route(
                                admin_direct_message.get_correlation_id(),
                                Ok((
                                    CircuitMessageType::ADMIN_DIRECT_MESSAGE,
                                    circuit_msg.take_payload(),
                                )),
                            )
                            .map_err(|err| OrchestratorError::Internal(Box::new(err)))?;
                    }
                    CircuitMessageType::CIRCUIT_DIRECT_MESSAGE => {
                        let direct_message: CircuitDirectMessage =
                            protobuf::parse_from_bytes(circuit_msg.get_payload())
                                .map_err(|err| OrchestratorError::Internal(Box::new(err)))?;
                        inbound_router
                            .route(
                                direct_message.get_correlation_id(),
                                Ok((
                                    CircuitMessageType::CIRCUIT_DIRECT_MESSAGE,
                                    circuit_msg.take_payload(),
                                )),
                            )
                            .map_err(|err| OrchestratorError::Internal(Box::new(err)))?;
                    }
                    CircuitMessageType::SERVICE_CONNECT_RESPONSE => {
                        let response: ServiceConnectResponse =
                            protobuf::parse_from_bytes(circuit_msg.get_payload())
                                .map_err(|err| OrchestratorError::Internal(Box::new(err)))?;
                        inbound_router
                            .route(
                                response.get_correlation_id(),
                                Ok((
                                    CircuitMessageType::SERVICE_CONNECT_RESPONSE,
                                    circuit_msg.take_payload(),
                                )),
                            )
                            .map_err(|err| OrchestratorError::Internal(Box::new(err)))?;
                    }
                    CircuitMessageType::SERVICE_DISCONNECT_RESPONSE => {
                        let response: ServiceDisconnectResponse =
                            protobuf::parse_from_bytes(circuit_msg.get_payload())
                                .map_err(|err| OrchestratorError::Internal(Box::new(err)))?;
                        inbound_router
                            .route(
                                response.get_correlation_id(),
                                Ok((
                                    CircuitMessageType::SERVICE_DISCONNECT_RESPONSE,
                                    circuit_msg.take_payload(),
                                )),
                            )
                            .map_err(|err| OrchestratorError::Internal(Box::new(err)))?;
                    }
                    CircuitMessageType::CIRCUIT_ERROR_MESSAGE => {
                        let response: CircuitError =
                            protobuf::parse_from_bytes(circuit_msg.get_payload())
                                .map_err(|err| OrchestratorError::Internal(Box::new(err)))?;
                        warn!("Received circuit error message {:?}", response);
                    }
                    msg_type => warn!("Received unimplemented message: {:?}", msg_type),
                }
            }
            NetworkMessageType::NETWORK_HEARTBEAT => trace!("Received network heartbeat"),
            _ => warn!("Received unimplemented message"),
        }
    }

    Ok(())
}

fn run_inbound_loop(
    services: Arc<Mutex<HashMap<ServiceDefinition, ManagedService>>>,
    inbound_receiver: Receiver<Result<(CircuitMessageType, Vec<u8>), channel::RecvError>>,
    inbound_running: Arc<AtomicBool>,
) -> Result<(), OrchestratorError> {
    let timeout = Duration::from_secs(TIMEOUT_SEC);
    while inbound_running.load(Ordering::SeqCst) {
        let service_message = match inbound_receiver.recv_timeout(timeout) {
            Ok(msg) => msg,
            Err(crossbeam_channel::RecvTimeoutError::Timeout) => continue,
            Err(err) => {
                debug!("inbound sender dropped; ending inbound message thread");
                return Err(OrchestratorError::Internal(Box::new(err)));
            }
        }
        .map_err(|err| OrchestratorError::Internal(Box::new(err)))?;

        match service_message {
            (CircuitMessageType::ADMIN_DIRECT_MESSAGE, msg) => {
                let mut admin_direct_message: AdminDirectMessage = protobuf::parse_from_bytes(&msg)
                    .map_err(|err| OrchestratorError::Internal(Box::new(err)))?;

                let services = services
                    .lock()
                    .map_err(|_| OrchestratorError::LockPoisoned)?;

                match services.iter().find_map(|(service_def, managed_service)| {
                    if service_def.circuit == admin_direct_message.get_circuit()
                        && service_def.service_id == admin_direct_message.get_recipient()
                    {
                        Some(&managed_service.service)
                    } else {
                        None
                    }
                }) {
                    Some(service) => {
                        let msg_context = ServiceMessageContext {
                            sender: admin_direct_message.take_sender(),
                            circuit: admin_direct_message.take_circuit(),
                            correlation_id: admin_direct_message.take_correlation_id(),
                        };

                        service
                            .handle_message(admin_direct_message.get_payload(), &msg_context)
                            .map_err(|err| OrchestratorError::Internal(Box::new(err)))?;
                    }
                    None => warn!(
                        "Service with id {} does not exist on circuit {}; ignoring message",
                        admin_direct_message.get_recipient(),
                        admin_direct_message.get_circuit(),
                    ),
                }
            }
            (CircuitMessageType::CIRCUIT_DIRECT_MESSAGE, msg) => {
                let mut circuit_direct_message: CircuitDirectMessage =
                    protobuf::parse_from_bytes(&msg)
                        .map_err(|err| OrchestratorError::Internal(Box::new(err)))?;

                let services = services
                    .lock()
                    .map_err(|_| OrchestratorError::LockPoisoned)?;

                match services.iter().find_map(|(service_def, managed_service)| {
                    if service_def.circuit == circuit_direct_message.get_circuit()
                        && service_def.service_id == circuit_direct_message.get_recipient()
                    {
                        Some(&managed_service.service)
                    } else {
                        None
                    }
                }) {
                    Some(service) => {
                        let msg_context = ServiceMessageContext {
                            sender: circuit_direct_message.take_sender(),
                            circuit: circuit_direct_message.take_circuit(),
                            correlation_id: circuit_direct_message.take_correlation_id(),
                        };

                        service
                            .handle_message(circuit_direct_message.get_payload(), &msg_context)
                            .map_err(|err| OrchestratorError::Internal(Box::new(err)))?;
                    }
                    None => warn!(
                        "Service with id {} does not exist on circuit {}; ignoring message",
                        circuit_direct_message.get_recipient(),
                        circuit_direct_message.get_circuit(),
                    ),
                }
            }
            (msg_type, _) => warn!(
                "Received message ({:?}) that does not have a correlation id",
                msg_type
            ),
        }
    }
    Ok(())
}

fn run_outgoing_loop(
    outgoing_mesh: Mesh,
    outgoing_running: Arc<AtomicBool>,
    outgoing_receiver: Receiver<Vec<u8>>,
    mesh_id: usize,
) -> Result<(), OrchestratorError> {
    while outgoing_running.load(Ordering::SeqCst) {
        let timeout = Duration::from_secs(TIMEOUT_SEC);
        let message_bytes = match outgoing_receiver.recv_timeout(timeout) {
            Ok(msg) => msg,
            Err(crossbeam_channel::RecvTimeoutError::Timeout) => continue,
            Err(err) => {
                error!("channel dropped while handling outgoing messages: {}", err);
                break;
            }
        };

        // Send message to splinter node
        outgoing_mesh
            .send(Envelope::new(mesh_id, message_bytes))
            .map_err(|err| OrchestratorError::Internal(Box::new(err)))?;
    }
    Ok(())
}
