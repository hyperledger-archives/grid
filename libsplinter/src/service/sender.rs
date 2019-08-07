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
use crossbeam_channel::Sender;
use protobuf::Message;
use uuid::Uuid;

use crate::network::reply::InboundRouter;
use crate::protos::circuit::{
    AdminDirectMessage, CircuitDirectMessage, CircuitMessage, CircuitMessageType,
};
use crate::protos::network::{NetworkMessage, NetworkMessageType};
use crate::service::error::ServiceSendError;
use crate::service::{ServiceMessageContext, ServiceNetworkSender};

#[derive(Debug, Clone)]
pub enum ServiceMessage {
    AdminDirectMessage(AdminDirectMessage),
    CircuitDirectMessage(CircuitDirectMessage),
}

#[derive(Debug, Clone)]
pub enum ProcessorMessage {
    ServiceMessage(ServiceMessage),
    Shutdown,
}

/// An implementation of a ServiceNetworkSender that should be used for AdminDirectMessage.
/// AdminDirectMessage needs special handling since this message can be sent over the admin circuit
/// or over any other circuit that exists.
#[derive(Clone)]
pub struct AdminServiceNetworkSender {
    outgoing_sender: Sender<Vec<u8>>,
    message_sender: String,
    inbound_router: InboundRouter<CircuitMessageType>,
}

impl AdminServiceNetworkSender {
    pub fn new(
        outgoing_sender: Sender<Vec<u8>>,
        message_sender: String,
        inbound_router: InboundRouter<CircuitMessageType>,
    ) -> Self {
        AdminServiceNetworkSender {
            outgoing_sender,
            message_sender,
            inbound_router,
        }
    }
}

impl ServiceNetworkSender for AdminServiceNetworkSender {
    /// the service will create the admin direct message themselves so it can set which circuit
    /// this message should be sent over.
    fn send(&self, recipient: &str, message: &[u8]) -> Result<(), ServiceSendError> {
        let mut admin_direct_message = AdminDirectMessage::new();
        admin_direct_message.set_circuit("admin".into());
        admin_direct_message.set_sender(self.message_sender.to_string());
        admin_direct_message.set_recipient(recipient.into());
        admin_direct_message.set_payload(message.to_vec());

        let bytes = admin_direct_message
            .write_to_bytes()
            .map_err(|err| ServiceSendError(Box::new(err)))?;

        let msg = create_message(bytes, CircuitMessageType::ADMIN_DIRECT_MESSAGE)
            .map_err(|err| ServiceSendError(Box::new(err)))?;

        self.outgoing_sender
            .send(msg)
            .map_err(|err| ServiceSendError(Box::new(err)))?;
        Ok(())
    }

    /// Send the message bytes to the given recipient (another admin service)
    /// and await the reply. This function blocks until the reply is
    /// returned.
    fn send_and_await(&self, recipient: &str, message: &[u8]) -> Result<Vec<u8>, ServiceSendError> {
        let mut admin_direct_message = AdminDirectMessage::new();
        admin_direct_message.set_circuit("admin".into());
        admin_direct_message.set_sender(self.message_sender.to_string());
        admin_direct_message.set_recipient(recipient.into());
        admin_direct_message.set_payload(message.to_vec());
        let correlation_id = Uuid::new_v4().to_string();
        admin_direct_message.set_correlation_id(correlation_id.to_string());

        let bytes = admin_direct_message
            .write_to_bytes()
            .map_err(|err| ServiceSendError(Box::new(err)))?;

        let message = create_message(bytes, CircuitMessageType::ADMIN_DIRECT_MESSAGE)
            .map_err(|err| ServiceSendError(Box::new(err)))?;

        let mut future = self.inbound_router.expect_reply(correlation_id);

        self.outgoing_sender
            .send(message)
            .map_err(|err| ServiceSendError(Box::new(err)))?;

        // block until the response is received
        future
            .get::<AdminDirectMessage>()
            .map(|mut res| res.take_payload())
            .map_err(|err| ServiceSendError(Box::new(err)))
    }

