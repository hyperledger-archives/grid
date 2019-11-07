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
    CircuitMessageType, ServiceConnectRequest, ServiceConnectResponse,
    ServiceConnectResponse_Status, ServiceDisconnectRequest, ServiceDisconnectResponse,
    ServiceDisconnectResponse_Status,
};
use crate::service::error::{ServiceConnectionError, ServiceDisconnectionError};
use crate::service::sender::create_message;
use crate::service::sender::{AdminServiceNetworkSender, StandardServiceNetworkSender};
use crate::service::{ServiceNetworkRegistry, ServiceNetworkSender};

const ADMIN_CIRCUIT_NAME: &str = "admin";

pub struct StandardServiceNetworkRegistry {
    circuit: String,
    outgoing_sender: Sender<Vec<u8>>,
    inbound_router: InboundRouter<CircuitMessageType>,
}

/// This is an implementation of ServiceNetworkRegistry that can be used by a standard service
/// that does not require special funcationality
impl StandardServiceNetworkRegistry {
    pub fn new(
        circuit: String,
        outgoing_sender: Sender<Vec<u8>>,
        inbound_router: InboundRouter<CircuitMessageType>,
    ) -> Self {
        StandardServiceNetworkRegistry {
            circuit,
            outgoing_sender,
            inbound_router,
        }
    }
}

impl ServiceNetworkRegistry for StandardServiceNetworkRegistry {
    /// Sends a ServiceConnectRequest for the provided service_id and blocks
    /// until the connection response is returned from the splinter node
    fn connect(
        &self,
        service_id: &str,
    ) -> Result<Box<dyn ServiceNetworkSender>, ServiceConnectionError> {
        let correlation_id = Uuid::new_v4().to_string();
        let mut connect_msg = ServiceConnectRequest::new();
        connect_msg.set_circuit(self.circuit.to_string());
        connect_msg.set_service_id(service_id.to_string());
        connect_msg.set_correlation_id(correlation_id.clone());

        let connect_msg_bytes = connect_msg
            .write_to_bytes()
            .map_err(|err| ServiceConnectionError::ConnectionError(Box::new(err)))?;

        let msg_bytes = create_message(
            connect_msg_bytes,
            CircuitMessageType::SERVICE_CONNECT_REQUEST,
        )
        .map_err(|err| ServiceConnectionError::ConnectionError(Box::new(err)))?;

        let mut future = self.inbound_router.expect_reply(correlation_id);

        self.outgoing_sender
            .send(msg_bytes)
            .map_err(|err| ServiceConnectionError::ConnectionError(Box::new(err)))?;

        let mut response: ServiceConnectResponse = future
            .get()
            .map_err(|err| ServiceConnectionError::ConnectionError(Box::new(err)))?;

        if response.get_status() != ServiceConnectResponse_Status::OK {
            return Err(ServiceConnectionError::RejectedError(
                response.take_error_message(),
            ));
        }

        if self.circuit == ADMIN_CIRCUIT_NAME {
            let admin_network_sender = AdminServiceNetworkSender::new(
                self.outgoing_sender.clone(),
                service_id.to_string(),
                self.inbound_router.clone(),
            );
            Ok(Box::new(admin_network_sender))
        } else {
            let standard_network_sender = StandardServiceNetworkSender::new(
                self.outgoing_sender.clone(),
                self.circuit.to_string(),
                service_id.to_string(),
                self.inbound_router.clone(),
            );
            Ok(Box::new(standard_network_sender))
        }
    }

