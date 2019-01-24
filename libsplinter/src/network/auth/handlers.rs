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

use ::log::{debug, log};
use protobuf::Message;

use crate::channel::Sender;
use crate::network::auth::{AuthorizationAction, AuthorizationManager, AuthorizationState};
use crate::network::dispatch::{
    DispatchError, Dispatcher, Handler, MessageContext,
};
use crate::network::sender::SendRequest;
use crate::protos::authorization::{
    AuthorizationMessage, AuthorizationMessageType, AuthorizedMessage, ConnectRequest,
    ConnectResponse, ConnectResponse_AuthorizationType, TrustRequest,
};
use crate::protos::network::{NetworkMessage, NetworkMessageType};

/// Create a Dispatcher for Authorization messages
///
/// Creates and configures a Dispatcher to handle messages from an AuthorizationMessage envelope.
/// The dispatcher is provided the given network sender for response messages, and the network
/// itself to handle updating identities (or removing connections with authorization failures).
///
/// The identity provided is sent to connections for Trust authorizations.
pub fn create_authorization_dispatcher(
    auth_manager: AuthorizationManager,
    network_sender: Box<dyn Sender<SendRequest>>,
) -> Dispatcher<AuthorizationMessageType> {
    let mut auth_dispatcher = Dispatcher::new(network_sender);

    auth_dispatcher.set_handler(
        AuthorizationMessageType::CONNECT_REQUEST,
        Box::new(ConnectRequestHandler::new(auth_manager.clone())),
    );

    auth_dispatcher.set_handler(
        AuthorizationMessageType::CONNECT_RESPONSE,
        Box::new(ConnectResponseHandler::new(auth_manager.clone())),
    );

    auth_dispatcher.set_handler(
        AuthorizationMessageType::TRUST_REQUEST,
        Box::new(TrustRequestHandler::new(auth_manager.clone())),
    );

    auth_dispatcher
}

/// Handler for the Connect Request Authorization Message Type
struct ConnectRequestHandler {
    auth_manager: AuthorizationManager,
}

impl ConnectRequestHandler {
    fn new(auth_manager: AuthorizationManager) -> Self {
        ConnectRequestHandler { auth_manager }
    }
}

impl Handler<AuthorizationMessageType, ConnectRequest> for ConnectRequestHandler {
    fn handle(
        &self,
        msg: ConnectRequest,
        context: &MessageContext<AuthorizationMessageType>,
        sender: &dyn Sender<SendRequest>,
    ) -> Result<(), DispatchError> {
        match self
            .auth_manager
            .next_state(context.source_peer_id(), AuthorizationAction::Connecting)
        {
            Err(err) => {
                debug!(
                    "Ignoring connect message from peer {} ({}): {}",
                    context.source_peer_id(),
                    msg.get_endpoint(),
                    err
                );
            }
            Ok(AuthorizationState::Connecting) => {
                debug!(
                    "Beginning handshake for peer {} ({})",
                    context.source_peer_id(),
                    msg.get_endpoint()
                );
                let mut response = ConnectResponse::new();
                response.set_accepted_authorization_types(
                    vec![ConnectResponse_AuthorizationType::TRUST].into(),
                );
                sender.send(SendRequest::new(
                    context.source_peer_id().to_string(),
                    wrap_in_network_auth_envelopes(
                        AuthorizationMessageType::CONNECT_RESPONSE,
                        response,
                    )?,
                ))?;
            }
            Ok(next_state) => panic!("Should not have been able to transition to {}", next_state),
        }

        Ok(())
    }
}

/// Handler for the ConnectResponse Authorization Message Type
struct ConnectResponseHandler {
    auth_manager: AuthorizationManager,
}

impl ConnectResponseHandler {
    fn new(auth_manager: AuthorizationManager) -> Self {
        ConnectResponseHandler { auth_manager }
    }
}

