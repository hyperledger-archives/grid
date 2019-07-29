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

use crossbeam_channel::{Receiver, Sender};
use protobuf::Message;

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use crate::channel;
use crate::mesh::{Envelope, Mesh, RecvTimeoutError as MeshRecvTimeoutError};
use crate::network::reply::InboundRouter;
use crate::protos::authorization::{
    AuthorizationMessage, AuthorizationMessageType, ConnectRequest, ConnectRequest_HandshakeMode,
};
use crate::protos::circuit::{
    AdminDirectMessage, CircuitDirectMessage, CircuitMessage, CircuitMessageType,
    ServiceConnectResponse, ServiceDisconnectResponse,
};
use crate::protos::network::{NetworkMessage, NetworkMessageType};
use crate::service::error::ServiceProcessorError;
use crate::service::registry::StandardServiceNetworkRegistry;
use crate::service::sender::{ProcessorMessage, ServiceMessage};
use crate::service::{Service, ServiceMessageContext};
use crate::transport::Connection;
use crate::{rwlock_read_unwrap, rwlock_write_unwrap};

// Recv timeout in secs
const TIMEOUT_SEC: u64 = 2;

/// State that can be passed between threads.
/// Includes the service senders and join_handles for the service threads.
struct SharedState {
    pub services: HashMap<String, Sender<ProcessorMessage>>,
    pub join_handles: Vec<JoinHandle<Result<(), ServiceProcessorError>>>,
}

/// The ServiceProcessor handles the networking for services. This includes talking to the
/// splinter node, connecting for authorization, registering the services, and routing
/// direct messages to the correct service.
pub struct ServiceProcessor {
    shared_state: Arc<RwLock<SharedState>>,
    mesh: Mesh,
    circuit: String,
    node_mesh_id: usize,
    network_sender: Sender<Vec<u8>>,
    network_receiver: Receiver<Vec<u8>>,
    running: Arc<AtomicBool>,
    inbound_router: InboundRouter<ServiceMessage>,
    inbound_receiver: Receiver<Result<ServiceMessage, channel::RecvError>>,
    channel_capacity: usize,
}

impl ServiceProcessor {
    pub fn new(
        connection: Box<dyn Connection>,
        circuit: String,
        incoming_capacity: usize,
        outgoing_capacity: usize,
        channel_capacity: usize,
        running: Arc<AtomicBool>,
    ) -> Result<Self, ServiceProcessorError> {
        let mesh = Mesh::new(incoming_capacity, outgoing_capacity);
        let node_mesh_id = mesh
            .add(connection)
            .map_err(|err| ServiceProcessorError::ProcessError(Box::new(err)))?;
        let (network_sender, network_receiver) = crossbeam_channel::bounded(channel_capacity);
        let (inbound_sender, inbound_receiver) = crossbeam_channel::bounded(channel_capacity);
        Ok(ServiceProcessor {
            shared_state: Arc::new(RwLock::new(SharedState {
                services: HashMap::new(),
                join_handles: vec![],
            })),
            mesh,
            circuit,
            node_mesh_id,
            network_sender,
            network_receiver,
            running,
            inbound_router: InboundRouter::new(Box::new(inbound_sender)),
            inbound_receiver,
            channel_capacity,
        })
    }