    /// Send the message bytes back to the origin specified in the given
    /// message context.
    fn reply(
        &self,
        message_origin: &ServiceMessageContext,
        message: &[u8],
    ) -> Result<(), ServiceSendError> {
        let mut admin_direct_message = AdminDirectMessage::new();
        admin_direct_message.set_circuit(message_origin.circuit.to_string());
        admin_direct_message.set_sender(self.message_sender.to_string());
        admin_direct_message.set_recipient(message_origin.sender.to_string());
        admin_direct_message.set_payload(message.to_vec());
        admin_direct_message.set_correlation_id(message_origin.correlation_id.to_string());

        let bytes = admin_direct_message
            .write_to_bytes()
            .map_err(|err| ServiceSendError(Box::new(err)))?;

        let message = create_message(bytes, CircuitMessageType::ADMIN_DIRECT_MESSAGE)
            .map_err(|err| ServiceSendError(Box::new(err)))?;

        self.outgoing_sender
            .send(message)
            .map_err(|err| ServiceSendError(Box::new(err)))?;
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn ServiceNetworkSender> {
        Box::new(self.clone())
    }
}

/// This implementation of ServiceNetworkSender can be used by any service that does not require
/// any special handling.
#[derive(Clone)]
pub struct StandardServiceNetworkSender {
    outgoing_sender: Sender<Vec<u8>>,
    circuit: String,
    message_sender: String,
    inbound_router: InboundRouter<CircuitMessageType>,
}

impl StandardServiceNetworkSender {
    pub fn new(
        outgoing_sender: Sender<Vec<u8>>,
        circuit: String,
        message_sender: String,
        inbound_router: InboundRouter<CircuitMessageType>,
    ) -> Self {
        StandardServiceNetworkSender {
            outgoing_sender,
            circuit,
            message_sender,
            inbound_router,
        }
    }
}

impl ServiceNetworkSender for StandardServiceNetworkSender {
    /// Send the message bytes to the given recipient (another service)
    fn send(&self, recipient: &str, message: &[u8]) -> Result<(), ServiceSendError> {
        let mut direct_message = CircuitDirectMessage::new();
        direct_message.set_circuit(self.circuit.to_string());
        direct_message.set_sender(self.message_sender.to_string());
        direct_message.set_recipient(recipient.to_string());
        direct_message.set_payload(message.to_vec());

        let bytes = direct_message
            .write_to_bytes()
            .map_err(|err| ServiceSendError(Box::new(err)))?;

        let message = create_message(bytes, CircuitMessageType::CIRCUIT_DIRECT_MESSAGE)
            .map_err(|err| ServiceSendError(Box::new(err)))?;

        self.outgoing_sender
            .send(message)
            .map_err(|err| ServiceSendError(Box::new(err)))?;
        Ok(())
    }

    /// Send the message bytes to the given recipient (another service)
    /// and await the reply.  This function blocks until the reply is
    /// returned.
    fn send_and_await(&self, recipient: &str, message: &[u8]) -> Result<Vec<u8>, ServiceSendError> {
        let mut direct_message = CircuitDirectMessage::new();
        direct_message.set_circuit(self.circuit.to_string());
        direct_message.set_sender(self.message_sender.to_string());
        direct_message.set_recipient(recipient.to_string());
        direct_message.set_payload(message.to_vec());

        let correlation_id = Uuid::new_v4().to_string();
        direct_message.set_correlation_id(correlation_id.to_string());

        let bytes = direct_message
            .write_to_bytes()
            .map_err(|err| ServiceSendError(Box::new(err)))?;

        let message = create_message(bytes, CircuitMessageType::CIRCUIT_DIRECT_MESSAGE)
            .map_err(|err| ServiceSendError(Box::new(err)))?;

        let mut future = self.inbound_router.expect_reply(correlation_id);

        self.outgoing_sender
            .send(message)
            .map_err(|err| ServiceSendError(Box::new(err)))?;

        // block until the response is received
        future
            .get::<CircuitDirectMessage>()
            .map(|mut res| res.take_payload())
            .map_err(|err| ServiceSendError(Box::new(err)))
    }

