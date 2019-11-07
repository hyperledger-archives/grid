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
use std::thread::{self, JoinHandle};
use std::time::Duration;

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

/// Helper macro for generating ServiceProcessorError::ProcessError
macro_rules! process_err {
    ($err:ident, $ctx_msg:expr) => {
        ServiceProcessorError::ProcessError($ctx_msg.into(), Box::new($err))
    };
    ($err:ident, $ctx_msg:tt, $($fmt_arg:tt)*) => {
        ServiceProcessorError::ProcessError(format!($ctx_msg, $($fmt_arg)*), Box::new($err))
    }
}

/// Helper macro for generating map_err functions that convert errors into
/// ServiceProcessorError::ProcessError values.
macro_rules! to_process_err {
    ($($arg:tt)*) => {
        |err| process_err!(err, $($arg)*)
    }
}

/// The ServiceProcessor handles the networking for services. This includes talking to the
/// splinter node, connecting for authorization, registering the services, and routing
/// direct messages to the correct service.
pub struct ServiceProcessor {
    shared_state: Arc<RwLock<SharedState>>,
    services: Vec<Box<dyn Service>>,
    mesh: Mesh,
    circuit: String,
    node_mesh_id: usize,
    network_sender: Sender<Vec<u8>>,
    network_receiver: Receiver<Vec<u8>>,
    running: Arc<AtomicBool>,
    inbound_router: InboundRouter<CircuitMessageType>,
    inbound_receiver: Receiver<Result<(CircuitMessageType, Vec<u8>), channel::RecvError>>,
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
            .map_err(|err| process_err!(err, "unable to add connection to mesh"))?;
        let (network_sender, network_receiver) = crossbeam_channel::bounded(channel_capacity);
        let (inbound_sender, inbound_receiver) = crossbeam_channel::bounded(channel_capacity);
        Ok(ServiceProcessor {
            shared_state: Arc::new(RwLock::new(SharedState {
                services: HashMap::new(),
                join_handles: vec![],
            })),
            services: vec![],
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
    pub fn add_service(&mut self, service: Box<dyn Service>) -> Result<(), ServiceProcessorError> {
        if self
            .services
            .iter()
            .any(|s| s.service_id() == service.service_id())
        {
            Err(ServiceProcessorError::AddServiceError(format!(
                "{} already exists",
                service.service_id()
            )))
        } else {
            self.services.push(service);

            Ok(())
        }
    }

    /// Once the service processor is started it will handle incoming messages from the splinter
    /// node and route it to a running service.
    ///
    /// Returns a ShutdownHandle and join_handles so the service can be properly shutdown.
    pub fn start(
        self,
    ) -> Result<
        (
            ShutdownHandle,
            JoinHandles<Result<(), ServiceProcessorError>>,
        ),
        ServiceProcessorError,
    > {
        // Starts the authorization process with the splinter node
        // If running over inproc connection, this is the only authorization message required
        let connect_request = create_connect_request()
            .map_err(|err| process_err!(err, "unable to create connect request"))?;
        self.mesh
            .send(Envelope::new(self.node_mesh_id, connect_request))
            .map_err(|err| process_err!(err, "unable to send connect request"))?;

        // Wait for the auth response.  Currently, this is on an inproc transport, so this will be
        // an "ok" response
        let _authed_response = self
            .mesh
            .recv()
            .map_err(|err| process_err!(err, "Unable to receive auth response"))?;

        for service in self.services.into_iter() {
            let mut shared_state = rwlock_write_unwrap!(self.shared_state);
            let service_id = service.service_id().to_string();

            let (send, recv) = crossbeam_channel::bounded(self.channel_capacity);
            let network_sender = self.network_sender.clone();
            let circuit = self.circuit.clone();
            let inbound_router = self.inbound_router.clone();
            let join_handle = thread::Builder::new()
                .name(format!("Service {}", service_id))
                .spawn(move || {
                    let service_id = service.service_id().to_string();
                    if let Err(err) =
                        run_service_loop(circuit, service, network_sender, recv, inbound_router)
                    {
                        error!("Terminating service {} due to error: {}", service_id, err);
                        Err(err)
                    } else {
                        Ok(())
                    }
                })?;
            shared_state.join_handles.push(join_handle);
            shared_state.services.insert(service_id.to_string(), send);
        }

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

                        if let Err(err) = process_incoming_msg(&message_bytes, &mut inbound_router)
                        {
                            error!("Unable to process message: {}", err);
                            continue;
                        }
                    }

                    Ok(())
                })?;

