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

use std;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::{sync_channel, Receiver, SyncSender},
    Arc,
};
use std::thread;
use std::time::Duration;

pub use error::ConnectionManagerError;
pub use messages::{CmMessage, CmNotification, CmPayload, CmRequest, CmResponse, CmResponseStatus};
use protobuf::Message;
use uuid::Uuid;

use crate::mesh::{Envelope, Mesh};
use crate::protos::network::{NetworkHeartbeat, NetworkMessage, NetworkMessageType};
use crate::transport::Transport;

const DEFAULT_HEARTBEAT_INTERVAL: u64 = 10;
const CHANNEL_CAPACITY: usize = 15;

pub struct ConnectionManager {
    hb_monitor: HeartbeatMonitor,
    connection_state: Option<ConnectionState>,
    join_handle: Option<thread::JoinHandle<()>>,
    sender: Option<SyncSender<CmMessage>>,
    shutdown_handle: Option<ShutdownHandle>,
}

impl ConnectionManager {
    pub fn new(mesh: Mesh, transport: Box<dyn Transport + Send>) -> Self {
        let connection_state = Some(ConnectionState::new(mesh, transport));
        let hb_monitor = HeartbeatMonitor::new(DEFAULT_HEARTBEAT_INTERVAL);

        Self {
            hb_monitor,
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
                let mut subscribers = HashMap::new();
                loop {
                    match recv.recv() {
                        Ok(CmMessage::Shutdown) => break,
                        Ok(CmMessage::Subscribe(id, sender)) => {
                            subscribers.insert(id, sender);
                        }
                        Ok(CmMessage::UnSubscribe(ref id)) => {
                            subscribers.remove(id);
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

        self.hb_monitor.start(sender.clone())?;
        self.join_handle = Some(join_handle);
        self.shutdown_handle = Some(ShutdownHandle {
            sender: sender.clone(),
            hb_shutdown_handle: self.hb_monitor.shutdown_handle().unwrap(),
        });
        self.sender = Some(sender.clone());

        Ok(Connector { sender })
    }

    pub fn shutdown_handle(&self) -> Option<ShutdownHandle> {
        self.shutdown_handle.clone()
    }

    pub fn await_shutdown(self) {
        self.hb_monitor.await_shutdown();

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

struct HeartbeatMonitor {
    interval: u64,
    join_handle: Option<thread::JoinHandle<()>>,
    shutdown_handle: Option<HbShutdownHandle>,
}

impl HeartbeatMonitor {
    fn new(interval: u64) -> Self {
        Self {
            interval,
            join_handle: None,
            shutdown_handle: None,
        }
    }

    fn start(&mut self, cm_sender: SyncSender<CmMessage>) -> Result<(), ConnectionManagerError> {
        if self.join_handle.is_some() {
            return Ok(());
        }

        let running = Arc::new(AtomicBool::new(true));

        let running_clone = running.clone();
        let interval = self.interval;
        let join_handle = thread::Builder::new()
            .name("Heartbeat Monitor".into())
            .spawn(move || {
                info!("Starting heartbeat manager");

                while running_clone.load(Ordering::SeqCst) {
                    thread::sleep(Duration::from_secs(interval));
                    if let Err(err) = cm_sender.send(CmMessage::SendHeartbeats) {
                        error!("Connection manager has disconnected before shutting down heartbeat monitor {:?}", err);
                        break;
                    }
                }
            })?;

        self.join_handle = Some(join_handle);
        self.shutdown_handle = Some(HbShutdownHandle { running });

        Ok(())
    }

    fn shutdown_handle(&self) -> Option<HbShutdownHandle> {
        self.shutdown_handle.clone()
    }

    fn await_shutdown(self) {
        let join_handle = if let Some(jh) = self.join_handle {
            jh
        } else {
            return;
        };

        if let Err(err) = join_handle.join() {
            error!("Failed to shutdown heartbeat monitor gracefully: {:?}", err);
        }
    }
}

#[derive(Clone)]
pub struct Connector {
    sender: SyncSender<CmMessage>,
}

impl Connector {
    pub fn request_connection(&self, endpoint: &str) -> Result<CmResponse, ConnectionManagerError> {
        let (sender, recv) = sync_channel(1);

        let message = CmMessage::Request(CmRequest {
            sender,
            payload: CmPayload::AddConnection {
                endpoint: endpoint.to_string(),
            },
        });

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

    pub fn subscribe(&self) -> Result<NotificationHandler, ConnectionManagerError> {
        let id = Uuid::new_v4().to_string();
        let (send, recv) = sync_channel(1);
        match self.sender.send(CmMessage::Subscribe(id.clone(), send)) {
            Ok(()) => Ok(NotificationHandler {
                id,
                recv,
                sender: self.sender.clone(),
            }),
            Err(_) => Err(ConnectionManagerError::SendMessageError(
                "The connection manager is no longer running".into(),
            )),
        }
    }
}

#[derive(Clone)]
pub struct ShutdownHandle {
    sender: SyncSender<CmMessage>,
    hb_shutdown_handle: HbShutdownHandle,
}

impl ShutdownHandle {
    pub fn shutdown(&self) {
        self.hb_shutdown_handle.shutdown();

        if let Err(_) = self.sender.send(CmMessage::Shutdown) {
            warn!("Connection manager is no longer running");
        }
    }
}

#[derive(Clone)]
struct HbShutdownHandle {
    running: Arc<AtomicBool>,
}

impl HbShutdownHandle {
    pub fn shutdown(&self) {
        self.running.store(false, Ordering::SeqCst)
    }
}

pub struct NotificationHandler {
    id: String,
    sender: SyncSender<CmMessage>,
    recv: Receiver<Vec<CmNotification>>,
}

impl NotificationHandler {
    pub fn listen(&self) -> Result<Vec<CmNotification>, ConnectionManagerError> {
        match self.recv.recv() {
            Ok(notifications) => Ok(notifications),
            Err(_) => Err(ConnectionManagerError::SendMessageError(
                "The connection manager is no longer running".into(),
            )),
        }
    }

    pub fn unsubscribe(&self) -> Result<(), ConnectionManagerError> {
        let message = CmMessage::UnSubscribe(self.id.clone());
        match self.sender.send(message) {
            Ok(()) => Ok(()),
            Err(_) => Err(ConnectionManagerError::SendMessageError(
                "Unsubscribe request timed out".into(),
            )),
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
            meta.ref_count = meta.ref_count + 1;
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
            meta.ref_count = meta.ref_count - 1;
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
    let result = match req.payload {
        CmPayload::AddConnection { ref endpoint } => state.add_connection(endpoint),
    };

    let response = match result {
        Ok(()) => CmResponse::AddConnection {
            status: CmResponseStatus::OK,
            error_message: None,
        },
        Err(err) => CmResponse::AddConnection {
            status: CmResponseStatus::Error,
            error_message: Some(format!("{:?}", err)),
        },
    };

    if let Err(_) = req.sender.send(response) {
        error!("Requester has dropped its connection to connection manager");
    }
}

fn notify_subscribers(
    subscribers: &mut HashMap<String, SyncSender<Vec<CmNotification>>>,
    notifications: Vec<CmNotification>,
) {
    for (id, sender) in subscribers.clone() {
        if let Err(_) = sender.send(notifications.clone()) {
            warn!("subscriber has dropped its connection to connection manager");
            subscribers.remove(&id);
        }
    }
}

fn send_heartbeats(
    state: &mut ConnectionState,
    subscribers: &mut HashMap<String, SyncSender<Vec<CmNotification>>>,
) {
    let heartbeat_message = match create_heartbeat() {
        Ok(h) => h,
        Err(err) => {
            error!("Failed to create heartbeat message: {:?}", err);
            return;
        }
    };
    let mut notifications = Vec::new();

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

            notifications.push(CmNotification::HeartbeatSendFail {
                endpoint: endpoint.clone(),
                message: format!("{:?}", err),
            });

            if let Err(err) = state.reconnect(&endpoint) {
                error!("Connection reattempt failed: {:?}", err);
                notifications.push(CmNotification::ReconnectAttemptFailed {
                    endpoint: endpoint.clone(),
                    message: format!("{:?}", err),
                });
            } else {
                notifications.push(CmNotification::ReconnectAttemptSuccess {
                    endpoint: endpoint.clone(),
                });
            }
        } else {
            notifications.push(CmNotification::HeartbeatSent {
                endpoint: endpoint.clone(),
            });
        }
    }

    notify_subscribers(subscribers, notifications);
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
    fn test_notification_handler_subscribe_unsubscribe() {
        let mut transport = Box::new(InprocTransport::default());
        transport.listen("inproc://test").unwrap();
        let mesh = Mesh::new(512, 128);

        let mut cm = ConnectionManager::new(mesh, transport);

        let connector = cm.start().unwrap();

        let subscriber = connector.subscribe().unwrap();
        subscriber.unsubscribe().unwrap();

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

        let subscriber = connector.subscribe().unwrap();

        let notifications = subscriber.listen().unwrap();

        assert!(notifications.iter().any(|x| *x
            == CmNotification::HeartbeatSent {
                endpoint: "inproc://test".to_string(),
            }));

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
        let mut listener = transport.listen("tcp://localhost:8080").unwrap();
        let mesh = Mesh::new(512, 128);
        let mesh_clone = mesh.clone();

        thread::spawn(move || {
            let conn = listener.accept().unwrap();
            mesh_clone.add(conn).unwrap();
        });

        let mut cm = ConnectionManager::new(mesh.clone(), transport);
        let connector = cm.start().unwrap();

        let response = connector
            .request_connection("tcp://localhost:8080")
            .unwrap();

        assert_eq!(
            response,
            CmResponse::AddConnection {
                status: CmResponseStatus::OK,
                error_message: None
            }
        );

        let subscriber = connector.subscribe().unwrap();

        let notifications = subscriber.listen().unwrap();

        assert!(notifications.iter().any(|x| *x
            == CmNotification::HeartbeatSent {
                endpoint: "tcp://localhost:8080".to_string(),
            }));

        // Verify mesh received heartbeat

        let envelope = mesh.recv().unwrap();
        let heartbeat: NetworkMessage = protobuf::parse_from_bytes(&envelope.payload()).unwrap();
        assert_eq!(
            heartbeat.get_message_type(),
            NetworkMessageType::NETWORK_HEARTBEAT
        );
    }
}
