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
use crate::network::dispatch::{DispatchError, DispatchMessage, Handler, MessageContext};
use crate::network::sender::SendRequest;
use crate::protos::circuit::{CircuitMessage, CircuitMessageType};
use crate::protos::network::NetworkMessageType;

use ::log::{debug, log};

// Implements a handler that pass messages to another dispatcher loop
pub struct CircuitMessageHandler {
    sender: Box<Sender<DispatchMessage<CircuitMessageType>>>,
}

impl Handler<NetworkMessageType, CircuitMessage> for CircuitMessageHandler {
    fn handle(
        &self,
        msg: CircuitMessage,
        context: &MessageContext<NetworkMessageType>,
        _: &dyn Sender<SendRequest>,
    ) -> Result<(), DispatchError> {
        debug!("Handle CircuitMessage: {:?}", msg);
        let dispatch_msg = DispatchMessage::new(
            msg.get_message_type(),
            msg.get_payload().to_vec(),
            context.source_peer_id().to_string(),
        );
        self.sender.send(dispatch_msg)?;
        Ok(())
    }
}

impl CircuitMessageHandler {
    pub fn new(sender: Box<Sender<DispatchMessage<CircuitMessageType>>>) -> Self {
        CircuitMessageHandler { sender }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::channel::mock::MockSender;
    use crate::channel::Sender;
    use crate::network::dispatch::Dispatcher;
    use crate::protos::network::NetworkMessageType;

    use protobuf::Message;

    #[test]
    // Test that circuit message is sent to the circuit dispatch sender
    fn test_circuit_message_handler() {
        // Set up the dispatcher and handler
        let network_sender = Box::new(MockSender::default());
        let circuit_sender = Box::new(MockSender::default());
        let mut network_dispatcher = Dispatcher::new(network_sender.box_clone());

        let handler = CircuitMessageHandler::new(circuit_sender.box_clone());
        network_dispatcher.set_handler(NetworkMessageType::CIRCUIT, Box::new(handler));

        // Create a CircuitMessage wrapped in a NetworkMessage to be dispatched
        let mut circuit_msg = CircuitMessage::new();
        circuit_msg.set_message_type(CircuitMessageType::SERVICE_CONNECT_REQUEST);
        circuit_msg.set_payload(b"test".to_vec());
        let circuit_bytes = circuit_msg.write_to_bytes().unwrap();

        // Dispatch network message
        network_dispatcher
            .dispatch("PEER", &NetworkMessageType::CIRCUIT, circuit_bytes.clone())
            .unwrap();

        // Check that the CircuitMessage is put in the DispatchMessage and send over the sender
        let dispatched_messages = circuit_sender.sent();
        let message = dispatched_messages.get(0).unwrap();

        assert_eq!(message.source_peer_id(), "PEER");
        assert_eq!(b"test".to_vec(), message.message_bytes());
        assert_eq!(
            message.message_type(),
            &CircuitMessageType::SERVICE_CONNECT_REQUEST
        );
    }
}
