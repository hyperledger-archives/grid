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

use crate::channel::Sender;
use crate::circuit::handlers::create_message;
use crate::circuit::{ServiceId, SplinterState};
use crate::network::dispatch::{DispatchError, Handler, MessageContext};
use crate::network::sender::SendRequest;
use crate::protos::circuit::{CircuitError, CircuitMessageType};
use crate::rwlock_read_unwrap;

use std::sync::{Arc, RwLock};

// Implements a handler that handles CircuitError messages
pub struct CircuitErrorHandler {
    node_id: String,
    state: Arc<RwLock<SplinterState>>,
}

// In most cases the error message will be returned directly back to service, but in the case
// where it is returned back to a different node, this node will do its best effort to
// return it back to the service or node who sent the original message.
impl Handler<CircuitMessageType, CircuitError> for CircuitErrorHandler {
    fn handle(
        &self,
        msg: CircuitError,
        context: &MessageContext<CircuitMessageType>,
        sender: &dyn Sender<SendRequest>,
    ) -> Result<(), DispatchError> {
        debug!("Handle Circuit Error Message {:?}", msg);
        let circuit_name = msg.get_circuit_name();
        let service_id = msg.get_service_id();
        let unique_id = ServiceId::new(circuit_name.to_string(), service_id.to_string());

        // Get read lock on state
        let state = rwlock_read_unwrap!(self.state);

        // check if the msg_sender is in the service directory
        let recipient = match state.service_directory().get(&unique_id) {
            Some(service) => {
                let node_id = service.node().id();
                if node_id == self.node_id {
                    // If the service is connected to this node, send the error to the service
                    match service.peer_id() {
                        Some(peer_id) => peer_id,
                        None => {
                            // This should never happen, as a peer id will always
                            // be set on a service that is connected to the local node.
                            warn!("No peer id for service:{} ", service.service_id());
                            return Ok(());
                        }
                    }
                } else {
                    // If the service is connected to another node, send the error to that node
                    service.node().id()
                }
            }
            None => {
                // If the service is not in the service directory, the nodes does not know who to
                // forward this message to, so the message is dropped
                warn!(
                    "Original message sender is not connected: {}, cannot send Circuit Error",
                    service_id
                );
                return Ok(());
            }
        };

        let network_msg_bytes = create_message(
            context.message_bytes().to_vec(),
            CircuitMessageType::CIRCUIT_ERROR_MESSAGE,
        )?;

        // forward error message
        let send_request = SendRequest::new(recipient.to_string(), network_msg_bytes);
        sender.send(send_request)?;
        Ok(())
    }
}

impl CircuitErrorHandler {
    pub fn new(node_id: String, state: Arc<RwLock<SplinterState>>) -> Self {
        CircuitErrorHandler { node_id, state }
    }
}

#[cfg(test)]
mod tests {
    use protobuf::Message;

    use std::sync::Arc;

    use super::*;
    use crate::channel::mock::MockSender;
    use crate::channel::Sender;
    use crate::circuit::directory::CircuitDirectory;
    use crate::circuit::service::{Service, SplinterNode};
    use crate::circuit::Circuit;
    use crate::network::dispatch::Dispatcher;
    use crate::protos::circuit::{CircuitError_Error, CircuitMessage};
    use crate::protos::network::NetworkMessage;

