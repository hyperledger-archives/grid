// Copyright 2018 Cargill Incorporated
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
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

use crossbeam_channel;

use crate::node_registry::yaml::YamlNodeRegistry;
use libsplinter::admin::AdminService;
use libsplinter::circuit::directory::CircuitDirectory;
use libsplinter::circuit::handlers::{
    AdminDirectMessageHandler, CircuitDirectMessageHandler, CircuitErrorHandler,
    CircuitMessageHandler, ServiceConnectForwardHandler, ServiceConnectRequestHandler,
    ServiceDisconnectForwardHandler, ServiceDisconnectRequestHandler,
};
use libsplinter::circuit::SplinterState;
use libsplinter::mesh::Mesh;
use libsplinter::network::auth::handlers::{
    create_authorization_dispatcher, AuthorizationMessageHandler, NetworkAuthGuardHandler,
};
use libsplinter::network::auth::AuthorizationManager;
use libsplinter::network::dispatch::{DispatchLoop, DispatchMessage, Dispatcher};
use libsplinter::network::handlers::NetworkEchoHandler;
use libsplinter::network::peer::PeerConnector;
use libsplinter::network::sender::{NetworkMessageSender, SendRequest};
use libsplinter::network::{
    ConnectionError, Network, PeerUpdateError, RecvTimeoutError, SendError,
};
use libsplinter::node_registry::NodeRegistry;
use libsplinter::orchestrator::ServiceOrchestrator;
use libsplinter::protos::authorization::AuthorizationMessageType;
use libsplinter::protos::circuit::CircuitMessageType;
use libsplinter::protos::network::{NetworkMessage, NetworkMessageType};
use libsplinter::rest_api::{
    Method, Resource, RestApiBuilder, RestApiServerError, RestResourceProvider,
};
use libsplinter::rwlock_read_unwrap;
use libsplinter::service::scabbard::ScabbardFactory;
use libsplinter::service::{self, Service, ServiceProcessor};
use libsplinter::storage::get_storage;
use libsplinter::transport::{
    inproc::InprocTransport, multi::MultiTransport, AcceptError, ConnectError, Incoming,
    ListenError, Transport,
};

use crate::registry_config::{RegistryConfig, RegistryConfigBuilder, RegistryConfigError};
use crate::routes;

// Recv timeout in secs
const TIMEOUT_SEC: u64 = 2;
const ADMIN_SERVICE_ADDRESS: &str = "inproc://admin-service";

pub struct SplinterDaemon {
    storage_location: String,
    service_endpoint: String,
    network_endpoint: String,
    initial_peers: Vec<String>,
    network: Network,
    node_id: String,
    rest_api_endpoint: String,
    registry_config: RegistryConfig,
}

