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
use std::sync::{Arc, RwLock};
use std::thread;

use libsplinter::circuit::directory::CircuitDirectory;
use libsplinter::circuit::handlers::{
    CircuitMessageHandler, ServiceConnectForwardHandler, ServiceConnectRequestHandler,
};
use libsplinter::circuit::SplinterState;
use libsplinter::mesh::Mesh;
use libsplinter::network::auth::handlers::{
    create_authorization_dispatcher, AuthorizationMessageHandler, NetworkAuthGuardHandler,
};
use libsplinter::network::auth::AuthorizationManager;
use libsplinter::network::dispatch::{DispatchLoop, DispatchMessage, Dispatcher};
use libsplinter::network::handlers::NetworkEchoHandler;
use libsplinter::network::sender::{NetworkMessageSender, SendRequest};
use libsplinter::network::{ConnectionError, Network, PeerUpdateError, SendError};
use libsplinter::protos::authorization::{
    AuthorizationMessage, AuthorizationMessageType, ConnectRequest,
};
use libsplinter::protos::circuit::CircuitMessageType;
use libsplinter::protos::network::{NetworkMessage, NetworkMessageType};
use libsplinter::rwlock_read_unwrap;
use libsplinter::storage::get_storage;
use libsplinter::transport::{AcceptError, ConnectError, Incoming, ListenError, Transport};

use ::log::{debug, error, info, log};
use crossbeam_channel;
use protobuf::Message;

pub struct SplinterDaemon {
    transport: Box<dyn Transport + Send>,
    storage_location: String,
    service_endpoint: String,
    network_endpoint: String,
    initial_peers: Vec<String>,
    network: Network,
    node_id: String,
}

impl SplinterDaemon {
    pub fn new(
        storage_location: String,
        transport: Box<dyn Transport + Send>,
        network_endpoint: String,
        service_endpoint: String,
        initial_peers: Vec<String>,
        node_id: String,
    ) -> Result<SplinterDaemon, CreateError> {
        let mesh = Mesh::new(512, 128);
        let network = Network::new(mesh.clone());

        Ok(SplinterDaemon {
            transport,
            storage_location,
            service_endpoint,
            network_endpoint,
            initial_peers,
            network,
            node_id,
        })
    }