    // Test that if an error message recieved is meant for the service connected to a node,
    // the error message is sent to the service
    #[test]
    fn test_circuit_error_handler_service() {
        // Set up disptacher and mock sender
        let sender = Box::new(MockSender::default());
        let mut dispatcher = Dispatcher::new(sender.box_clone());

        // Add circuit and service to splinter state
        let circuit = Circuit::new(
            "alpha".into(),
            "trust".into(),
            vec!["123".into()],
            vec!["abc".into(), "def".into()],
            "any".into(),
            "none".into(),
            "require_direct".into(),
        );

        let mut circuit_directory = CircuitDirectory::new();
        circuit_directory.add_circuit("alpha".to_string(), circuit);

        let state = Arc::new(RwLock::new(SplinterState::new(
            "memory".to_string(),
            circuit_directory,
        )));

        let node_123 = SplinterNode::new("123".to_string(), vec!["123.0.0.1:0".to_string()]);
        let node_345 = SplinterNode::new("345".to_string(), vec!["123.0.0.1:1".to_string()]);

        let service_abc =
            Service::new("abc".to_string(), Some("abc_network".to_string()), node_123);
        let service_def =
            Service::new("def".to_string(), Some("def_network".to_string()), node_345);

        let abc_id = ServiceId::new("alpha".into(), "abc".into());
        let def_id = ServiceId::new("alpha".into(), "def".into());
        state.write().unwrap().add_service(abc_id, service_abc);
        state.write().unwrap().add_service(def_id, service_def);

        // Add circuit error handler to the the dispatcher
        let handler = CircuitErrorHandler::new("123".to_string(), state);
        dispatcher.set_handler(CircuitMessageType::CIRCUIT_ERROR_MESSAGE, Box::new(handler));

        // Create the error message
        let mut circuit_error = CircuitError::new();
        circuit_error.set_service_id("abc".into());
        circuit_error.set_circuit_name("alpha".into());
        circuit_error.set_correlation_id("1234".into());
        circuit_error.set_error(CircuitError_Error::ERROR_RECIPIENT_NOT_IN_DIRECTORY);
        circuit_error.set_error_message("TEST".into());
        let error_bytes = circuit_error.write_to_bytes().unwrap();

        // dispatch the error message
        dispatcher
            .dispatch(
                "345",
                &CircuitMessageType::CIRCUIT_ERROR_MESSAGE,
                error_bytes.clone(),
            )
            .unwrap();

        // verify that the error message was sent to the abc service
        let send_request = sender.sent().get(0).unwrap().clone();

        assert_eq!(send_request.recipient(), "abc_network");

        let network_msg: NetworkMessage =
            protobuf::parse_from_bytes(send_request.payload()).unwrap();
        let circuit_msg: CircuitMessage =
            protobuf::parse_from_bytes(network_msg.get_payload()).unwrap();
        let circuit_error: CircuitError =
            protobuf::parse_from_bytes(circuit_msg.get_payload()).unwrap();

        assert_eq!(
            circuit_msg.get_message_type(),
            CircuitMessageType::CIRCUIT_ERROR_MESSAGE
        );

        assert_eq!(circuit_error.get_service_id(), "abc");
        assert_eq!(
            circuit_error.get_error(),
            CircuitError_Error::ERROR_RECIPIENT_NOT_IN_DIRECTORY
        );
        assert_eq!(circuit_error.get_error_message(), "TEST");
        assert_eq!(circuit_error.get_correlation_id(), "1234");
    }

