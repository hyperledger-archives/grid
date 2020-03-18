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
mod notification;
mod pacemaker;

use std;
use std::cmp::min;
use std::collections::HashMap;
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::time::Instant;

use uuid::Uuid;

pub use error::ConnectionManagerError;
pub use notification::{ConnectionManagerNotification, NotificationIter};
use pacemaker::Pacemaker;
use protobuf::Message;

use crate::matrix::{MatrixLifeCycle, MatrixSender};
use crate::protos::network::{NetworkHeartbeat, NetworkMessage, NetworkMessageType};
use crate::transport::{Connection, Transport};

const DEFAULT_HEARTBEAT_INTERVAL: u64 = 10;
const INITIAL_RETRY_FREQUENCY: u64 = 10;
const DEFAULT_MAXIMUM_RETRY_FREQUENCY: u64 = 300;

enum CmMessage {
    Shutdown,
    Subscribe(Sender<ConnectionManagerNotification>),
    Request(CmRequest),
    SendHeartbeats,
}

enum CmRequest {
    AddConnection {
        endpoint: String,
        connection_id: String,
        sender: Sender<Result<(), ConnectionManagerError>>,
    },
    RemoveConnection {
        endpoint: String,
        sender: Sender<Result<Option<String>, ConnectionManagerError>>,
    },
    ListConnections {
        sender: Sender<Result<Vec<String>, ConnectionManagerError>>,
    },
    AddInboundConnection {
        connection: Box<dyn Connection>,
        sender: Sender<Result<(), ConnectionManagerError>>,
    },
}

pub struct ConnectionManager<T: 'static, U: 'static>
where
    T: MatrixLifeCycle,
    U: MatrixSender,
{
    pacemaker: Pacemaker,
    connection_state: Option<ConnectionState<T, U>>,
    join_handle: Option<thread::JoinHandle<()>>,
    sender: Option<Sender<CmMessage>>,
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
        let (sender, recv) = channel();
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

        self.pacemaker
            .start(sender.clone(), || CmMessage::SendHeartbeats)?;
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
    sender: Sender<CmMessage>,
}

impl Connector {
    /// Request a connection to the given endpoint.
    ///
    /// This operation is idempotent: if a connection to that endpoint already exists, a new
    /// connection is not created.
    ///
    /// # Errors
    ///
    /// An error is returned if the connection cannot be created
    pub fn request_connection(
        &self,
        endpoint: &str,
        id: &str,
    ) -> Result<(), ConnectionManagerError> {
        let (sender, recv) = channel();
        self.sender
            .send(CmMessage::Request(CmRequest::AddConnection {
                sender,
                endpoint: endpoint.to_string(),
                connection_id: id.to_string(),
            }))
            .map_err(|_| {
                ConnectionManagerError::SendMessageError(
                    "The connection manager is no longer running".into(),
                )
            })?;

        recv.recv().map_err(|_| {
            ConnectionManagerError::SendMessageError(
                "The connection manager is no longer running".into(),
            )
        })?
    }

    /// Removes a connection
    ///
    ///  # Returns
    ///
    ///  The endpoint, if the connection exists; None, otherwise.
    ///
    ///  # Errors
    ///
    ///  Returns a ConnectionManagerError if the query cannot be performed.
    pub fn remove_connection(
        &self,
        endpoint: &str,
    ) -> Result<Option<String>, ConnectionManagerError> {
        let (sender, recv) = channel();
        self.sender
            .send(CmMessage::Request(CmRequest::RemoveConnection {
                sender,
                endpoint: endpoint.to_string(),
            }))
            .map_err(|_| {
                ConnectionManagerError::SendMessageError(
                    "The connection manager is no longer running".into(),
                )
            })?;

        recv.recv().map_err(|_| {
            ConnectionManagerError::SendMessageError(
                "The connection manager is no longer running".into(),
            )
        })?
    }

    /// Subscribe to notfications for connection events
    ///
    /// # Returns
    ///
    /// Notifications are received via an iterator.  The iterator will block when using standard
    /// `next`, but also provides a `try_next`
    ///
    /// # Errors
    ///
    /// Return a ConnectionManagerError if the notification iterator cannot be created.
    pub fn subscribe(&self) -> Result<NotificationIter, ConnectionManagerError> {
        let (send, recv) = channel();
        match self.sender.send(CmMessage::Subscribe(send)) {
            Ok(()) => Ok(NotificationIter { recv }),
            Err(_) => Err(ConnectionManagerError::SendMessageError(
                "The connection manager is no longer running".into(),
            )),
        }
    }

