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

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, RwLock};

use crate::network::Network;


/// The states of a connection during authorization.
#[derive(PartialEq, Debug, Clone)]
enum AuthorizationState {
    Unknown,
    Connecting,
    Authorized,
    Unauthorized,
}

impl fmt::Display for AuthorizationState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            AuthorizationState::Unknown => "Unknown",
            AuthorizationState::Connecting => "Connecting",
            AuthorizationState::Authorized => "Authorized",
            AuthorizationState::Unauthorized => "Unauthorized",
        })
    }
}

type Identity = String;

/// The state transitions that can be applied on an connection during authorization.
#[derive(PartialEq, Debug)]
enum AuthorizationAction {
    Connecting,
    TrustIdentifying(Identity),
    Unauthorizing,
}

impl fmt::Display for AuthorizationAction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            AuthorizationAction::Connecting => "Connecting",
            AuthorizationAction::TrustIdentifying(_) => "TrustIdentifying",
            AuthorizationAction::Unauthorizing => "Unauthorizing",
        })
    }
}

/// The errors that may occur for a connection during authorization.
#[derive(PartialEq, Debug)]
enum AuthorizationActionError {
    AlreadyConnecting,
    InvalidMessageOrder(AuthorizationState, AuthorizationAction),
    ConnectionLost,
}

impl fmt::Display for AuthorizationActionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AuthorizationActionError::AlreadyConnecting => {
                f.write_str("Already attempting to connect.")
            }
            AuthorizationActionError::InvalidMessageOrder(start, action) => {
                write!(f, "Attempting to transition from {} via {}.", start, action)
            }
            AuthorizationActionError::ConnectionLost => {
                f.write_str("Connection lost while authorizing peer")
            }
        }
    }
}

/// Manages authorization states for connections on a network.
#[derive(Clone)]
pub struct AuthorizationManager {
    states: Arc<RwLock<HashMap<String, AuthorizationState>>>,
    network: Network,
    identity: Identity,
}

impl AuthorizationManager {
    /// Constructs an AuthorizationManager
    pub fn new(network: Network, identity: Identity) -> Self {
        AuthorizationManager {
            states: Default::default(),
            network,
            identity,
        }
    }

    /// Indicated whether or not a peer is authorized.
    pub fn is_authorized(&self, peer_id: &str) -> bool {
        let states = rwlock_read_unwrap!(self.states);
        if let Some(state) = states.get(peer_id) {
            state == &AuthorizationState::Authorized
        } else {
            false
        }
    }