    /// add_service takes a Service and sets up the thread that the service will run in.
    /// The service will be started, including registration and then messages are routed to the
    /// the services using a channel.
    pub fn add_service(
        &mut self,
        mut service: Box<dyn Service>,
    ) -> Result<(), ServiceProcessorError> {
        let mut shared_state = rwlock_write_unwrap!(self.shared_state);
        let service_id = service.service_id().to_string();

        let (send, recv) = crossbeam_channel::bounded(self.channel_capacity);
        let network_sender = self.network_sender.clone();
        let circuit = self.circuit.clone();
        let inbound_router = self.inbound_router.clone();
        let join_handle = thread::Builder::new()
            .name(format!("Service {}", service_id))
            .spawn(move || {
                info!("Starting Service: {}", service.service_id());
                let registry = StandardServiceNetworkRegistry::new(
                    circuit.to_string(),
                    network_sender.clone(),
                    inbound_router.clone(),
                );
                service
                    .start(&registry)
                    .map_err(|err| ServiceProcessorError::ProcessError(Box::new(err)))?;

                loop {
                    let service_message: ServiceMessage = match recv.recv() {
                        Ok(ProcessorMessage::ServiceMessage(message)) => Ok(message),
                        Ok(ProcessorMessage::Shutdown) => {
                            info!("Shutting down {}", service.service_id());
                            service.stop(&registry).map_err(|err| {
                                ServiceProcessorError::ProcessError(Box::new(err))
                            })?;
                            service.destroy().map_err(|err| {
                                ServiceProcessorError::ProcessError(Box::new(err))
                            })?;
                            break;
                        }
                        Err(err) => Err(ServiceProcessorError::ProcessError(Box::new(err))),
                    }?;

                    match service_message {
                        ServiceMessage::AdminDirectMessage(mut admin_direct_message) => {
                            let msg_context = ServiceMessageContext {
                                sender: admin_direct_message.take_sender(),
                                circuit: admin_direct_message.take_circuit(),
                                correlation_id: admin_direct_message.take_correlation_id(),
                            };

                            service
                                .handle_message(admin_direct_message.get_payload(), &msg_context)
                                .map_err(|err| {
                                    ServiceProcessorError::ProcessError(Box::new(err))
                                })?;
                        }
                        ServiceMessage::CircuitDirectMessage(mut direct_message) => {
                            let msg_context = ServiceMessageContext {
                                sender: direct_message.take_sender(),
                                circuit: direct_message.take_circuit(),
                                correlation_id: direct_message.take_correlation_id(),
                            };

                            service
                                .handle_message(direct_message.get_payload(), &msg_context)
                                .map_err(|err| {
                                    ServiceProcessorError::ProcessError(Box::new(err))
                                })?;
                        }
                        other_msg => warn!(
                            "{} received unexpected message type: {:?}",
                            service.service_id(),
                            other_msg
                        ),
                    }
                }
                Ok(())
            })?;
        shared_state.join_handles.push(join_handle);

        if shared_state.services.get(&service_id).is_none() {
            shared_state.services.insert(service_id.to_string(), send);
            Ok(())
        } else {
            Err(ServiceProcessorError::AddServiceError(format!(
                "{} already exists",
                service_id
            )))
        }
    }

    /// Once the service processor is started it will handle incoming messages from the splinter
    /// node and route it to a running service.
    ///
    /// Returns a ShutdownHandle and join_handles so the service can be properly shutdown.
    pub fn start(self) -> Result<ShutdownHandle, ServiceProcessorError> {
        // Starts the authorization process with the splinter node
        // If running over inproc connection, this is the only authroization message required
        let connect_request = create_connect_request()
            .map_err(|err| ServiceProcessorError::ProcessError(Box::new(err)))?;;
        self.mesh
            .send(Envelope::new(self.node_mesh_id, connect_request))
            .map_err(|err| ServiceProcessorError::ProcessError(Box::new(err)))?;

        let incoming_mesh = self.mesh.clone();
        let shared_state = self.shared_state.clone();
        let incoming_running = self.running.clone();
        let mut inbound_router = self.inbound_router.clone();
        // Thread to handle incoming messages from a splinter node.
        let incoming_join_handle: JoinHandle<Result<(), ServiceProcessorError>> =
            thread::Builder::new()
                .name("ServiceProcessor incoming".into())
                .spawn(move || {
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
                            .map_err(|err| ServiceProcessorError::ProcessError(Box::new(err)))?;

                        // if a service is waiting on a reply the inbound router will
                        // route back the reponse to the service based on the correlation id in
                        // the message, otherwise it will be sent to the inbound thread
                        match msg.get_message_type() {
                            NetworkMessageType::CIRCUIT => {
                                let circuit_msg: CircuitMessage =
                                    protobuf::parse_from_bytes(&msg.get_payload()).map_err(
                                        |err| ServiceProcessorError::ProcessError(Box::new(err)),
                                    )?;

                                match circuit_msg.get_message_type() {
                                    CircuitMessageType::ADMIN_DIRECT_MESSAGE => {
                                        let admin_direct_message: AdminDirectMessage =
                                            protobuf::parse_from_bytes(&circuit_msg.get_payload())
                                                .map_err(|err| {
                                                    ServiceProcessorError::ProcessError(Box::new(
                                                        err,
                                                    ))
                                                })?;
                                        inbound_router
                                            .route(Ok(ServiceMessage::AdminDirectMessage(
                                                admin_direct_message,
                                            )))
                                            .map_err(|err| {
                                                ServiceProcessorError::ProcessError(Box::new(err))
                                            })?;
                                    }
                                    CircuitMessageType::CIRCUIT_DIRECT_MESSAGE => {
                                        let direct_message: CircuitDirectMessage =
                                            protobuf::parse_from_bytes(&circuit_msg.get_payload())
                                                .map_err(|err| {
                                                    ServiceProcessorError::ProcessError(Box::new(
                                                        err,
                                                    ))
                                                })?;
                                        inbound_router
                                            .route(Ok(ServiceMessage::CircuitDirectMessage(
                                                direct_message,
                                            )))
                                            .map_err(|err| {
                                                ServiceProcessorError::ProcessError(Box::new(err))
                                            })?;
                                    }
                                    CircuitMessageType::SERVICE_CONNECT_RESPONSE => {
                                        let response: ServiceConnectResponse =
                                            protobuf::parse_from_bytes(&circuit_msg.get_payload())
                                                .map_err(|err| {
                                                    ServiceProcessorError::ProcessError(Box::new(
                                                        err,
                                                    ))
                                                })?;
                                        inbound_router
                                            .route(Ok(ServiceMessage::ServiceConnectResponse(
                                                response,
                                            )))
                                            .map_err(|err| {
                                                ServiceProcessorError::ProcessError(Box::new(err))
                                            })?;
                                    }
                                    CircuitMessageType::SERVICE_DISCONNECT_RESPONSE => {
                                        let response: ServiceDisconnectResponse =
                                            protobuf::parse_from_bytes(&circuit_msg.get_payload())
                                                .map_err(|err| {
                                                    ServiceProcessorError::ProcessError(Box::new(
                                                        err,
                                                    ))
                                                })?;
                                        inbound_router
                                            .route(Ok(ServiceMessage::ServiceDisconnectResponse(
                                                response,
                                            )))
                                            .map_err(|err| {
                                                ServiceProcessorError::ProcessError(Box::new(err))
                                            })?;
                                    }
                                    msg_type => {
                                        warn!("Received unimplemented message: {:?}", msg_type)
                                    }
                                }
                            }
                            _ => warn!("Received unimplemented message"),
                        }
                    }

                    Ok(())
                })?;