    /// List the connections available to this Connector instance.
    ///
    /// # Returns
    ///
    /// Returns a vector of connection endpoints.
    ///
    /// # Errors
    ///
    /// Returns a ConnectionManagerError if the connections cannot be queried.
    pub fn list_connections(&self) -> Result<Vec<String>, ConnectionManagerError> {
        let (sender, recv) = channel();
        self.sender
            .send(CmMessage::Request(CmRequest::ListConnections { sender }))
            .map_err(|_| {
                ConnectionManagerError::SendMessageError(
                    "The connection manager is no longer running".into(),
                )
            })?;

        recv.recv().map_err(|_| {
            ConnectionManagerError::SendMessageError(
                "The connection manager is no longer running".into(),
            )
        })?
    }

    pub fn add_inbound_connection(
        &self,
        connection: Box<dyn Connection>,
    ) -> Result<(), ConnectionManagerError> {
        let (sender, recv) = channel();
        self.sender
            .send(CmMessage::Request(CmRequest::AddInboundConnection {
                connection,
                sender,
            }))
            .map_err(|_| {
                ConnectionManagerError::SendMessageError(
                    "The connection manager is no longer running".into(),
                )
            })?;

        recv.recv().map_err(|_| {
            ConnectionManagerError::SendMessageError(
                "The connection manager is no longer running".into(),
            )
        })?
    }
}

/// Signals shutdown to the ConnectionManager
#[derive(Clone)]
pub struct ShutdownHandle {
    sender: Sender<CmMessage>,
    pacemaker_shutdown_handle: pacemaker::ShutdownHandle,
}

impl ShutdownHandle {
    /// Signal the ConnectionManager to shutdown.
    pub fn shutdown(self) {
        self.pacemaker_shutdown_handle.shutdown();

        if self.sender.send(CmMessage::Shutdown).is_err() {
            warn!("Connection manager is no longer running");
        }
    }
}

#[derive(Clone, Debug)]
enum ConnectionMetadata {
    Outbound {
        id: String,
        outbound: OutboundConnection,
    },

    Inbound {
        id: String,
        inbound: InboundConnection,
    },
}

impl ConnectionMetadata {
    fn is_outbound(&self) -> bool {
        match self {
            ConnectionMetadata::Outbound { .. } => true,
            _ => false,
        }
    }

    fn id(&self) -> &str {
        match self {
            ConnectionMetadata::Outbound { id, .. } => id,
            ConnectionMetadata::Inbound { id, .. } => id,
        }
    }

    fn endpoint(&self) -> &str {
        match self {
            ConnectionMetadata::Outbound { outbound, .. } => &outbound.endpoint,
            ConnectionMetadata::Inbound { inbound, .. } => &inbound.endpoint,
        }
    }
}

#[derive(Clone, Debug)]
struct OutboundConnection {
    endpoint: String,
    reconnecting: bool,
    retry_frequency: u64,
    last_connection_attempt: Instant,
    reconnection_attempts: u64,
}

#[derive(Clone, Debug)]
struct InboundConnection {
    endpoint: String,
    disconnected: bool,
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

    fn add_inbound_connection(
        &mut self,
        connection: Box<dyn Connection>,
    ) -> Result<(), ConnectionManagerError> {
        let endpoint = connection.remote_endpoint();

        let id = Uuid::new_v4().to_string();

        self.life_cycle
            .add(connection, id.clone())
            .map_err(|err| ConnectionManagerError::ConnectionCreationError(format!("{:?}", err)))?;

        self.connections.insert(
            endpoint.clone(),
            ConnectionMetadata::Inbound {
                id,
                inbound: InboundConnection {
                    endpoint,
                    disconnected: false,
                },
            },
        );

        Ok(())
    }