impl SplinterDaemon {
    pub fn start(&mut self, transport: Box<dyn Transport + Send>) -> Result<(), StartError> {
        let inproc_tranport = InprocTransport::default();
        let mut transport = MultiTransport::new(vec![transport, Box::new(inproc_tranport.clone())]);

        // Setup up ctrlc handling
        let running = Arc::new(AtomicBool::new(true));
        let r = running.clone();

        let registry = create_node_registry(&self.registry_config)?;

        let node_registry_manager =
            routes::NodeRegistryManager::new(self.node_id.clone(), registry);

        // set up the listeners on the transport
        let mut network_listener = transport.listen(&self.network_endpoint)?;
        debug!(
            "Listening for peer connections on {}",
            network_listener.endpoint()
        );
        let mut service_listener = transport.listen(&self.service_endpoint)?;
        debug!(
            "Listening for service connections on {}",
            service_listener.endpoint()
        );
        let mut admin_service_listener = transport.listen(ADMIN_SERVICE_ADDRESS)?;

        let orchestrator = ServiceOrchestrator::new(
            vec![Box::new(ScabbardFactory::new(None, None))],
            self.service_endpoint.clone(),
            inproc_tranport.clone(),
        );
        let peer_connector = PeerConnector::new(self.network.clone(), Box::new(transport));
        let admin_service = AdminService::new(&self.node_id, orchestrator, peer_connector.clone());

        let node_id = self.node_id.clone();
        let service_endpoint = self.service_endpoint.clone();
        let (rest_api_shutdown_handle, rest_api_join_handle) = RestApiBuilder::new()
            .with_bind(&self.rest_api_endpoint)
            .add_resource(Resource::new(
                Method::Get,
                "/openapi.yml",
                routes::get_openapi,
            ))
            .add_resource(Resource::new(Method::Get, "/status", move |_, _| {
                routes::get_status(node_id.clone(), service_endpoint.clone())
            }))
            .add_resources(node_registry_manager.resources())
            .add_resources(admin_service.resources())
            .build()?
            .run()?;

        ctrlc::set_handler(move || {
            info!("Recieved Shutdown");
            r.store(false, Ordering::SeqCst);
            if let Err(err) = rest_api_shutdown_handle.shutdown() {
                error!("Unable to cleanly shutdown REST API server: {}", err);
            }
        })
        .expect("Error setting Ctrl-C handler");

        info!("Starting SpinterNode with id {}", self.node_id);

        // Load initial state from the configured storage location and create the new
        // SplinterState from the retrieved circuit directory
        let storage = get_storage(&self.storage_location, CircuitDirectory::new)
            .map_err(|err| StartError::StorageError(format!("Storage Error: {}", err)))?;

        let circuit_directory = storage.read().clone();
        let state = Arc::new(RwLock::new(SplinterState::new(
            self.storage_location.to_string(),
            circuit_directory,
        )));

        let network = self.network.clone();
        let (send, recv) = crossbeam_channel::bounded(5);
        let r = running.clone();
        let network_message_sender_thread = thread::spawn(move || {
            let network_sender = NetworkMessageSender::new(Box::new(recv), network, r);
            network_sender.run()
        });

        // Set up the Circuit dispatcher
        let (circuit_dispatch_send, circuit_dispatch_recv) = crossbeam_channel::bounded(5);
        let circuit_dispatcher = set_up_circuit_dispatcher(
            send.clone(),
            &self.node_id,
            &self.network_endpoint,
            state.clone(),
        );
        let circuit_dispatch_loop = DispatchLoop::new(
            Box::new(circuit_dispatch_recv),
            circuit_dispatcher,
            running.clone(),
        );
        let circuit_dispatcher_thread = thread::spawn(move || circuit_dispatch_loop.run());

        // Set up the Auth dispatcher
        let auth_manager = AuthorizationManager::new(self.network.clone(), self.node_id.clone());
        let (auth_dispatch_send, auth_dispatch_recv) = crossbeam_channel::bounded(5);
        let auth_dispatcher =
            create_authorization_dispatcher(auth_manager.clone(), Box::new(send.clone()));
        let auth_dispatch_loop = DispatchLoop::new(
            Box::new(auth_dispatch_recv),
            auth_dispatcher,
            running.clone(),
        );
        let auth_dispatcher_thread = thread::spawn(move || auth_dispatch_loop.run());

        // Set up the Network dispatcher
        let (network_dispatch_send, network_dispatch_recv) = crossbeam_channel::bounded(5);
        let network_dispatcher = set_up_network_dispatcher(
            send,
            &self.node_id,
            auth_manager,
            circuit_dispatch_send,
            auth_dispatch_send,
        );
        let network_dispatch_loop = DispatchLoop::new(
            Box::new(network_dispatch_recv),
            network_dispatcher,
            running.clone(),
        );
        let network_dispatcher_thread = thread::spawn(move || network_dispatch_loop.run());

        // setup a thread to listen on the network port and add incoming connection to the network
        let network_clone = self.network.clone();

        // this thread will just be dropped on shutdown
        let _ = thread::spawn(move || {
            for connection_result in network_listener.incoming() {
                let connection = match connection_result {
                    Ok(connection) => connection,
                    Err(err) => {
                        return Err(StartError::TransportError(format!(
                            "Accept Error: {:?}",
                            err
                        )));
                    }
                };
                debug!("Received connection from {}", connection.remote_endpoint());
                network_clone.add_connection(connection)?;
            }
            Ok(())
        });

        // setup a thread to listen on the service port and add incoming connection to the network
        let service_clone = self.network.clone();

        // this thread will just be dropped on shutdown
        let admin_service_peer_id = admin_service.service_id().to_string();
        let _ = thread::spawn(move || {
            // accept the admin service's connection
            match admin_service_listener.incoming().next() {
                Some(Ok(connection)) => {
                    service_clone.add_peer(admin_service_peer_id, connection)?;
                }
                Some(Err(err)) => {
                    return Err(StartError::TransportError(format!(
                        "Accept Error: {:?}",
                        err
                    )));
                }
                None => {}
            }

            for connection_result in service_listener.incoming() {
                let connection = match connection_result {
                    Ok(connection) => connection,
                    Err(err) => {
                        return Err(StartError::TransportError(format!(
                            "Accept Error: {:?}",
                            err
                        )));
                    }
                };
                debug!(
                    "Received service connection from {}",
                    connection.remote_endpoint()
                );
                service_clone.add_connection(connection)?;
            }
            Ok(())
        });

        // For provided initial peers, try to connect to them
        for peer in self.initial_peers.iter() {
            if let Err(err) = peer_connector.connect_unidentified_peer(&peer) {
                error!("Connect Error: {}", err);
            }
        }

        // For each node in the circuit_directory, try to connect and add them to the network
        for (node_id, node) in rwlock_read_unwrap!(state).nodes().iter() {
            if let Some(endpoint) = node.endpoints().get(0) {
                // if the node is this node do not try to connect.
                let node_endpoint = {
                    if endpoint.starts_with("tcp://") {
                        &endpoint[6..]
                    } else {
                        endpoint
                    }
                };
                if node_endpoint != self.network_endpoint {
                    if let Err(err) = peer_connector.connect_peer(node_id, &node_endpoint) {
                        debug!("Unable to connect to node: {} Error: {:?}", node_id, err);
                    }
                }
            } else {
                debug!("node {} has no known endpoints", node_id);
            }
        }

        let timeout = Duration::from_secs(TIMEOUT_SEC);

        let service_processor_join_handle =
            Self::start_admin_service(inproc_tranport, admin_service, Arc::clone(&running))?;

        // start the recv loop
        while running.load(Ordering::SeqCst) {
            match self.network.recv_timeout(timeout) {
                // This is where the message should be dispatched
                Ok(message) => {
                    let mut msg: NetworkMessage =
                        protobuf::parse_from_bytes(message.payload()).unwrap();
                    let dispatch_msg = DispatchMessage::new(
                        msg.get_message_type(),
                        msg.take_payload(),
                        message.peer_id().to_string(),
                    );
                    debug!("Received Message from {}: {:?}", message.peer_id(), msg);
                    match network_dispatch_send.send(dispatch_msg) {
                        Ok(()) => (),
                        Err(err) => error!("Dispatch Error {}", err.to_string()),
                    }
                }
                Err(RecvTimeoutError::Disconnected) => {
                    // if the reciever has disconnected, shutdown
                    warn!("Recieved Disconnected Error from Network");
                    break;
                }
                Err(_) => {
                    // Timeout or NoPeerError are ignored
                    continue;
                }
            }
        }
        info!("Shutting down");
        // Join network sender and dispatcher threads
        let _ = network_message_sender_thread.join();
        let _ = circuit_dispatcher_thread.join();
        let _ = auth_dispatcher_thread.join();
        let _ = network_dispatcher_thread.join();
        let _ = rest_api_join_handle.join();
        let _ = service_processor_join_handle.join_all();

        Ok(())
    }