    // Test that if an error message recieved is meant for the service not connected to this node,
    // the error message is sent to the node the service is connected to
    #[test]
    fn test_circuit_error_handler_node() {
        // Set up disptacher and mock sender
        let sender = Box::new(MockSender::default());
        let mut dispatcher = Dispatcher::new(sender.box_clone());

        // Add circuit and service to splinter state
        let circuit = Circuit::new(
            "alpha".into(),
            "trust".into(),
            vec!["123".into()],
            vec!["abc".into(), "def".into()],
            "any".into(),
            "none".into(),
            "require_direct".into(),
        );

        let mut circuit_directory = CircuitDirectory::new();
        circuit_directory.add_circuit("alpha".to_string(), circuit);

        let state = Arc::new(RwLock::new(SplinterState::new(
            "memory".to_string(),
            circuit_directory,
        )));

        let node_123 = SplinterNode::new("123".to_string(), vec!["123.0.0.1:0".to_string()]);
        let node_345 = SplinterNode::new("345".to_string(), vec!["123.0.0.1:1".to_string()]);

        let service_abc =
            Service::new("abc".to_string(), Some("abc_network".to_string()), node_123);
        let service_def =
            Service::new("def".to_string(), Some("def_network".to_string()), node_345);

        let abc_id = ServiceId::new("alpha".into(), "abc".into());
        let def_id = ServiceId::new("alpha".into(), "def".into());
        state.write().unwrap().add_service(abc_id, service_abc);
        state.write().unwrap().add_service(def_id, service_def);

        // Add circuit error handler to the the dispatcher
        let handler = CircuitErrorHandler::new("123".to_string(), state);
        dispatcher.set_handler(CircuitMessageType::CIRCUIT_ERROR_MESSAGE, Box::new(handler));

        // Create the error message
        let mut circuit_error = CircuitError::new();
        circuit_error.set_service_id("def".into());
        circuit_error.set_circuit_name("alpha".into());
        circuit_error.set_correlation_id("1234".into());
        circuit_error.set_error(CircuitError_Error::ERROR_RECIPIENT_NOT_IN_DIRECTORY);
        circuit_error.set_error_message("TEST".into());
        let error_bytes = circuit_error.write_to_bytes().unwrap();

        // dispatch the error message
        dispatcher
            .dispatch(
                "568",
                &CircuitMessageType::CIRCUIT_ERROR_MESSAGE,
                error_bytes.clone(),
            )
            .unwrap();

        // verify that the error message was sent to the 345 node
        let send_request = sender.sent().get(0).unwrap().clone();

        assert_eq!(send_request.recipient(), "345");

        let network_msg: NetworkMessage =
            protobuf::parse_from_bytes(send_request.payload()).unwrap();
        let circuit_msg: CircuitMessage =
            protobuf::parse_from_bytes(network_msg.get_payload()).unwrap();
        let circuit_error: CircuitError =
            protobuf::parse_from_bytes(circuit_msg.get_payload()).unwrap();

        assert_eq!(
            circuit_msg.get_message_type(),
            CircuitMessageType::CIRCUIT_ERROR_MESSAGE
        );

        assert_eq!(circuit_error.get_service_id(), "def");
        assert_eq!(
            circuit_error.get_error(),
            CircuitError_Error::ERROR_RECIPIENT_NOT_IN_DIRECTORY
        );
        assert_eq!(circuit_error.get_error_message(), "TEST");
        assert_eq!(circuit_error.get_correlation_id(), "1234");
    }

    // Test that if the service the error message is meant for is not connected, the message is
    // dropped because there is no way to know where to send it
    #[test]
    fn test_circuit_error_handler_no_service() {
        // Set up disptacher and mock sender
        let sender = Box::new(MockSender::default());
        let mut dispatcher = Dispatcher::new(sender.box_clone());

        // create empty state
        let circuit_directory = CircuitDirectory::new();

        let state = Arc::new(RwLock::new(SplinterState::new(
            "memory".to_string(),
            circuit_directory,
        )));

        // Add circuit error handler to the the dispatcher
        let handler = CircuitErrorHandler::new("123".to_string(), state);
        dispatcher.set_handler(CircuitMessageType::CIRCUIT_ERROR_MESSAGE, Box::new(handler));

        // Create the circuit error message
        let mut circuit_error = CircuitError::new();
        circuit_error.set_service_id("abc".into());
        circuit_error.set_circuit_name("alpha".into());
        circuit_error.set_correlation_id("1234".into());
        circuit_error.set_error(CircuitError_Error::ERROR_RECIPIENT_NOT_IN_DIRECTORY);
        circuit_error.set_error_message("TEST".into());
        let error_bytes = circuit_error.write_to_bytes().unwrap();

        // dispatch the error message
        dispatcher
            .dispatch(
                "def",
                &CircuitMessageType::CIRCUIT_ERROR_MESSAGE,
                error_bytes.clone(),
            )
            .unwrap();

        // verify that the direct message was dropped
        let send_request = sender.sent();

        assert_eq!(send_request.len(), 0);
    }
}