    pub fn start(&mut self) -> Result<(), StartError> {
        info!("Starting SpinterNode with id {}", self.node_id);

        // Load initial state from the configured storage location and create the new
        // SplinterState from the retrieved circuit directory
        let storage = get_storage(&self.storage_location, || CircuitDirectory::new())
            .map_err(|err| StartError::StorageError(format!("Storage Error: {}", err)))?;

        let circuit_directory = storage.read().clone();
        let state = Arc::new(RwLock::new(SplinterState::new(
            self.storage_location.to_string(),
            circuit_directory,
        )));

        let network = self.network.clone();
        let (send, recv) = crossbeam_channel::bounded(5);

        let _ = thread::spawn(move || {
            let network_sender = NetworkMessageSender::new(Box::new(recv), network);
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
        let circuit_dispatch_loop =
            DispatchLoop::new(Box::new(circuit_dispatch_recv), circuit_dispatcher);
        let _ = thread::spawn(move || circuit_dispatch_loop.run());

        // Set up the Auth dispatcher
        let auth_manager = AuthorizationManager::new(
            self.network.clone(),
            self.node_id.clone(),
            self.network_endpoint.clone(),
        );
        let (auth_dispatch_send, auth_dispatch_recv) = crossbeam_channel::bounded(5);
        let auth_dispatcher =
            create_authorization_dispatcher(auth_manager.clone(), Box::new(send.clone()));
        let auth_dispatch_loop = DispatchLoop::new(Box::new(auth_dispatch_recv), auth_dispatcher);
        let _ = thread::spawn(move || auth_dispatch_loop.run());

        // Set up the Network dispatcher
        let (network_dispatch_send, network_dispatch_recv) = crossbeam_channel::bounded(5);
        let network_dispatcher = set_up_network_dispatcher(
            send,
            &self.node_id,
            auth_manager,
            circuit_dispatch_send,
            auth_dispatch_send,
        );
        let network_dispatch_loop =
            DispatchLoop::new(Box::new(network_dispatch_recv), network_dispatcher);
        let _ = thread::spawn(move || network_dispatch_loop.run());

        // setup a thread to listen on the network port and add incoming connection to the network
        let mut network_listener = self.transport.listen(&self.network_endpoint)?;
        let network_clone = self.network.clone();
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
        let mut service_listener = self.transport.listen(&self.service_endpoint)?;
        let service_clone = self.network.clone();
        let _ = thread::spawn(move || {
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
            let connection_result = self.transport.connect(&peer);
            match connection_result {
                Ok(connection) => {
                    debug!("Successfully connected to {}", connection.remote_endpoint());
                    self.network.add_connection(connection)?;
                }
                Err(err) => {
                    error!("Connect Error: {:?}", err);
                }
            };
        }

        let connect_request_msg_bytes = create_connect_request(self.network_endpoint.clone())?;
        for peer_id in self.network.peer_ids() {
            debug!("Sending connect request to peer {}", peer_id);
            self.network.send(peer_id, &connect_request_msg_bytes)?;
        }

        // For each node in the circuit_directory, try to connect and add them to the network
        for (node_id, node) in rwlock_read_unwrap!(state).nodes().iter() {
            if let Some(endpoint) = node.endpoints().get(0) {
                // if the node is this node do not try to connect.
                if endpoint != &self.network_endpoint {
                    let connection_result = self.transport.connect(&endpoint);
                    let connection = match connection_result {
                        Ok(connection) => connection,
                        Err(err) => {
                            debug!("Unable to connect to node: {} Error: {:?}", node_id, err);
                            continue;
                        }
                    };
                    debug!(
                        "Successfully connected to node {}: {}",
                        node_id,
                        connection.remote_endpoint()
                    );
                    self.network.add_peer(node_id.to_string(), connection)?;
                }
            } else {
                debug!("Unable to connect to node: {}", node_id);
            }
        }

        // start the recv loop
        loop {
            match self.network.recv() {
                // This is where the message should be dispatched
                Ok(message) => {
                    let msg: NetworkMessage =
                        protobuf::parse_from_bytes(message.payload()).unwrap();
                    let dispatch_msg = DispatchMessage::new(
                        msg.get_message_type(),
                        msg.get_payload().to_vec(),
                        message.peer_id().to_string(),
                    );
                    debug!("Received Message from {}: {:?}", message.peer_id(), msg);
                    match network_dispatch_send.send(dispatch_msg) {
                        Ok(()) => (),
                        Err(err) => error!("Dispatch Error {}", err.to_string()),
                    }
                }
                Err(err) => {
                    debug!("Error: {:?}", err);
                    continue;
                }
            }
        }
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

    let service_connect_forward_handler = ServiceConnectForwardHandler::new(state);
    dispatcher.set_handler(
        CircuitMessageType::SERVICE_CONNECT_FORWARD,
        Box::new(service_connect_forward_handler),
    );

    dispatcher
}

fn create_connect_request(endpoint: String) -> Result<Vec<u8>, protobuf::ProtobufError> {
    let mut connect_request = ConnectRequest::new();
    connect_request.set_endpoint(endpoint);

    let mut auth_msg_env = AuthorizationMessage::new();
    auth_msg_env.set_message_type(AuthorizationMessageType::CONNECT_REQUEST);
    auth_msg_env.set_payload(connect_request.write_to_bytes()?);

    let mut network_msg = NetworkMessage::new();
    network_msg.set_message_type(NetworkMessageType::AUTHORIZATION);
    network_msg.set_payload(auth_msg_env.write_to_bytes()?);

    network_msg.write_to_bytes()
}

#[derive(Debug)]
pub enum CreateError {}

#[derive(Debug)]
pub enum StartError {
    TransportError(String),
    NetworkError(String),
    StorageError(String),
    ProtocolError(String),
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
