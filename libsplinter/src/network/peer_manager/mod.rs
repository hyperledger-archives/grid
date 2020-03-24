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

mod connector;
mod error;
mod notification;
mod peer_map;

use std::sync::mpsc::{channel, Sender};
use std::thread;

use self::error::{
    PeerListError, PeerManagerError, PeerRefAddError, PeerRefRemoveError, PeerRefUpdateError,
};
use crate::network::connection_manager::ConnectionManagerNotification;
use crate::network::connection_manager::Connector;
pub use crate::network::peer_manager::connector::PeerManagerConnector;
use crate::network::peer_manager::connector::PeerRemover;
pub use crate::network::peer_manager::notification::{
    PeerManagerNotification, PeerNotificationIter,
};
use crate::network::peer_manager::peer_map::{PeerMap, PeerMetadata, PeerStatus};
use crate::network::ref_map::RefMap;

use uuid::Uuid;

// the number of retry attempts for an active endpoint before the PeerManager will try other
// endpoints associated with a peer
const DEFAULT_MAXIMUM_RETRY_ATTEMPTS: u64 = 5;

#[derive(Debug, Clone)]
pub(crate) enum PeerManagerMessage {
    Shutdown,
    Request(PeerManagerRequest),
    Subscribe(Sender<PeerManagerNotification>),
    InternalNotification(ConnectionManagerNotification),
}

impl From<ConnectionManagerNotification> for PeerManagerMessage {
    fn from(notification: ConnectionManagerNotification) -> Self {
        PeerManagerMessage::InternalNotification(notification)
    }
}

#[derive(Debug, Clone)]
pub(crate) enum PeerManagerRequest {
    AddPeer {
        peer_id: String,
        endpoints: Vec<String>,
        sender: Sender<Result<PeerRef, PeerRefAddError>>,
    },
    UpdatePeer {
        old_peer_id: String,
        new_peer_id: String,
        sender: Sender<Result<(), PeerRefUpdateError>>,
    },
    RemovePeer {
        peer_id: String,
        sender: Sender<Result<(), PeerRefRemoveError>>,
    },
    ListPeers {
        sender: Sender<Result<Vec<String>, PeerListError>>,
    },
}

/// A PeerRef is used to keep track of peer references. When dropped, the PeerRef will send
/// a request to the PeerManager to remove a reference to the peer, thus removing the peer if no
/// more references exists.

#[derive(Debug, PartialEq)]
pub struct PeerRef {
    peer_id: String,
    peer_remover: PeerRemover,
}

impl PeerRef {
    pub(super) fn new(peer_id: String, peer_remover: PeerRemover) -> Self {
        PeerRef {
            peer_id,
            peer_remover,
        }
    }

    pub fn peer_id(&mut self) -> &str {
        &self.peer_id
    }
}

impl Drop for PeerRef {
    fn drop(&mut self) {
        match self.peer_remover.remove_peer_ref(&self.peer_id) {
            Ok(_) => (),
            Err(err) => error!(
                "Unable to remove reference to {} on drop: {}",
                self.peer_id, err
            ),
        }
    }
}

/// The PeerManager is in charge of keeping track of peers and their ref count, as well as
/// requesting connections from the ConnectionManager. If a peer has disconnected, the PeerManager
/// will also try the peer's other endpoints until one is successful.
pub struct PeerManager {
    connection_manager_connector: Connector,
    join_handle: Option<thread::JoinHandle<()>>,
    sender: Option<Sender<PeerManagerMessage>>,
    shutdown_handle: Option<ShutdownHandle>,
    max_retry_attempts: Option<u64>,
}

impl PeerManager {
    pub fn new(connector: Connector, max_retry_attempts: Option<u64>) -> Self {
        PeerManager {
            connection_manager_connector: connector,
            join_handle: None,
            sender: None,
            shutdown_handle: None,
            max_retry_attempts,
        }
    }