impl Handler<AuthorizationMessageType, ConnectResponse> for ConnectResponseHandler {
    fn handle(
        &self,
        msg: ConnectResponse,
        context: &MessageContext<AuthorizationMessageType>,
        sender: &dyn Sender<SendRequest>,
    ) -> Result<(), DispatchError> {
        debug!(
            "Receive connect response from peer {}: {:?}",
            context.source_peer_id(),
            msg
        );
        if msg
            .get_accepted_authorization_types()
            .iter()
            .any(|t| t == &ConnectResponse_AuthorizationType::TRUST)
        {
            let mut trust_request = TrustRequest::new();
            trust_request.set_identity(self.auth_manager.identity.clone());
            sender.send(SendRequest::new(
                context.source_peer_id().to_string(),
                wrap_in_network_auth_envelopes(
                    AuthorizationMessageType::TRUST_REQUEST,
                    trust_request,
                )?,
            ))?;
        }
        Ok(())
    }
}

/// Handler for the TrustRequest Authorization Message Type
struct TrustRequestHandler {
    auth_manager: AuthorizationManager,
}

impl TrustRequestHandler {
    fn new(auth_manager: AuthorizationManager) -> Self {
        TrustRequestHandler { auth_manager }
    }
}

impl Handler<AuthorizationMessageType, TrustRequest> for TrustRequestHandler {
    fn handle(
        &self,
        msg: TrustRequest,
        context: &MessageContext<AuthorizationMessageType>,
        sender: &dyn Sender<SendRequest>,
    ) -> Result<(), DispatchError> {
        match self.auth_manager.next_state(
            context.source_peer_id(),
            AuthorizationAction::TrustIdentifying(msg.get_identity().to_string()),
        ) {
            Err(err) => {
                debug!(
                    "Ignoring trust request message from peer {}: {}",
                    context.source_peer_id(),
                    err
                );
            }
            Ok(AuthorizationState::Authorized) => {
                debug!(
                    "Sending Authorized message to peer {} (formerly {})",
                    msg.get_identity(),
                    context.source_peer_id()
                );
                let auth_msg = AuthorizedMessage::new();
                sender.send(SendRequest::new(
                    msg.get_identity().to_string(),
                    wrap_in_network_auth_envelopes(AuthorizationMessageType::AUTHORIZE, auth_msg)?,
                ))?;
            }
            Ok(next_state) => panic!("Should not have been able to transition to {}", next_state),
        }
        Ok(())
    }
}