        let inbound_receiver = self.inbound_receiver;
        let inbound_running = self.running.clone();
        // Thread that handles messages that do not have a matching correlation id
        let inbound_join_handle: JoinHandle<Result<(), ServiceProcessorError>> =
            thread::Builder::new()
                .name("Handle message with correlation_id".into())
                .spawn(move || {
                    let timeout = Duration::from_secs(TIMEOUT_SEC);
                    while inbound_running.load(Ordering::SeqCst) {
                        let service_message = match inbound_receiver.recv_timeout(timeout) {
                            Ok(msg) => msg,
                            Err(crossbeam_channel::RecvTimeoutError::Timeout) => continue,
                            Err(err) => {
                                debug!("inbound sender dropped; ending inbound message thread");
                                return Err(process_err!(err, "inbound sender dropped"));
                            }
                        }
                        .map_err(to_process_err!("received service message error"))?;

                        if let Err(err) =
                            process_inbound_msg_with_correlation_id(service_message, &shared_state)
                        {
                            error!("Unable to process inbound message: {}", err);
                        }
                    }
                    Ok(())
                })?;

        let outgoing_mesh = self.mesh;
        let outgoing_running = self.running.clone();
        let outgoing_receiver = self.network_receiver;
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
                                error!("channel dropped while handling outgoing messages: {}", err);
                                break;
                            }
                        };

                        // Send message to splinter node
                        if let Err(err) =
                            outgoing_mesh.send(Envelope::new(node_mesh_id, message_bytes))
                        {
                            error!(
                                "Unable to send message via mesh to {}: {}",
                                node_mesh_id, err
                            );
                            continue;
                        }
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

        Ok((
            ShutdownHandle { do_shutdown },
            JoinHandles::new(vec![
                incoming_join_handle,
                outgoing_join_handle,
                inbound_join_handle,
            ]),
        ))
    }
}