    /// Start the PeerManager
    ///
    /// Returns a PeerManagerConnector that can be used to send requests to the PeerManager.
    pub fn start(&mut self) -> Result<PeerManagerConnector, PeerManagerError> {
        let (sender, recv) = channel();
        if self.sender.is_some() {
            return Err(PeerManagerError::StartUpError(
                "PeerManager has already been started".to_string(),
            ));
        }
        let connector = self.connection_manager_connector.clone();
        let peer_remover = PeerRemover {
            sender: sender.clone(),
        };

        let subscriber_id = connector.subscribe(sender.clone()).map_err(|err| {
            PeerManagerError::StartUpError(format!(
                "Unable to subscribe to connection manager notifications: {}",
                err
            ))
        })?;

        let max_retry_attempts = self
            .max_retry_attempts
            .unwrap_or(DEFAULT_MAXIMUM_RETRY_ATTEMPTS);
        let join_handle = thread::Builder::new()
            .name("Peer Manager".into())
            .spawn(move || {
                let mut peers = PeerMap::new();
                let mut ref_map = RefMap::new();
                let mut subscribers = Vec::new();
                loop {
                    match recv.recv() {
                        Ok(PeerManagerMessage::Shutdown) => break,
                        Ok(PeerManagerMessage::Request(request)) => {
                            handle_request(
                                request,
                                connector.clone(),
                                &mut peers,
                                &peer_remover,
                                &mut ref_map,
                            );
                        }
                        Ok(PeerManagerMessage::Subscribe(sender)) => {
                            subscribers.push(sender);
                        }
                        Ok(PeerManagerMessage::InternalNotification(notification)) => {
                            handle_notifications(
                                notification,
                                &mut peers,
                                connector.clone(),
                                &mut subscribers,
                                max_retry_attempts,
                            )
                        }
                        Err(_) => {
                            warn!("All senders have disconnected");
                            break;
                        }
                    }
                }

                if let Err(err) = connector.unsubscribe(subscriber_id) {
                    error!(
                        "Unable to unsubscribe from connection manager notifications: {}",
                        err
                    );
                }
            });

        match join_handle {
            Ok(join_handle) => {
                self.join_handle = Some(join_handle);
            }
            Err(err) => {
                return Err(PeerManagerError::StartUpError(format!(
                    "Unable to start PeerManager thread {}",
                    err
                )))
            }
        }

        self.shutdown_handle = Some(ShutdownHandle {
            sender: sender.clone(),
        });
        self.sender = Some(sender.clone());
        Ok(PeerManagerConnector::new(sender))
    }

    pub fn shutdown_handle(&self) -> Option<ShutdownHandle> {
        self.shutdown_handle.clone()
    }

