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
pub mod auth;
pub mod dispatch;
mod dispatch_proto;
pub mod handlers;
pub mod peer;
pub(crate) mod reply;
pub mod sender;

use protobuf::Message;
use uuid::Uuid;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

use crate::collections::BiHashMap;
use crate::mesh::{
    AddError, Envelope, Mesh, RecvError as MeshRecvError, RecvTimeoutError as MeshRecvTimeoutError,
    RemoveError, SendError as MeshSendError,
};
use crate::protos::network::{NetworkHeartbeat, NetworkMessage, NetworkMessageType};
use crate::transport::Connection;

#[derive(Debug)]
pub struct NetworkMessageWrapper {
    peer_id: String,
    payload: Vec<u8>,
}

impl NetworkMessageWrapper {
    pub fn new(peer_id: String, payload: Vec<u8>) -> Self {
        NetworkMessageWrapper { peer_id, payload }
    }
    pub fn peer_id(&self) -> &str {
        &self.peer_id
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
}

struct PeerMap {
    peers: BiHashMap<String, usize>,
    redirects: HashMap<String, String>,
    endpoints: BiHashMap<String, String>,
}

/// A map of Peer IDs to mesh IDs, which also maintains a redirect table for updated peer ids.
impl PeerMap {
    fn new() -> Self {
        PeerMap {
            peers: BiHashMap::new(),
            redirects: HashMap::new(),
            endpoints: BiHashMap::new(),
        }
    }

    /// Returns the current list of peer ids.
    ///
    /// This list does not include any of the redirected peer ids.
    fn peer_ids(&self) -> Vec<String> {
        self.peers
            .keys()
            .map(std::string::ToString::to_string)
            .collect()
    }

    /// Insert a new peer id for a given mesh id
    fn insert(&mut self, peer_id: String, mesh_id: usize, endpoint: String) {
        self.peers.insert(peer_id.clone(), mesh_id);
        self.endpoints.insert(peer_id, endpoint);
    }

    /// Remove a peer id, its endpoint and all of its redirects
    fn remove(&mut self, peer_id: &str) -> Option<usize> {
        info!("Removing peer: {}", peer_id);
        let peer_id_key = peer_id.to_string();
        self.redirects
            .retain(|_, target_peer_id| target_peer_id != peer_id);
        self.endpoints.remove_by_key(&peer_id_key);
        self.peers
            .remove_by_key(&peer_id_key)
            .map(|(_, mesh_id)| mesh_id)
    }

    /// Updates a peer id, and creates a redirect for the old id to the given new one.
    ///
    /// Additionally, it updates all of the old redirects to point to the given new one.
    fn update(&mut self, old_peer_id: String, new_peer_id: String) -> Result<(), PeerUpdateError> {
        if let Some((_, mesh_id)) = self.peers.remove_by_key(&old_peer_id) {
            self.peers.insert(new_peer_id.clone(), mesh_id);

            if let Some((_, endpoint)) = self.endpoints.remove_by_key(&old_peer_id) {
                self.endpoints.insert(new_peer_id.clone(), endpoint);
            }
            // update the old forwards
            for (_, v) in self
                .redirects
                .iter_mut()
                .filter(|(_, v)| **v == old_peer_id)
            {
                *v = new_peer_id.clone()
            }

            self.redirects.insert(old_peer_id, new_peer_id);

            Ok(())
        } else {
            Err(PeerUpdateError {})
        }
    }

    /// Returns the mesh id for the given peer id, following redirects if necessary.
    fn get_mesh_id(&self, peer_id: &str) -> Option<&usize> {
        self.redirects
            .get(peer_id)
            .and_then(|target_peer_id| self.peers.get_by_key(target_peer_id))
            .or_else(|| self.peers.get_by_key(&peer_id.to_string()))
    }

    /// Returns the direct peer id for the given mesh_id
    fn get_peer_id(&self, mesh_id: usize) -> Option<&String> {
        self.peers.get_by_value(&mesh_id)
    }

    /// Returns the endpoint for the given peer id
    fn get_peer_endpoint(&self, peer_id: &str) -> Option<String> {
        let endpoint_opt = self
            .redirects
            .get(peer_id)
            .and_then(|target_peer_id| self.endpoints.get_by_key(target_peer_id))
            .or_else(|| self.endpoints.get_by_key(&peer_id.to_string()));

        endpoint_opt.cloned()
    }

