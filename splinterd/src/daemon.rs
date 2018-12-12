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

use libsplinter::mesh::Mesh;
use libsplinter::network::{ConnectionError, Network};
use libsplinter::storage::state::State;
use libsplinter::storage::Storage;
use libsplinter::transport::{AcceptError, ConnectError, Incoming, ListenError, Transport};

use std::thread;

pub struct SplinterDaemon {
    transport: Box<dyn Transport + Send>,
    storage: Box<dyn Storage<S = State>>,
    service_endpoint: String,
    network_endpoint: String,
    initial_peers: Vec<String>,
    network: Network,
}

impl SplinterDaemon {
    pub fn new(
        storage: Box<dyn Storage<S = State>>,
        transport: Box<dyn Transport + Send>,
        network_endpoint: String,
        service_endpoint: String,
        initial_peers: Vec<String>,
    ) -> Result<SplinterDaemon, CreateError> {
        // create SplinterD node
        let mesh = Mesh::new(512, 128);
        let network = Network::new(mesh.clone());

        Ok(SplinterDaemon {
            transport,
            storage,
            service_endpoint,
            network_endpoint,
            initial_peers,
            network,
        })
    }

    pub fn start(&mut self) -> Result<(), StartError> {
        let mut network_listener = self.transport.listen(&self.network_endpoint)?;
        let mut network_clone = self.network.clone();
        let _ = thread::spawn(move || {
            for connection_result in network_listener.incoming() {
                let connection = match connection_result {
                    Ok(connection) => connection,
                    Err(err) => {
                        return Err(StartError::TransportError(format!(
                            "Accept Error: {:?}",
                            err
                        )))
                    }
                };
                debug!(
                    "Received network connection from {}",
                    connection.remote_endpoint()
                );
                network_clone.add_connection(connection)?;
            }
            Ok(())
        });

        let mut service_listener = self.transport.listen(&self.service_endpoint)?;
        let mut service_clone = self.network.clone();
        let _ = thread::spawn(move || {
            for connection_result in service_listener.incoming() {
                let connection = match connection_result {
                    Ok(connection) => connection,
                    Err(err) => {
                        return Err(StartError::TransportError(format!(
                            "Accept Error: {:?}",
                            err
                        )))
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

        for peer in self.initial_peers.iter() {
            let connection_result = self.transport.connect(&peer);
            let connection = match connection_result {
                Ok(connection) => connection,
                Err(err) => {
                    return Err(StartError::TransportError(format!(
                        "Connect Error: {:?}",
                        err
                    )))
                }
            };
            debug!("Successfully connected to {}", connection.remote_endpoint());
            self.network.add_connection(connection)?;
        }

        for (node_id, node) in self.storage.read().nodes().iter() {
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

        loop {
            match self.network.recv() {
                // This is where the message should be dispatched
                Ok(message) => {
                    let msg_str = String::from_utf8(message.payload().to_vec()).unwrap();
                    debug!("Received Message from {}: {:?}", message.peer_id(), msg_str);
                }
                Err(err) => {
                    debug!("Error: {:?}", err);
                    continue;
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum CreateError {}

#[derive(Debug)]
pub enum StartError {
    TransportError(String),
    NetworkError(String),
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
