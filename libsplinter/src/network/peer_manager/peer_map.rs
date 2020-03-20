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

use std::collections::HashMap;

use super::error::PeerUpdateError;

#[derive(Clone, PartialEq, Debug)]
pub enum PeerStatus {
    Connected,
    Disconnected { retry_attempts: u64 },
}

#[derive(Clone, PartialEq, Debug)]
pub struct PeerMetadata {
    pub id: String,
    pub connection_id: String,
    pub endpoints: Vec<String>,
    pub active_endpoint: String,
    pub status: PeerStatus,
}

pub struct PeerMap {
    peers: HashMap<String, PeerMetadata>,
    redirects: HashMap<String, String>,
    // Endpoint to peer id
    endpoints: HashMap<String, String>,
}

/// A map of Peer IDs to peer metadata, which also maintains a redirect table for updated peer IDs.
///
/// Peer metadata includes the peer_id, the list of endpoints and the current active endpoint.
impl PeerMap {
    pub fn new() -> Self {
        PeerMap {
            peers: HashMap::new(),
            redirects: HashMap::new(),
            endpoints: HashMap::new(),
        }
    }

    /// Returns the current list of peer ids.
    ///
    /// This list does not include any of the redirected peer ids.
    pub fn peer_ids(&self) -> Vec<String> {
        self.peers
            .iter()
            .map(|(_, metadata)| metadata.id.to_string())
            .collect()
    }

    /// Insert a new peer id and endpoints
    pub fn insert(
        &mut self,
        peer_id: String,
        connection_id: String,
        endpoints: Vec<String>,
        active_endpoint: String,
    ) {
        let peer_metadata = PeerMetadata {
            id: peer_id.clone(),
            endpoints: endpoints.clone(),
            active_endpoint,
            status: PeerStatus::Connected,
            connection_id,
        };

        self.peers.insert(peer_id.clone(), peer_metadata);

        for endpoint in endpoints {
            self.endpoints.insert(endpoint, peer_id.clone());
        }
    }

    /// Remove a peer id, its endpoint and all of its redirects. Returns the active_endpoint of
    /// the peer.
    pub fn remove(&mut self, peer_id: &str) -> Option<String> {
        self.redirects
            .retain(|_, target_peer_id| target_peer_id != peer_id);
        if let Some(peer_metadata) = self.peers.remove(&peer_id.to_string()) {
            for endpoint in peer_metadata.endpoints.iter() {
                self.endpoints.remove(endpoint);
            }

            Some(peer_metadata.active_endpoint)
        } else {
            None
        }
    }

    /// Updates a peer id, and creates a redirect for the old id to the given new one.
    ///
    /// Additionally, it updates all of the old redirects to point to the given new one.
    pub fn update_peer_id(
        &mut self,
        old_peer_id: String,
        new_peer_id: String,
    ) -> Result<(), PeerUpdateError> {
        if let Some(mut peer_metadata) = self.peers.remove(&old_peer_id) {
            // let mut new_peer_metadata = peer_metadata.clone();
            for endpoint in peer_metadata.endpoints.iter() {
                self.endpoints
                    .insert(endpoint.to_string(), new_peer_id.clone());
            }

            peer_metadata.id = new_peer_id.clone();
            self.peers.insert(new_peer_id.clone(), peer_metadata);

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
            Err(PeerUpdateError(format!(
                "Unable to update {} to {}",
                old_peer_id, new_peer_id
            )))
        }
    }

    /// Updates an existing peer, all fields can be updated except peer_id.
    pub fn update_peer(&mut self, peer_metadata: PeerMetadata) -> Result<(), PeerUpdateError> {
        // Only valid if the peer already exists
        if self.peers.contains_key(&peer_metadata.id) {
            for endpoint in peer_metadata.endpoints.iter() {
                self.endpoints
                    .insert(endpoint.to_string(), peer_metadata.id.clone());
            }

            self.peers
                .insert(peer_metadata.id.to_string(), peer_metadata);

            Ok(())
        } else {
            Err(PeerUpdateError(format!(
                "Unable to update peer {}, does not exist",
                peer_metadata.id
            )))
        }
    }

