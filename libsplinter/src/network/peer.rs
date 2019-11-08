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
use std::fmt;
use std::sync::{Arc, Mutex};

use protobuf::Message;

use crate::protos::{
    authorization::{
        AuthorizationMessage, AuthorizationMessageType, ConnectRequest,
        ConnectRequest_HandshakeMode,
    },
    network::{NetworkMessage, NetworkMessageType},
};
use crate::transport::Transport;

use super::Network;

#[derive(Debug, PartialEq)]
pub struct ErrorInfo {
    pub node_id: String,
    pub message: String,
}

#[derive(Debug, PartialEq)]
pub enum PeerConnectorError {
    PoisonedLock(String),
    ConnectionFailed(ErrorInfo),
    AddPeerFailed(ErrorInfo),
}

impl PeerConnectorError {
    fn connection_failed(peer_id: &str, message: String) -> Self {
        PeerConnectorError::ConnectionFailed(ErrorInfo {
            node_id: peer_id.to_string(),
            message,
        })
    }

    fn add_peer_failed(peer_id: &str, message: String) -> Self {
        PeerConnectorError::AddPeerFailed(ErrorInfo {
            node_id: peer_id.to_string(),
            message,
        })
    }
}

impl std::error::Error for PeerConnectorError {}

impl fmt::Display for PeerConnectorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PeerConnectorError::PoisonedLock(msg) => write!(f, "unable to acquire lock: {}", msg),
            PeerConnectorError::ConnectionFailed(info) => {
                write!(f, "failed to connect to {}: {}", info.node_id, info.message)
            }
            PeerConnectorError::AddPeerFailed(info) => write!(
                f,
                "failed to add peer for {}: {}",
                info.node_id, info.message
            ),
        }
    }
}

#[derive(Clone)]
pub struct PeerConnector {
    transport: Arc<Mutex<Box<dyn Transport + Send>>>,
    network: Network,
}

impl PeerConnector {
    pub fn new(network: Network, transport: Box<dyn Transport + Send>) -> PeerConnector {
        Self {
            network,
            transport: Arc::new(Mutex::new(transport)),
        }
    }

    pub fn connect_peer(&self, node_id: &str, endpoint: &str) -> Result<(), PeerConnectorError> {
        let mut transport = self
            .transport
            .lock()
            .map_err(|err| PeerConnectorError::PoisonedLock(err.to_string()))?;

        if self.network.get_peer_by_endpoint(endpoint).is_some() {
            return Ok(());
        }

        debug!("Connecting to {} at {}...", node_id, endpoint);
        let connection = transport
            .connect(&endpoint)
            .map_err(|err| PeerConnectorError::connection_failed(node_id, format!("{:?}", err)))?;
        debug!(
            "Successfully connected to node {}: {}",
            node_id,
            connection.remote_endpoint()
        );
        self.network
            .add_peer(node_id.to_string(), connection)
            .map_err(|err| PeerConnectorError::add_peer_failed(node_id, err.to_string()))?;

        let connect_request_msg_bytes = create_connect_request().map_err(|err| {
            PeerConnectorError::connection_failed(
                node_id,
                format!("unable to create message: {}", err),
            )
        })?;
        self.network
            .send(&node_id, &connect_request_msg_bytes)
            .map_err(|err| {
                PeerConnectorError::connection_failed(
                    node_id,
                    format!("unable to send connect request: {:?}", err),
                )
            })?;

        Ok(())
    }

    pub fn connect_unidentified_peer(&self, endpoint: &str) -> Result<(), PeerConnectorError> {
        let mut transport = self.transport.lock().map_err(|err| {
            PeerConnectorError::PoisonedLock(format!("Unable to acquire transport lock: {}", err))
        })?;

        if self.network.get_peer_by_endpoint(endpoint).is_some() {
            return Ok(());
        }

        let connection = transport
            .connect(&endpoint)
            .map_err(|err| PeerConnectorError::connection_failed(endpoint, format!("{:?}", err)))?;
        debug!("Successfully connected to {}", connection.remote_endpoint());
        let temp_peer_id = self
            .network
            .add_connection(connection)
            .map_err(|err| PeerConnectorError::add_peer_failed(endpoint, err.to_string()))?;

        let connect_request_msg_bytes = create_connect_request().map_err(|err| {
            PeerConnectorError::connection_failed(
                endpoint,
                format!("unable to create message: {}", err),
            )
        })?;
        self.network
            .send(&temp_peer_id, &connect_request_msg_bytes)
            .map_err(|err| {
                PeerConnectorError::connection_failed(
                    endpoint,
                    format!("unable to send connect request: {:?}", err),
                )
            })?;

        Ok(())
    }
}