        let inbound_receiver = self.inbound_receiver.clone();
        let inbound_running = self.running.clone();
        // Thread that handles messages that do not have a matching correlation id
        let inbound_join_handle: JoinHandle<Result<(), ServiceProcessorError>> =
            thread::Builder::new()
                .name("Handle message with correlation_id".into())
                .spawn(move || {
                    let timeout = Duration::from_secs(TIMEOUT_SEC);
                    while inbound_running.load(Ordering::SeqCst) {
                        let service_message = inbound_receiver
                            .recv_timeout(timeout)
                            .map_err(|err| ServiceProcessorError::ProcessError(Box::new(err)))?
                            .map_err(|err| ServiceProcessorError::ProcessError(Box::new(err)))?;
                        match service_message {
                            ServiceMessage::AdminDirectMessage(msg) => {
                                handle_admin_direct_msg(msg, &shared_state).map_err(|err| {
                                    ServiceProcessorError::ProcessError(Box::new(err))
                                })?;
                            }
                            ServiceMessage::CircuitDirectMessage(msg) => {
                                handle_circuit_direct_msg(msg, &shared_state).map_err(|err| {
                                    ServiceProcessorError::ProcessError(Box::new(err))
                                })?;
                            }
                            _ => warn!("Received message that does not have a correlation id"),
                        }
                    }
                    Ok(())
                })?;

        let outgoing_mesh = self.mesh.clone();
        let outgoing_running = self.running.clone();
        let outgoing_receiver = self.network_receiver.clone();
        let node_mesh_id = self.node_mesh_id;

        // Thread that handles outgoing messages that need to be sent to the splinter node
        let outgoing_join_handle: JoinHandle<Result<(), ServiceProcessorError>> =
            thread::Builder::new()
                .name("ServiceProcessor outgoing".into())
                .spawn(move || {
                    while outgoing_running.load(Ordering::SeqCst) {
                        let timeout = Duration::from_secs(TIMEOUT_SEC);
                        let message_bytes = match outgoing_receiver.recv_timeout(timeout) {
                            Ok(msg) => msg,
                            Err(crossbeam_channel::RecvTimeoutError::Timeout) => continue,
                            Err(err) => {
                                error!("{}", err);
                                break;
                            }
                        };

                        // Send message to splinter node
                        outgoing_mesh
                            .send(Envelope::new(node_mesh_id, message_bytes))
                            .map_err(|err| ServiceProcessorError::ProcessError(Box::new(err)))?;
                    }
                    Ok(())
                })?;