    fn get_peer_by_endpoint(&self, endpoint: &str) -> Option<String> {
        self.endpoints.get_by_value(&endpoint.to_string()).cloned()
    }
}

#[derive(Clone)]
pub struct Network {
    peers: Arc<RwLock<PeerMap>>,
    mesh: Mesh,
}

impl Network {
    pub fn new(mesh: Mesh, heartbeat_interval: u64) -> Result<Self, NetworkStartUpError> {
        let network = Network {
            peers: Arc::new(RwLock::new(PeerMap::new())),
            mesh,
        };

        if heartbeat_interval != 0 {
            let heartbeat_network = network.clone();
            let heartbeat = NetworkHeartbeat::new().write_to_bytes().map_err(|_| {
                NetworkStartUpError("cannot create NetworkHeartbeat message".to_string())
            })?;
            let mut heartbeat_message = NetworkMessage::new();
            heartbeat_message.set_message_type(NetworkMessageType::NETWORK_HEARTBEAT);
            heartbeat_message.set_payload(heartbeat);
            let heartbeat_bytes = heartbeat_message
                .write_to_bytes()
                .map_err(|_| NetworkStartUpError("cannot create NetworkMessage".to_string()))?;
            let _ = thread::spawn(move || {
                let interval = Duration::from_secs(heartbeat_interval);
                thread::sleep(interval);
                loop {
                    let peers = rwlock_read_unwrap!(heartbeat_network.peers).peer_ids();
                    for peer in peers {
                        heartbeat_network
                            .send(&peer, &heartbeat_bytes)
                            .unwrap_or_else(|err| {
                                error!("Unable to send heartbeat to {}: {:?}", peer, err)
                            });
                    }
                    thread::sleep(interval);
                }
            });
        }

        Ok(network)
    }

    pub fn peer_ids(&self) -> Vec<String> {
        rwlock_read_unwrap!(self.peers).peer_ids()
    }

    pub fn get_peer_endpoint(&self, peer_id: &str) -> Option<String> {
        rwlock_read_unwrap!(self.peers).get_peer_endpoint(peer_id)
    }

    pub fn get_peer_by_endpoint(&self, endpoint: &str) -> Option<String> {
        rwlock_read_unwrap!(self.peers).get_peer_by_endpoint(endpoint)
    }

    pub fn add_connection(
        &self,
        connection: Box<dyn Connection>,
    ) -> Result<String, ConnectionError> {
        let mut peers = rwlock_write_unwrap!(self.peers);
        let endpoint = connection.remote_endpoint();
        let mesh_id = self.mesh.add(connection)?;
        // Temp peer id until the connection has completed authorization
        let peer_id = format!("temp-{}", Uuid::new_v4());
        peers.insert(peer_id.clone(), mesh_id, endpoint);
        Ok(peer_id)
    }

    pub fn remove_connection(&self, peer_id: &str) -> Result<(), ConnectionError> {
        if let Some(mesh_id) = rwlock_write_unwrap!(self.peers).remove(peer_id) {
            self.mesh.remove(mesh_id)?;
        }

        Ok(())
    }

    pub fn add_peer(
        &self,
        peer_id: String,
        connection: Box<dyn Connection>,
    ) -> Result<(), ConnectionError> {
        // we already know the peers unique id
        let mut peers = rwlock_write_unwrap!(self.peers);
        let endpoint = connection.remote_endpoint();
        let mesh_id = self.mesh.add(connection)?;
        peers.insert(peer_id, mesh_id, endpoint);
        Ok(())
    }

    pub fn update_peer_id(&self, old_id: String, new_id: String) -> Result<(), PeerUpdateError> {
        rwlock_write_unwrap!(self.peers).update(old_id, new_id)
    }

    pub fn send(&self, peer_id: &str, msg: &[u8]) -> Result<(), SendError> {
        let mesh_id = match rwlock_read_unwrap!(self.peers).get_mesh_id(peer_id) {
            Some(mesh_id) => *mesh_id,
            None => {
                return Err(SendError::NoPeerError(format!(
                    "Send Error: No peer with peer_id {} found",
                    peer_id
                )));
            }
        };

        match self.mesh.send(Envelope::new(mesh_id, msg.to_vec())) {
            Ok(()) => (),
            Err(MeshSendError::Disconnected(err)) => {
                rwlock_write_unwrap!(self.peers).remove(peer_id);
                return Err(SendError::from(MeshSendError::Disconnected(err)));
            }
            Err(err) => return Err(SendError::from(err)),
        }

        Ok(())
    }

    pub fn recv(&self) -> Result<NetworkMessageWrapper, RecvError> {
        let envelope = self.mesh.recv()?;
        let peer_id = match rwlock_read_unwrap!(self.peers).get_peer_id(envelope.id()) {
            Some(peer_id) => peer_id.to_string(),
            None => {
                return Err(RecvError::NoPeerError(format!(
                    "Recv Error: No Peer with mesh id {} found",
                    envelope.id()
                )));
            }
        };

        Ok(NetworkMessageWrapper::new(peer_id, envelope.take_payload()))
    }

