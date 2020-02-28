// Copyright 2018-2020 Cargill Incorporated
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
use std::cmp::min;
use std::collections::HashMap;
#[cfg(feature = "connection-manager-notification-iter-try-next")]
use std::sync::mpsc::TryRecvError;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::thread;
use std::time::Instant;

pub use error::ConnectionManagerError;
pub use messages::{
    CmMessage, CmPayload, CmRequest, CmResponse, CmResponseStatus, ConnectionManagerNotification,
};
use pacemaker::Pacemaker;
use protobuf::Message;

use crate::matrix::{MatrixLifeCycle, MatrixSender};
use crate::protos::network::{NetworkHeartbeat, NetworkMessage, NetworkMessageType};
use crate::transport::Transport;

const DEFAULT_HEARTBEAT_INTERVAL: u64 = 10;
const CHANNEL_CAPACITY: usize = 15;
const INITIAL_RETRY_FREQUENCY: u64 = 10;
const DEFAULT_MAXIMUM_RETRY_FREQUENCY: u64 = 300;

pub struct ConnectionManager<T: 'static, U: 'static>
where
    T: MatrixLifeCycle,
    U: MatrixSender,
{
    pacemaker: Pacemaker,
    connection_state: Option<ConnectionState<T, U>>,
    join_handle: Option<thread::JoinHandle<()>>,
    sender: Option<SyncSender<CmMessage>>,
    shutdown_handle: Option<ShutdownHandle>,
}

