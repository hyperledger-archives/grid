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

use std::sync::{Arc, RwLock};

use crate::channel::Sender;
use crate::circuit::handlers::create_message;
use crate::circuit::SplinterState;
use crate::network::dispatch::{DispatchError, Handler, MessageContext};
use crate::network::sender::SendRequest;
use crate::protos::circuit::{
    AdminDirectMessage, CircuitError, CircuitError_Error, CircuitMessageType,
};
use crate::rwlock_read_unwrap;
use protobuf::Message;

const ADMIN_SERVICE_ID_PREFIX: &str = "admin::";

// Implements a handler that handles AdminDirectMessage
pub struct AdminDirectMessageHandler {
    node_id: String,
    state: Arc<RwLock<SplinterState>>,
}

impl Handler<CircuitMessageType, AdminDirectMessage> for AdminDirectMessageHandler {
    fn handle(
        &self,
        msg: AdminDirectMessage,
        context: &MessageContext<CircuitMessageType>,
        sender: &dyn Sender<SendRequest>,
    ) -> Result<(), DispatchError> {
        debug!(
            "Handle Admin Direct Message {} on {} ({} => {}) [{} byte{}]",
            msg.get_correlation_id(),
            msg.get_circuit(),
            msg.get_sender(),
            msg.get_recipient(),
            msg.get_payload().len(),
            if msg.get_payload().len() == 1 {
                ""
            } else {
                "s"
            }
        );

        // msg bytes will either be message bytes of a direct message or an error message
        // the msg_recipient is either the service/node id to send the message to or is the
        // peer_id to send back the error message
        let (msg_bytes, msg_recipient) = self.create_response(msg, context)?;
        // either forward the direct message or send back an error message.
        let send_request = SendRequest::new(msg_recipient.to_string(), msg_bytes);
        sender.send(send_request)?;
        Ok(())
    }
}

impl AdminDirectMessageHandler {
    pub fn new(node_id: String, state: Arc<RwLock<SplinterState>>) -> Self {
        Self { node_id, state }
    }

    fn create_response(
        &self,
        msg: AdminDirectMessage,
        context: &MessageContext<CircuitMessageType>,
    ) -> Result<(Vec<u8>, String), DispatchError> {
        let circuit_name = msg.get_circuit();
        let msg_sender = msg.get_sender();
        let recipient = msg.get_recipient();

        if !is_admin_service_id(msg_sender) {
            let err_msg_bytes = create_circuit_error_msg(
                &msg,
                CircuitError_Error::ERROR_SENDER_NOT_IN_CIRCUIT_ROSTER,
                format!(
                    "Sender is not allowed to send admin messages: {}",
                    msg_sender
                ),
            )?;
            return Ok((
                create_message(err_msg_bytes, CircuitMessageType::CIRCUIT_ERROR_MESSAGE)?,
                context.source_peer_id().into(),
            ));
        }

        if !is_admin_service_id(recipient) {
            let err_msg_bytes = create_circuit_error_msg(
                &msg,
                CircuitError_Error::ERROR_RECIPIENT_NOT_IN_CIRCUIT_ROSTER,
                format!(
                    "Recipient is not allowed to receive admin messages: {}",
                    recipient
                ),
            )?;
            return Ok((
                create_message(err_msg_bytes, CircuitMessageType::CIRCUIT_ERROR_MESSAGE)?,
                context.source_peer_id().into(),
            ));
        }

        // Get read lock on state
        let state = rwlock_read_unwrap!(self.state);

        // msg bytes will either be message bytes of a direct message or an error message
        // the msg_recipient is either the service/node id to send the message to or is the
        // peer_id to send back the error message
        let response = if state.circuit(circuit_name).is_some() {
            let node_id = &recipient[ADMIN_SERVICE_ID_PREFIX.len()..];
            // If the service is on this node send message to the service, otherwise
            // send the message to the node the service is connected to
            let target_node = if node_id != self.node_id {
                node_id
            } else {
                // The internal admin service is at the node id with an identical name
                recipient
            };

            let msg_bytes = context.message_bytes().to_vec();
            let network_msg_bytes =
                create_message(msg_bytes, CircuitMessageType::ADMIN_DIRECT_MESSAGE)?;
            (network_msg_bytes, target_node.to_string())
        } else {
            // if the circuit does not exist, send circuit error
            let msg_bytes = create_circuit_error_msg(
                &msg,
                CircuitError_Error::ERROR_CIRCUIT_DOES_NOT_EXIST,
                format!("Circuit does not exist: {}", circuit_name),
            )?;

            let network_msg_bytes =
                create_message(msg_bytes, CircuitMessageType::CIRCUIT_ERROR_MESSAGE)?;
            (network_msg_bytes, context.source_peer_id().to_string())
        };
        Ok(response)
    }
}

