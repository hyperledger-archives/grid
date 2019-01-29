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
pub mod dispatch;
mod dispatch_proto;
pub mod handlers;
pub mod sender;

use uuid::Uuid;

use std::sync::{Arc, RwLock};

use crate::collections::BiHashMap;
use crate::mesh::{
    AddError, Envelope, Mesh, RecvError as MeshRecvError, RemoveError, SendError as MeshSendError,
};
use crate::transport::Connection;

#[derive(Debug)]
pub struct NetworkMessage {
    peer_id: String,
    payload: Vec<u8>,
}

impl NetworkMessage {
    pub fn new(peer_id: String, payload: Vec<u8>) -> Self {
        NetworkMessage { peer_id, payload }
    }
    pub fn peer_id(&self) -> &str {
        &self.peer_id
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
}

#[derive(Clone)]
pub struct Network {
    // Peer Id to Connection Id
    peers: Arc<RwLock<BiHashMap<String, usize>>>,
    mesh: Mesh,
}

impl Network {
    pub fn new(mesh: Mesh) -> Self {
        Network {
            peers: Arc::new(RwLock::new(BiHashMap::new())),
            mesh,
        }
    }

    pub fn peer_ids(&self) -> Vec<String> {
        rwlock_read_unwrap!(self.peers)
            .keys()
            .map(|left| left.to_string())
            .collect()
    }

    pub fn add_connection(
        &self,
        connection: Box<dyn Connection>,
    ) -> Result<String, ConnectionError> {
        let mut peers = rwlock_write_unwrap!(self.peers);
        let mesh_id = self.mesh.add(connection)?;
        // Temp peer id until the connection has completed authorization
        let peer_id = format!("temp-{}", Uuid::new_v4());
        peers.insert(peer_id.clone(), mesh_id);
        Ok(peer_id)
    }

    pub fn remove_connection(&self, peer_id: &String) -> Result<(), ConnectionError> {
        if let Some((_, mesh_id)) = rwlock_write_unwrap!(self.peers).remove_by_key(peer_id) {
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
        let mesh_id = self.mesh.add(connection)?;
        peers.insert(peer_id, mesh_id);
        Ok(())
    }

    pub fn update_peer_id(&self, old_id: String, new_id: String) -> Result<(), PeerUpdateError> {
        let mut peers = rwlock_write_unwrap!(self.peers);
        if let Some((_, mesh_id)) = peers.remove_by_key(&old_id) {
            peers.insert(new_id, mesh_id);
            Ok(())
        } else {
            Err(PeerUpdateError {})
        }
    }

    pub fn send(&self, peer_id: String, msg: &[u8]) -> Result<(), SendError> {
        let mesh_id = match rwlock_read_unwrap!(self.peers).get_by_key(&peer_id) {
            Some(mesh_id) => *mesh_id,
            None => {
                return Err(SendError::NoPeerError(format!(
                    "Send Error: No peer with peer_id {} found",
                    peer_id
                )));
            }
        };

        self.mesh.send(Envelope::new(mesh_id, msg.to_vec()))?;
        Ok(())
    }

    pub fn recv(&self) -> Result<NetworkMessage, RecvError> {
        let envelope = self.mesh.recv()?;
        let peer_id = match rwlock_read_unwrap!(self.peers).get_by_value(&envelope.id()) {
            Some(peer_id) => peer_id.to_string(),
            None => {
                return Err(RecvError::NoPeerError(format!(
                    "Recv Error: No Peer with mesh id {} found",
                    envelope.id()
                )));
            }
        };

        Ok(NetworkMessage::new(peer_id, envelope.take_payload()))
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
        let network_one = Network::new(mesh_one);

        let mut transport = RawTransport::default();

        let mut listener = assert_ok(transport.listen("127.0.0.1:0"));
        let endpoint = listener.endpoint();

        thread::spawn(move || {
            // Setup second network
            let mesh_two = Mesh::new(5, 5);
            let network_two = Network::new(mesh_two);

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
            assert_ok(network_two.send(peer_id, b"hello_world"));
        });

        // accept connection
        let connection = assert_ok(listener.accept());

        // add peer with peer id 123
        assert_ok(network_one.add_peer("123".into(), connection));
        // send 123 a peer id
        assert_ok(network_one.send("123".into(), b"345"));

        // wait to receive hello world from peer 123
        let message = assert_ok(network_one.recv());
        assert_eq!("123", message.peer_id());
        assert_eq!(b"hello_world", message.payload());
    }

}