    fn start_admin_service(
        transport: InprocTransport,
        admin_service: AdminService,
        running: Arc<AtomicBool>,
    ) -> Result<service::JoinHandles<Result<(), service::error::ServiceProcessorError>>, StartError>
    {
        let start_admin: std::thread::JoinHandle<
            Result<
                service::JoinHandles<Result<(), service::error::ServiceProcessorError>>,
                StartError,
            >,
        > = thread::spawn(move || {
            let mut transport = transport;

            // use a match statement here, to inform
            let connection = transport.connect(ADMIN_SERVICE_ADDRESS).map_err(|err| {
                StartError::AdminServiceError(format!(
                    "unable to initiate admin service connection: {:?}",
                    err
                ))
            })?;
            let mut admin_service_processor =
                ServiceProcessor::new(connection, "admin".into(), 1, 1, 128, running).map_err(
                    |err| {
                        StartError::AdminServiceError(format!(
                            "unable to create admin service processor: {}",
                            err
                        ))
                    },
                )?;

            admin_service_processor
                .add_service(Box::new(admin_service))
                .map_err(|err| {
                    StartError::AdminServiceError(format!(
                        "unable to add admin service to processor: {}",
                        err
                    ))
                })?;

            admin_service_processor
                .start()
                .map(|(_, join_handles)| join_handles)
                .map_err(|err| {
                    StartError::AdminServiceError(format!(
                        "unable to start service processor: {}",
                        err
                    ))
                })
        });

        start_admin.join().map_err(|_| {
            StartError::AdminServiceError(
                "unable to start admin service, due to thread join error".into(),
            )
        })?
    }
}

#[derive(Default)]
pub struct SplinterDaemonBuilder {
    storage_location: Option<String>,
    service_endpoint: Option<String>,
    network_endpoint: Option<String>,
    initial_peers: Option<Vec<String>>,
    node_id: Option<String>,
    rest_api_endpoint: Option<String>,
    registry_backend: Option<String>,
    registry_file: Option<String>,
}