fn create_circuit_error_msg(
    msg: &AdminDirectMessage,
    error_type: CircuitError_Error,
    error_msg: String,
) -> Result<Vec<u8>, DispatchError> {
    let mut error_message = CircuitError::new();
    error_message.set_correlation_id(msg.get_correlation_id().into());
    error_message.set_service_id(msg.get_sender().into());
    error_message.set_circuit_name(msg.get_circuit().into());
    error_message.set_error(error_type);
    error_message.set_error_message(error_msg);

    error_message.write_to_bytes().map_err(DispatchError::from)
}

fn is_admin_service_id(service_id: &str) -> bool {
    service_id.starts_with(ADMIN_SERVICE_ID_PREFIX)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::{Arc, Mutex};

    use crate::channel::{SendError, Sender};
    use crate::circuit::directory::CircuitDirectory;
    use crate::circuit::{AuthorizationType, Circuit, DurabilityType, PersistenceType, RouteType};
    use crate::network::dispatch::Dispatcher;
    use crate::protos::circuit::CircuitMessage;
    use crate::protos::network::NetworkMessage;

    /// Send a message from a non-admin service. Expect that the message is ignored and an error
    /// is returned to sender.
    #[test]
    fn test_ignore_non_admin_sender() {
        // Set up dispatcher and mock sender
        let sender = Box::new(MockNetworkSender::default());
        let mut dispatcher = Dispatcher::new(sender.box_clone());

        // Add circuit and service to splinter state
        let circuit = Circuit::builder()
            .with_id("alpha".into())
            .with_auth(AuthorizationType::Trust)
            .with_members(vec!["1234".into(), "5678".into()])
            .with_roster(vec!["abc".into(), "def".into()])
            .with_persistence(PersistenceType::Any)
            .with_durability(DurabilityType::NoDurabilty)
            .with_routes(RouteType::Any)
            .with_circuit_management_type("admin_test_app".into())
            .build()
            .expect("Should have built a correct circuit");

        let mut circuit_directory = CircuitDirectory::new();
        circuit_directory.add_circuit("alpha".to_string(), circuit);

        let state = Arc::new(RwLock::new(SplinterState::new(
            "memory".to_string(),
            circuit_directory,
        )));

        let handler = AdminDirectMessageHandler::new("1234".into(), state);
        dispatcher.set_handler(CircuitMessageType::ADMIN_DIRECT_MESSAGE, Box::new(handler));

        let mut direct_message = AdminDirectMessage::new();
        direct_message.set_circuit("admin".into());
        direct_message.set_sender("abc".into());
        direct_message.set_recipient("admin::1234".into());
        direct_message.set_payload(b"test".to_vec());
        direct_message.set_correlation_id("random_corr_id".into());
        let direct_bytes = direct_message.write_to_bytes().unwrap();

        assert_eq!(
            Ok(()),
            dispatcher.dispatch(
                "5678",
                &CircuitMessageType::ADMIN_DIRECT_MESSAGE,
                direct_bytes
            )
        );

        let send_request = sender.sent().lock().unwrap().get(0).unwrap().clone();

        assert_send_request(
            send_request,
            "5678",
            CircuitMessageType::CIRCUIT_ERROR_MESSAGE,
            |error_msg: CircuitError| {
                assert_eq!(error_msg.get_service_id(), "abc");
                assert_eq!(
                    error_msg.get_error(),
                    CircuitError_Error::ERROR_SENDER_NOT_IN_CIRCUIT_ROSTER
                );
                assert_eq!(error_msg.get_correlation_id(), "random_corr_id");
            },
        )
    }

    /// Send a message to a non-admin service. Expect that the message is ignored and an error is
    /// returned to sender.
    #[test]
    fn test_ignore_non_admin_recipient() {
        // Set up dispatcher and mock sender
        let sender = Box::new(MockNetworkSender::default());
        let mut dispatcher = Dispatcher::new(sender.box_clone());

        // Add circuit and service to splinter state
        let circuit = Circuit::builder()
            .with_id("alpha".into())
            .with_auth(AuthorizationType::Trust)
            .with_members(vec!["1234".into(), "5678".into()])
            .with_roster(vec!["abc".into(), "def".into()])
            .with_persistence(PersistenceType::Any)
            .with_durability(DurabilityType::NoDurabilty)
            .with_routes(RouteType::Any)
            .with_circuit_management_type("admin_test_app".into())
            .build()
            .expect("Should have built a correct circuit");

        let mut circuit_directory = CircuitDirectory::new();
        circuit_directory.add_circuit("alpha".to_string(), circuit);

        let state = Arc::new(RwLock::new(SplinterState::new(
            "memory".to_string(),
            circuit_directory,
        )));

        let handler = AdminDirectMessageHandler::new("1234".into(), state);
        dispatcher.set_handler(CircuitMessageType::ADMIN_DIRECT_MESSAGE, Box::new(handler));

        let mut direct_message = AdminDirectMessage::new();
        direct_message.set_circuit("admin".into());
        direct_message.set_sender("admin::5678".into());
        direct_message.set_recipient("def".into());
        direct_message.set_payload(b"test".to_vec());
        direct_message.set_correlation_id("random_corr_id".into());
        let direct_bytes = direct_message.write_to_bytes().unwrap();

        assert_eq!(
            Ok(()),
            dispatcher.dispatch(
                "5678",
                &CircuitMessageType::ADMIN_DIRECT_MESSAGE,
                direct_bytes
            )
        );

        let send_request = sender.sent().lock().unwrap().get(0).unwrap().clone();

        assert_send_request(
            send_request,
            "5678",
            CircuitMessageType::CIRCUIT_ERROR_MESSAGE,
            |error_msg: CircuitError| {
                assert_eq!(error_msg.get_service_id(), "admin::5678");
                assert_eq!(
                    error_msg.get_error(),
                    CircuitError_Error::ERROR_RECIPIENT_NOT_IN_CIRCUIT_ROSTER,
                );
                assert_eq!(error_msg.get_correlation_id(), "random_corr_id");
            },
        )
    }

    /// Send a message to an admin service via the standard circuit. Expect that the message is
    /// sent to the current node's target admin service.
    #[test]
    fn test_send_admin_direct_message_via_standard_circuit() {
        // Set up dispatcher and mock sender
        let sender = Box::new(MockNetworkSender::default());
        let mut dispatcher = Dispatcher::new(sender.box_clone());

        // Add circuit and service to splinter state
        let circuit = Circuit::builder()
            .with_id("alpha".into())
            .with_auth(AuthorizationType::Trust)
            .with_members(vec!["1234".into(), "5678".into()])
            .with_roster(vec!["abc".into(), "def".into()])
            .with_persistence(PersistenceType::Any)
            .with_durability(DurabilityType::NoDurabilty)
            .with_routes(RouteType::Any)
            .with_circuit_management_type("admin_test_app".into())
            .build()
            .expect("Should have built a correct circuit");

        let mut circuit_directory = CircuitDirectory::new();
        circuit_directory.add_circuit("alpha".to_string(), circuit);

        let state = Arc::new(RwLock::new(SplinterState::new(
            "memory".to_string(),
            circuit_directory,
        )));

        let handler = AdminDirectMessageHandler::new("1234".into(), state);
        dispatcher.set_handler(CircuitMessageType::ADMIN_DIRECT_MESSAGE, Box::new(handler));

        let mut direct_message = AdminDirectMessage::new();
        direct_message.set_circuit("alpha".into());
        direct_message.set_sender("admin::1234".into());
        direct_message.set_recipient("admin::5678".into());
        direct_message.set_payload(b"test".to_vec());
        direct_message.set_correlation_id("random_corr_id".into());
        let direct_bytes = direct_message.write_to_bytes().unwrap();

        assert_eq!(
            Ok(()),
            dispatcher.dispatch(
                "1234",
                &CircuitMessageType::ADMIN_DIRECT_MESSAGE,
                direct_bytes
            )
        );

        let send_request = sender.sent().lock().unwrap().get(0).unwrap().clone();

        assert_send_request(
            send_request,
            "5678",
            CircuitMessageType::ADMIN_DIRECT_MESSAGE,
            |error_msg: AdminDirectMessage| {
                assert_eq!(error_msg.get_circuit(), "alpha");
                assert_eq!(error_msg.get_sender(), "admin::1234");
                assert_eq!(error_msg.get_recipient(), "admin::5678");
                assert_eq!(error_msg.get_payload(), b"test");
                assert_eq!(error_msg.get_correlation_id(), "random_corr_id");
            },
        )
    }

    /// Send a message to an admin service via the admin circuit. Expect that the message is
    /// sent to the appropriate node that hosts the target admin service.
    #[test]
    fn test_send_admin_direct_message_via_admin_circuit() {
        // Set up dispatcher and mock sender
        let sender = Box::new(MockNetworkSender::default());
        let mut dispatcher = Dispatcher::new(sender.box_clone());

        let circuit_directory = CircuitDirectory::new();

        let state = Arc::new(RwLock::new(SplinterState::new(
            "memory".to_string(),
            circuit_directory,
        )));

        let handler = AdminDirectMessageHandler::new("1234".into(), state);
        dispatcher.set_handler(CircuitMessageType::ADMIN_DIRECT_MESSAGE, Box::new(handler));

        let mut direct_message = AdminDirectMessage::new();
        direct_message.set_circuit("admin".into());
        direct_message.set_sender("admin::1234".into());
        direct_message.set_recipient("admin::5678".into());
        direct_message.set_payload(b"test".to_vec());
        direct_message.set_correlation_id("random_corr_id".into());
        let direct_bytes = direct_message.write_to_bytes().unwrap();

        assert_eq!(
            Ok(()),
            dispatcher.dispatch(
                "1234",
                &CircuitMessageType::ADMIN_DIRECT_MESSAGE,
                direct_bytes
            )
        );

        let send_request = sender.sent().lock().unwrap().get(0).unwrap().clone();

        assert_send_request(
            send_request,
            "5678",
            CircuitMessageType::ADMIN_DIRECT_MESSAGE,
            |error_msg: AdminDirectMessage| {
                assert_eq!(error_msg.get_circuit(), "admin");
                assert_eq!(error_msg.get_sender(), "admin::1234");
                assert_eq!(error_msg.get_recipient(), "admin::5678");
                assert_eq!(error_msg.get_payload(), b"test");
                assert_eq!(error_msg.get_correlation_id(), "random_corr_id");
            },
        )
    }

    /// Send a message to an admin service via the standard circuit.  Expect that the message is
    /// sent to the current node's target admin service.
    #[test]
    fn test_send_admin_direct_message_via_admin_circuit_to_local_service() {
        // Set up dispatcher and mock sender
        let sender = Box::new(MockNetworkSender::default());
        let mut dispatcher = Dispatcher::new(sender.box_clone());

        let circuit_directory = CircuitDirectory::new();

        let state = Arc::new(RwLock::new(SplinterState::new(
            "memory".to_string(),
            circuit_directory,
        )));

        let handler = AdminDirectMessageHandler::new("1234".into(), state);
        dispatcher.set_handler(CircuitMessageType::ADMIN_DIRECT_MESSAGE, Box::new(handler));

        let mut direct_message = AdminDirectMessage::new();
        direct_message.set_circuit("admin".into());
        direct_message.set_sender("admin::5678".into());
        direct_message.set_recipient("admin::1234".into());
        direct_message.set_payload(b"test".to_vec());
        direct_message.set_correlation_id("random_corr_id".into());
        let direct_bytes = direct_message.write_to_bytes().unwrap();

        assert_eq!(
            Ok(()),
            dispatcher.dispatch(
                "1234",
                &CircuitMessageType::ADMIN_DIRECT_MESSAGE,
                direct_bytes
            )
        );

        let send_request = sender.sent().lock().unwrap().get(0).unwrap().clone();

        assert_send_request(
            send_request,
            "admin::1234",
            CircuitMessageType::ADMIN_DIRECT_MESSAGE,
            |error_msg: AdminDirectMessage| {
                assert_eq!(error_msg.get_circuit(), "admin");
                assert_eq!(error_msg.get_sender(), "admin::5678");
                assert_eq!(error_msg.get_recipient(), "admin::1234");
                assert_eq!(error_msg.get_payload(), b"test");
                assert_eq!(error_msg.get_correlation_id(), "random_corr_id");
            },
        )
    }

    fn assert_send_request<M: protobuf::Message, F: Fn(M)>(
        send_request: SendRequest,
        expected_recipient: &str,
        expected_circuit_msg_type: CircuitMessageType,
        detail_assertions: F,
    ) {
        assert_eq!(expected_recipient, send_request.recipient());

        let network_msg: NetworkMessage =
            protobuf::parse_from_bytes(send_request.payload()).unwrap();
        let circuit_msg: CircuitMessage =
            protobuf::parse_from_bytes(network_msg.get_payload()).unwrap();
        assert_eq!(expected_circuit_msg_type, circuit_msg.get_message_type(),);
        let circuit_msg: M = protobuf::parse_from_bytes(circuit_msg.get_payload()).unwrap();

        detail_assertions(circuit_msg);
    }

    #[derive(Default)]
    struct MockNetworkSender {
        sent: Arc<Mutex<Vec<SendRequest>>>,
    }

    impl MockNetworkSender {
        pub fn sent(&self) -> &Arc<Mutex<Vec<SendRequest>>> {
            &self.sent
        }
    }

    impl Sender<SendRequest> for MockNetworkSender {
        fn send(&self, message: SendRequest) -> Result<(), SendError> {
            self.sent.lock().unwrap().push(message);
            Ok(())
        }

        fn box_clone(&self) -> Box<dyn Sender<SendRequest>> {
            Box::new(MockNetworkSender {
                sent: self.sent.clone(),
            })
        }
    }
}