    /// Returns the endpoint for the given peer id
    pub fn get_peer_from_endpoint(&self, endpoint: &str) -> Option<&PeerMetadata> {
        if let Some(peer) = self.endpoints.get(endpoint) {
            self.peers.get(peer)
        } else {
            None
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    // Test that peer_ids() are returned correctly
    //  1. Test that an empty peer_map returns an empty vec of peer IDs
    //  2. Add two peers and test that their id are returned from peer_ids()
    //  3. Update the first peer and test the updated peer id is returned in place of the old id.
    #[test]
    fn test_get_peer_ids() {
        let mut peer_map = PeerMap::new();

        let peers = peer_map.peer_ids();
        assert_eq!(peers, Vec::<String>::new());

        peer_map.insert(
            "test_peer".to_string(),
            "connection_id_1".to_string(),
            vec!["test_endpoint1".to_string(), "test_endpoint2".to_string()],
            "test_endpoint2".to_string(),
        );

        peer_map.insert(
            "next_peer".to_string(),
            "connection_id_2".to_string(),
            vec!["endpoint1".to_string(), "endpoint2".to_string()],
            "next_endpoint1".to_string(),
        );

        let mut peers = peer_map.peer_ids();
        peers.sort();
        assert_eq!(
            peers,
            vec!["next_peer".to_string(), "test_peer".to_string()]
        );

        peer_map
            .update_peer_id("test_peer".to_string(), "new_peer".to_string())
            .expect("Unable to update peer id");

        let mut peers = peer_map.peer_ids();
        peers.sort();
        assert_eq!(peers, vec!["new_peer".to_string(), "next_peer".to_string()]);
    }

    // Test that peer_metadata() return the correct PeerMetadata for the provided id
    //  1. Test that None is retured for a peer ID that does not exist
    //  2. Insert a peer
    //  3. Validate the expected PeerMetadata is returned from peer_metadata()
    #[test]
    fn test_get_peer_by_endpoint() {
        let mut peer_map = PeerMap::new();

        let peer_metadata = peer_map.get_peer_from_endpoint("bad_endpoint");
        assert_eq!(peer_metadata, None);

        peer_map.insert(
            "test_peer".to_string(),
            "connection_id".to_string(),
            vec!["test_endpoint1".to_string(), "test_endpoint2".to_string()],
            "test_endpoint2".to_string(),
        );

        let peer_metadata = peer_map.get_peer_from_endpoint("test_endpoint1");
        assert_eq!(
            peer_metadata,
            Some(&PeerMetadata {
                id: "test_peer".to_string(),
                connection_id: "connection_id".to_string(),
                endpoints: vec!["test_endpoint1".to_string(), "test_endpoint2".to_string()],
                active_endpoint: "test_endpoint2".to_string(),
                status: PeerStatus::Connected
            })
        );

        let peer_metadata = peer_map.get_peer_from_endpoint("test_endpoint2");
        assert_eq!(
            peer_metadata,
            Some(&PeerMetadata {
                id: "test_peer".to_string(),
                connection_id: "connection_id".to_string(),
                endpoints: vec!["test_endpoint1".to_string(), "test_endpoint2".to_string()],
                active_endpoint: "test_endpoint2".to_string(),
                status: PeerStatus::Connected
            })
        );
    }

    // Test that a peer can properly be added
    //  1. Insert a peer
    //  2. Check that the peer is in self.peers
    //  3. Check that the correct metadata is returned from self.peers.get()
    #[test]
    fn test_insert_peer() {
        let mut peer_map = PeerMap::new();

        peer_map.insert(
            "test_peer".to_string(),
            "connection_id".to_string(),
            vec!["test_endpoint1".to_string(), "test_endpoint2".to_string()],
            "test_endpoint2".to_string(),
        );
        assert!(peer_map.peers.contains_key("test_peer"));

        let peer_metadata = peer_map.peers.get("test_peer");
        assert_eq!(
            peer_metadata,
            Some(&PeerMetadata {
                id: "test_peer".to_string(),
                connection_id: "connection_id".to_string(),
                endpoints: vec!["test_endpoint1".to_string(), "test_endpoint2".to_string()],
                active_endpoint: "test_endpoint2".to_string(),
                status: PeerStatus::Connected
            })
        );
    }

    // Test that a peer can be properly removed
    //  1. Test that removing a peer_id that is not in the peer map will return None
    //  2. Insert peer test_peer and verify id is in self.peers
    //  3. Verify that the correct active endpoint is returned when removing test_peer
    #[test]
    fn test_remove_peer() {
        let mut peer_map = PeerMap::new();

        let active_endpoint = peer_map.remove("test_peer");

        assert_eq!(active_endpoint, None,);

        peer_map.insert(
            "test_peer".to_string(),
            "connection_id".to_string(),
            vec!["test_endpoint1".to_string(), "test_endpoint2".to_string()],
            "test_endpoint2".to_string(),
        );
        assert!(peer_map.peers.contains_key("test_peer"));

        let active_endpoint = peer_map.remove("test_peer");
        assert!(!peer_map.peers.contains_key("test_peer"));

        assert_eq!(active_endpoint, Some("test_endpoint2".to_string()),);
    }

    // Test that update_peer_id() works correctly
    //  1. Test that an error is returned if the old peer id does not exist
    //  2. Insert test_peer and check it is in self.peers
    //  3. Update test_peer to have the id new_peer
    //  4. Verify that peers contains new_peer and redirects contains a redirect from test_peer
    //     to new_peer
    #[test]
    fn test_update_peer_id() {
        let mut peer_map = PeerMap::new();

        if let Ok(()) = peer_map.update_peer_id("test_peer".to_string(), "new_peer".to_string()) {
            panic!("Should not be able to update peer because old peer does not exist")
        }

        peer_map.insert(
            "test_peer".to_string(),
            "connection_id".to_string(),
            vec!["test_endpoint1".to_string(), "test_endpoint2".to_string()],
            "test_endpoint2".to_string(),
        );
        assert!(peer_map.peers.contains_key("test_peer"));

        peer_map
            .update_peer_id("test_peer".to_string(), "new_peer".to_string())
            .expect("Unable to update peer id");

        assert!(peer_map.peers.contains_key("new_peer"));
        assert!(peer_map.redirects.contains_key("test_peer"));
    }

    // Test that a peer can be updated
    //  1. Check that an error is returned if the peer does not exist
    //  2. Insert test_peer with active endpoint test_endpoint2
    //  3. Update the active enpdoint for test_peer to test_endpoint1 and set the status to
    //     disconnected
    //  4. Check that the peer's metadata now points to test_endpoint1 and the peer is disconnected
    #[test]
    fn test_get_update_active_endpoint() {
        let mut peer_map = PeerMap::new();
        let no_peer_metadata = PeerMetadata {
            id: "test_peer".to_string(),
            connection_id: "connection_id".to_string(),
            endpoints: vec!["test_endpoint1".to_string(), "test_endpoint2".to_string()],
            active_endpoint: "test_endpoint1".to_string(),
            status: PeerStatus::Connected,
        };

        if let Ok(()) = peer_map.update_peer(no_peer_metadata) {
            panic!("Should not have been able to update peer because test_peer does not exist")
        }

        peer_map.insert(
            "test_peer".to_string(),
            "connection_id".to_string(),
            vec!["test_endpoint1".to_string(), "test_endpoint2".to_string()],
            "test_endpoint2".to_string(),
        );
        assert!(peer_map.peers.contains_key("test_peer"));

        let mut peer_metadata = peer_map
            .get_peer_from_endpoint("test_endpoint2")
            .cloned()
            .expect("Unable to retrieve peer metadata with endpoint");

        peer_metadata.active_endpoint = "test_endpoint1".to_string();
        peer_metadata.endpoints.push("new_endpoint".to_string());
        peer_metadata.status = PeerStatus::Disconnected { retry_attempts: 5 };

        peer_map
            .update_peer(peer_metadata)
            .expect("Unable to update endpoint");

        let peer_metadata = peer_map.peers.get("test_peer");
        assert_eq!(
            peer_metadata,
            Some(&PeerMetadata {
                id: "test_peer".to_string(),
                endpoints: vec![
                    "test_endpoint1".to_string(),
                    "test_endpoint2".to_string(),
                    "new_endpoint".to_string()
                ],
                connection_id: "connection_id".to_string(),
                active_endpoint: "test_endpoint1".to_string(),
                status: PeerStatus::Disconnected { retry_attempts: 5 },
            })
        );
    }
}
