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

mod error;
mod messages;
mod pacemaker;

use std;
use std::collections::HashMap;
#[cfg(feature = "connection-manager-notification-iter-try-next")]
use std::sync::mpsc::TryRecvError;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::thread;

pub use error::ConnectionManagerError;
pub use messages::{CmMessage, CmNotification, CmPayload, CmRequest, CmResponse, CmResponseStatus};
use pacemaker::Pacemaker;
use protobuf::Message;

use crate::mesh::{Envelope, Mesh};
use crate::protos::network::{NetworkHeartbeat, NetworkMessage, NetworkMessageType};
use crate::transport::Transport;

const DEFAULT_HEARTBEAT_INTERVAL: u64 = 10;
const CHANNEL_CAPACITY: usize = 15;

pub struct ConnectionManager {
    pacemaker: Pacemaker,
    connection_state: Option<ConnectionState>,
    join_handle: Option<thread::JoinHandle<()>>,
    sender: Option<SyncSender<CmMessage>>,
    shutdown_handle: Option<ShutdownHandle>,
}

impl ConnectionManager {
    pub fn new(mesh: Mesh, transport: Box<dyn Transport + Send>) -> Self {
        let connection_state = Some(ConnectionState::new(mesh, transport));
        let pacemaker = Pacemaker::new(DEFAULT_HEARTBEAT_INTERVAL);

        Self {
            pacemaker,
            connection_state,
            join_handle: None,
            sender: None,
            shutdown_handle: None,
        }
    }

    pub fn start(&mut self) -> Result<Connector, ConnectionManagerError> {
        let (sender, recv) = sync_channel(CHANNEL_CAPACITY);
        let mut state = self.connection_state.take().ok_or_else(|| {
            ConnectionManagerError::StartUpError("Service has already started".into())
        })?;

        let join_handle = thread::Builder::new()
            .name("Connection Manager".into())
            .spawn(move || {
                let mut subscribers = Vec::new();
                loop {
                    match recv.recv() {
                        Ok(CmMessage::Shutdown) => break,
                        Ok(CmMessage::Subscribe(sender)) => {
                            subscribers.push(sender);
                        }
                        Ok(CmMessage::Request(req)) => {
                            handle_request(req, &mut state);
                        }
                        Ok(CmMessage::SendHeartbeats) => {
                            send_heartbeats(&mut state, &mut subscribers)
                        }
                        Err(_) => {
                            warn!("All senders have disconnected");
                            break;
                        }
                    }
                }
            })?;

        self.pacemaker.start(sender.clone())?;
        self.join_handle = Some(join_handle);
        self.shutdown_handle = Some(ShutdownHandle {
            sender: sender.clone(),
            pacemaker_shutdown_handle: self.pacemaker.shutdown_handle().unwrap(),
        });
        self.sender = Some(sender.clone());

        Ok(Connector { sender })
    }

    pub fn shutdown_handle(&self) -> Option<ShutdownHandle> {
        self.shutdown_handle.clone()
    }

    pub fn await_shutdown(self) {
        self.pacemaker.await_shutdown();

        let join_handle = if let Some(jh) = self.join_handle {
            jh
        } else {
            return;
        };

        if let Err(err) = join_handle.join() {
            error!(
                "Connection manager thread did not shutdown correctly: {:?}",
                err
            );
        }
    }

    pub fn shutdown_and_wait(self) {
        if let Some(sh) = self.shutdown_handle.clone() {
            sh.shutdown();
        } else {
            return;
        }

        self.await_shutdown();
    }
}

#[derive(Clone)]
pub struct Connector {
    sender: SyncSender<CmMessage>,
}

impl Connector {
    pub fn request_connection(&self, endpoint: &str) -> Result<CmResponse, ConnectionManagerError> {
        self.send_payload(CmPayload::AddConnection {
            endpoint: endpoint.to_string(),
        })
    }

    pub fn remove_connection(&self, endpoint: &str) -> Result<CmResponse, ConnectionManagerError> {
        self.send_payload(CmPayload::RemoveConnection {
            endpoint: endpoint.to_string(),
        })
    }

