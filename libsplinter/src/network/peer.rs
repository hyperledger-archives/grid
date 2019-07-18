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

use crate::transport::Transport;

use super::Network;

#[derive(Debug, PartialEq)]
pub struct PeerConnectorError(String);

impl std::error::Error for PeerConnectorError {}

impl fmt::Display for PeerConnectorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a peering error occurred: {}", self.0)
    }
}

#[derive(Clone)]
pub struct PeerConnector {
    transport: Arc<Mutex<Box<dyn Transport>>>,
    network: Network,
}

impl PeerConnector {
    pub fn new(network: Network, transport: Box<dyn Transport>) -> PeerConnector {
        Self {
            network,
            transport: Arc::new(Mutex::new(transport)),
        }
    }

    pub fn connect_peer(&self, node_id: &str, endpoint: &str) -> Result<(), PeerConnectorError> {
        let mut transport = self.transport.lock().map_err(|err| {
            PeerConnectorError(format!("Unable to acquire transport lock: {}", err))
        })?;

        let connection = transport.connect(&endpoint).map_err(|err| {
            PeerConnectorError(format!("Unable to connect to node {}: {:?}", node_id, err))
        })?;
        debug!(
            "Successfully connected to node {}: {}",
            node_id,
            connection.remote_endpoint()
        );
        self.network
            .add_peer(node_id.to_string(), connection)
            .map_err(|err| {
                PeerConnectorError(format!("Unable to add peer {}: {}", node_id, err))
            })?;

        Ok(())
    }

    pub fn connect_unidentified_peer(&self, endpoint: &str) -> Result<(), PeerConnectorError> {
        let mut transport = self.transport.lock().map_err(|err| {
            PeerConnectorError(format!("Unable to acquire transport lock: {}", err))
        })?;

        let connection = transport.connect(&endpoint).map_err(|err| {
            PeerConnectorError(format!("Unable to connect to {}: {:?}", endpoint, err))
        })?;
        debug!("Successfully connected to {}", connection.remote_endpoint());
        self.network.add_connection(connection).map_err(|err| {
            PeerConnectorError(format!("Unable to add peer endpoint {}: {}", endpoint, err))
        })?;

        Ok(())
    }
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
    fn test_connect_undentified_peer() {
        let mesh = Mesh::new(4, 16);
        let network = Network::new(mesh.clone());
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

    /// Add a connection with a known node id (e.g. a node that was connected and known in the
    /// past).
    #[test]
    fn test_connect_peer() {
        let mesh = Mesh::new(4, 16);
        let network = Network::new(mesh.clone());
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
        let network = Network::new(mesh.clone());
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
        let network = Network::new(mesh.clone());
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