fn process_incoming_msg(
    message_bytes: &[u8],
    inbound_router: &mut InboundRouter<CircuitMessageType>,
) -> Result<(), ServiceProcessorError> {
    let msg: NetworkMessage = protobuf::parse_from_bytes(message_bytes)
        .map_err(to_process_err!("unable parse network message"))?;

    // if a service is waiting on a reply the inbound router will
    // route back the reponse to the service based on the correlation id in
    // the message, otherwise it will be sent to the inbound thread
    match msg.get_message_type() {
        NetworkMessageType::CIRCUIT => {
            let mut circuit_msg: CircuitMessage = protobuf::parse_from_bytes(&msg.get_payload())
                .map_err(to_process_err!("unable to parse circuit message"))?;

            match circuit_msg.get_message_type() {
                CircuitMessageType::ADMIN_DIRECT_MESSAGE => {
                    let admin_direct_message: AdminDirectMessage =
                        protobuf::parse_from_bytes(circuit_msg.get_payload())
                            .map_err(to_process_err!("unable to parse admin direct message"))?;
                    inbound_router
                        .route(
                            admin_direct_message.get_correlation_id(),
                            Ok((
                                CircuitMessageType::ADMIN_DIRECT_MESSAGE,
                                circuit_msg.take_payload(),
                            )),
                        )
                        .map_err(to_process_err!("unable to route message"))?;
                }
                CircuitMessageType::CIRCUIT_DIRECT_MESSAGE => {
                    let direct_message: CircuitDirectMessage =
                        protobuf::parse_from_bytes(circuit_msg.get_payload())
                            .map_err(to_process_err!("unable to parse circuit direct message"))?;
                    inbound_router
                        .route(
                            direct_message.get_correlation_id(),
                            Ok((
                                CircuitMessageType::CIRCUIT_DIRECT_MESSAGE,
                                circuit_msg.take_payload(),
                            )),
                        )
                        .map_err(to_process_err!("unable to route message"))?;
                }
                CircuitMessageType::SERVICE_CONNECT_RESPONSE => {
                    let response: ServiceConnectResponse =
                        protobuf::parse_from_bytes(circuit_msg.get_payload())
                            .map_err(to_process_err!("unable to parse service connect response"))?;
                    inbound_router
                        .route(
                            response.get_correlation_id(),
                            Ok((
                                CircuitMessageType::SERVICE_CONNECT_RESPONSE,
                                circuit_msg.take_payload(),
                            )),
                        )
                        .map_err(to_process_err!("unable to route message"))?;
                }
                CircuitMessageType::SERVICE_DISCONNECT_RESPONSE => {
                    let response: ServiceDisconnectResponse =
                        protobuf::parse_from_bytes(circuit_msg.get_payload()).map_err(|err| {
                            process_err!(err, "unable to parse service disconnect response")
                        })?;
                    inbound_router
                        .route(
                            response.get_correlation_id(),
                            Ok((
                                CircuitMessageType::SERVICE_DISCONNECT_RESPONSE,
                                circuit_msg.take_payload(),
                            )),
                        )
                        .map_err(to_process_err!("unable to route message"))?;
                }
                msg_type => warn!("Received unimplemented message: {:?}", msg_type),
            }
        }
        NetworkMessageType::NETWORK_HEARTBEAT => trace!("Received network heartbeat"),
        _ => warn!("Received unimplemented message"),
    }

    Ok(())
}

fn process_inbound_msg_with_correlation_id(
    service_message: (CircuitMessageType, Vec<u8>),
    shared_state: &Arc<RwLock<SharedState>>,
) -> Result<(), ServiceProcessorError> {
    match service_message {
        (CircuitMessageType::ADMIN_DIRECT_MESSAGE, msg) => {
            let admin_direct_message: AdminDirectMessage = protobuf::parse_from_bytes(&msg)
                .map_err(to_process_err!(
                    "unable to parse inbound admin direct message"
                ))?;

            handle_admin_direct_msg(admin_direct_message, &shared_state).map_err(
                to_process_err!("unable to handle inbound admin direct message"),
            )?;
        }
        (CircuitMessageType::CIRCUIT_DIRECT_MESSAGE, msg) => {
            let circuit_direct_message: CircuitDirectMessage = protobuf::parse_from_bytes(&msg)
                .map_err(to_process_err!(
                    "unable to parse inbound circuit direct message"
                ))?;

            handle_circuit_direct_msg(circuit_direct_message, &shared_state).map_err(
                to_process_err!("unable to handle inbound circuit direct message"),
            )?;
        }
        (CircuitMessageType::CIRCUIT_ERROR_MESSAGE, msg) => {
            let response: CircuitError = protobuf::parse_from_bytes(&msg)
                .map_err(to_process_err!("unable to parse circuit error message"))?;
            warn!("Received circuit error message {:?}", response);
        }
        (msg_type, _) => warn!(
            "Received message ({:?}) that does not have a correlation id",
            msg_type
        ),
    }
    Ok(())
}