impl SplinterDaemonBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_storage_location(mut self, value: String) -> Self {
        self.storage_location = Some(value);
        self
    }

    pub fn with_service_endpoint(mut self, value: String) -> Self {
        self.service_endpoint = Some(value);
        self
    }

    pub fn with_network_endpoint(mut self, value: String) -> Self {
        self.network_endpoint = Some(value);
        self
    }

    pub fn with_initial_peers(mut self, value: Vec<String>) -> Self {
        self.initial_peers = Some(value);
        self
    }

    pub fn with_node_id(mut self, value: String) -> Self {
        self.node_id = Some(value);
        self
    }

    pub fn with_rest_api_endpoint(mut self, value: String) -> Self {
        self.rest_api_endpoint = Some(value);
        self
    }

    pub fn with_registry_backend(mut self, value: String) -> Self {
        self.registry_backend = Some(value);
        self
    }

    pub fn with_registry_file(mut self, value: String) -> Self {
        self.registry_file = Some(value);
        self
    }

    pub fn build(self) -> Result<SplinterDaemon, CreateError> {
        let mesh = Mesh::new(512, 128);
        let network = Network::new(mesh.clone());

        let storage_location = self.storage_location.ok_or_else(|| {
            CreateError::MissingRequiredField("Missing field: storage_location".to_string())
        })?;

        let service_endpoint = self.service_endpoint.ok_or_else(|| {
            CreateError::MissingRequiredField("Missing field: service_location".to_string())
        })?;

        let network_endpoint = self.network_endpoint.ok_or_else(|| {
            CreateError::MissingRequiredField("Missing field: network_endpoint".to_string())
        })?;

        let initial_peers = self.initial_peers.ok_or_else(|| {
            CreateError::MissingRequiredField("Missing field: initial_peers".to_string())
        })?;

        let node_id = self.node_id.ok_or_else(|| {
            CreateError::MissingRequiredField("Missing field: node_id".to_string())
        })?;

        let rest_api_endpoint = self.rest_api_endpoint.ok_or_else(|| {
            CreateError::MissingRequiredField("Missing field: rest_api_endpoint".to_string())
        })?;

        let mut registry_config_builder = RegistryConfigBuilder::default();
        if let Some(value) = self.registry_backend {
            registry_config_builder = registry_config_builder.with_registry_backend(value);
        }

        if let Some(value) = self.registry_file {
            registry_config_builder = registry_config_builder.with_registry_file(value);
        }

        let registry_config = registry_config_builder.build()?;

        Ok(SplinterDaemon {
            storage_location,
            service_endpoint,
            network_endpoint,
            initial_peers,
            network,
            node_id,
            rest_api_endpoint,
            registry_config,
        })
    }
}

fn set_up_network_dispatcher(
    send: crossbeam_channel::Sender<SendRequest>,
    node_id: &str,
    auth_manager: AuthorizationManager,
    circuit_sender: crossbeam_channel::Sender<DispatchMessage<CircuitMessageType>>,
    auth_sender: crossbeam_channel::Sender<DispatchMessage<AuthorizationMessageType>>,
) -> Dispatcher<NetworkMessageType> {
    let mut dispatcher = Dispatcher::<NetworkMessageType>::new(Box::new(send));

    let network_echo_handler = NetworkEchoHandler::new(node_id.to_string());
    dispatcher.set_handler(
        NetworkMessageType::NETWORK_ECHO,
        Box::new(NetworkAuthGuardHandler::new(
            auth_manager.clone(),
            Box::new(network_echo_handler),
        )),
    );

    let circuit_message_handler = CircuitMessageHandler::new(Box::new(circuit_sender));
    dispatcher.set_handler(
        NetworkMessageType::CIRCUIT,
        Box::new(NetworkAuthGuardHandler::new(
            auth_manager,
            Box::new(circuit_message_handler),
        )),
    );

    let auth_message_handler = AuthorizationMessageHandler::new(Box::new(auth_sender));
    dispatcher.set_handler(
        NetworkMessageType::AUTHORIZATION,
        Box::new(auth_message_handler),
    );

    dispatcher
}