impl<T, U> ConnectionManager<T, U>
where
    T: MatrixLifeCycle,
    U: MatrixSender,
{
    pub fn new(
        life_cycle: T,
        matrix_sender: U,
        transport: Box<dyn Transport + Send>,
        heartbeat_interval: Option<u64>,
        maximum_retry_frequency: Option<u64>,
    ) -> Self {
        let heartbeat = heartbeat_interval.unwrap_or(DEFAULT_HEARTBEAT_INTERVAL);
        let retry_frequency = maximum_retry_frequency.unwrap_or(DEFAULT_MAXIMUM_RETRY_FREQUENCY);
        let connection_state = Some(ConnectionState::new(
            life_cycle,
            matrix_sender,
            transport,
            retry_frequency,
        ));
        let pacemaker = Pacemaker::new(heartbeat);

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

    pub fn subscribe(&self) -> Result<NotificationIter, ConnectionManagerError> {
        let (send, recv) = sync_channel(CHANNEL_CAPACITY);
        match self.sender.send(CmMessage::Subscribe(send)) {
            Ok(()) => Ok(NotificationIter { recv }),
            Err(_) => Err(ConnectionManagerError::SendMessageError(
                "The connection manager is no longer running".into(),
            )),
        }
    }

    pub fn list_connections(&self) -> Result<Vec<String>, ConnectionManagerError> {
        match self.send_payload(CmPayload::ListConnections) {
            Ok(CmResponse::ListConnections { endpoints }) => Ok(endpoints),
            Err(err) => Err(err),
            _ => {
                panic!("This cannot return a response type that is not CmResponse::ListConnections")
            }
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

pub struct NotificationIter {
    recv: Receiver<ConnectionManagerNotification>,
}

#[cfg(feature = "connection-manager-notification-iter-try-next")]
impl NotificationIter {
    pub fn try_next(
        &self,
    ) -> Result<Option<ConnectionManagerNotification>, ConnectionManagerError> {
        match self.recv.try_recv() {
            Ok(notifications) => Ok(Some(notifications)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => Err(ConnectionManagerError::SendMessageError(
                "The connection manager is no longer running".into(),
            )),
        }
    }
}

impl Iterator for NotificationIter {
    type Item = ConnectionManagerNotification;

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

#[derive(Clone, Debug)]
struct ConnectionMetadata {
    id: usize,
    endpoint: String,
    ref_count: u64,
    reconnecting: bool,
    retry_frequency: u64,
    last_connection_attempt: Instant,
}

struct ConnectionState<T, U>
where
    T: MatrixLifeCycle,
    U: MatrixSender,
{
    connections: HashMap<String, ConnectionMetadata>,
    life_cycle: T,
    matrix_sender: U,
    transport: Box<dyn Transport>,
    maximum_retry_frequency: u64,
}

impl<T, U> ConnectionState<T, U>
where
    T: MatrixLifeCycle,
    U: MatrixSender,
{
    fn new(
        life_cycle: T,
        matrix_sender: U,
        transport: Box<dyn Transport + Send>,
        maximum_retry_frequency: u64,
    ) -> Self {
        Self {
            life_cycle,
            matrix_sender,
            transport,
            connections: HashMap::new(),
            maximum_retry_frequency,
        }
    }

    fn add_connection(&mut self, endpoint: &str) -> Result<(), ConnectionManagerError> {
        if let Some(meta) = self.connections.get_mut(endpoint) {
            meta.ref_count += 1;
        } else {
            let connection = self.transport.connect(endpoint).map_err(|err| {
                ConnectionManagerError::ConnectionCreationError(format!("{:?}", err))
            })?;

            let id = self.life_cycle.add(connection).map_err(|err| {
                ConnectionManagerError::ConnectionCreationError(format!("{:?}", err))
            })?;

            self.connections.insert(
                endpoint.to_string(),
                ConnectionMetadata {
                    id,
                    endpoint: endpoint.to_string(),
                    ref_count: 1,
                    reconnecting: false,
                    retry_frequency: INITIAL_RETRY_FREQUENCY,
                    last_connection_attempt: Instant::now(),
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
            self.life_cycle.remove(meta.id).map_err(|err| {
                ConnectionManagerError::ConnectionRemovalError(format!("{:?}", err))
            })?;
        }

        Ok(Some(meta))
    }

    fn reconnect(
        &mut self,
        endpoint: &str,
        subscribers: &mut Vec<SyncSender<ConnectionManagerNotification>>,
    ) -> Result<(), ConnectionManagerError> {
        let mut meta = if let Some(meta) = self.connections.get_mut(endpoint) {
            meta.clone()
        } else {
            return Err(ConnectionManagerError::ConnectionRemovalError(
                "Cannot reconnect to endpoint without metadata".into(),
            ));
        };

        if let Ok(connection) = self.transport.connect(endpoint) {
            // remove old mesh id, this may happen before reconnection is attempted
            if self.life_cycle.remove(meta.id).is_err() {
                trace!(
                    "Connection was already removed from life_cycle: {}",
                    endpoint
                );
            }

            // add new connection to mesh
            let id = self.life_cycle.add(connection).map_err(|err| {
                ConnectionManagerError::ConnectionReconnectError(format!("{:?}", err))
            })?;

            // replace mesh id and reset reconnecting fields
            meta.id = id;
            meta.reconnecting = false;
            meta.retry_frequency = INITIAL_RETRY_FREQUENCY;
            meta.last_connection_attempt = Instant::now();
            self.connections.insert(endpoint.to_string(), meta);

            // Notify subscribers of success
            notify_subscribers(
                subscribers,
                ConnectionManagerNotification::Connected {
                    endpoint: endpoint.to_string(),
                },
            );
        } else {
            meta.reconnecting = true;
            meta.retry_frequency = min(meta.retry_frequency * 2, self.maximum_retry_frequency);
            meta.last_connection_attempt = Instant::now();
            self.connections.insert(endpoint.to_string(), meta);
        }
        Ok(())
    }

    fn connection_metadata(&self) -> HashMap<String, ConnectionMetadata> {
        self.connections.clone()
    }

    fn matrix_sender(&self) -> U {
        self.matrix_sender.clone()
    }
}

fn handle_request<T: MatrixLifeCycle, U: MatrixSender>(
    req: CmRequest,
    state: &mut ConnectionState<T, U>,
) {
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
        CmPayload::ListConnections => CmResponse::ListConnections {
            endpoints: state
                .connection_metadata()
                .iter()
                .map(|(key, _)| key.to_string())
                .collect(),
        },
    };

    if req.sender.send(response).is_err() {
        error!("Requester has dropped its connection to connection manager");
    }
}

fn notify_subscribers(
    subscribers: &mut Vec<SyncSender<ConnectionManagerNotification>>,
    notification: ConnectionManagerNotification,
) {
    subscribers.retain(|sender| sender.send(notification.clone()).is_ok());
}

fn send_heartbeats<T: MatrixLifeCycle, U: MatrixSender>(
    state: &mut ConnectionState<T, U>,
    subscribers: &mut Vec<SyncSender<ConnectionManagerNotification>>,
) {
    let heartbeat_message = match create_heartbeat() {
        Ok(h) => h,
        Err(err) => {
            error!("Failed to create heartbeat message: {:?}", err);
            return;
        }
    };

    for (endpoint, metadata) in state.connection_metadata() {
        if state
            .matrix_sender()
            .send(metadata.id, heartbeat_message.clone())
            .is_err()
        {
            // if connection is already attempting reconnection, call reconnect
            if metadata.reconnecting {
                if metadata.last_connection_attempt.elapsed().as_secs() > metadata.retry_frequency {
                    if let Err(err) = state.reconnect(&endpoint, subscribers) {
                        error!("Reconnection attempt to {} failed: {:?}", endpoint, err);
                    }
                }
            } else {
                info!("Sending heartbeat to {}", endpoint);
                if let Err(err) = state
                    .matrix_sender()
                    .send(metadata.id, heartbeat_message.clone())
                {
                    error!(
                        "failed to send heartbeat: {:?} attempting reconnection",
                        err
                    )
                }
                notify_subscribers(
                    subscribers,
                    ConnectionManagerNotification::Disconnected {
                        endpoint: endpoint.clone(),
                    },
                );
                if let Err(err) = state.reconnect(&endpoint, subscribers) {
                    error!("Reconnection attempt to {} failed: {:?}", endpoint, err);
                }
            }
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
    use crate::mesh::Mesh;
    use crate::transport::inproc::InprocTransport;
    use crate::transport::raw::RawTransport;

    #[test]
    fn test_connection_manager_startup_and_shutdown() {
        let mut transport = Box::new(InprocTransport::default());
        transport.listen("inproc://test").unwrap();
        let mesh = Mesh::new(512, 128);

        let mut cm = ConnectionManager::new(
            mesh.get_life_cycle(),
            mesh.get_sender(),
            transport,
            None,
            None,
        );

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
        let mut cm = ConnectionManager::new(
            mesh.get_life_cycle(),
            mesh.get_sender(),
            transport,
            None,
            None,
        );
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
        let mut cm = ConnectionManager::new(
            mesh.get_life_cycle(),
            mesh.get_sender(),
            transport,
            None,
            None,
        );
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

    /// test_heartbeat_inproc
    ///
    /// Test that heartbeats are correctly sent to connections
    #[test]
    fn test_heartbeat_inproc() {
        let mut transport = Box::new(InprocTransport::default());
        let mut listener = transport.listen("inproc://test").unwrap();
        let mesh = Mesh::new(512, 128);
        let mesh_clone = mesh.clone();

        thread::spawn(move || {
            let conn = listener.accept().unwrap();
            mesh_clone.add(conn).unwrap();
        });

        let mut cm = ConnectionManager::new(
            mesh.get_life_cycle(),
            mesh.get_sender(),
            transport,
            Some(1),
            None,
        );
        let connector = cm.start().unwrap();

        let response = connector.request_connection("inproc://test").unwrap();

        assert_eq!(
            response,
            CmResponse::AddConnection {
                status: CmResponseStatus::OK,
                error_message: None
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

    // test_heartbeat_raw_tcp
    ///
    /// Test that heartbeats are correctly sent to connections
    #[test]
    fn test_heartbeat_raw_tcp() {
        let mut transport = Box::new(RawTransport::default());
        let mut listener = transport.listen("tcp://localhost:0").unwrap();
        let endpoint = listener.endpoint();

        let mesh = Mesh::new(512, 128);
        let mesh_clone = mesh.clone();

        thread::spawn(move || {
            let conn = listener.accept().unwrap();
            mesh_clone.add(conn).unwrap();
        });

        let mut cm = ConnectionManager::new(
            mesh.get_life_cycle(),
            mesh.get_sender(),
            transport,
            None,
            None,
        );
        let connector = cm.start().unwrap();

        let response = connector.request_connection(&endpoint).unwrap();

        assert_eq!(
            response,
            CmResponse::AddConnection {
                status: CmResponseStatus::OK,
                error_message: None
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
        let mut listener = transport.listen("tcp://localhost:0").unwrap();
        let endpoint = listener.endpoint();
        let mesh = Mesh::new(512, 128);
        let mesh_clone = mesh.clone();

        thread::spawn(move || {
            let conn = listener.accept().unwrap();
            mesh_clone.add(conn).unwrap();
        });

        let mut cm = ConnectionManager::new(
            mesh.get_life_cycle(),
            mesh.get_sender(),
            transport,
            None,
            None,
        );
        let connector = cm.start().unwrap();

        let add_response = connector.request_connection(&endpoint).unwrap();

        assert_eq!(
            add_response,
            CmResponse::AddConnection {
                status: CmResponseStatus::OK,
                error_message: None
            }
        );

        let remove_response = connector.remove_connection(&endpoint).unwrap();

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
        let mut listener = transport.listen("tcp://localhost:0").unwrap();
        let endpoint = listener.endpoint();
        let mesh = Mesh::new(512, 128);
        let mesh_clone = mesh.clone();

        thread::spawn(move || {
            let conn = listener.accept().unwrap();
            mesh_clone.add(conn).unwrap();
        });

        let mut cm = ConnectionManager::new(
            mesh.get_life_cycle(),
            mesh.get_sender(),
            transport,
            None,
            None,
        );
        let connector = cm.start().unwrap();

        let remove_response = connector.remove_connection(&endpoint).unwrap();

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
    /// creates a thread that send AttemptingReconnect notifications to
    /// the notifier.
    ///
    /// Asserts:
    ///
    /// The notifications sent are received by the NotificationIter
    /// correctly
    ///
    /// That the total number of notifications sent equals 5
    fn test_notifications_handler_iterator() {
        let (send, recv) = sync_channel(2);

        let nh = NotificationIter { recv };

        let join_handle = thread::spawn(move || {
            for _ in 0..5 {
                send.send(ConnectionManagerNotification::Connected {
                    endpoint: "tcp://localhost:3030".to_string(),
                })
                .unwrap();
            }
        });

        let mut notifications_sent = 0;
        for n in nh {
            assert_eq!(
                n,
                ConnectionManagerNotification::Connected {
                    endpoint: "tcp://localhost:3030".to_string()
                }
            );
            notifications_sent += 1;
        }

        assert_eq!(notifications_sent, 5);

        join_handle.join().unwrap();
    }

    /// test_reconnect_raw_tcp
    ///
    /// Test that if a connection disconnects, the connection manager will detect the connection
    /// has disconnected by trying to send a heartbeat. Then connection manger will try to
    /// reconnect to the endpoint.
    #[test]
    fn test_reconnect_raw_tcp() {
        let mut transport = Box::new(RawTransport::default());
        let mut listener = transport
            .listen("tcp://localhost:0")
            .expect("Cannot listen for connections");
        let endpoint = listener.endpoint();
        let mesh1 = Mesh::new(512, 128);
        let mesh2 = Mesh::new(512, 128);

        thread::spawn(move || {
            // accept incoming connection and add it to mesh2
            let conn = listener.accept().expect("Cannot accept connection");
            let id = mesh2.add(conn).expect("Cannot add connection to mesh");

            // Verify mesh received heartbeat
            let envelope = mesh2.recv().expect("Cannot receive message");
            let heartbeat: NetworkMessage = protobuf::parse_from_bytes(&envelope.payload())
                .expect("Cannot parse NetworkMessage");
            assert_eq!(
                heartbeat.get_message_type(),
                NetworkMessageType::NETWORK_HEARTBEAT
            );

            // remove connection to cause reconnection attempt
            let mut connection = mesh2
                .remove(id)
                .expect("Cannot remove connection from mesh");
            connection
                .disconnect()
                .expect("Connection failed to disconnect");

            // wait for reconnection attempt
            listener.accept().expect("Unable to accept connection");
        });

        let mut cm = ConnectionManager::new(
            mesh1.get_life_cycle(),
            mesh1.get_sender(),
            transport,
            Some(1),
            None,
        );
        let connector = cm.start().expect("Unable to start ConnectionManager");

        let response = connector
            .request_connection(&endpoint)
            .expect("Unable to request connection");

        assert_eq!(
            response,
            CmResponse::AddConnection {
                status: CmResponseStatus::OK,
                error_message: None
            }
        );

        let mut subscriber = connector.subscribe().expect("Cannot get subscriber");
        // receive reconnecting attempt
        let reconnecting_notification = subscriber
            .next()
            .expect("Cannot get message from subscriber");
        assert!(
            reconnecting_notification
                == ConnectionManagerNotification::Disconnected {
                    endpoint: endpoint.clone(),
                }
        );

        // receive successful reconnect attempt
        let reconnection_notification = subscriber
            .next()
            .expect("Cannot get message from subscriber");
        assert!(
            reconnection_notification
                == ConnectionManagerNotification::Connected {
                    endpoint: endpoint.clone(),
                }
        );
    }
}