        let shutdown_shared_state = self.shared_state.clone();
        // Creates the shutdown handle that will be called by the process starting up the
        // Service processor
        let do_shutdown = Box::new(move || {
            debug!("Shutting down service processor");
            let mut shared_state = rwlock_write_unwrap!(shutdown_shared_state);
            // send shutdown to the services and wait for join
            for (service_id, service_sender) in shared_state.services.iter() {
                info!("Shutting down {}", service_id);
                service_sender
                    .send(ProcessorMessage::Shutdown)
                    .map_err(|err| {
                        ServiceProcessorError::ShutdownError(format!(
                            "unable to send shutdown message: {:?}",
                            err
                        ))
                    })?;
            }

            while let Some(join_handle) = shared_state.join_handles.pop() {
                join_handle.join().map_err(|err| {
                    ServiceProcessorError::ShutdownError(format!(
                        "unable to cleanly join a Service thread: {:?}",
                        err
                    ))
                })??;
            }
            Ok(())
        });

        Ok(ShutdownHandle {
            do_shutdown,
            incoming_join_handle,
            outgoing_join_handle,
            inbound_join_handle,
        })
    }
}

pub struct ShutdownHandle {
    do_shutdown: Box<dyn Fn() -> Result<(), ServiceProcessorError>>,
    incoming_join_handle: JoinHandle<Result<(), ServiceProcessorError>>,
    outgoing_join_handle: JoinHandle<Result<(), ServiceProcessorError>>,
    inbound_join_handle: JoinHandle<Result<(), ServiceProcessorError>>,
}

impl ShutdownHandle {
    pub fn shutdown(self) -> Result<(), ServiceProcessorError> {
        (*self.do_shutdown)()?;

        self.incoming_join_handle.join().map_err(|err| {
            ServiceProcessorError::ShutdownError(format!(
                "unable to shutdown incoming thread: {:?}",
                err
            ))
        })??;
        self.outgoing_join_handle.join().map_err(|err| {
            ServiceProcessorError::ShutdownError(format!(
                "unable to shutdown outgoing thread: {:?}",
                err
            ))
        })??;
        self.inbound_join_handle.join().map_err(|err| {
            ServiceProcessorError::ShutdownError(format!(
                "unable to shutdown inbound thread: {:?}",
                err
            ))
        })??;

        Ok(())
    }
}

fn handle_circuit_direct_msg(
    direct_message: CircuitDirectMessage,
    shared_state: &Arc<RwLock<SharedState>>,
) -> Result<(), ServiceProcessorError> {
    let shared_state = rwlock_read_unwrap!(shared_state);

    if let Some(service_sender) = shared_state.services.get(direct_message.get_recipient()) {
        service_sender
            .send(ProcessorMessage::ServiceMessage(
                ServiceMessage::CircuitDirectMessage(direct_message),
            ))
            .map_err(|err| ServiceProcessorError::ProcessError(Box::new(err)))?;
    } else {
        warn!(
            "Service with id {} does not exist, ignoring message",
            direct_message.get_recipient()
        );
    }
    Ok(())
}