fn set_up_circuit_dispatcher(
    send: crossbeam_channel::Sender<SendRequest>,
    node_id: &str,
    endpoint: &str,
    state: Arc<RwLock<SplinterState>>,
) -> Dispatcher<CircuitMessageType> {
    let mut dispatcher = Dispatcher::<CircuitMessageType>::new(Box::new(send));

    let service_connect_request_handler =
        ServiceConnectRequestHandler::new(node_id.to_string(), endpoint.to_string(), state.clone());
    dispatcher.set_handler(
        CircuitMessageType::SERVICE_CONNECT_REQUEST,
        Box::new(service_connect_request_handler),
    );

    let service_connect_forward_handler = ServiceConnectForwardHandler::new(state.clone());
    dispatcher.set_handler(
        CircuitMessageType::SERVICE_CONNECT_FORWARD,
        Box::new(service_connect_forward_handler),
    );

    let service_disconnect_request_handler =
        ServiceDisconnectRequestHandler::new(node_id.to_string(), state.clone());
    dispatcher.set_handler(
        CircuitMessageType::SERVICE_DISCONNECT_REQUEST,
        Box::new(service_disconnect_request_handler),
    );

    let service_disconnect_forward_handler = ServiceDisconnectForwardHandler::new(state.clone());
    dispatcher.set_handler(
        CircuitMessageType::SERVICE_DISCONNECT_FORWARD,
        Box::new(service_disconnect_forward_handler),
    );

    let direct_message_handler =
        CircuitDirectMessageHandler::new(node_id.to_string(), state.clone());
    dispatcher.set_handler(
        CircuitMessageType::CIRCUIT_DIRECT_MESSAGE,
        Box::new(direct_message_handler),
    );

    let circuit_error_handler = CircuitErrorHandler::new(node_id.to_string(), state.clone());
    dispatcher.set_handler(
        CircuitMessageType::CIRCUIT_ERROR_MESSAGE,
        Box::new(circuit_error_handler),
    );

    // Circuit Admin handlers
    let admin_direct_message_handler =
        AdminDirectMessageHandler::new(node_id.to_string(), state.clone());
    dispatcher.set_handler(
        CircuitMessageType::ADMIN_DIRECT_MESSAGE,
        Box::new(admin_direct_message_handler),
    );

    dispatcher
}

fn create_node_registry(
    registry_config: &RegistryConfig,
) -> Result<Box<dyn NodeRegistry>, RestApiServerError> {
    match &registry_config.registry_backend() as &str {
        "FILE" => Ok(Box::new(
            YamlNodeRegistry::new(&registry_config.registry_file()).map_err(|err| {
                RestApiServerError::StartUpError(format!(
                    "Failed to initialize YamlNodeRegistry: {}",
                    err
                ))
            })?,
        )),
        _ => Err(RestApiServerError::StartUpError(
            "NodeRegistry type is not supported".to_string(),
        )),
    }
}

#[derive(Debug)]
pub enum CreateError {
    MissingRequiredField(String),
    NodeRegistryError(String),
}

impl From<RegistryConfigError> for CreateError {
    fn from(err: RegistryConfigError) -> Self {
        CreateError::NodeRegistryError(format!("Error configuring Node Registry: {}", err))
    }
}

#[derive(Debug)]
pub enum StartError {
    TransportError(String),
    NetworkError(String),
    StorageError(String),
    ProtocolError(String),
    RestApiError(String),
    AdminServiceError(String),
}

impl From<RestApiServerError> for StartError {
    fn from(rest_api_error: RestApiServerError) -> Self {
        StartError::RestApiError(format!("Rest Api Server Error: {}", rest_api_error))
    }
}

impl From<ListenError> for StartError {
    fn from(listen_error: ListenError) -> Self {
        StartError::TransportError(format!("Listen Error: {:?}", listen_error))
    }
}

impl From<AcceptError> for StartError {
    fn from(accept_error: AcceptError) -> Self {
        StartError::TransportError(format!("Accept Error: {:?}", accept_error))
    }
}

impl From<ConnectError> for StartError {
    fn from(connect_error: ConnectError) -> Self {
        StartError::TransportError(format!("Connect Error: {:?}", connect_error))
    }
}

impl From<ConnectionError> for StartError {
    fn from(connection_error: ConnectionError) -> Self {
        StartError::NetworkError(format!("Network Error: {:?}", connection_error))
    }
}

impl From<SendError> for StartError {
    fn from(send_error: SendError) -> Self {
        StartError::NetworkError(format!("Network Error: {:?}", send_error))
    }
}

impl From<PeerUpdateError> for StartError {
    fn from(update_error: PeerUpdateError) -> Self {
        StartError::NetworkError(format!("Network Peer Update Error: {:?}", update_error))
    }
}

impl From<protobuf::ProtobufError> for StartError {
    fn from(err: protobuf::ProtobufError) -> Self {
        StartError::ProtocolError(format!("Protocol Format Error: {:?}", err))
    }
}
