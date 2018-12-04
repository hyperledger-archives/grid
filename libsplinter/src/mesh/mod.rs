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

mod control;
mod incoming;
mod outgoing;
mod pool;
mod reactor;

use std::collections::HashMap;
use std::io;
use std::sync::{Arc, RwLock};

pub use mesh::control::{AddError, Control, RemoveError};
pub use mesh::incoming::Incoming;
pub use mesh::outgoing::Outgoing;

use mesh::reactor::Reactor;
use transport::Connection;

/// Wrapper around payload to include connection id
#[derive(Debug, Default, PartialEq)]
pub struct Envelope {
    id: usize,
    payload: Vec<u8>,
}

impl Envelope {
    pub fn new(id: usize, payload: Vec<u8>) -> Self {
        Envelope { id, payload }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    pub fn take_payload(self) -> Vec<u8> {
        self.payload
    }
}

/// A Connection reactor
#[derive(Clone)]
pub struct Mesh {
    outgoings: Arc<RwLock<HashMap<usize, Outgoing>>>,
    incoming: Incoming,
    ctrl: Control,
}

impl Mesh {
    /// Create a new mesh, spawning a background thread for sending and receiving, and setting up
    /// channels to communicate with it.
    pub fn new(incoming_capacity: usize, outgoing_capacity: usize) -> Self {
        let (ctrl, incoming) = Reactor::spawn(incoming_capacity, outgoing_capacity);
        Mesh {
            outgoings: Arc::new(RwLock::new(HashMap::new())),
            incoming,
            ctrl,
        }
    }

    /// Add a new connection to the mesh, moving it to the background thread, and return its id.
    pub fn add(&self, connection: Box<dyn Connection>) -> Result<usize, AddError> {
        let outgoing = self.ctrl.add(connection)?;
        let id = outgoing.id();
        self.outgoings.write().unwrap().insert(id, outgoing);

        Ok(id)
    }

    /// Remove an existing connection from the mesh and return it.
    pub fn remove(&self, id: usize) -> Result<Box<dyn Connection>, RemoveError> {
        let connection = self.ctrl.remove(id)?;
        // The outgoing channel needs to be removed after the control request completes, or else
        // the reactor will detect that the outgoing sender has dropped and clean it up
        // automatically, causing the control request to fail with NotFound.
        self.outgoings.write().unwrap().remove(&id);
        Ok(connection)
    }

    /// Send the envelope on the mesh.
    ///
    /// This is a convenience function and is equivalent to
    /// `mesh.outgoing(envelope.id()).send(envelope.take_payload())`.
    pub fn send(&self, envelope: Envelope) -> Result<(), SendError> {
        let outgoings = self.outgoings.read().unwrap();
        let id = envelope.id();
        match outgoings.get(&envelope.id()) {
            Some(ref outgoing) => match outgoing.send(envelope.take_payload()) {
                Ok(()) => Ok(()),
                Err(err) => Err(SendError::from_outgoing_send_error(err, id)),
            },
            None => Err(SendError::NotFound),
        }
    }

    /// Receive a new envelope from the mesh.
    ///
    /// This is a convenience function that avoids an extra clone, and is equivalent to
    /// `mesh.incoming().recv()`.
    pub fn recv(&self) -> Result<Envelope, RecvError> {
        self.incoming.recv().map_err(|_| RecvError)
    }

    /// Create a new handle for sending to the existing connection with the given id.
    ///
    /// This may be faster if many sends on the same Connection are going to be performed because
    // the internal lock around the pool of senders does not need to be reacquired.
    pub fn outgoing(&self, id: usize) -> Option<Outgoing> {
        self.outgoings.read().unwrap().get(&id).cloned()
    }

    /// Create a new handle for receiving envelopes from the mesh.
    ///
    /// This is useful if an object only needs to receive and doesn't need to send.
    pub fn incoming(&self) -> Incoming {
        self.incoming.clone()
    }
}

#[derive(Debug)]
pub enum SendError {
    NotFound,
    IoError(io::Error),
    Full(Envelope),
    Disconnected(Envelope),
}

impl SendError {
    fn from_outgoing_send_error(err: outgoing::SendError, id: usize) -> Self {
        match err {
            outgoing::SendError::IoError(err) => SendError::IoError(err),
            outgoing::SendError::Full(payload) => SendError::Full(Envelope::new(id, payload)),
            outgoing::SendError::Disconnected(payload) => {
                SendError::Disconnected(Envelope::new(id, payload))
            }
        }
    }
}

#[derive(Debug)]
pub struct RecvError;

#[cfg(test)]
mod tests {
    use super::*;

    use std::fmt::Debug;
    use std::sync::mpsc::channel;
    use std::thread;

    use transport::{raw::RawTransport, tls::tests::create_test_tls_transport, Transport};

    fn assert_ok<T, E: Debug>(result: Result<T, E>) -> T {
        match result {
            Ok(ok) => ok,
            Err(err) => panic!("Expected Ok(...), got Err({:?})", err),
        }
    }

