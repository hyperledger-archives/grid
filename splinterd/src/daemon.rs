use libsplinter::storage::state::State;
use libsplinter::storage::Storage;
use libsplinter::transport::{AcceptError, ConnectError, Incoming, ListenError, Transport};

use std::thread;

pub struct SplinterDaemon {
    storage: Box<dyn Storage<S = State>>,
    transport: Box<dyn Transport + Send>,
    service_endpoint: String,
    network_endpoint: String,
    initial_peers: Vec<String>,
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
        Ok(SplinterDaemon {
            storage,
            transport,
            service_endpoint,
            network_endpoint,
            initial_peers,
        })
    }

    pub fn start(&mut self) -> Result<(), StartError> {
        let mut network_listener = self.transport.listen(&self.network_endpoint)?;
        let network_thread = thread::spawn(move || {
            for connection_result in network_listener.incoming() {
                let connection = match connection_result {
                    Ok(connection) => connection,
                    Err(err) => return Err(err),
                };
                println!("Recieved connection from {}", connection.remote_endpoint());
            }
            Ok(())
        });

        let mut service_listener = self.transport.listen(&self.service_endpoint)?;
        let service_thread = thread::spawn(move || {
            for connection_result in service_listener.incoming() {
                let connection = match connection_result {
                    Ok(connection) => connection,
                    Err(err) => return Err(err),
                };
                println!("Recieved connection from {}", connection.remote_endpoint());
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
            println!("Successfully connected to {}", connection.remote_endpoint());
        }

        for (node_id, node) in self.storage.read().nodes().iter() {
            if let Some(endpoint) = node.endpoints().get(1) {
                let connection_result = self.transport.connect(&endpoint);
                let connection = match connection_result {
                    Ok(connection) => connection,
                    Err(err) => {
                        return Err(StartError::TransportError(format!(
                            "Connect Error: {:?}",
                            err
                        )))
                    }
                };
                println!(
                    "Successfully connected to node {}: {}",
                    node_id,
                    connection.remote_endpoint()
                );
            } else {
                println!("Unable to connect to node: {}", node_id);
            }
        }
        let _ = network_thread.join();
        let _ = service_thread.join();
        Ok(())
    }
}

#[derive(Debug)]
pub enum CreateError {}

#[derive(Debug)]
pub enum StartError {
    TransportError(String),
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