    fn add_connection(&mut self, endpoint: &str, id: String) -> Result<(), ConnectionManagerError> {
        if self.connections.get_mut(endpoint).is_some() {
            return Ok(());
        } else {
            let connection = self.transport.connect(endpoint).map_err(|err| {
                ConnectionManagerError::ConnectionCreationError(format!("{:?}", err))
            })?;

            self.life_cycle
                .add(connection, id.to_string())
                .map_err(|err| {
                    ConnectionManagerError::ConnectionCreationError(format!("{:?}", err))
                })?;

            self.connections.insert(
                endpoint.to_string(),
                ConnectionMetadata::Outbound {
                    id,
                    outbound: OutboundConnection {
                        endpoint: endpoint.to_string(),
                        reconnecting: false,
                        retry_frequency: INITIAL_RETRY_FREQUENCY,
                        last_connection_attempt: Instant::now(),
                        reconnection_attempts: 0,
                    },
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
            meta.clone()
        } else {
            return Ok(None);
        };

        self.connections.remove(endpoint);
        // remove mesh id, this may happen before reconnection is attempted
        self.life_cycle.remove(meta.id()).map_err(|err| {
            ConnectionManagerError::ConnectionRemovalError(format!(
                "Cannot remove connection {} from life cycle: {}",
                endpoint, err
            ))
        })?;

        Ok(Some(meta))
    }

    fn reconnect(
        &mut self,
        endpoint: &str,
        subscribers: &mut Vec<Sender<ConnectionManagerNotification>>,
    ) -> Result<(), ConnectionManagerError> {
        let mut meta = if let Some(meta) = self.connections.get_mut(endpoint) {
            meta.clone()
        } else {
            return Err(ConnectionManagerError::ConnectionRemovalError(
                "Cannot reconnect to endpoint without metadata".into(),
            ));
        };

        if !meta.is_outbound() {
            // Do not attempt to reconnect inbound connections.
            return Ok(());
        }

        if let Ok(connection) = self.transport.connect(endpoint) {
            // remove old mesh id, this may happen before reconnection is attempted
            self.life_cycle.remove(meta.id()).map_err(|err| {
                ConnectionManagerError::ConnectionRemovalError(format!(
                    "Cannot remove connection {} from life cycle: {}",
                    endpoint, err
                ))
            })?;

            // add new connection to mesh
            self.life_cycle
                .add(connection, meta.id().to_string())
                .map_err(|err| {
                    ConnectionManagerError::ConnectionReconnectError(format!("{:?}", err))
                })?;

            // replace mesh id and reset reconnecting fields
            match meta {
                ConnectionMetadata::Outbound {
                    ref mut outbound, ..
                } => {
                    outbound.reconnecting = false;
                    outbound.retry_frequency = INITIAL_RETRY_FREQUENCY;
                    outbound.last_connection_attempt = Instant::now();
                    outbound.reconnection_attempts = 0;
                }
                // We checked earlier that this was an outbound connection
                _ => unreachable!(),
            }

            self.connections.insert(endpoint.to_string(), meta);

            // Notify subscribers of success
            notify_subscribers(
                subscribers,
                ConnectionManagerNotification::Connected {
                    endpoint: endpoint.to_string(),
                },
            );
        } else {
            let reconnection_attempts = match meta {
                ConnectionMetadata::Outbound {
                    ref mut outbound, ..
                } => {
                    outbound.reconnecting = true;
                    outbound.retry_frequency =
                        min(outbound.retry_frequency * 2, self.maximum_retry_frequency);
                    outbound.last_connection_attempt = Instant::now();
                    outbound.reconnection_attempts += 1;

                    outbound.reconnection_attempts
                }
                // We checked earlier that this was an outbound connection
                _ => unreachable!(),
            };

            self.connections.insert(endpoint.to_string(), meta);

            // Notify subscribers of reconnection failure
            notify_subscribers(
                subscribers,
                ConnectionManagerNotification::ReconnectionFailed {
                    endpoint: endpoint.to_string(),
                    attempts: reconnection_attempts,
                },
            );
        }
        Ok(())
    }

    fn connection_metadata(&self) -> &HashMap<String, ConnectionMetadata> {
        &self.connections
    }

    fn connection_metadata_mut(&mut self) -> &mut HashMap<String, ConnectionMetadata> {
        &mut self.connections
    }

    fn matrix_sender(&self) -> U {
        self.matrix_sender.clone()
    }
}

fn handle_request<T: MatrixLifeCycle, U: MatrixSender>(
    req: CmRequest,
    state: &mut ConnectionState<T, U>,
) {
    match req {
        CmRequest::AddConnection {
            endpoint,
            sender,
            connection_id,
        } => {
            if sender
                .send(state.add_connection(&endpoint, connection_id))
                .is_err()
            {
                warn!("connector dropped before receiving result of add connection");
            }
        }
        CmRequest::RemoveConnection { endpoint, sender } => {
            let response = state
                .remove_connection(&endpoint)
                .map(|meta_opt| meta_opt.map(|meta| meta.endpoint().to_owned()));

            if sender.send(response).is_err() {
                warn!("connector dropped before receiving result of remove connection");
            }
        }
        CmRequest::ListConnections { sender } => {
            if sender
                .send(Ok(state
                    .connection_metadata()
                    .iter()
                    .map(|(key, _)| key.to_string())
                    .collect()))
                .is_err()
            {
                warn!("connector dropped before receiving result of list connections");
            }
        }
        CmRequest::AddInboundConnection { sender, connection } => {
            let res = state.add_inbound_connection(connection);
            if sender.send(res).is_err() {
                warn!("connector dropped before receiving result of add inbound callback");
            }
        }
    };
}

fn notify_subscribers(
    subscribers: &mut Vec<Sender<ConnectionManagerNotification>>,
    notification: ConnectionManagerNotification,
) {
    subscribers.retain(|sender| sender.send(notification.clone()).is_ok());
}

fn send_heartbeats<T: MatrixLifeCycle, U: MatrixSender>(
    state: &mut ConnectionState<T, U>,
    subscribers: &mut Vec<Sender<ConnectionManagerNotification>>,
) {
    let heartbeat_message = match create_heartbeat() {
        Ok(h) => h,
        Err(err) => {
            error!("Failed to create heartbeat message: {:?}", err);
            return;
        }
    };

    let matrix_sender = state.matrix_sender();
    let mut reconnections = vec![];
    for (endpoint, metadata) in state.connection_metadata_mut().iter_mut() {
        match metadata {
            ConnectionMetadata::Outbound { id, outbound } => {
                // if connection is already attempting reconnection, call reconnect
                if outbound.reconnecting {
                    if outbound.last_connection_attempt.elapsed().as_secs()
                        > outbound.retry_frequency
                    {
                        reconnections.push(endpoint.to_string());
                    }
                } else {
                    info!("Sending heartbeat to {}", endpoint);
                    if let Err(err) =
                        matrix_sender.send((*id).to_string(), heartbeat_message.clone())
                    {
                        error!(
                            "failed to send heartbeat: {:?} attempting reconnection",
                            err
                        );

                        notify_subscribers(
                            subscribers,
                            ConnectionManagerNotification::Disconnected {
                                endpoint: endpoint.clone(),
                            },
                        );
                        reconnections.push(endpoint.to_string());
                    }
                }
            }
            ConnectionMetadata::Inbound {
                id,
                ref mut inbound,
            } => {
                info!("Sending heartbeat to {}", endpoint);
                if let Err(err) = matrix_sender.send((*id).to_string(), heartbeat_message.clone()) {
                    error!(
                        "failed to send heartbeat: {:?} attempting reconnection",
                        err
                    );

                    notify_subscribers(
                        subscribers,
                        ConnectionManagerNotification::Disconnected {
                            endpoint: endpoint.clone(),
                        },
                    );
                }
                inbound.disconnected = false;
            }
        }
    }

    for endpoint in reconnections {
        if let Err(err) = state.reconnect(&endpoint, subscribers) {
            error!("Reconnection attempt to {} failed: {:?}", endpoint, err);
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
    use crate::transport::socket::TcpTransport;

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

        connector
            .request_connection("inproc://test", "test_id")
            .expect("A connection could not be created");

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

        connector
            .request_connection("inproc://test", "test_id")
            .expect("A connection could not be created");

        connector
            .request_connection("inproc://test", "test_id")
            .expect("A connection could not be re-requested");

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
            mesh_clone.add(conn, "test_id".to_string()).unwrap();
        });

        let mut cm = ConnectionManager::new(
            mesh.get_life_cycle(),
            mesh.get_sender(),
            transport,
            Some(1),
            None,
        );
        let connector = cm.start().unwrap();

        connector
            .request_connection("inproc://test", "test_id")
            .expect("A connection could not be created");

        // Verify mesh received heartbeat

        let envelope = mesh.recv().unwrap();
        let heartbeat: NetworkMessage = protobuf::parse_from_bytes(&envelope.payload()).unwrap();
        assert_eq!(
            heartbeat.get_message_type(),
            NetworkMessageType::NETWORK_HEARTBEAT
        );

        cm.shutdown_and_wait();
    }

    // test_heartbeat_raw_tcp
    ///
    /// Test that heartbeats are correctly sent to connections
    #[test]
    fn test_heartbeat_raw_tcp() {
        let mut transport = Box::new(TcpTransport::default());
        let mut listener = transport.listen("tcp://localhost:0").unwrap();
        let endpoint = listener.endpoint();

        let mesh = Mesh::new(512, 128);
        let mesh_clone = mesh.clone();

        thread::spawn(move || {
            let conn = listener.accept().unwrap();
            mesh_clone.add(conn, "test_id".to_string()).unwrap();
        });

        let mut cm = ConnectionManager::new(
            mesh.get_life_cycle(),
            mesh.get_sender(),
            transport,
            None,
            None,
        );
        let connector = cm.start().unwrap();

        connector
            .request_connection(&endpoint, "test_id")
            .expect("A connection could not be created");

        // Verify mesh received heartbeat

        let envelope = mesh.recv().unwrap();
        let heartbeat: NetworkMessage = protobuf::parse_from_bytes(&envelope.payload()).unwrap();
        assert_eq!(
            heartbeat.get_message_type(),
            NetworkMessageType::NETWORK_HEARTBEAT
        );
        cm.shutdown_and_wait();
    }

    #[test]
    fn test_remove_connection() {
        let mut transport = Box::new(TcpTransport::default());
        let mut listener = transport.listen("tcp://localhost:0").unwrap();
        let endpoint = listener.endpoint();
        let mesh = Mesh::new(512, 128);
        let mesh_clone = mesh.clone();

        thread::spawn(move || {
            let conn = listener.accept().unwrap();
            mesh_clone.add(conn, "test_id".to_string()).unwrap();
        });

        let mut cm = ConnectionManager::new(
            mesh.get_life_cycle(),
            mesh.get_sender(),
            transport,
            None,
            None,
        );
        let connector = cm.start().unwrap();

        connector
            .request_connection(&endpoint, "test_id")
            .expect("A connection could not be created");

        assert_eq!(
            vec![endpoint.clone()],
            connector
                .list_connections()
                .expect("Unable to list connections")
        );

        let endpoint_removed = connector
            .remove_connection(&endpoint)
            .expect("Unable to remove connection");

        assert_eq!(Some(endpoint.clone()), endpoint_removed);

        assert!(connector
            .list_connections()
            .expect("Unable to list connections")
            .is_empty());

        cm.shutdown_and_wait();
    }

    #[test]
    fn test_remove_nonexistent_connection() {
        let mut transport = Box::new(TcpTransport::default());
        let mut listener = transport.listen("tcp://localhost:0").unwrap();
        let endpoint = listener.endpoint();
        let mesh = Mesh::new(512, 128);
        let mesh_clone = mesh.clone();

        thread::spawn(move || {
            let conn = listener.accept().unwrap();
            mesh_clone.add(conn, "test_id".to_string()).unwrap();
        });

        let mut cm = ConnectionManager::new(
            mesh.get_life_cycle(),
            mesh.get_sender(),
            transport,
            None,
            None,
        );
        let connector = cm.start().unwrap();

        let endpoint_removed = connector
            .remove_connection(&endpoint)
            .expect("Unable to remove connection");

        assert_eq!(None, endpoint_removed);
        cm.shutdown_and_wait();
    }

    #[test]
    /// Tests that notifier iterator correctly exists when sender
    /// is dropped.
    ///
    /// Procedure:
    ///
    /// The test creates a sync channel and a notifier, then it
    /// creates a thread that send Connected notifications to
    /// the notifier.
    ///
    /// Asserts:
    ///
    /// The notifications sent are received by the NotificationIter
    /// correctly
    ///
    /// That the total number of notifications sent equals 5
    fn test_notifications_handler_iterator() {
        let (send, recv) = channel();

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
        let mut transport = Box::new(TcpTransport::default());
        let mut listener = transport
            .listen("tcp://localhost:0")
            .expect("Cannot listen for connections");
        let endpoint = listener.endpoint();
        let mesh1 = Mesh::new(512, 128);
        let mesh2 = Mesh::new(512, 128);

        thread::spawn(move || {
            // accept incoming connection and add it to mesh2
            let conn = listener.accept().expect("Cannot accept connection");
            mesh2
                .add(conn, "test_id".to_string())
                .expect("Cannot add connection to mesh");

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
                .remove(&"test_id".to_string())
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

        connector
            .request_connection(&endpoint, "test_id")
            .expect("Unable to request connection");

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
        cm.shutdown_and_wait();
    }
}