    /// Send the message bytes back to the origin specified in the given
    /// message context.
    fn reply(
        &self,
        message_origin: &ServiceMessageContext,
        message: &[u8],
    ) -> Result<(), ServiceSendError> {
        let mut direct_message = CircuitDirectMessage::new();
        direct_message.set_circuit(message_origin.circuit.to_string());
        direct_message.set_sender(self.message_sender.to_string());
        direct_message.set_recipient(message_origin.sender.to_string());
        direct_message.set_payload(message.to_vec());
        direct_message.set_correlation_id(message_origin.correlation_id.to_string());

        let bytes = direct_message
            .write_to_bytes()
            .map_err(|err| ServiceSendError(Box::new(err)))?;

        let message = create_message(bytes, CircuitMessageType::CIRCUIT_DIRECT_MESSAGE)
            .map_err(|err| ServiceSendError(Box::new(err)))?;

        self.outgoing_sender
            .send(message)
            .map_err(|err| ServiceSendError(Box::new(err)))?;
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn ServiceNetworkSender> {
        Box::new(self.clone())
    }
}

/// Helper function for creating a NetworkMessge with a Circuit message type
///
/// # Arguments
///
/// * `payload` - The payload in bytes that should be set in the Circuit message get_payload
/// * `circuit_message_type` - The message type that should be set in teh Circuit message
pub fn create_message(
    payload: Vec<u8>,
    circuit_message_type: CircuitMessageType,
) -> Result<Vec<u8>, protobuf::error::ProtobufError> {
    let mut circuit_msg = CircuitMessage::new();
    circuit_msg.set_message_type(circuit_message_type);
    circuit_msg.set_payload(payload);
    let circuit_bytes = circuit_msg.write_to_bytes()?;

    let mut network_msg = NetworkMessage::new();
    network_msg.set_message_type(NetworkMessageType::CIRCUIT);
    network_msg.set_payload(circuit_bytes);
    network_msg.write_to_bytes()
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use std::thread;

    #[test]
    // test that a StandardServiceNetworkSender properly sends a message to the outgoing thread
    fn test_standard_send() {
        let (outgoing_sender, outgoing_receiver) = crossbeam_channel::bounded(3);
        let (internal_sender, _) = crossbeam_channel::bounded(3);
        let inbound_router: InboundRouter<CircuitMessageType> =
            InboundRouter::new(Box::new(internal_sender));
        let network_sender = StandardServiceNetworkSender::new(
            outgoing_sender,
            "test_circuit".to_string(),
            "service_a".to_string(),
            inbound_router,
        );

        thread::Builder::new()
            .name("test_standard_send".to_string())
            .spawn(move || {
                network_sender.send("service_b", b"test_message").unwrap();
            })
            .unwrap();

        let msg_bytes = match outgoing_receiver.recv() {
            Ok(msg_bytes) => msg_bytes,
            Err(err) => panic!("Received error: {}", err),
        };

        let network_msg: NetworkMessage = protobuf::parse_from_bytes(&msg_bytes).unwrap();
        let circuit_msg: CircuitMessage =
            protobuf::parse_from_bytes(network_msg.get_payload()).unwrap();
        let direct_message: CircuitDirectMessage =
            protobuf::parse_from_bytes(circuit_msg.get_payload()).unwrap();

        assert_eq!(direct_message.get_recipient(), "service_b");
        assert_eq!(direct_message.get_sender(), "service_a");
        assert_eq!(direct_message.get_circuit(), "test_circuit");
        assert_eq!(direct_message.get_payload(), b"test_message");
    }

    #[test]
    // test that a StandardServiceNetworkSender properly send_and_awaits. Sends a message and
    // waits for a reply.
    fn test_standard_send_and_await_send() {
        let (outgoing_sender, outgoing_receiver) = crossbeam_channel::bounded(3);
        let (internal_sender, _) = crossbeam_channel::bounded(3);
        let mut inbound_router: InboundRouter<CircuitMessageType> =
            InboundRouter::new(Box::new(internal_sender));
        let network_sender = StandardServiceNetworkSender::new(
            outgoing_sender,
            "test_circuit".to_string(),
            "service_a".to_string(),
            inbound_router.clone(),
        );

        thread::Builder::new()
            .name("test_standard_send_and_await_send".to_string())
            .spawn(move || {
                let response = network_sender
                    .send_and_await("service_b", b"test_message")
                    .unwrap();
                assert_eq!(&response, b"test_response");

                // send message to shutdown the test
                network_sender.send("service_b", b"shutdown").unwrap();
            })
            .unwrap();

        let msg_bytes = match outgoing_receiver.recv() {
            Ok(msg_bytes) => msg_bytes,
            Err(err) => panic!("Received error: {}", err),
        };

        let network_msg: NetworkMessage = protobuf::parse_from_bytes(&msg_bytes).unwrap();
        let circuit_msg: CircuitMessage =
            protobuf::parse_from_bytes(network_msg.get_payload()).unwrap();
        let mut direct_message: CircuitDirectMessage =
            protobuf::parse_from_bytes(circuit_msg.get_payload()).unwrap();

        assert_eq!(direct_message.get_recipient(), "service_b");
        assert_eq!(direct_message.get_sender(), "service_a");
        assert_eq!(direct_message.get_circuit(), "test_circuit");
        assert_eq!(direct_message.get_payload(), b"test_message");

        let correlation_id = direct_message.take_correlation_id();

        let mut direct_response = CircuitDirectMessage::new();
        direct_response.set_recipient(direct_message.take_sender());
        direct_response.set_sender(direct_message.take_recipient());
        direct_response.set_circuit(direct_message.take_circuit());
        direct_response.set_correlation_id(correlation_id);
        direct_response.set_payload(b"test_response".to_vec());
        inbound_router
            .route(
                direct_response.get_correlation_id(),
                Ok((
                    CircuitMessageType::CIRCUIT_DIRECT_MESSAGE,
                    direct_response
                        .write_to_bytes()
                        .expect("Failed to write bytes"),
                )),
            )
            .unwrap();

        // block until the network sender test is finished
        outgoing_receiver.recv().unwrap();
    }

    #[test]
    // test that a StandardServiceNetworkSender properly replies to a message based on the provided
    // message context
    fn test_standard_reply() {
        let (outgoing_sender, outgoing_receiver) = crossbeam_channel::bounded(3);
        let (internal_sender, _) = crossbeam_channel::bounded(3);
        let inbound_router: InboundRouter<CircuitMessageType> =
            InboundRouter::new(Box::new(internal_sender));
        let network_sender = StandardServiceNetworkSender::new(
            outgoing_sender,
            "test_circuit".to_string(),
            "service_a".to_string(),
            inbound_router,
        );

        thread::Builder::new()
            .name("test_standard_reply".to_string())
            .spawn(move || {
                let msg_context = ServiceMessageContext {
                    sender: "service_b".to_string(),
                    circuit: "test_circuit".to_string(),
                    correlation_id: "test_correlation_id".to_string(),
                };
                network_sender.reply(&msg_context, b"test_message").unwrap();
            })
            .unwrap();;

        let msg_bytes = match outgoing_receiver.recv() {
            Ok(msg_bytes) => msg_bytes,
            Err(err) => panic!("Received error: {}", err),
        };

        let network_msg: NetworkMessage = protobuf::parse_from_bytes(&msg_bytes).unwrap();
        let circuit_msg: CircuitMessage =
            protobuf::parse_from_bytes(network_msg.get_payload()).unwrap();
        let direct_message: CircuitDirectMessage =
            protobuf::parse_from_bytes(circuit_msg.get_payload()).unwrap();

        assert_eq!(direct_message.get_recipient(), "service_b");
        assert_eq!(direct_message.get_sender(), "service_a");
        assert_eq!(direct_message.get_circuit(), "test_circuit");
        assert_eq!(direct_message.get_payload(), b"test_message");
        assert_eq!(direct_message.get_correlation_id(), "test_correlation_id");
    }

    #[test]
    // test that a AdminServiceNetworkSender properly sends a message to the outgoing thread
    fn test_admin_send() {
        let (outgoing_sender, outgoing_receiver) = crossbeam_channel::bounded(3);
        let (internal_sender, _) = crossbeam_channel::bounded(3);
        let inbound_router: InboundRouter<CircuitMessageType> =
            InboundRouter::new(Box::new(internal_sender));
        let network_sender = AdminServiceNetworkSender::new(
            outgoing_sender,
            "service_b".to_string(),
            inbound_router,
        );

        thread::Builder::new()
            .name("test_admin_send".to_string())
            .spawn(move || {
                network_sender.send("service_a", b"test_admin").unwrap();
            })
            .unwrap();;

        let msg_bytes = match outgoing_receiver.recv() {
            Ok(msg_bytes) => msg_bytes,
            Err(err) => panic!("Received error: {}", err),
        };

        let network_msg: NetworkMessage = protobuf::parse_from_bytes(&msg_bytes).unwrap();
        let circuit_msg: CircuitMessage =
            protobuf::parse_from_bytes(network_msg.get_payload()).unwrap();
        let direct_message: AdminDirectMessage =
            protobuf::parse_from_bytes(circuit_msg.get_payload()).unwrap();

        assert_eq!(direct_message.get_recipient(), "service_a");
        assert_eq!(direct_message.get_sender(), "service_b");
        assert_eq!(direct_message.get_circuit(), "admin");
        assert_eq!(direct_message.get_payload(), b"test_admin");
    }

    #[test]
    // test that a AdminServiceNetworkSender properly send_and_awaits. Sends a message and
    // waits for a reply.
    fn test_admin_send_and_await_send() {
        let (outgoing_sender, outgoing_receiver) = crossbeam_channel::bounded(3);
        let (internal_sender, _) = crossbeam_channel::bounded(3);
        let mut inbound_router: InboundRouter<CircuitMessageType> =
            InboundRouter::new(Box::new(internal_sender));
        let network_sender = AdminServiceNetworkSender::new(
            outgoing_sender,
            "service_b".to_string(),
            inbound_router.clone(),
        );

        thread::Builder::new()
            .name("test_admin_send_and_await_send".to_string())
            .spawn(move || {
                let response = network_sender
                    .send_and_await("service_a", b"test_admin")
                    .unwrap();
                assert_eq!(&response, b"test_response");

                // send message to shutdown the test
                network_sender.send("service_b", b"shutdown").unwrap();
            })
            .unwrap();;

        let msg_bytes = match outgoing_receiver.recv() {
            Ok(msg_bytes) => msg_bytes,
            Err(err) => panic!("Received error: {}", err),
        };

        let network_msg: NetworkMessage = protobuf::parse_from_bytes(&msg_bytes).unwrap();
        let circuit_msg: CircuitMessage =
            protobuf::parse_from_bytes(network_msg.get_payload()).unwrap();
        let mut direct_message: AdminDirectMessage =
            protobuf::parse_from_bytes(circuit_msg.get_payload()).unwrap();

        assert_eq!(direct_message.get_recipient(), "service_a");
        assert_eq!(direct_message.get_sender(), "service_b");
        assert_eq!(direct_message.get_circuit(), "admin");
        assert_eq!(direct_message.get_payload(), b"test_admin");

        let correlation_id = direct_message.take_correlation_id();

        let mut direct_response = AdminDirectMessage::new();
        direct_response.set_recipient(direct_message.take_sender());
        direct_response.set_sender(direct_message.take_recipient());
        direct_response.set_circuit(direct_message.take_circuit());
        direct_response.set_correlation_id(correlation_id);
        direct_response.set_payload(b"test_response".to_vec());
        inbound_router
            .route(
                direct_response.get_correlation_id(),
                Ok((
                    CircuitMessageType::ADMIN_DIRECT_MESSAGE,
                    direct_response
                        .write_to_bytes()
                        .expect("Failed to write bytes"),
                )),
            )
            .unwrap();

        // block until the network sender test is finished
        outgoing_receiver.recv().unwrap();
    }

    #[test]
    // test that a StandardServiceNetworkSender properly replies to a message based on the provided
    // message context
    fn test_admin_reply() {
        let (outgoing_sender, outgoing_receiver) = crossbeam_channel::bounded(3);
        let (internal_sender, _) = crossbeam_channel::bounded(3);
        let inbound_router: InboundRouter<CircuitMessageType> =
            InboundRouter::new(Box::new(internal_sender));
        let network_sender = AdminServiceNetworkSender::new(
            outgoing_sender,
            "service_a".to_string(),
            inbound_router,
        );

        thread::Builder::new()
            .name("test_admin_reply".to_string())
            .spawn(move || {
                let msg_context = ServiceMessageContext {
                    sender: "service_b".to_string(),
                    circuit: "admin".to_string(),
                    correlation_id: "test_correlation_id".to_string(),
                };
                network_sender.reply(&msg_context, b"test_message").unwrap();
            })
            .unwrap();;

        let msg_bytes = match outgoing_receiver.recv() {
            Ok(msg_bytes) => msg_bytes,
            Err(err) => panic!("Received error: {}", err),
        };

        let network_msg: NetworkMessage = protobuf::parse_from_bytes(&msg_bytes).unwrap();
        let circuit_msg: CircuitMessage =
            protobuf::parse_from_bytes(network_msg.get_payload()).unwrap();
        let direct_message: AdminDirectMessage =
            protobuf::parse_from_bytes(circuit_msg.get_payload()).unwrap();

        assert_eq!(direct_message.get_recipient(), "service_b");
        assert_eq!(direct_message.get_sender(), "service_a");
        assert_eq!(direct_message.get_circuit(), "admin");
        assert_eq!(direct_message.get_payload(), b"test_message");
        assert_eq!(direct_message.get_correlation_id(), "test_correlation_id");
    }
}