    pub fn await_shutdown(self) {
        let join_handle = if let Some(jh) = self.join_handle {
            jh
        } else {
            return;
        };

        if let Err(err) = join_handle.join() {
            error!("Peer manager thread did not shutdown correctly: {:?}", err);
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
pub struct ShutdownHandle {
    sender: Sender<PeerManagerMessage>,
}

impl ShutdownHandle {
    pub fn shutdown(&self) {
        if self.sender.send(PeerManagerMessage::Shutdown).is_err() {
            warn!("Connection manager is no longer running");
        }
    }
}

fn handle_request(
    request: PeerManagerRequest,
    connector: Connector,
    peers: &mut PeerMap,
    peer_remover: &PeerRemover,
    ref_map: &mut RefMap,
) {
    match request {
        PeerManagerRequest::AddPeer {
            peer_id,
            endpoints,
            sender,
        } => {
            if sender
                .send(add_peer(
                    peer_id,
                    endpoints,
                    connector,
                    peers,
                    peer_remover,
                    ref_map,
                ))
                .is_err()
            {
                warn!("connector dropped before receiving result of adding peer");
            }
        }
        PeerManagerRequest::UpdatePeer {
            old_peer_id,
            new_peer_id,
            sender,
        } => {
            if sender
                .send(update_peer(old_peer_id, new_peer_id, peers, ref_map))
                .is_err()
            {
                warn!("connector dropped before receiving result of updating peer");
            }
        }
        PeerManagerRequest::RemovePeer { peer_id, sender } => {
            if sender
                .send(remove_peer(peer_id, connector, peers, ref_map))
                .is_err()
            {
                warn!("connector dropped before receiving result of removing peer");
            }
        }
        PeerManagerRequest::ListPeers { sender } => {
            if sender.send(Ok(peers.peer_ids())).is_err() {
                warn!("connector dropped before receiving result of list peers");
            }
        }
    };
}

fn add_peer(
    peer_id: String,
    endpoints: Vec<String>,
    connector: Connector,
    peers: &mut PeerMap,
    peer_remover: &PeerRemover,
    ref_map: &mut RefMap,
) -> Result<PeerRef, PeerRefAddError> {
    let new_ref_count = ref_map.add_ref(peer_id.to_string());

    // if this is not a new peer, return success
    if new_ref_count > 1 {
        let peer_ref = PeerRef::new(peer_id, peer_remover.clone());
        return Ok(peer_ref);
    };

    debug!("Attempting to peer with {}", peer_id);
    let connection_id = format!("{}", Uuid::new_v4());
    // If new, try to create a connection
    for endpoint in endpoints.iter() {
        match connector.request_connection(&endpoint, &connection_id) {
            Ok(()) => {
                debug!("Peer {} connected via {}", peer_id, endpoint);
                peers.insert(
                    peer_id.clone(),
                    connection_id,
                    endpoints.clone(),
                    endpoint.to_string(),
                );
                let peer_ref = PeerRef::new(peer_id, peer_remover.clone());
                return Ok(peer_ref);
            }
            Err(err) => {
                warn!("Unable to connect to endpoint {}: {}", endpoint, err);
                continue;
            }
        }
    }

    warn!("Unable to peer with {}", peer_id);
    // remove the reference created above
    ref_map.remove_ref(&peer_id);
    // unable to connect to any of the endpoints provided
    Err(PeerRefAddError::AddError(format!(
        "Unable to connect any endpoint that was provided for peer {}",
        peer_id
    )))
}

fn update_peer(
    peer_id: String,
    new_peer_id: String,
    peers: &mut PeerMap,
    ref_map: &mut RefMap,
) -> Result<(), PeerRefUpdateError> {
    // update the ref_map, so old PeerRef can still be used to drop references
    if ref_map
        .update_ref(peer_id.clone(), new_peer_id.clone())
        .is_err()
    {
        return Err(PeerRefUpdateError::UpdateError(format!(
            "Unable to update peer, {} does not exist",
            peer_id
        )));
    }

    // update the peer in the peer map
    match peers.update_peer_id(peer_id.clone(), new_peer_id.clone()) {
        Ok(()) => {
            debug!("Updated peer id from {} to {}", peer_id, new_peer_id);
            Ok(())
        }
        Err(_) => Err(PeerRefUpdateError::UpdateError(format!(
            "Unable to update peer, {} does not exist",
            peer_id
        ))),
    }
}

fn remove_peer(
    peer_id: String,
    connector: Connector,
    peers: &mut PeerMap,
    ref_map: &mut RefMap,
) -> Result<(), PeerRefRemoveError> {
    debug!("Removing peer: {}", peer_id);
    // remove the reference
    let removed_peer = ref_map.remove_ref(&peer_id);
    if let Some(removed_peer) = removed_peer {
        let active_endpoint = peers.remove(&removed_peer).ok_or_else(|| {
            PeerRefRemoveError::RemoveError(format!(
                "Peer {} has already been removed from the peer map",
                peer_id
            ))
        })?;

        match connector.remove_connection(&active_endpoint) {
            Ok(Some(_)) => {
                debug!(
                    "Peer {} has been removed and connection {} has been closed",
                    peer_id, active_endpoint
                );
                Ok(())
            }
            Ok(None) => Err(PeerRefRemoveError::RemoveError(
                "No connection to remove, something has gone wrong".to_string(),
            )),
            Err(err) => Err(PeerRefRemoveError::RemoveError(format!("{}", err))),
        }
    } else {
        // if the peer has not been fully removed, return OK
        Ok(())
    }
}

// If a connection has reached the retry limit before it could be reestablished, the peer manager
// will try the peer's other endpoints.
fn retry_endpoints(
    peer_metadata: &mut PeerMetadata,
    connector: Connector,
) -> Result<bool, PeerManagerError> {
    debug!("Trying to find active endpoint for {}", peer_metadata.id);
    for endpoint in peer_metadata.endpoints.iter() {
        match connector.request_connection(&endpoint, &peer_metadata.connection_id) {
            Ok(()) => {
                debug!("Peered with {}: {}", peer_metadata.id, endpoint);
                if endpoint != &peer_metadata.active_endpoint {
                    // Remove old active endpoint from peer_manager
                    match connector.remove_connection(&peer_metadata.active_endpoint) {
                        Ok(Some(_)) => (),
                        Ok(None) => (),
                        Err(err) => {
                            return Err(PeerManagerError::RetryEndpoints(format!(
                                "Unable to remove active endpoint {} from connection manager: {}",
                                &peer_metadata.active_endpoint, err
                            )))
                        }
                    }
                }
                peer_metadata.active_endpoint = endpoint.to_string();
                return Ok(true);
            }
            Err(err) => {
                warn!("Unable to connect to endpoint {}: {}", endpoint, err);
                continue;
            }
        }
    }

    warn!(
        "Unable to find new active endpoint for peer {}",
        peer_metadata.id
    );
    // unable to connect to any of the endpoints provided
    Ok(false)
}

fn handle_notifications(
    notification: ConnectionManagerNotification,
    peers: &mut PeerMap,
    connector: Connector,
    subscribers: &mut Vec<Sender<PeerManagerNotification>>,
    max_retry_attempts: u64,
) {
    match notification {
        // If a connection has been successful, forward notification to subscribers
        ConnectionManagerNotification::Connected { endpoint } => {
            // if we have a corresponding peer for for endpoint, send notification; otherwise
            // ignore
            if let Some(mut peer_metadata) = peers.get_peer_from_endpoint(&endpoint).cloned() {
                let notification = PeerManagerNotification::Connected {
                    peer: peer_metadata.id.to_string(),
                };
                subscribers.retain(|sender| sender.send(notification.clone()).is_ok());
                // if a peer was previously disconnected, remove from disconnected list
                peer_metadata.status = PeerStatus::Connected;
                if let Err(err) = peers.update_peer(peer_metadata) {
                    error!("Unable to update peer: {}", err);
                }
            }
        }
        // If a connection has disconnected, forward notification to subscribers
        ConnectionManagerNotification::Disconnected { endpoint } => {
            if let Some(mut peer_metadata) = peers.get_peer_from_endpoint(&endpoint).cloned() {
                let notification = PeerManagerNotification::Disconnected {
                    peer: peer_metadata.id.to_string(),
                };
                subscribers.retain(|sender| sender.send(notification.clone()).is_ok());
                // set peer to disconnected
                peer_metadata.status = PeerStatus::Disconnected { retry_attempts: 1 };
                if let Err(err) = peers.update_peer(peer_metadata) {
                    error!("Unable to update peer: {}", err);
                }
            }
        }
        ConnectionManagerNotification::ReconnectionFailed { endpoint, attempts } => {
            // Check if the disconnected peer has reached the retry limit, if so try to find a
            // different endpoint that can be connected to
            if let Some(mut peer_metadata) = peers.get_peer_from_endpoint(&endpoint).cloned() {
                if attempts >= max_retry_attempts {
                    match retry_endpoints(&mut peer_metadata, connector) {
                        Ok(true) => {
                            peer_metadata.status = PeerStatus::Connected;
                            let notification = PeerManagerNotification::Connected {
                                peer: peer_metadata.id.to_string(),
                            };
                            subscribers.retain(|sender| sender.send(notification.clone()).is_ok());
                        }
                        // if a new endpoint could not be found, reset timeout and try again later
                        Ok(false) => {
                            peer_metadata.status = PeerStatus::Disconnected {
                                retry_attempts: attempts,
                            };
                        }
                        Err(err) => {
                            error!("Error returned from retry_endpoints: {}", err);
                            peer_metadata.status = PeerStatus::Disconnected {
                                retry_attempts: attempts,
                            };
                        }
                    }
                    if let Err(err) = peers.update_peer(peer_metadata) {
                        error!("Unable to update peer: {}", err);
                    }
                }
            }
        }
        ConnectionManagerNotification::InboundConnection { .. } => (),
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::mesh::Mesh;
    use crate::network::connection_manager::ConnectionManager;
    use crate::protos::network::{NetworkMessage, NetworkMessageType};
    use crate::transport::inproc::InprocTransport;
    use crate::transport::raw::RawTransport;
    use crate::transport::Transport;

    // Test that a call to add_peer_ref returns the correct PeerRef
    //
    // 1. add test_peer
    // 2. verify that the returned PeerRef contains the test_peer id
    #[test]
    fn test_peer_manager_add_peer() {
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
        let mut peer_manager = PeerManager::new(connector, None);
        let peer_connector = peer_manager.start().expect("Cannot start peer_manager");
        let peer_ref = peer_connector
            .add_peer_ref("test_peer".to_string(), vec!["inproc://test".to_string()])
            .expect("Unable to add peer");

        assert_eq!(peer_ref.peer_id, "test_peer");
        peer_manager.shutdown_and_wait();
        cm.shutdown_and_wait();
    }

    // Test that a call to add_peer_ref with a peer with multiple endpoints is successful, even if
    // the first endpoint is not available
    //
    // 1. add test_peer with two endpoints. The first endpoint will fail and cause the peer
    //    manager to try the second
    // 2. verify that the returned PeerRef contains the test_peer id
    #[test]
    fn test_peer_manager_add_peer_endpoints() {
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
        let mut peer_manager = PeerManager::new(connector, None);
        let peer_connector = peer_manager.start().expect("Cannot start peer_manager");
        let peer_ref = peer_connector
            .add_peer_ref(
                "test_peer".to_string(),
                vec![
                    "inproc://bad_endpoint".to_string(),
                    "inproc://test".to_string(),
                ],
            )
            .expect("Unable to add peer");

        assert_eq!(peer_ref.peer_id, "test_peer");
        peer_manager.shutdown_and_wait();
        cm.shutdown_and_wait();
    }

    // Test that the same peer can be added multiple times.
    //
    // 1. add test_peer
    // 2. add the same peer, and see it is successful
    #[test]
    fn test_peer_manager_add_peer_multiple_times() {
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
        let mut peer_manager = PeerManager::new(connector, None);
        let peer_connector = peer_manager.start().expect("Cannot start peer_manager");
        let peer_ref = peer_connector
            .add_peer_ref("test_peer".to_string(), vec!["inproc://test".to_string()])
            .expect("Unable to add peer");

        assert_eq!(peer_ref.peer_id, "test_peer");

        let peer_ref = peer_connector
            .add_peer_ref("test_peer".to_string(), vec!["inproc://test".to_string()])
            .expect("Unable to add peer");

        assert_eq!(peer_ref.peer_id, "test_peer");
        peer_manager.shutdown_and_wait();
        cm.shutdown_and_wait();
    }

    // Test that list_peer returns the correct list of peers
    //
    // 1. add test_peer
    // 2. add next_peer
    // 3. call list_peers
    // 4. verify that the sorted list of peers contains both test_peer and next_peer
    #[test]
    fn test_peer_manager_list_peer() {
        let mut transport = Box::new(InprocTransport::default());
        let mut listener = transport.listen("inproc://test").unwrap();

        thread::spawn(move || {
            listener.accept().unwrap();
        });

        let mut listener = transport.listen("inproc://test_2").unwrap();
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
        let mut peer_manager = PeerManager::new(connector, None);
        let peer_connector = peer_manager.start().expect("Cannot start peer_manager");
        let peer_ref_1 = peer_connector
            .add_peer_ref("test_peer".to_string(), vec!["inproc://test".to_string()])
            .expect("Unable to add peer");

        assert_eq!(peer_ref_1.peer_id, "test_peer");

        let peer_ref_2 = peer_connector
            .add_peer_ref("next_peer".to_string(), vec!["inproc://test_2".to_string()])
            .expect("Unable to add peer");

        assert_eq!(peer_ref_2.peer_id, "next_peer");

        let mut peer_list = peer_connector
            .list_peers()
            .expect("Unable to get peer list");

        peer_list.sort();

        assert_eq!(
            peer_list,
            vec!["next_peer".to_string(), "test_peer".to_string()]
        );
        peer_manager.shutdown_and_wait();
        cm.shutdown_and_wait();
    }

    // Test that if a peer is updated, it is properly put in the list_peer list
    //
    // 1. add test_peer
    // 2. update test_peer to have a new id, new_peer
    // 3. call list_peers
    // 4. verify that list peers contains only new_peer
    #[test]
    fn test_peer_manager_update_peer() {
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
        let mut peer_manager = PeerManager::new(connector, None);
        let peer_connector = peer_manager.start().expect("Cannot start peer_manager");
        let peer_ref = peer_connector
            .add_peer_ref("test_peer".to_string(), vec!["inproc://test".to_string()])
            .expect("Unable to add peer");

        assert_eq!(peer_ref.peer_id, "test_peer");

        peer_connector
            .update_peer_ref("test_peer", "new_peer")
            .expect("Unable to update peer id");

        let peer_list = peer_connector
            .list_peers()
            .expect("Unable to get peer list");

        assert_eq!(peer_list, vec!["new_peer".to_string()]);
        peer_manager.shutdown_and_wait();
        cm.shutdown_and_wait();
    }

    // Test that when a PeerRef is dropped, a remove peer request is properly sent and the peer
    // is removed
    //
    // 1. add test_peer
    // 2. call list peers
    // 3. verify that the peer list contains test_peer
    // 4. drop the PeerRef
    // 5. call list peers
    // 6. verify that the new peer list is empty
    #[test]
    fn test_peer_manager_drop_peer_ref() {
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
        let mut peer_manager = PeerManager::new(connector, None);

        let peer_connector = peer_manager.start().expect("Cannot start peer_manager");

        {
            let peer_ref = peer_connector
                .add_peer_ref("test_peer".to_string(), vec!["inproc://test".to_string()])
                .expect("Unable to add peer");

            assert_eq!(peer_ref.peer_id, "test_peer");

            let peer_list = peer_connector
                .list_peers()
                .expect("Unable to get peer list");

            assert_eq!(peer_list, vec!["test_peer".to_string()]);
        }
        // drop peer_ref

        let peer_list = peer_connector
            .list_peers()
            .expect("Unable to get peer list");

        assert_eq!(peer_list, Vec::<String>::new());
        peer_manager.shutdown_and_wait();
        cm.shutdown_and_wait();
    }

    // Test that if a peer is updated, an old peer_ref (with the old peer id) can still remove
    // a reference for that peer.
    //
    // 1. add test_peer
    // 2. update test_peer id to new_peer
    // 3. call list peers
    // 4. verify that the peer list contains new_peer
    // 5. Add reference to new_peer, and verify new_peer is the only peer in the peer lst
    // 6. drop the PeerRef for new_peer
    // 7. call list peers
    // 8. verify that the peer list still contains new_peer
    // 9. drop the originally PeerREf for test_peer
    // 10. call list peers and verify that it is empty
    #[test]
    fn test_peer_manager_drop_updated_peer_ref() {
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
        let mut peer_manager = PeerManager::new(connector, None);

        let peer_connector = peer_manager.start().expect("Cannot start peer_manager");

        {
            // create peer_ref with peer_id test_peer
            let peer_ref = peer_connector
                .add_peer_ref("test_peer".to_string(), vec!["inproc://test".to_string()])
                .expect("Unable to add peer");

            assert_eq!(peer_ref.peer_id, "test_peer");

            // update peer id
            peer_connector
                .update_peer_ref("test_peer", "new_peer")
                .expect("Unable to update peer id");

            let peer_list = peer_connector
                .list_peers()
                .expect("Unable to get peer list");

            assert_eq!(peer_list, vec!["new_peer".to_string()]);

            {
                // add another reference to new_peer
                let peer_ref_2 = peer_connector
                    .add_peer_ref("new_peer".to_string(), vec!["inproc://test".to_string()])
                    .expect("Unable to add peer");

                assert_eq!(peer_ref_2.peer_id, "new_peer");

                // verify that only 1 peer is listed
                let peer_list = peer_connector
                    .list_peers()
                    .expect("Unable to get peer list");

                assert_eq!(peer_list, vec!["new_peer".to_string()]);
            }
            // drop peer ref 2, reference has peer id "new_peer"

            // verify that new_peer has not been removed
            let peer_list = peer_connector
                .list_peers()
                .expect("Unable to get peer list");

            assert_eq!(peer_list, vec!["new_peer".to_string()]);
        }
        // drop peer ref with old peer id test_peer

        // verify that the peer has been removed
        let peer_list = peer_connector
            .list_peers()
            .expect("Unable to get peer list");

        assert_eq!(peer_list, Vec::<String>::new());
        peer_manager.shutdown_and_wait();
        cm.shutdown_and_wait();
    }

    // Test that if a peer's endpoint disconnects and does not reconnect during a set timeout, the
    // PeerManager will retry the peers list of endpoints trying to find an endpoint that is
    // available.
    //
    // 1. add test_peer, this will connected to the first endpoint
    // 2. verify that the test_peer connection receives a heartbeat
    // 3. disconnect the connection made to test_peer
    // 4. verify that subscribers will receive a Disconnected notification
    // 5. drop the listener for the first endpoint to force the attempt on the second endpoint
    // 6. verify that subscribers will receive a Connected notfication when the new active endpoint
    //    is successfully connected to.
    #[test]
    fn test_peer_manager_update_active_endpoint() {
        let mut transport = Box::new(RawTransport::default());
        let mut listener = transport
            .listen("tcp://localhost:0")
            .expect("Cannot listen for connections");
        let endpoint = listener.endpoint();
        let mesh1 = Mesh::new(512, 128);
        let mesh2 = Mesh::new(512, 128);

        let mut listener2 = transport
            .listen("tcp://localhost:0")
            .expect("Cannot listen for connections");
        let endpoint2 = listener2.endpoint();

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
                .remove("test_id")
                .expect("Cannot remove connection from mesh");
            connection
                .disconnect()
                .expect("Connection failed to disconnect");
            // force drop of first listener
            drop(listener);
            // wait for the peer manager to switch endpoints
            let conn = listener2.accept().expect("Unable to accept connection");
            mesh2
                .add(conn, "test_id".to_string())
                .expect("Cannot add connection to mesh");
            mesh2.recv().expect("Cannot receive message");
        });

        let mut cm = ConnectionManager::new(
            mesh1.get_life_cycle(),
            mesh1.get_sender(),
            transport,
            Some(1),
            None,
        );
        let connector = cm.start().unwrap();
        let mut peer_manager = PeerManager::new(connector, Some(1));
        let peer_connector = peer_manager.start().expect("Cannot start peer_manager");
        let mut subscriber = peer_connector.subscribe().expect("Unable to subscribe");
        let peer_ref = peer_connector
            .add_peer_ref("test_peer".to_string(), vec![endpoint, endpoint2])
            .expect("Unable to add peer");

        assert_eq!(peer_ref.peer_id, "test_peer");

        // receive reconnecting attempt
        let disconnected_notification = subscriber
            .next()
            .expect("Cannot get message from subscriber");
        assert!(
            disconnected_notification
                == PeerManagerNotification::Disconnected {
                    peer: "test_peer".to_string(),
                }
        );

        // receive notifications that the peer is connected to new endpoint
        let connected_notification = subscriber
            .next()
            .expect("Cannot get message from subscriber");

        assert!(
            connected_notification
                == PeerManagerNotification::Connected {
                    peer: "test_peer".to_string(),
                }
        );

        peer_manager.shutdown_and_wait();
        cm.shutdown_and_wait();
    }

    // Test that the PeerManager can be started and stopped
    #[test]
    fn test_peer_manager_shutdown() {
        let transport = Box::new(InprocTransport::default());

        let mesh = Mesh::new(512, 128);
        let mut cm = ConnectionManager::new(
            mesh.get_life_cycle(),
            mesh.get_sender(),
            transport,
            None,
            None,
        );
        let connector = cm.start().unwrap();
        let mut peer_manager = PeerManager::new(connector, None);
        peer_manager.start().expect("Cannot start peer_manager");

        peer_manager.shutdown_and_wait();
    }
}