    /// Transitions from one authorization state to another
    ///
    /// Errors
    ///
    /// The errors are error messages that should be returned on the appropriate message
    fn next_state(
        &self,
        peer_id: &str,
        action: AuthorizationAction,
    ) -> Result<AuthorizationState, AuthorizationActionError> {
        let mut states = rwlock_write_unwrap!(self.states);

        let cur_state = states.get(peer_id).unwrap_or(&AuthorizationState::Unknown);
        match cur_state {
            &AuthorizationState::Unknown => match action {
                AuthorizationAction::Connecting => {
                    // Here the decision for Challenges will be made.
                    states.insert(peer_id.to_string(), AuthorizationState::Connecting);
                    Ok(AuthorizationState::Connecting)
                }
                AuthorizationAction::Unauthorizing => {
                    self.network
                        .remove_connection(&peer_id.to_string())
                        .map_err(|_| AuthorizationActionError::ConnectionLost)?;
                    Ok(AuthorizationState::Unauthorized)
                }
                _ => Err(AuthorizationActionError::InvalidMessageOrder(
                    AuthorizationState::Unknown,
                    action,
                )),
            },
            &AuthorizationState::Connecting => match action {
                AuthorizationAction::Connecting => Err(AuthorizationActionError::AlreadyConnecting),
                AuthorizationAction::TrustIdentifying(new_peer_id) => {
                    // Verify pub key allowed
                    states.remove(peer_id);
                    self.network
                        .update_peer_id(peer_id.to_string(), new_peer_id.clone())
                        .map_err(|_| AuthorizationActionError::ConnectionLost)?;
                    states.insert(new_peer_id, AuthorizationState::Authorized);
                    Ok(AuthorizationState::Authorized)
                }
                AuthorizationAction::Unauthorizing => {
                    states.remove(peer_id);
                    self.network
                        .remove_connection(&peer_id.to_string())
                        .map_err(|_| AuthorizationActionError::ConnectionLost)?;
                    Ok(AuthorizationState::Unauthorized)
                }
            },
            &AuthorizationState::Authorized => match action {
                AuthorizationAction::Unauthorizing => {
                    states.remove(peer_id);
                    self.network
                        .remove_connection(&peer_id.to_string())
                        .map_err(|_| AuthorizationActionError::ConnectionLost)?;
                    Ok(AuthorizationState::Unauthorized)
                }
                _ => Err(AuthorizationActionError::InvalidMessageOrder(
                    AuthorizationState::Authorized,
                    action,
                )),
            },
            _ => Err(AuthorizationActionError::InvalidMessageOrder(
                cur_state.clone(),
                action,
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::mesh::Mesh;
    use crate::network::Network;
    use crate::transport::inproc::InprocTransport;
    use crate::transport::Transport;

    /// This test runs through the trust authorization state machine happy path. It traverses
    /// through each state, Unknown -> Connecting -> Authorized and verifies that the response
    /// for is_authorized is correct at each stage.
    #[test]
    fn trust_state_machine_valid() {
        let (network, peer_id) = create_network_with_initial_temp_peer();

        let auth_manager = AuthorizationManager::new(network.clone(), "mock_identity".into());

        assert!(!auth_manager.is_authorized(&peer_id));

        assert_eq!(
            Ok(AuthorizationState::Connecting),
            auth_manager.next_state(&peer_id, AuthorizationAction::Connecting)
        );

        assert!(!auth_manager.is_authorized(&peer_id));

        // verify that it cannot be connected again.
        assert_eq!(
            Err(AuthorizationActionError::AlreadyConnecting),
            auth_manager.next_state(&peer_id, AuthorizationAction::Connecting)
        );
        assert!(!auth_manager.is_authorized(&peer_id));

        // Supply the TrustIdentifying action and verify that it is authorized
        let new_peer_id = "abcd".to_string();
        assert_eq!(
            Ok(AuthorizationState::Authorized),
            auth_manager.next_state(
                &peer_id,
                AuthorizationAction::TrustIdentifying(new_peer_id.clone())
            )
        );
        // we no longer have the temp id
        assert!(!auth_manager.is_authorized(&peer_id));
        // but we now have the new identified peer
        assert!(auth_manager.is_authorized(&new_peer_id));
        assert_eq!(vec![new_peer_id.clone()], network.peer_ids());
    }

    /// This test begins a connection, and then unauthorizes the peer.  Verify that the auth
    /// manager reports the correct value for is_authorized, and that the peer is removed.
    #[test]
    fn trust_state_machine_unauthorize_while_connecting() {
        let (network, peer_id) = create_network_with_initial_temp_peer();

        let auth_manager = AuthorizationManager::new(network.clone(), "mock_identity".into());

        assert!(!auth_manager.is_authorized(&peer_id));
        assert_eq!(
            Ok(AuthorizationState::Connecting),
            auth_manager.next_state(&peer_id, AuthorizationAction::Connecting)
        );

        assert_eq!(
            Ok(AuthorizationState::Unauthorized),
            auth_manager.next_state(&peer_id, AuthorizationAction::Unauthorizing)
        );

        assert!(!auth_manager.is_authorized(&peer_id));
        let empty_vec: Vec<String> = Vec::with_capacity(0);
        assert_eq!(empty_vec, network.peer_ids());
    }

    /// This test begins a connection, trusts it, and then unauthorizes the peer.  Verify that
    /// the auth manager reports the correct values for is_authorized, and that the peer is removed.
    #[test]
    fn trust_state_machine_unauthorize_when_authorized() {
        let (network, peer_id) = create_network_with_initial_temp_peer();

        let auth_manager = AuthorizationManager::new(network.clone(), "mock_identity".into());

        assert!(!auth_manager.is_authorized(&peer_id));
        assert_eq!(
            Ok(AuthorizationState::Connecting),
            auth_manager.next_state(&peer_id, AuthorizationAction::Connecting)
        );
        let new_peer_id = "abcd".to_string();
        assert_eq!(
            Ok(AuthorizationState::Authorized),
            auth_manager.next_state(
                &peer_id,
                AuthorizationAction::TrustIdentifying(new_peer_id.clone())
            )
        );
        assert!(!auth_manager.is_authorized(&peer_id));
        assert!(auth_manager.is_authorized(&new_peer_id));
        assert_eq!(vec![new_peer_id.clone()], network.peer_ids());

        assert_eq!(
            Ok(AuthorizationState::Unauthorized),
            auth_manager.next_state(&new_peer_id, AuthorizationAction::Unauthorizing)
        );

        assert!(!auth_manager.is_authorized(&new_peer_id));
        let empty_vec: Vec<String> = Vec::with_capacity(0);
        assert_eq!(empty_vec, network.peer_ids());
    }

    fn create_network_with_initial_temp_peer() -> (Network, String) {
        let network = Network::new(Mesh::new(5, 5));

        let mut transport = InprocTransport::default();

        let mut _listener = transport
            .listen("local")
            .expect("Unable to create the listener");
        let connection = transport
            .connect("local")
            .expect("Unable to create the connection");

        network
            .add_connection(connection)
            .expect("Unable to add connection to network");

        // We only have one peer, so we can grab this id as the temp id.
        let peer_id = network.peer_ids()[0].clone();

        (network, peer_id)
    }
}