    pub fn recv_timeout(
        &self,
        timeout: Duration,
    ) -> Result<NetworkMessageWrapper, RecvTimeoutError> {
        let envelope = self.mesh.recv_timeout(timeout)?;
        let peer_id = match rwlock_read_unwrap!(self.peers).get_peer_id(envelope.id()) {
            Some(peer_id) => peer_id.to_string(),
            None => {
                return Err(RecvTimeoutError::NoPeerError(format!(
                    "Recv Error: No Peer with mesh id {} found",
                    envelope.id()
                )));
            }
        };

        Ok(NetworkMessageWrapper::new(peer_id, envelope.take_payload()))
    }
}

// -------------- Errors --------------

#[derive(Debug)]
pub enum RecvError {
    NoPeerError(String),
    MeshError(String),
}

impl From<MeshRecvError> for RecvError {
    fn from(recv_error: MeshRecvError) -> Self {
        RecvError::MeshError(format!("Recv Error: {:?}", recv_error))
    }
}

#[derive(Debug)]
pub enum RecvTimeoutError {
    NoPeerError(String),
    Timeout,
    Disconnected,
}

impl From<MeshRecvTimeoutError> for RecvTimeoutError {
    fn from(recv_error: MeshRecvTimeoutError) -> Self {
        match recv_error {
            MeshRecvTimeoutError::Timeout => RecvTimeoutError::Timeout,
            MeshRecvTimeoutError::Disconnected => RecvTimeoutError::Disconnected,
        }
    }
}

#[derive(Debug)]
pub enum SendError {
    NoPeerError(String),
    MeshError(String),
}

impl From<MeshSendError> for SendError {
    fn from(recv_error: MeshSendError) -> Self {
        SendError::MeshError(format!("Send Error: {:?}", recv_error))
    }
}

#[derive(Debug)]
pub enum ConnectionError {
    AddError(String),
    RemoveError(String),
}

impl std::error::Error for ConnectionError {}

impl std::fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ConnectionError::AddError(msg) => write!(f, "unable to add connection: {}", msg),
            ConnectionError::RemoveError(msg) => write!(f, "unable to remove connection: {}", msg),
        }
    }
}

#[derive(Debug)]
pub struct NetworkStartUpError(String);

impl std::error::Error for NetworkStartUpError {}

impl std::fmt::Display for NetworkStartUpError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "network failed to startup: {}", self.0)
    }
}

impl From<AddError> for ConnectionError {
    fn from(add_error: AddError) -> Self {
        ConnectionError::AddError(format!("Add Error: {:?}", add_error))
    }
}

impl From<RemoveError> for ConnectionError {
    fn from(remove_error: RemoveError) -> Self {
        ConnectionError::RemoveError(format!("Remove Error: {:?}", remove_error))
    }
}

#[derive(Debug)]
pub struct PeerUpdateError {}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::transport::raw::RawTransport;
    use crate::transport::Transport;
    use std::fmt::Debug;
    use std::thread;

    fn assert_ok<T, E: Debug>(result: Result<T, E>) -> T {
        match result {
            Ok(ok) => ok,
            Err(err) => panic!("Expected Ok(...), got Err({:?})", err),
        }
    }

    #[test]
    fn test_network() {
        // Setup the first network
        let mesh_one = Mesh::new(5, 5);
        let network_one = Network::new(mesh_one, 2).unwrap();

        let mut transport = RawTransport::default();

        let mut listener = assert_ok(transport.listen("127.0.0.1:0"));
        let endpoint = listener.endpoint();

        thread::spawn(move || {
            // Setup second network
            let mesh_two = Mesh::new(5, 5);
            let network_two = Network::new(mesh_two, 2).unwrap();

            // connect to listener and add connection to network
            let connection = assert_ok(transport.connect(&endpoint));
            assert_ok(network_two.add_connection(connection));

            // block until the message is received that contains the connection peer_id
            let message = assert_ok(network_two.recv());
            assert_eq!(b"345", message.payload());

            // update connection to have correct peer_id
            let peer_id = String::from_utf8(message.payload().to_vec()).unwrap();
            assert_ok(network_two.update_peer_id(message.peer_id().into(), peer_id.clone()));
            // verify that the peer has been updated
            assert_eq!(vec![peer_id.clone()], network_two.peer_ids());

            // send hello world
            assert_ok(network_two.send(&peer_id, b"hello_world"));
        });

        // accept connection
        let connection = assert_ok(listener.accept());
        let remote_endpoint = connection.remote_endpoint();

        // add peer with peer id 123
        assert_ok(network_one.add_peer("123".into(), connection));
        assert_eq!(
            Some("123".into()),
            network_one.get_peer_by_endpoint(&remote_endpoint)
        );
        // send 123 a peer id
        assert_ok(network_one.send("123".into(), b"345"));

        // wait to receive hello world from peer 123
        let message = assert_ok(network_one.recv());
        assert_eq!("123", message.peer_id());
        assert_eq!(b"hello_world", message.payload());

        let heartbeat = NetworkHeartbeat::new().write_to_bytes().unwrap();
        let mut heartbeat_message = NetworkMessage::new();
        heartbeat_message.set_message_type(NetworkMessageType::NETWORK_HEARTBEAT);
        heartbeat_message.set_payload(heartbeat);
        let heartbeat_bytes = heartbeat_message.write_to_bytes().unwrap();

        // wait for heartbeat
        let message = assert_ok(network_one.recv());
        assert_eq!("123", message.peer_id());
        assert_eq!(heartbeat_bytes, message.payload());
    }
}