fn handle_admin_direct_msg(
    admin_direct_message: AdminDirectMessage,
    shared_state: &Arc<RwLock<SharedState>>,
) -> Result<(), ServiceProcessorError> {
    let shared_state = rwlock_read_unwrap!(shared_state);

    if let Some(service_sender) = shared_state
        .services
        .get(admin_direct_message.get_recipient())
    {
        service_sender
            .send(ProcessorMessage::ServiceMessage(
                ServiceMessage::AdminDirectMessage(admin_direct_message),
            ))
            .map_err(|err| ServiceProcessorError::ProcessError(Box::new(err)))?;
    } else {
        warn!(
            "Service with id {} does not exist, ignoring message",
            admin_direct_message.get_recipient()
        );
    }
    Ok(())
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

#[cfg(test)]
pub mod tests {
    use super::*;

    use std::thread;

    use crate::network::Network;
    use crate::protos::circuit::{ServiceConnectRequest, ServiceConnectResponse_Status};
    use crate::service::error::{
        ServiceDestroyError, ServiceError, ServiceStartError, ServiceStopError,
    };
    use crate::service::sender::create_message;
    use crate::service::{ServiceNetworkRegistry, ServiceNetworkSender};
    use crate::transport::inproc::InprocTransport;
    use crate::transport::Transport;

    #[test]
    // This test uses a MockService that will call the corresponding network_sender function.
    // Verifies that the ServiceProcessor sends a connect request, starts up the service, and
    // route the messages to the service, including routing through the inbound router when there
    // is a matching correlation id.
    fn standard_direct_message() {
        let mut transport = InprocTransport::default();
        let mut inproc_listener = transport.listen("internal").unwrap();
        let running = Arc::new(AtomicBool::new(true));
        let r = running.clone();

        let mesh = Mesh::new(512, 128);
        let network = Network::new(mesh.clone());

        thread::Builder::new()
            .name("standard_direct_message".to_string())
            .spawn(move || {
                let connection = transport.connect("internal").unwrap();
                let mut processor =
                    ServiceProcessor::new(connection, "alpha".to_string(), 3, 3, 3, running)
                        .unwrap();

                // Add MockService to the processor and start the processor.
                let service = MockService::new();
                processor.add_service(Box::new(service)).unwrap();
                let _ = processor.start().unwrap();
            })
            .unwrap();;

        // this part of the test mimics the splinter daemon sending message to the connected
        // service
        let connection = inproc_listener.accept().unwrap();
        network
            .add_peer("service_processor".to_string(), connection)
            .unwrap();

        // Receive connect request from service
        let auth_response = get_auth_msg(network.recv().unwrap().payload().to_vec());
        assert_eq!(
            auth_response.get_message_type(),
            AuthorizationMessageType::CONNECT_REQUEST
        );

        // Receive service connect request and respond with ServiceConnectionResposne with status
        // OK
        let mut service_request = get_service_connect(network.recv().unwrap().payload().to_vec());
        assert_eq!(service_request.get_service_id(), "mock_service");
        assert_eq!(service_request.get_circuit(), "alpha");

        let service_response = create_service_connect_response(
            service_request.take_correlation_id(),
            "alpha".to_string(),
        )
        .unwrap();
        network
            .send("service_processor", &service_response)
            .unwrap();

        // request the mock service sends a message without caring about correlation id
        let send_msg = create_circuit_direct_msg(b"send".to_vec()).unwrap();
        network.send("service_processor", &send_msg).unwrap();

        let send_response = get_circuit_direct_msg(network.recv().unwrap().payload().to_vec());
        assert_eq!(send_response.get_payload(), b"send_response");

        // request the mock service send_and_await a message and blocks until correlation id is
        // returned
        let send_and_await_msg = create_circuit_direct_msg(b"send_and_await".to_vec()).unwrap();
        network
            .send("service_processor", &send_and_await_msg)
            .unwrap();

        let mut waiting_response =
            get_circuit_direct_msg(network.recv().unwrap().payload().to_vec());
        assert_eq!(waiting_response.get_payload(), b"waiting for response");

        // respond to send_and_await
        let wait_response = create_circuit_direct_msg_with_correlation_id(
            b"respond to waiting".to_vec(),
            waiting_response.take_correlation_id(),
        )
        .unwrap();
        network.send("service_processor", &wait_response).unwrap();

        // reply to this provided message
        let reply_request = create_circuit_direct_msg_with_correlation_id(
            b"reply".to_vec(),
            "reply_correlation_id".to_string(),
        )
        .unwrap();
        network.send("service_processor", &reply_request).unwrap();

        let reply_response = get_circuit_direct_msg(network.recv().unwrap().payload().to_vec());
        assert_eq!(reply_response.get_payload(), b"reply response");
        assert_eq!(reply_response.get_correlation_id(), "reply_correlation_id");

        r.store(false, Ordering::SeqCst);
    }

    #[test]
    fn test_admin_direct_message() {
        let mut transport = InprocTransport::default();
        let mut inproc_listener = transport.listen("internal").unwrap();
        let running = Arc::new(AtomicBool::new(true));
        let r = running.clone();

        let mesh = Mesh::new(512, 128);
        let network = Network::new(mesh.clone());

        thread::Builder::new()
            .name("test_admin_direct_message".to_string())
            .spawn(move || {
                let connection = transport.connect("internal").unwrap();
                let mut processor =
                    ServiceProcessor::new(connection, "admin".to_string(), 3, 3, 3, running)
                        .unwrap();

                // Add MockService to the processor and start the processor.
                let service = MockAdminService::new();
                processor.add_service(Box::new(service)).unwrap();
                let _ = processor.start().unwrap();
            })
            .unwrap();;

        // this part of the test mimics the splinter daemon sending message to the connected
        // service
        let connection = inproc_listener.accept().unwrap();
        network
            .add_peer("service_processor".to_string(), connection)
            .unwrap();;

        // Receive connect request from service
        let auth_response = get_auth_msg(network.recv().unwrap().payload().to_vec());
        assert_eq!(
            auth_response.get_message_type(),
            AuthorizationMessageType::CONNECT_REQUEST
        );

        // Receive service connect request and respond with ServiceConnectionResposne with status
        // OK
        let mut service_request = get_service_connect(network.recv().unwrap().payload().to_vec());
        assert_eq!(service_request.get_service_id(), "mock_service");
        assert_eq!(service_request.get_circuit(), "admin");

        let service_response = create_service_connect_response(
            service_request.take_correlation_id(),
            "admin".to_string(),
        )
        .unwrap();
        network
            .send("service_processor", &service_response)
            .unwrap();

        // request the mock service sends a message without caring about correlation id
        let send_msg = create_admin_direct_msg(b"send".to_vec()).unwrap();
        network.send("service_processor", &send_msg).unwrap();

        let send_response = get_admin_direct_msg(network.recv().unwrap().payload().to_vec());
        assert_eq!(send_response.get_payload(), b"send_response");

        // request the mock service send_and_await a message and blocks until correlation id is
        // returned
        let send_and_await_msg = create_admin_direct_msg(b"send_and_await".to_vec()).unwrap();
        network
            .send("service_processor", &send_and_await_msg)
            .unwrap();

        let mut waiting_response = get_admin_direct_msg(network.recv().unwrap().payload().to_vec());
        assert_eq!(waiting_response.get_payload(), b"waiting for response");

        // respond to send_and_await
        let wait_response = create_admin_direct_msg_with_correlation_id(
            b"respond to waiting".to_vec(),
            waiting_response.take_correlation_id(),
        )
        .unwrap();
        network.send("service_processor", &wait_response).unwrap();

        // reply to this provided message
        let reply_request = create_admin_direct_msg_with_correlation_id(
            b"reply".to_vec(),
            "reply_correlation_id".to_string(),
        )
        .unwrap();
        network.send("service_processor", &reply_request).unwrap();

        let reply_response = get_admin_direct_msg(network.recv().unwrap().payload().to_vec());
        assert_eq!(reply_response.get_payload(), b"reply response");
        assert_eq!(reply_response.get_correlation_id(), "reply_correlation_id");

        r.store(false, Ordering::SeqCst);
    }

    // Service that can be used for testing a standard service's functionality
    struct MockService {
        service_id: String,
        service_type: String,
        network_sender: Option<Box<dyn ServiceNetworkSender>>,
    }

    impl MockService {
        pub fn new() -> Self {
            MockService {
                service_id: "mock_service".to_string(),
                service_type: "mock".to_string(),
                network_sender: None,
            }
        }
    }

    impl Service for MockService {
        /// This service's id
        fn service_id(&self) -> &str {
            &self.service_id
        }

        /// This service's message family
        fn service_type(&self) -> &str {
            &self.service_type
        }

        /// Starts the service
        fn start(
            &mut self,
            service_registry: &dyn ServiceNetworkRegistry,
        ) -> Result<(), ServiceStartError> {
            let network_sender = service_registry
                .connect(self.service_id())
                .map_err(|err| ServiceStartError(Box::new(err)))?;
            self.network_sender = Some(network_sender);
            Ok(())
        }

        /// Stops the service
        fn stop(
            &mut self,
            service_registry: &dyn ServiceNetworkRegistry,
        ) -> Result<(), ServiceStopError> {
            service_registry
                .disconnect(self.service_id())
                .map_err(|err| ServiceStopError(Box::new(err)))?;
            Ok(())
        }

        /// Clean-up any resources before the service is removed.
        /// Consumes the service (which, given the use of dyn traits,
        /// this must take a boxed Service instance).
        fn destroy(self: Box<Self>) -> Result<(), ServiceDestroyError> {
            unimplemented!()
        }

        fn handle_message(
            &self,
            message_bytes: &[u8],
            message_context: &ServiceMessageContext,
        ) -> Result<(), ServiceError> {
            if message_bytes == b"send" {
                if let Some(network_sender) = &self.network_sender {
                    network_sender
                        .send(&message_context.sender, b"send_response")
                        .unwrap();;
                }
            } else if message_bytes == b"send_and_await" {
                if let Some(network_sender) = &self.network_sender {
                    let response = network_sender
                        .send_and_await(&message_context.sender, b"waiting for response")
                        .unwrap();
                    assert_eq!(response, b"respond to waiting");
                }
            } else if message_bytes == b"reply" {
                if let Some(network_sender) = &self.network_sender {
                    network_sender
                        .reply(&message_context, b"reply response")
                        .unwrap();;
                }
            }
            Ok(())
        }
    }

    // Service that can be used for testing a Admin service's functionality
    struct MockAdminService {
        service_id: String,
        service_type: String,
        network_sender: Option<Box<dyn ServiceNetworkSender>>,
    }

    impl MockAdminService {
        pub fn new() -> Self {
            MockAdminService {
                service_id: "mock_service".to_string(),
                service_type: "mock".to_string(),
                network_sender: None,
            }
        }
    }

    impl Service for MockAdminService {
        /// This service's id
        fn service_id(&self) -> &str {
            &self.service_id
        }

        /// This service's message family
        fn service_type(&self) -> &str {
            &self.service_type
        }

        /// Starts the service
        fn start(
            &mut self,
            service_registry: &dyn ServiceNetworkRegistry,
        ) -> Result<(), ServiceStartError> {
            let network_sender = service_registry
                .connect(self.service_id())
                .map_err(|err| ServiceStartError(Box::new(err)))?;
            self.network_sender = Some(network_sender);
            Ok(())
        }

        /// Stops the service
        fn stop(
            &mut self,
            service_registry: &dyn ServiceNetworkRegistry,
        ) -> Result<(), ServiceStopError> {
            service_registry
                .disconnect(self.service_id())
                .map_err(|err| ServiceStopError(Box::new(err)))?;
            Ok(())
        }

        /// Clean-up any resources before the service is removed.
        /// Consumes the service (which, given the use of dyn traits,
        /// this must take a boxed Service instance).
        fn destroy(self: Box<Self>) -> Result<(), ServiceDestroyError> {
            unimplemented!()
        }

        // for send and send_and_await the handle_message returns the bytes of an admin direct
        // message so it can choose which circuit the message is sent over
        fn handle_message(
            &self,
            message_bytes: &[u8],
            message_context: &ServiceMessageContext,
        ) -> Result<(), ServiceError> {
            if message_bytes == b"send" {
                if let Some(network_sender) = &self.network_sender {
                    network_sender
                        .send(&message_context.sender, b"send_response")
                        .unwrap();;
                }
            } else if message_bytes == b"send_and_await" {
                if let Some(network_sender) = &self.network_sender {
                    let response = network_sender
                        .send_and_await(&message_context.sender, b"waiting for response")
                        .unwrap();
                    assert_eq!(response, b"respond to waiting");
                }
            } else if message_bytes == b"reply" {
                if let Some(network_sender) = &self.network_sender {
                    network_sender
                        .reply(&message_context, b"reply response")
                        .unwrap();
                }
            }
            Ok(())
        }
    }

    fn create_circuit_direct_msg(payload: Vec<u8>) -> Result<Vec<u8>, protobuf::ProtobufError> {
        let mut direct_response = CircuitDirectMessage::new();
        direct_response.set_recipient("mock_service".to_string());
        direct_response.set_sender("service_a".to_string());
        direct_response.set_circuit("alpha".to_string());
        direct_response.set_payload(payload);
        let bytes = direct_response.write_to_bytes().unwrap();

        let msg = create_message(bytes, CircuitMessageType::CIRCUIT_DIRECT_MESSAGE)?;
        Ok(msg)
    }

    // this message routes back to the mock service so the message can be send and handled by the
    // same service during send_and_await and reply
    fn create_circuit_direct_msg_with_correlation_id(
        payload: Vec<u8>,
        correlation_id: String,
    ) -> Result<Vec<u8>, protobuf::ProtobufError> {
        let mut direct_response = CircuitDirectMessage::new();
        direct_response.set_recipient("mock_service".to_string());
        direct_response.set_sender("mock_service".to_string());
        direct_response.set_circuit("alpha".to_string());
        direct_response.set_correlation_id(correlation_id);
        direct_response.set_payload(payload);
        let bytes = direct_response.write_to_bytes().unwrap();

        let msg = create_message(bytes, CircuitMessageType::CIRCUIT_DIRECT_MESSAGE)?;
        Ok(msg)
    }

    fn create_admin_direct_msg(payload: Vec<u8>) -> Result<Vec<u8>, protobuf::ProtobufError> {
        let mut direct_response = AdminDirectMessage::new();
        direct_response.set_recipient("mock_service".to_string());
        direct_response.set_sender("service_a".to_string());
        direct_response.set_circuit("admin".to_string());
        direct_response.set_payload(payload);
        let bytes = direct_response.write_to_bytes().unwrap();

        let msg = create_message(bytes, CircuitMessageType::ADMIN_DIRECT_MESSAGE)?;
        Ok(msg)
    }

    // this message routes back to the mock service so the message can be send and handled by the
    // same service during send_and_await and reply
    fn create_admin_direct_msg_with_correlation_id(
        payload: Vec<u8>,
        correlation_id: String,
    ) -> Result<Vec<u8>, protobuf::ProtobufError> {
        let mut direct_response = AdminDirectMessage::new();
        direct_response.set_recipient("mock_service".to_string());
        direct_response.set_sender("mock_service".to_string());
        direct_response.set_circuit("admin".to_string());
        direct_response.set_correlation_id(correlation_id);
        direct_response.set_payload(payload);
        let bytes = direct_response.write_to_bytes().unwrap();

        let msg = create_message(bytes, CircuitMessageType::ADMIN_DIRECT_MESSAGE)?;
        Ok(msg)
    }

    fn create_service_connect_response(
        correlation_id: String,
        circuit: String,
    ) -> Result<Vec<u8>, protobuf::ProtobufError> {
        let mut response = ServiceConnectResponse::new();
        response.set_circuit(circuit);
        response.set_service_id("mock_service".to_string());
        response.set_status(ServiceConnectResponse_Status::OK);
        response.set_correlation_id(correlation_id);
        let bytes = response.write_to_bytes().unwrap();

        let msg = create_message(bytes, CircuitMessageType::SERVICE_CONNECT_RESPONSE)?;
        Ok(msg)
    }

    fn get_auth_msg(network_msg_bytes: Vec<u8>) -> AuthorizationMessage {
        let network_msg: NetworkMessage = protobuf::parse_from_bytes(&network_msg_bytes).unwrap();
        let auth_msg: AuthorizationMessage =
            protobuf::parse_from_bytes(network_msg.get_payload()).unwrap();
        auth_msg
    }

    fn get_service_connect(network_msg_bytes: Vec<u8>) -> ServiceConnectRequest {
        let network_msg: NetworkMessage = protobuf::parse_from_bytes(&network_msg_bytes).unwrap();
        let circuit_msg: CircuitMessage =
            protobuf::parse_from_bytes(network_msg.get_payload()).unwrap();
        let request: ServiceConnectRequest =
            protobuf::parse_from_bytes(circuit_msg.get_payload()).unwrap();
        request
    }

    fn get_circuit_direct_msg(network_msg_bytes: Vec<u8>) -> CircuitDirectMessage {
        let network_msg: NetworkMessage = protobuf::parse_from_bytes(&network_msg_bytes).unwrap();
        let circuit_msg: CircuitMessage =
            protobuf::parse_from_bytes(network_msg.get_payload()).unwrap();
        let direct_message: CircuitDirectMessage =
            protobuf::parse_from_bytes(circuit_msg.get_payload()).unwrap();
        direct_message
    }

    fn get_admin_direct_msg(network_msg_bytes: Vec<u8>) -> AdminDirectMessage {
        let network_msg: NetworkMessage = protobuf::parse_from_bytes(&network_msg_bytes).unwrap();
        let circuit_msg: CircuitMessage =
            protobuf::parse_from_bytes(network_msg.get_payload()).unwrap();
        let direct_message: AdminDirectMessage =
            protobuf::parse_from_bytes(circuit_msg.get_payload()).unwrap();
        direct_message
    }
}