    pub fn subscribe(&self) -> Result<NotificationHandler, ConnectionManagerError> {
        let (send, recv) = sync_channel(CHANNEL_CAPACITY);
        match self.sender.send(CmMessage::Subscribe(send)) {
            Ok(()) => Ok(NotificationHandler { recv }),
            Err(_) => Err(ConnectionManagerError::SendMessageError(
                "The connection manager is no longer running".into(),
            )),
        }
    }

    fn send_payload(&self, payload: CmPayload) -> Result<CmResponse, ConnectionManagerError> {
        let (sender, recv) = sync_channel(1);

        let message = CmMessage::Request(CmRequest { sender, payload });

        match self.sender.send(message) {
            Ok(()) => (),
            Err(_) => {
                return Err(ConnectionManagerError::SendMessageError(
                    "The connection manager is no longer running".into(),
                ))
            }
        };

        recv.recv()
            .map_err(|err| ConnectionManagerError::SendMessageError(format!("{:?}", err)))
    }
}

#[derive(Clone)]
pub struct ShutdownHandle {
    sender: SyncSender<CmMessage>,
    pacemaker_shutdown_handle: pacemaker::ShutdownHandle,
}

impl ShutdownHandle {
    pub fn shutdown(&self) {
        self.pacemaker_shutdown_handle.shutdown();

        if self.sender.send(CmMessage::Shutdown).is_err() {
            warn!("Connection manager is no longer running");
        }
    }
}

pub struct NotificationHandler {
    recv: Receiver<CmNotification>,
}

#[cfg(feature = "connection-manager-notification-iter-try-next")]
impl NotificationHandler {
    pub fn try_next(&self) -> Result<Option<CmNotification>, ConnectionManagerError> {
        match self.recv.try_recv() {
            Ok(notifications) => Ok(Some(notifications)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => Err(ConnectionManagerError::SendMessageError(
                "The connection manager is no longer running".into(),
            )),
        }
    }
}

impl Iterator for NotificationHandler {
    type Item = CmNotification;

    fn next(&mut self) -> Option<Self::Item> {
        match self.recv.recv() {
            Ok(notification) => Some(notification),
            Err(_) => {
                // This is expected if the connection manager shuts down before
                // this end
                None
            }
        }
    }
}

#[derive(Clone)]
struct ConnectionMetadata {
    id: usize,
    endpoint: String,
    ref_count: u64,
}

struct ConnectionState {
    connections: HashMap<String, ConnectionMetadata>,
    mesh: Mesh,
    transport: Box<dyn Transport>,
}

impl ConnectionState {
    fn new(mesh: Mesh, transport: Box<dyn Transport + Send>) -> Self {
        Self {
            mesh,
            transport,
            connections: HashMap::new(),
        }
    }

    fn add_connection(&mut self, endpoint: &str) -> Result<(), ConnectionManagerError> {
        if let Some(meta) = self.connections.get_mut(endpoint) {
            meta.ref_count += 1;
        } else {
            let connection = self.transport.connect(endpoint).map_err(|err| {
                ConnectionManagerError::ConnectionCreationError(format!("{:?}", err))
            })?;

            let id = self.mesh.add(connection).map_err(|err| {
                ConnectionManagerError::ConnectionCreationError(format!("{:?}", err))
            })?;

            self.connections.insert(
                endpoint.to_string(),
                ConnectionMetadata {
                    id,
                    endpoint: endpoint.to_string(),
                    ref_count: 1,
                },
            );
        };

        Ok(())
    }

    fn remove_connection(
        &mut self,
        endpoint: &str,
    ) -> Result<Option<ConnectionMetadata>, ConnectionManagerError> {
        let meta = if let Some(meta) = self.connections.get_mut(endpoint) {
            meta.ref_count -= 1;
            meta.clone()
        } else {
            return Ok(None);
        };

        if meta.ref_count < 1 {
            self.connections.remove(endpoint);
            self.mesh.remove(meta.id).map_err(|err| {
                ConnectionManagerError::ConnectionRemovalError(format!("{:?}", err))
            })?;
        }

        Ok(Some(meta))
    }