    // Test that connections added to a Mesh can send and receive successfully
    fn test_single_connection_send_receive<T: Transport + Send + 'static>(
        mut transport: T,
        bind: &str,
    ) {
        let mut listener = assert_ok(transport.listen(bind));
        let endpoint = listener.endpoint();

        let handle = thread::spawn(move || {
            let client = assert_ok(transport.connect(&endpoint));

            let mesh = Mesh::new(1, 1);
            let id = assert_ok(mesh.add(client));

            assert_ok(mesh.send(Envelope::new(id, b"hello".to_vec())));
        });

        let mesh = Mesh::new(1, 1);
        let server = assert_ok(listener.accept());
        assert_ok(mesh.add(server));

        let envelope = assert_ok(mesh.recv());

        assert_eq!(b"hello", envelope.payload());

        handle.join().unwrap();
    }

    // Test that connections can be added and removed from a Mesh
    fn test_add_remove_connections<T: Transport + Send + 'static>(mut transport: T, bind: &str) {
        let mut listener = assert_ok(transport.listen(bind));
        let endpoint = listener.endpoint();

        let mesh = Mesh::new(0, 0);

        let mut ids = Vec::new();

        let mesh_clone = mesh.clone();
        let handle = thread::spawn(move || {
            let mesh = mesh_clone;

            let mut ids = Vec::new();
            for _ in 0..8 {
                let conn = assert_ok(transport.connect(&endpoint));
                let id = assert_ok(mesh.add(conn));
                ids.push(id);
            }

            ids
        });

        for _ in 0..8 {
            let conn = assert_ok(listener.accept());
            let id = assert_ok(mesh.add(conn));
            ids.push(id);
        }

        ids.extend(handle.join().unwrap().as_slice());
        ids.sort();

        for id in &ids {
            assert_ok(mesh.remove(*id));
        }
    }

    // Test that many connections can be added to a Mesh and sent and received from
    fn test_many_connections<T: Transport + Send + 'static>(mut transport: T, bind: &str) {
        const CONNECTIONS: usize = 16;

        let mut listener = assert_ok(transport.listen(bind));
        let endpoint = listener.endpoint();

        let mesh = Mesh::new(CONNECTIONS, CONNECTIONS);

        let (client_ready_tx, client_ready_rx) = channel();
        let (server_ready_tx, server_ready_rx) = channel();

        let mesh_clone = mesh.clone();
        let handle = thread::spawn(move || {
            let mesh = mesh_clone;

            for _ in 0..CONNECTIONS {
                assert_ok(mesh.add(assert_ok(transport.connect(&endpoint))));
            }

            // Block waiting for other thread to send everything
            client_ready_rx.recv().unwrap();

            let mut ids = Vec::with_capacity(CONNECTIONS);
            for _ in 0..CONNECTIONS {
                let envelope = assert_ok(mesh.recv());
                assert_eq!(b"hello", envelope.payload());
                ids.push(envelope.id());
            }

            // Signal to other thread we are done receiving
            server_ready_tx.send(()).unwrap();

            for id in ids {
                assert_ok(mesh.send(Envelope::new(id, b"world".to_vec())));
            }
        });

        for _ in 0..CONNECTIONS {
            let conn = assert_ok(listener.accept());
            let id = assert_ok(mesh.add(conn));
            assert_ok(mesh.send(Envelope::new(id, b"hello".to_vec())));
        }

        // Signal done sending to background thread
        client_ready_tx.send(()).unwrap();
        // Wait for other thread to drain the queue so we don't accidentally receive messages sent
        // to that thread
        server_ready_rx.recv().unwrap();

        let incoming = mesh.incoming();
        for _ in 0..CONNECTIONS {
            let envelope = assert_ok(incoming.recv());
            assert_eq!(b"world", envelope.payload());
        }

        handle.join().unwrap();
    }

    #[test]
    fn test_connection_send_receive_raw() {
        let raw = RawTransport::default();
        test_single_connection_send_receive(raw, "127.0.0.1:0");
    }

    #[test]
    fn test_connection_send_receive_tls() {
        let tls = create_test_tls_transport();
        test_single_connection_send_receive(tls, "127.0.0.1:0");
    }

    #[test]
    fn test_add_remove_connections_raw() {
        let raw = RawTransport::default();
        test_add_remove_connections(raw, "127.0.0.1:0");
    }

    #[test]
    fn test_many_connections_raw() {
        let raw = RawTransport::default();
        test_many_connections(raw, "127.0.0.1:0");
    }

    #[test]
    fn test_many_connections_tls() {
        let tls = create_test_tls_transport();
        test_many_connections(tls, "127.0.0.1:0");
    }

    #[test]
    fn test_add_remove_connections_tls() {
        let tls = create_test_tls_transport();
        test_add_remove_connections(tls, "127.0.0.1:0");
    }
}
