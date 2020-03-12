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

use std::sync::mpsc::{channel, Sender};

use super::error::{
    PeerListError, PeerManagerError, PeerRefAddError, PeerRefRemoveError, PeerRefUpdateError,
};
use super::notification::PeerNotificationIter;
use super::PeerRef;
use super::{PeerManagerMessage, PeerManagerRequest};

/// The PeerManagerConnector will be used to make requests to the PeerManager.
///
/// The connector includes functions to add a new peer reference, update a peer and list the
/// existing peers.
#[derive(Clone, Debug)]
pub struct PeerManagerConnector {
    sender: Sender<PeerManagerMessage>,
}

impl PeerManagerConnector {
    pub(crate) fn new(sender: Sender<PeerManagerMessage>) -> Self {
        PeerManagerConnector { sender }
    }

    /// Request that a peer is added to the PeerManager. If a peer already exists, the peer's ref
    /// count will be incremented
    ///
    /// # Arguments
    ///
    /// * `peer_id` -  The unique id for the peer.
    /// * `endpoints` -  The list of endpoints associated with the peer. The list should be in
    ///     preference order, with the first endpoint being the first attempted.
    ///
    /// Returns a PeerRef, that when dropped, will automatically send a removal request to the
    /// PeerManager.
    pub fn add_peer_ref(
        &self,
        peer_id: String,
        endpoints: Vec<String>,
    ) -> Result<PeerRef, PeerRefAddError> {
        let (sender, recv) = channel();

        let message = PeerManagerMessage::Request(PeerManagerRequest::AddPeer {
            peer_id,
            endpoints,
            sender,
        });

        match self.sender.send(message) {
            Ok(()) => (),
            Err(_) => {
                return Err(PeerRefAddError::InternalError(
                    "Unable to send message to PeerManager, receiver dropped".to_string(),
                ))
            }
        };

        recv.recv()
            .map_err(|err| PeerRefAddError::ReceiveError(format!("{:?}", err)))?
    }

    /// Request that a peer is updated. If a peer already exists, update a peer id. Redirections
    /// will be added to the peer map and reference count so the old PeerRef will still work.
    ///
    /// # Arguments
    ///
    /// * `old_peer_id` -  The old peer_id of the peer.
    /// * `new_peer_id` -  The new peer_id of the peer.
    pub fn update_peer_ref(
        &self,
        old_peer_id: &str,
        new_peer_id: &str,
    ) -> Result<(), PeerRefUpdateError> {
        let (sender, recv) = channel();

        let message = PeerManagerMessage::Request(PeerManagerRequest::UpdatePeer {
            old_peer_id: old_peer_id.to_string(),
            new_peer_id: new_peer_id.to_string(),
            sender,
        });

        match self.sender.send(message) {
            Ok(()) => (),
            Err(_) => {
                return Err(PeerRefUpdateError::InternalError(
                    "Unable to send message to PeerManager, receiver dropped".to_string(),
                ))
            }
        };

        recv.recv()
            .map_err(|err| PeerRefUpdateError::ReceiveError(format!("{:?}", err)))?
    }

    /// Request the list of currently connected peers.
    ///
    /// Returns the list of peer ids.
    pub fn list_peers(&self) -> Result<Vec<String>, PeerListError> {
        let (sender, recv) = channel();
        let message = PeerManagerMessage::Request(PeerManagerRequest::ListPeers { sender });

        match self.sender.send(message) {
            Ok(()) => (),
            Err(_) => {
                return Err(PeerListError::InternalError(
                    "Unable to send message to PeerManager, receiver dropped".to_string(),
                ))
            }
        };

        recv.recv()
            .map_err(|err| PeerListError::ReceiveError(format!("{:?}", err)))?
    }

    /// Subscribe to PeerManager notifications.
    ///
    /// Returns a PeerNotificationIter that can be used to receive notications about connected and
    /// disconnected peers
    pub fn subscribe(&self) -> Result<PeerNotificationIter, PeerManagerError> {
        let (send, recv) = channel();
        match self.sender.send(PeerManagerMessage::Subscribe(send)) {
            Ok(()) => Ok(PeerNotificationIter { recv }),
            Err(_) => Err(PeerManagerError::SendMessageError(
                "The peer manager is no longer running".into(),
            )),
        }
    }
}

/// The PeerRemover will be used in the PeerRef to decrement the reference count for a peer when
/// the PeerRef is dropped.
#[derive(Clone, Debug)]
pub(crate) struct PeerRemover {
    pub sender: Sender<PeerManagerMessage>,
}

impl PeerRemover {
    /// This function will only be called when the PeerRef is dropped.
    ///
    /// Sends a request to the PeerManager to remove a peer.
    ///
    /// # Arguments
    /// * `peer_id` - the peer_id of the PeerRef that has been dropped
    pub fn remove_peer_ref(&self, peer_id: &str) -> Result<(), PeerRefRemoveError> {
        let (sender, recv) = channel();

        let message = PeerManagerMessage::Request(PeerManagerRequest::RemovePeer {
            peer_id: peer_id.to_string(),
            sender,
        });

        match self.sender.send(message) {
            Ok(()) => (),
            Err(_) => {
                return Err(PeerRefRemoveError::InternalError(
                    "Unable to send message to PeerManager, receiver dropped".to_string(),
                ))
            }
        };

        recv.recv()
            .map_err(|err| PeerRefRemoveError::ReceiveError(format!("{:?}", err)))?
    }
}

impl PartialEq for PeerRemover {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}