    fn reconnect(&mut self, endpoint: &str) -> Result<(), ConnectionManagerError> {
        self.remove_connection(endpoint)?;
        self.add_connection(endpoint)
    }

    fn connection_metadata(&self) -> HashMap<String, ConnectionMetadata> {
        self.connections.clone()
    }

    fn mesh(&self) -> Mesh {
        self.mesh.clone()
    }
}

fn handle_request(req: CmRequest, state: &mut ConnectionState) {
    let response = match req.payload {
        CmPayload::AddConnection { ref endpoint } => {
            if let Err(err) = state.add_connection(endpoint) {
                CmResponse::AddConnection {
                    status: CmResponseStatus::Error,
                    error_message: Some(format!("{:?}", err)),
                }
            } else {
                CmResponse::AddConnection {
                    status: CmResponseStatus::OK,
                    error_message: None,
                }
            }
        }
        CmPayload::RemoveConnection { ref endpoint } => match state.remove_connection(endpoint) {
            Ok(Some(_)) => CmResponse::RemoveConnection {
                status: CmResponseStatus::OK,
                error_message: None,
            },
            Ok(None) => CmResponse::RemoveConnection {
                status: CmResponseStatus::ConnectionNotFound,
                error_message: None,
            },
            Err(err) => CmResponse::RemoveConnection {
                status: CmResponseStatus::Error,
                error_message: Some(format!("{:?}", err)),
            },
        },
    };

    if req.sender.send(response).is_err() {
        error!("Requester has dropped its connection to connection manager");
    }
}

fn notify_subscribers(
    subscribers: &mut Vec<SyncSender<CmNotification>>,
    notification: CmNotification,
) {
    subscribers.retain(|sender| {
        if sender.send(notification.clone()).is_err() {
            false
        } else {
            true
        }
    });
}

fn send_heartbeats(state: &mut ConnectionState, subscribers: &mut Vec<SyncSender<CmNotification>>) {
    let heartbeat_message = match create_heartbeat() {
        Ok(h) => h,
        Err(err) => {
            error!("Failed to create heartbeat message: {:?}", err);
            return;
        }
    };
    for (endpoint, metadata) in state.connection_metadata() {
        info!("Sending heartbeat to {}", endpoint);
        if let Err(err) = state
            .mesh()
            .send(Envelope::new(metadata.id, heartbeat_message.clone()))
        {
            error!(
                "failed to send heartbeat: {:?} attempting reconnection",
                err
            );

            notify_subscribers(
                subscribers,
                CmNotification::HeartbeatSendFail {
                    endpoint: endpoint.clone(),
                    message: format!("{:?}", err),
                },
            );

            if let Err(err) = state.reconnect(&endpoint) {
                error!("Connection reattempt failed: {:?}", err);
                notify_subscribers(
                    subscribers,
                    CmNotification::ReconnectAttemptFailed {
                        endpoint: endpoint.clone(),
                        message: format!("{:?}", err),
                    },
                );
            } else {
                notify_subscribers(
                    subscribers,
                    CmNotification::ReconnectAttemptSuccess {
                        endpoint: endpoint.clone(),
                    },
                );
            }
        } else {
            notify_subscribers(
                subscribers,
                CmNotification::HeartbeatSent {
                    endpoint: endpoint.clone(),
                },
            );
        }
    }
}

fn create_heartbeat() -> Result<Vec<u8>, ConnectionManagerError> {
    let heartbeat = NetworkHeartbeat::new().write_to_bytes().map_err(|_| {
        ConnectionManagerError::HeartbeatError("cannot create NetworkHeartbeat message".to_string())
    })?;
    let mut heartbeat_message = NetworkMessage::new();
    heartbeat_message.set_message_type(NetworkMessageType::NETWORK_HEARTBEAT);
    heartbeat_message.set_payload(heartbeat);
    let heartbeat_bytes = heartbeat_message.write_to_bytes().map_err(|_| {
        ConnectionManagerError::HeartbeatError("cannot create NetworkMessage".to_string())
    })?;
    Ok(heartbeat_bytes)
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::transport::inproc::InprocTransport;
    use crate::transport::raw::RawTransport;

    #[test]
    fn test_connection_manager_startup_and_shutdown() {
        let mut transport = Box::new(InprocTransport::default());
        transport.listen("inproc://test").unwrap();
        let mesh = Mesh::new(512, 128);

        let mut cm = ConnectionManager::new(mesh, transport);

        cm.start().unwrap();
        cm.shutdown_and_wait();
    }

    #[test]
    fn test_add_connection_request() {
        let mut transport = Box::new(InprocTransport::default());
        let mut listener = transport.listen("inproc://test").unwrap();

        thread::spawn(move || {
            listener.accept().unwrap();
        });

        let mesh = Mesh::new(512, 128);
        let mut cm = ConnectionManager::new(mesh, transport);
        let connector = cm.start().unwrap();

        let response = connector.request_connection("inproc://test").unwrap();

        assert_eq!(
            response,
            CmResponse::AddConnection {
                status: CmResponseStatus::OK,
                error_message: None
            }
        );

        cm.shutdown_and_wait();
    }

    /// Test that adding the same connection twice is an idempotent operation
    #[test]
    fn test_mutiple_add_connection_requests() {
        let mut transport = Box::new(InprocTransport::default());
        let mut listener = transport.listen("inproc://test").unwrap();

        thread::spawn(move || {
            listener.accept().unwrap();
        });

        let mesh = Mesh::new(512, 128);
        let mut cm = ConnectionManager::new(mesh, transport);
        let connector = cm.start().unwrap();

        let response = connector.request_connection("inproc://test").unwrap();

        assert_eq!(
            response,
            CmResponse::AddConnection {
                status: CmResponseStatus::OK,
                error_message: None
            }
        );

        let response = connector.request_connection("inproc://test").unwrap();
        assert_eq!(
            response,
            CmResponse::AddConnection {
                status: CmResponseStatus::OK,
                error_message: None
            }
        );

        cm.shutdown_and_wait();
    }

    /// test_heartbeat_notifications
    ///
    /// Test that heartbeats are correctly sent
    /// to subscribers
    #[test]
    fn test_heartbeat_notifications() {
        let mut transport = Box::new(InprocTransport::default());
        let mut listener = transport.listen("inproc://test").unwrap();
        let mesh = Mesh::new(512, 128);
        let mesh_clone = mesh.clone();

        thread::spawn(move || {
            let conn = listener.accept().unwrap();
            mesh_clone.add(conn).unwrap();
        });

        let mut cm = ConnectionManager::new(mesh.clone(), transport);
        let connector = cm.start().unwrap();

        let response = connector.request_connection("inproc://test").unwrap();

        assert_eq!(
            response,
            CmResponse::AddConnection {
                status: CmResponseStatus::OK,
                error_message: None
            }
        );

        let mut subscriber = connector.subscribe().unwrap();

        let notification = subscriber.next().unwrap();

        assert!(
            notification
                == CmNotification::HeartbeatSent {
                    endpoint: "inproc://test".to_string(),
                }
        );

        // Verify mesh received heartbeat

        let envelope = mesh.recv().unwrap();
        let heartbeat: NetworkMessage = protobuf::parse_from_bytes(&envelope.payload()).unwrap();
        assert_eq!(
            heartbeat.get_message_type(),
            NetworkMessageType::NETWORK_HEARTBEAT
        );
    }

    #[test]
    fn test_heartbeat_notifications_raw_tcp() {
        let mut transport = Box::new(RawTransport::default());
        let mut listener = transport.listen("tcp://localhost:3030").unwrap();
        let mesh = Mesh::new(512, 128);
        let mesh_clone = mesh.clone();

        thread::spawn(move || {
            let conn = listener.accept().unwrap();
            mesh_clone.add(conn).unwrap();
        });

        let mut cm = ConnectionManager::new(mesh.clone(), transport);
        let connector = cm.start().unwrap();

        let response = connector
            .request_connection("tcp://localhost:3030")
            .unwrap();

        assert_eq!(
            response,
            CmResponse::AddConnection {
                status: CmResponseStatus::OK,
                error_message: None
            }
        );

        let mut subscriber = connector.subscribe().unwrap();

        let notification = subscriber.next().unwrap();

        assert!(
            notification
                == CmNotification::HeartbeatSent {
                    endpoint: "tcp://localhost:3030".to_string(),
                }
        );

        // Verify mesh received heartbeat

        let envelope = mesh.recv().unwrap();
        let heartbeat: NetworkMessage = protobuf::parse_from_bytes(&envelope.payload()).unwrap();
        assert_eq!(
            heartbeat.get_message_type(),
            NetworkMessageType::NETWORK_HEARTBEAT
        );
    }

    #[test]
    fn test_remove_connection() {
        let mut transport = Box::new(RawTransport::default());
        let mut listener = transport.listen("tcp://localhost:3030").unwrap();
        let mesh = Mesh::new(512, 128);
        let mesh_clone = mesh.clone();

        thread::spawn(move || {
            let conn = listener.accept().unwrap();
            mesh_clone.add(conn).unwrap();
        });

        let mut cm = ConnectionManager::new(mesh.clone(), transport);
        let connector = cm.start().unwrap();

        let add_response = connector
            .request_connection("tcp://localhost:3030")
            .unwrap();

        assert_eq!(
            add_response,
            CmResponse::AddConnection {
                status: CmResponseStatus::OK,
                error_message: None
            }
        );

        let remove_response = connector.remove_connection("tcp://localhost:3030").unwrap();

        assert_eq!(
            remove_response,
            CmResponse::RemoveConnection {
                status: CmResponseStatus::OK,
                error_message: None
            }
        );
    }

    #[test]
    fn test_remove_nonexistent_connection() {
        let mut transport = Box::new(RawTransport::default());
        let mut listener = transport.listen("tcp://localhost:3030").unwrap();
        let mesh = Mesh::new(512, 128);
        let mesh_clone = mesh.clone();

        thread::spawn(move || {
            let conn = listener.accept().unwrap();
            mesh_clone.add(conn).unwrap();
        });

        let mut cm = ConnectionManager::new(mesh.clone(), transport);
        let connector = cm.start().unwrap();

        let remove_response = connector.remove_connection("tcp://localhost:3030").unwrap();

        assert_eq!(
            remove_response,
            CmResponse::RemoveConnection {
                status: CmResponseStatus::ConnectionNotFound,
                error_message: None,
            }
        );
    }

    #[test]
    /// Tests that notifier iterator correctly exists when sender
    /// is dropped.
    ///
    /// Procedure:
    ///
    /// The test creates a sync channel and a notifier, then it
    /// creates a thread that send HeartbeatSent notifications to
    /// the notifier.
    ///
    /// Asserts:
    ///
    /// The notifications sent are received by the Notifier
    /// correctly
    ///
    /// That the total number of notifications sent equals 5
    fn test_notifications_handler_iterator() {
        let (send, recv) = sync_channel(2);

        let nh = NotificationHandler { recv };

        let join_handle = thread::spawn(move || {
            for _ in 0..5 {
                send.send(CmNotification::HeartbeatSent {
                    endpoint: "tcp://localhost:3030".to_string(),
                })
                .unwrap();
            }
        });

        let mut notifications_sent = 0;
        for n in nh {
            assert_eq!(
                n,
                CmNotification::HeartbeatSent {
                    endpoint: "tcp://localhost:3030".to_string()
                }
            );
            notifications_sent += 1;
        }

        assert_eq!(notifications_sent, 5);

        join_handle.join().unwrap();
    }
}