fn create_connect_request() -> Result<Vec<u8>, protobuf::ProtobufError> {
    let mut connect_request = ConnectRequest::new();
    connect_request.set_handshake_mode(ConnectRequest_HandshakeMode::BIDIRECTIONAL);

    let mut auth_msg_env = AuthorizationMessage::new();
    auth_msg_env.set_message_type(AuthorizationMessageType::CONNECT_REQUEST);
    auth_msg_env.set_payload(connect_request.write_to_bytes()?);

    let mut network_msg = NetworkMessage::new();
    network_msg.set_message_type(NetworkMessageType::AUTHORIZATION);
    network_msg.set_payload(auth_msg_env.write_to_bytes()?);

    network_msg.write_to_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::VecDeque;

    use crate::mesh::Mesh;
    use crate::network::Network;
    use crate::transport::{
        ConnectError, Connection, DisconnectError, RecvError, SendError, Transport,
    };

    /// Add a connection without an existing node (peer) id.
    #[test]
    fn test_connect_unidentified_peer() {
        let mesh = Mesh::new(4, 16);
        let network = Network::new(mesh.clone(), 0).unwrap();
        let transport =
            MockConnectingTransport::expect_connections(vec![Ok(Box::new(MockConnection))]);

        let peer_connector = PeerConnector::new(network.clone(), Box::new(transport));

        assert!(network.peer_ids().is_empty());

        assert_eq!(
            Ok(()),
            peer_connector.connect_unidentified_peer("MockConnection")
        );

        assert!(!network.peer_ids().is_empty());
        assert!(network.peer_ids()[0].starts_with("temp-"));
    }

    /// Add a connection without an existing node (peer) id, and add the same peer a second time to
    /// determine that it is not being added more than once.
    #[test]
    fn test_connect_unidentified_peer_idempotent() {
        let mesh = Mesh::new(4, 16);
        let network = Network::new(mesh.clone(), 0).unwrap();
        let transport =
            MockConnectingTransport::expect_connections(vec![Ok(Box::new(MockConnection))]);

        let peer_connector = PeerConnector::new(network.clone(), Box::new(transport));

        assert!(network.peer_ids().is_empty());

        assert_eq!(
            Ok(()),
            peer_connector.connect_unidentified_peer("MockConnection")
        );

        assert_eq!(1, network.peer_ids().len());
        let peer_id = network.peer_ids()[0].to_string();
        assert!(peer_id.starts_with("temp-"));

        assert_eq!(
            Ok(()),
            peer_connector.connect_unidentified_peer("MockConnection")
        );

        assert_eq!(1, network.peer_ids().len());
        assert_eq!(peer_id, network.peer_ids()[0]);
    }

    /// Add a connection with a known node id (e.g. a node that was connected and known in the
    /// past).
    #[test]
    fn test_connect_peer() {
        let mesh = Mesh::new(4, 16);
        let network = Network::new(mesh.clone(), 0).unwrap();
        let transport =
            MockConnectingTransport::expect_connections(vec![Ok(Box::new(MockConnection))]);

        let peer_connector = PeerConnector::new(network.clone(), Box::new(transport));

        assert!(network.peer_ids().is_empty());

        assert_eq!(
            Ok(()),
            peer_connector.connect_peer("test_node_id", "MockConnection")
        );
        assert!(!network.peer_ids().is_empty());
        assert_eq!(Some(&"test_node_id".to_string()), network.peer_ids().get(0));
    }

    /// Test that an error is returned if the connection cannot be opened, no peer is added.
    #[test]
    fn test_connect_peer_unable_to_connect() {
        let mesh = Mesh::new(4, 16);
        let network = Network::new(mesh.clone(), 0).unwrap();
        let transport = MockConnectingTransport::expect_connections(vec![Err(
            ConnectError::ProtocolError("test error".into()),
        )]);

        let peer_connector = PeerConnector::new(network.clone(), Box::new(transport));

        assert!(network.peer_ids().is_empty());

        let result = peer_connector.connect_peer("test_node_id", "MockConnection");
        assert!(result.is_err());
        assert!(network.peer_ids().is_empty());
    }

    /// Test that an error is returned if the connection cannot be opened, no peer is added.
    #[test]
    fn test_connect_unidentified_peer_unable_to_connect() {
        let mesh = Mesh::new(4, 16);
        let network = Network::new(mesh.clone(), 0).unwrap();
        let transport = MockConnectingTransport::expect_connections(vec![Err(
            ConnectError::ProtocolError("test error".into()),
        )]);

        let peer_connector = PeerConnector::new(network.clone(), Box::new(transport));

        assert!(network.peer_ids().is_empty());

        let result = peer_connector.connect_unidentified_peer("MockConnection");
        assert!(result.is_err());
        assert!(network.peer_ids().is_empty());
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
            unimplemented!()
        }
    }

    struct MockConnection;

    impl Connection for MockConnection {
        fn send(&mut self, _message: &[u8]) -> Result<(), SendError> {
            Ok(())
        }

        fn recv(&mut self) -> Result<Vec<u8>, RecvError> {
            unimplemented!()
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
            &MockEvented
        }
    }

    struct MockEvented;

    impl mio::Evented for MockEvented {
        fn register(
            &self,
            _poll: &mio::Poll,
            _token: mio::Token,
            _interest: mio::Ready,
            _opts: mio::PollOpt,
        ) -> std::io::Result<()> {
            Ok(())
        }

        fn reregister(
            &self,
            _poll: &mio::Poll,
            _token: mio::Token,
            _interest: mio::Ready,
            _opts: mio::PollOpt,
        ) -> std::io::Result<()> {
            Ok(())
        }

        fn deregister(&self, _poll: &mio::Poll) -> std::io::Result<()> {
            Ok(())
        }
    }
}