fn wrap_in_network_auth_envelopes<M: protobuf::Message>(
    msg_type: AuthorizationMessageType,
    auth_msg: M,
) -> Result<Vec<u8>, DispatchError> {
    let mut auth_msg_env = AuthorizationMessage::new();
    auth_msg_env.set_message_type(msg_type);
    auth_msg_env.set_payload(auth_msg.write_to_bytes()?);

    let mut network_msg = NetworkMessage::new();
    network_msg.set_message_type(NetworkMessageType::AUTHORIZATION);
    network_msg.set_payload(auth_msg_env.write_to_bytes()?);

    network_msg.write_to_bytes().map_err(DispatchError::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    use protobuf::Message;

    use crate::channel::mock::MockSender;
    use crate::mesh::Mesh;
    use crate::network::Network;
    use crate::protos::authorization::{
        AuthorizationMessage, AuthorizedMessage, ConnectRequest, ConnectResponse,
        ConnectResponse_AuthorizationType, TrustRequest,
    };
    use crate::protos::network::{NetworkMessage, NetworkMessageType};
    use crate::transport::inproc::InprocTransport;
    use crate::transport::Transport;

    #[test]
    fn connect_request_dispatch() {
        let (network, peer_id) = create_network_with_initial_temp_peer();

        let auth_mgr = AuthorizationManager::new(network, "mock_identity".into());
        let network_sender = MockSender::default();
        let dispatcher =
            create_authorization_dispatcher(auth_mgr, Box::new(network_sender.clone()));

        let mut msg = ConnectRequest::new();
        msg.set_endpoint("local".into());
        let msg_bytes = msg.write_to_bytes().expect("Unable to serialize message");
        assert_eq!(
            Ok(()),
            dispatcher.dispatch(
                &peer_id,
                &AuthorizationMessageType::CONNECT_REQUEST,
                msg_bytes
            )
        );

        let send_request = network_sender
            .clear()
            .pop()
            .expect("A message should have been sent");

        let connect_res_msg: ConnectResponse = expect_auth_message(
            AuthorizationMessageType::CONNECT_RESPONSE,
            send_request.payload(),
        );
        assert_eq!(
            vec![ConnectResponse_AuthorizationType::TRUST],
            connect_res_msg.get_accepted_authorization_types().to_vec()
        );
    }

    // Test that a connect response is properly dispatched
    // There should be a trust request sent to the responding peer
    #[test]
    fn connect_response_dispatch() {
        let (network, peer_id) = create_network_with_initial_temp_peer();

        let auth_mgr = AuthorizationManager::new(network, "mock_identity".into());
        let network_sender = MockSender::default();
        let dispatcher =
            create_authorization_dispatcher(auth_mgr, Box::new(network_sender.clone()));

        let mut msg = ConnectResponse::new();
        msg.set_accepted_authorization_types(vec![ConnectResponse_AuthorizationType::TRUST].into());
        let msg_bytes = msg.write_to_bytes().expect("Unable to serialize message");
        assert_eq!(
            Ok(()),
            dispatcher.dispatch(
                &peer_id,
                &AuthorizationMessageType::CONNECT_RESPONSE,
                msg_bytes
            )
        );

        let send_request = network_sender
            .clear()
            .pop()
            .expect("A message should have been sent");

        let trust_req: TrustRequest = expect_auth_message(
            AuthorizationMessageType::TRUST_REQUEST,
            send_request.payload(),
        );
        assert_eq!("mock_identity", trust_req.get_identity());
    }

    // Test that the node can handle a trust response
    #[test]
    fn trust_request_dispatch() {
        let (network, peer_id) = create_network_with_initial_temp_peer();

        let auth_mgr = AuthorizationManager::new(network, "mock_identity".into());
        let network_sender = MockSender::default();
        let dispatcher =
            create_authorization_dispatcher(auth_mgr, Box::new(network_sender.clone()));

        // Begin the connection process, otherwise, the response will fail
        let mut msg = ConnectRequest::new();
        msg.set_endpoint("local".into());
        let msg_bytes = msg.write_to_bytes().expect("Unable to serialize message");
        assert_eq!(
            Ok(()),
            dispatcher.dispatch(
                &peer_id,
                &AuthorizationMessageType::CONNECT_REQUEST,
                msg_bytes
            )
        );

        let send_request = network_sender
            .clear()
            .pop()
            .expect("A message should have been sent");
        let _connect_res_msg: ConnectResponse = expect_auth_message(
            AuthorizationMessageType::CONNECT_RESPONSE,
            send_request.payload(),
        );

        let mut trust_req = TrustRequest::new();
        trust_req.set_identity("my_identity".into());
        let msg_bytes = trust_req
            .write_to_bytes()
            .expect("Unable to serialize message");
        assert_eq!(
            Ok(()),
            dispatcher.dispatch(
                &peer_id,
                &AuthorizationMessageType::TRUST_REQUEST,
                msg_bytes
            )
        );
        let send_request = network_sender
            .clear()
            .pop()
            .expect("A message should have been sent");

        let _auth_msg: AuthorizedMessage =
            expect_auth_message(AuthorizationMessageType::AUTHORIZE, send_request.payload());
    }

    fn expect_auth_message<M: protobuf::Message>(
        message_type: AuthorizationMessageType,
        msg_bytes: &[u8],
    ) -> M {
        let network_msg: NetworkMessage =
            protobuf::parse_from_bytes(msg_bytes).expect("Unable to parse network message");
        assert_eq!(NetworkMessageType::AUTHORIZATION, network_msg.message_type);

        let auth_msg: AuthorizationMessage = protobuf::parse_from_bytes(network_msg.get_payload())
            .expect("Unable to parse auth message");

        assert_eq!(message_type, auth_msg.message_type);

        match protobuf::parse_from_bytes(auth_msg.get_payload()) {
            Ok(msg) => msg,
            Err(err) => panic!(
                "unable to parse message for type {:?}: {:?}",
                message_type, err
            ),
        }
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