    /// Sends a ServiceDisconnectRequest for the provided service_id and blocks
    /// until the disconnection response is returned from the splinter node
    fn disconnect(&self, service_id: &str) -> Result<(), ServiceDisconnectionError> {
        let correlation_id = Uuid::new_v4().to_string();
        let mut disconnect_msg = ServiceDisconnectRequest::new();
        disconnect_msg.set_circuit(self.circuit.to_string());
        disconnect_msg.set_service_id(service_id.to_string());
        disconnect_msg.set_correlation_id(correlation_id.clone());

        let disconnect_msg_bytes = disconnect_msg
            .write_to_bytes()
            .map_err(|err| ServiceDisconnectionError::DisconnectionError(Box::new(err)))?;

        let msg_bytes = create_message(
            disconnect_msg_bytes,
            CircuitMessageType::SERVICE_DISCONNECT_REQUEST,
        )
        .map_err(|err| ServiceDisconnectionError::DisconnectionError(Box::new(err)))?;

        let mut future = self.inbound_router.expect_reply(correlation_id);

        self.outgoing_sender
            .send(msg_bytes)
            .map_err(|err| ServiceDisconnectionError::DisconnectionError(Box::new(err)))?;

        let mut response: ServiceDisconnectResponse = future
            .get()
            .map_err(|err| ServiceDisconnectionError::DisconnectionError(Box::new(err)))?;

        if response.get_status() != ServiceDisconnectResponse_Status::OK {
            return Err(ServiceDisconnectionError::RejectedError(
                response.take_error_message(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use std::thread;

    use crate::protos::circuit::{
        CircuitMessage, ServiceConnectResponse, ServiceDisconnectResponse,
    };
    use crate::protos::network::NetworkMessage;

    #[test]
    // Test connecting an admin service to a splinter daemon. The connect function will block
    // until a response is returned.
    fn test_admin_connect() {
        let (outgoing_sender, outgoing_receiver) = crossbeam_channel::bounded(3);
        let (internal_sender, _) = crossbeam_channel::bounded(3);
        let mut inbound_router: InboundRouter<CircuitMessageType> =
            InboundRouter::new(Box::new(internal_sender));
        let registry = StandardServiceNetworkRegistry::new(
            ADMIN_CIRCUIT_NAME.to_string(),
            outgoing_sender,
            inbound_router.clone(),
        );

        thread::Builder::new()
            .name("test_admin_connect".to_string())
            .spawn(move || {
                let msg_bytes = outgoing_receiver.recv().unwrap();
                let network_msg: NetworkMessage = protobuf::parse_from_bytes(&msg_bytes).unwrap();
                let circuit_msg: CircuitMessage =
                    protobuf::parse_from_bytes(network_msg.get_payload()).unwrap();
                let mut connect_request: ServiceConnectRequest =
                    protobuf::parse_from_bytes(circuit_msg.get_payload()).unwrap();

                assert_eq!(connect_request.get_service_id(), "service_a");
                assert_eq!(connect_request.get_circuit(), ADMIN_CIRCUIT_NAME);

                let mut response = ServiceConnectResponse::new();
                response.set_circuit(connect_request.take_circuit());
                response.set_service_id(connect_request.take_service_id());
                response.set_status(ServiceConnectResponse_Status::OK);
                response.set_correlation_id(connect_request.take_correlation_id());

                inbound_router
                    .route(
                        response.get_correlation_id(),
                        Ok((
                            CircuitMessageType::SERVICE_CONNECT_RESPONSE,
                            response.write_to_bytes().expect("Failed to write bytes"),
                        )),
                    )
                    .unwrap();
            })
            .unwrap();

        let _service_network_sender = registry.connect("service_a").unwrap();
    }

    #[test]
    // Test connecting a standard service to a splinter daemon. The connect function will block
    // until a response is returned.
    fn test_standard_connect() {
        let (outgoing_sender, outgoing_receiver) = crossbeam_channel::bounded(3);
        let (internal_sender, _) = crossbeam_channel::bounded(3);
        let mut inbound_router: InboundRouter<CircuitMessageType> =
            InboundRouter::new(Box::new(internal_sender));
        let registry = StandardServiceNetworkRegistry::new(
            "test".to_string(),
            outgoing_sender,
            inbound_router.clone(),
        );

        thread::Builder::new()
            .name("test_standard_connect".to_string())
            .spawn(move || {
                let msg_bytes = outgoing_receiver.recv().unwrap();
                let network_msg: NetworkMessage = protobuf::parse_from_bytes(&msg_bytes).unwrap();
                let circuit_msg: CircuitMessage =
                    protobuf::parse_from_bytes(network_msg.get_payload()).unwrap();
                let mut connect_request: ServiceConnectRequest =
                    protobuf::parse_from_bytes(circuit_msg.get_payload()).unwrap();

                assert_eq!(connect_request.get_service_id(), "service_a");
                assert_eq!(connect_request.get_circuit(), "test");

                let mut response = ServiceConnectResponse::new();
                response.set_circuit(connect_request.take_circuit());
                response.set_service_id(connect_request.take_service_id());
                response.set_status(ServiceConnectResponse_Status::OK);
                response.set_correlation_id(connect_request.take_correlation_id());

                inbound_router
                    .route(
                        response.get_correlation_id(),
                        Ok((
                            CircuitMessageType::SERVICE_CONNECT_RESPONSE,
                            response.write_to_bytes().expect("Failed to write bytes"),
                        )),
                    )
                    .unwrap();
            })
            .unwrap();

        let _service_network_sender = registry.connect("service_a").unwrap();
    }

    #[test]
    // Test disconnecting a standard service from a splinter daemon. The connect function will
    // block until a response is returned.
    fn test_disconnect() {
        let (outgoing_sender, outgoing_receiver) = crossbeam_channel::bounded(3);
        let (internal_sender, _) = crossbeam_channel::bounded(3);
        let mut inbound_router: InboundRouter<CircuitMessageType> =
            InboundRouter::new(Box::new(internal_sender));
        let registry = StandardServiceNetworkRegistry::new(
            "test".to_string(),
            outgoing_sender,
            inbound_router.clone(),
        );

        thread::Builder::new()
            .name("test_disconnect".to_string())
            .spawn(move || {
                let msg_bytes = outgoing_receiver.recv().unwrap();
                let network_msg: NetworkMessage = protobuf::parse_from_bytes(&msg_bytes).unwrap();
                let circuit_msg: CircuitMessage =
                    protobuf::parse_from_bytes(network_msg.get_payload()).unwrap();
                let mut disconnect_request: ServiceDisconnectRequest =
                    protobuf::parse_from_bytes(circuit_msg.get_payload()).unwrap();

                assert_eq!(disconnect_request.get_service_id(), "service_a");
                assert_eq!(disconnect_request.get_circuit(), "test");

                let mut response = ServiceDisconnectResponse::new();
                response.set_circuit(disconnect_request.take_circuit());
                response.set_service_id(disconnect_request.take_service_id());
                response.set_status(ServiceDisconnectResponse_Status::OK);
                response.set_correlation_id(disconnect_request.take_correlation_id());

                inbound_router
                    .route(
                        response.get_correlation_id(),
                        Ok((
                            CircuitMessageType::SERVICE_DISCONNECT_RESPONSE,
                            response.write_to_bytes().expect("Failed to write bytes"),
                        )),
                    )
                    .unwrap();
            })
            .unwrap();

        registry.disconnect("service_a").unwrap();
    }
}