pub struct ShutdownHandle {
    do_shutdown: Box<dyn Fn() -> Result<(), ServiceProcessorError> + Send>,
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

impl ShutdownHandle {
    pub fn shutdown(&self) -> Result<(), ServiceProcessorError> {
        (*self.do_shutdown)()
    }
}

fn run_service_loop(
    circuit: String,
    mut service: Box<dyn Service>,
    network_sender: Sender<Vec<u8>>,
    service_recv: Receiver<ProcessorMessage>,
    inbound_router: InboundRouter<CircuitMessageType>,
) -> Result<(), ServiceProcessorError> {
    info!("Starting Service: {}", service.service_id());
    let registry = StandardServiceNetworkRegistry::new(circuit, network_sender, inbound_router);
    service.start(&registry).map_err(to_process_err!(
        "unable to start service {}",
        service.service_id()
    ))?;

    loop {
        let service_message: ServiceMessage = match service_recv.recv() {
            Ok(ProcessorMessage::ServiceMessage(message)) => Ok(message),
            Ok(ProcessorMessage::Shutdown) => {
                info!("Shutting down {}", service.service_id());
                service
                    .stop(&registry)
                    .map_err(to_process_err!("unable to stop service"))?;
                service
                    .destroy()
                    .map_err(to_process_err!("unable to destroy service"))?;
                break;
            }
            Err(err) => Err(process_err!(err, "unable to receive service messages")),
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
                    .map_err(to_process_err!("unable to handle admin direct message"))?;
            }
            ServiceMessage::CircuitDirectMessage(mut direct_message) => {
                let msg_context = ServiceMessageContext {
                    sender: direct_message.take_sender(),
                    circuit: direct_message.take_circuit(),
                    correlation_id: direct_message.take_correlation_id(),
                };

                service
                    .handle_message(direct_message.get_payload(), &msg_context)
                    .map_err(to_process_err!("unable to handle circuit direct message"))?;
            }
        }
    }
    Ok(())
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
            .map_err(to_process_err!(
                "unable to send service (circuit direct) message"
            ))?;
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
            .map_err(to_process_err!(
                "unable to send service (admin direct) message"
            ))?;
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

    use std::any::Any;
    use std::thread;

    use crate::network::Network;
    use crate::protos::{
        authorization::AuthorizedMessage,
        circuit::{ServiceConnectRequest, ServiceConnectResponse_Status},
    };
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
        let network = Network::new(mesh.clone(), 0).unwrap();

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
            .unwrap();

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

        // Send authorized response
        network
            .send("service_processor", &authorized_response())
            .expect("Unable to send authorized response");

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
        let network = Network::new(mesh.clone(), 0).unwrap();

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
            .unwrap();

        // this part of the test mimics the splinter daemon sending message to the connected
        // service
        let connection = inproc_listener.accept().unwrap();
        network
            .add_peer("service_processor".to_string(), connection)
            .unwrap();

        // Receive connect request from service
        let auth_request = get_auth_msg(network.recv().unwrap().payload().to_vec());
        assert_eq!(
            auth_request.get_message_type(),
            AuthorizationMessageType::CONNECT_REQUEST
        );

        // Send authorized response
        network
            .send("service_processor", &authorized_response())
            .expect("Unable to send authorized response");

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
            let network_sender = service_registry.connect(self.service_id())?;
            self.network_sender = Some(network_sender);
            Ok(())
        }

        /// Stops the service
        fn stop(
            &mut self,
            service_registry: &dyn ServiceNetworkRegistry,
        ) -> Result<(), ServiceStopError> {
            service_registry.disconnect(self.service_id())?;
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
                        .unwrap();
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

        fn as_any(&self) -> &dyn Any {
            self
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
            let network_sender = service_registry.connect(self.service_id())?;
            self.network_sender = Some(network_sender);
            Ok(())
        }

        /// Stops the service
        fn stop(
            &mut self,
            service_registry: &dyn ServiceNetworkRegistry,
        ) -> Result<(), ServiceStopError> {
            service_registry.disconnect(self.service_id())?;
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
                        .unwrap();
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

        fn as_any(&self) -> &dyn Any {
            self
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
}
