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
use crate::circuit::service::{Service, SplinterNode};
use crate::circuit::SplinterState;
use crate::network::dispatch::{DispatchError, Handler, MessageContext};
use crate::network::sender::SendRequest;
use crate::protos::circuit::{
    CircuitMessageType, ServiceConnectForward, ServiceConnectRequest, ServiceConnectResponse,
    ServiceConnectResponse_Status, ServiceDisconnectForward, ServiceDisconnectRequest,
    ServiceDisconnectResponse, ServiceDisconnectResponse_Status,
};
use crate::rwlock_write_unwrap;

use std::sync::{Arc, RwLock};

use ::log::{debug, log, warn};
use protobuf::Message;

// Implements a handler that handles ServiceConnectRequest
pub struct ServiceConnectRequestHandler {
    node_id: String,
    endpoint: String,
    state: Arc<RwLock<SplinterState>>,
}

impl Handler<CircuitMessageType, ServiceConnectRequest> for ServiceConnectRequestHandler {
    fn handle(
        &self,
        msg: ServiceConnectRequest,
        context: &MessageContext<CircuitMessageType>,
        sender: &dyn Sender<SendRequest>,
    ) -> Result<(), DispatchError> {
        debug!("Handle Service Connect Request {:?}", msg);
        let circuit_name = msg.get_circuit();
        let service_id = msg.get_service_id();

        let mut response = ServiceConnectResponse::new();
        response.set_circuit(circuit_name.into());
        response.set_service_id(service_id.into());

        // hold on to the write lock for the entirety of the function
        let mut state = rwlock_write_unwrap!(self.state);
        let circuit_result = state.circuit(circuit_name).clone();
        if let Some(circuit) = circuit_result {
            // If the circuit has the service in its roster and the service is not yet connected
            // forward the connection to the rest of the nodes on the circuit and add the service
            // to splinter state
            if circuit.roster().contains(&service_id.to_string())
                && !state
                    .service_directory
                    .contains_key(&service_id.to_string())
            {
                let mut forward_message = ServiceConnectForward::new();
                forward_message.set_circuit(circuit_name.into());
                forward_message.set_service_id(service_id.into());
                forward_message.set_node_id(self.node_id.to_string());
                forward_message.set_node_endpoint(self.endpoint.to_string());
                let forward_bytes = forward_message.write_to_bytes()?;
                let network_msg_bytes =
                    create_message(forward_bytes, CircuitMessageType::SERVICE_CONNECT_FORWARD)?;

                for member in circuit.members() {
                    if member != &self.node_id {
                        let send_request =
                            SendRequest::new(member.to_string(), network_msg_bytes.clone());
                        sender.send(send_request)?;
                    }
                }
                let node =
                    SplinterNode::new(self.node_id.to_string(), vec![self.endpoint.to_string()]);
                let service = Service::new(service_id.to_string(), node);
                state.add_service(service_id.to_string(), service);
                response.set_status(ServiceConnectResponse_Status::OK);
            // If the circuit exists and has the service in the roster but the service is already
            // connected, return an error response
            } else if circuit.roster().contains(&service_id.to_string())
                && state
                    .service_directory
                    .contains_key(&service_id.to_string())
            {
                response
                    .set_status(ServiceConnectResponse_Status::ERROR_SERVICE_ALREADY_REGISTERED);
                response.set_error_message(format!("Service is already registered: {}", service_id))
            // If the circuit exists but does not have the service in its roster, return an error
            // response
            } else {
                response.set_status(
                    ServiceConnectResponse_Status::ERROR_SERVICE_NOT_IN_CIRCUIT_REGISTRY,
                );
                response.set_error_message(format!(
                    "Service is not allowed in the circuit: {}:{}",
                    circuit_name, service_id
                ))
            }
        // If the circuit does not exists, return an error response
        } else {
            response.set_status(ServiceConnectResponse_Status::ERROR_CIRCUIT_DOES_NOT_EXIST);
            response.set_error_message(format!("Circuit does not exist: {}", msg.get_circuit()))
        }

        // Return response
        let response_bytes = response.write_to_bytes()?;
        let network_msg_bytes =
            create_message(response_bytes, CircuitMessageType::SERVICE_CONNECT_RESPONSE)?;

        let recipient = context.source_peer_id().to_string();
        let send_request = SendRequest::new(recipient, network_msg_bytes);
        sender.send(send_request)?;
        Ok(())
    }
}

impl ServiceConnectRequestHandler {
    pub fn new(node_id: String, endpoint: String, state: Arc<RwLock<SplinterState>>) -> Self {
        ServiceConnectRequestHandler {
            node_id,
            endpoint,
            state,
        }
    }
}

// Implements a handler that handles ServiceDisconnectRequest
pub struct ServiceDisconnectRequestHandler {
    node_id: String,
    state: Arc<RwLock<SplinterState>>,
}

impl Handler<CircuitMessageType, ServiceDisconnectRequest> for ServiceDisconnectRequestHandler {
    fn handle(
        &self,
        msg: ServiceDisconnectRequest,
        context: &MessageContext<CircuitMessageType>,
        sender: &dyn Sender<SendRequest>,
    ) -> Result<(), DispatchError> {
        debug!("Handle Service Disconnect Request {:?}", msg);
        let circuit_name = msg.get_circuit();
        let service_id = msg.get_service_id();

        let mut response = ServiceDisconnectResponse::new();
        response.set_circuit(circuit_name.into());
        response.set_service_id(service_id.into());

        // hold on to the write lock for the entirety of the function
        let mut state = rwlock_write_unwrap!(self.state);
        let circuit_result = state.circuit(circuit_name).clone();
        if let Some(circuit) = circuit_result {
            // If the circuit has the service in its roster and the service is connected
            // forward the disconnection to the rest of the nodes on the circuit and remove the
            // service from splinter state
            if circuit.roster().contains(&service_id.to_string())
                && state
                    .service_directory
                    .contains_key(&service_id.to_string())
            {
                let mut forward_message = ServiceDisconnectForward::new();
                forward_message.set_circuit(circuit_name.into());
                forward_message.set_service_id(service_id.into());
                forward_message.set_node_id(self.node_id.to_string());
                let forward_bytes = forward_message.write_to_bytes()?;
                let network_msg_bytes = create_message(
                    forward_bytes,
                    CircuitMessageType::SERVICE_DISCONNECT_FORWARD,
                )?;

                for member in circuit.members() {
                    if member != &self.node_id {
                        let send_request =
                            SendRequest::new(member.to_string(), network_msg_bytes.clone());
                        sender.send(send_request)?;
                    }
                }

                state.remove_service(service_id);
                response.set_status(ServiceDisconnectResponse_Status::OK);
            // If the circuit exists and has the service in the roster but the service not
            // connected, return an error response
            } else if circuit.roster().contains(&service_id.to_string())
                && !state
                    .service_directory
                    .contains_key(&service_id.to_string())
            {
                response.set_status(ServiceDisconnectResponse_Status::ERROR_SERVICE_NOT_REGISTERED);
                response.set_error_message(format!("Service is not registered: {}", service_id))
            // If the circuit exists but does not have the service in its roster, return an error
            // response
            } else {
                response.set_status(
                    ServiceDisconnectResponse_Status::ERROR_SERVICE_NOT_IN_CIRCUIT_REGISTRY,
                );
                response.set_error_message(format!(
                    "Service is not allowed in the circuit: {}:{}",
                    circuit_name, service_id
                ))
            }
        // If the circuit does not exists, return an error response
        } else {
            response.set_status(ServiceDisconnectResponse_Status::ERROR_CIRCUIT_DOES_NOT_EXIST);
            response.set_error_message(format!("Circuit does not exist: {}", msg.get_circuit()))
        }

        // Return response
        let response_bytes = response.write_to_bytes()?;
        let network_msg_bytes = create_message(
            response_bytes,
            CircuitMessageType::SERVICE_DISCONNECT_RESPONSE,
        )?;

        let recipient = context.source_peer_id().to_string();
        let send_request = SendRequest::new(recipient, network_msg_bytes);
        sender.send(send_request)?;
        Ok(())
    }
}

impl ServiceDisconnectRequestHandler {
    pub fn new(node_id: String, state: Arc<RwLock<SplinterState>>) -> Self {
        ServiceDisconnectRequestHandler { node_id, state }
    }
}

// Implements a handler that handles NetworkEcho Messages
pub struct ServiceConnectForwardHandler {
    state: Arc<RwLock<SplinterState>>,
}

impl Handler<CircuitMessageType, ServiceConnectForward> for ServiceConnectForwardHandler {
    fn handle(
        &self,
        msg: ServiceConnectForward,
        _: &MessageContext<CircuitMessageType>,
        _: &dyn Sender<SendRequest>,
    ) -> Result<(), DispatchError> {
        debug!("Handle Service Connect Forward {:?}", msg);
        let circuit_name = msg.get_circuit();
        let service_id = msg.get_service_id();
        let node_id = msg.get_node_id();
        let node_endpoint = msg.get_node_endpoint();

        // hold on to the write lock for the entirety of the function
        let mut state = rwlock_write_unwrap!(self.state);
        let circuit_result = state.circuit(circuit_name).clone();
        if let Some(circuit) = circuit_result {
            // If the circuit has the service in its roster and the service is not yet connected
            // add the service to splinter state. Otherwise return
            if circuit.roster().contains(&service_id.to_string())
                && !state
                    .service_directory
                    .contains_key(&service_id.to_string())
            {
                let node = SplinterNode::new(node_id.to_string(), vec![node_endpoint.to_string()]);
                let service = Service::new(service_id.to_string(), node);
                state.add_service(service_id.to_string(), service);
            } else if circuit.roster().contains(&service_id.to_string())
                && state
                    .service_directory
                    .contains_key(&service_id.to_string())
            {
                warn!("Service is already registered: {}", service_id);
            } else {
                warn!(
                    "Service is not allowed in the circuit: {}:{}",
                    circuit_name, service_id
                );
            }
        } else {
            warn!("Circuit does not exist: {}", circuit_name);
        }

        Ok(())
    }
}

impl ServiceConnectForwardHandler {
    pub fn new(state: Arc<RwLock<SplinterState>>) -> Self {
        ServiceConnectForwardHandler { state }
    }
}

// Implements a handler that handles ServiceDisconnectForward Messages
pub struct ServiceDisconnectForwardHandler {
    state: Arc<RwLock<SplinterState>>,
}

impl Handler<CircuitMessageType, ServiceDisconnectForward> for ServiceDisconnectForwardHandler {
    fn handle(
        &self,
        msg: ServiceDisconnectForward,
        _: &MessageContext<CircuitMessageType>,
        _: &dyn Sender<SendRequest>,
    ) -> Result<(), DispatchError> {
        debug!("Handle Service Connect Forward {:?}", msg);
        let circuit_name = msg.get_circuit();
        let service_id = msg.get_service_id();

        // hold on to the write lock for the entirety of the function
        let mut state = rwlock_write_unwrap!(self.state);
        let circuit_result = state.circuit(circuit_name).clone();
        if let Some(circuit) = circuit_result {
            // If the circuit has the service in its roster and the service is connected
            // remove the service from splinter state. Otherwise return
            if circuit.roster().contains(&service_id.to_string())
                && state
                    .service_directory
                    .contains_key(&service_id.to_string())
            {
                state.remove_service(service_id);
            } else if circuit.roster().contains(&service_id.to_string())
                && !state
                    .service_directory
                    .contains_key(&service_id.to_string())
            {
                warn!("Service not registered: {}", service_id);
            } else {
                warn!(
                    "Service is not allowed in the circuit: {}:{}",
                    circuit_name, service_id
                );
            }
        } else {
            warn!("Circuit does not exist: {}", circuit_name);
        }

        Ok(())
    }
}

impl ServiceDisconnectForwardHandler {
    pub fn new(state: Arc<RwLock<SplinterState>>) -> Self {
        ServiceDisconnectForwardHandler { state }
    }
}

impl From<protobuf::error::ProtobufError> for DispatchError {
    fn from(e: protobuf::error::ProtobufError) -> Self {
        DispatchError::SerializationError(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::channel::mock::MockSender;
    use crate::channel::Sender;
    use crate::circuit::directory::CircuitDirectory;
    use crate::circuit::Circuit;
    use crate::network::dispatch::Dispatcher;
    use crate::protos::circuit::CircuitMessage;
    use crate::protos::network::NetworkMessage;
    use crate::storage::get_storage;

    #[test]
    fn test_service_request_handler_no_circuit() {
        let sender = Box::new(MockSender::default());
        let mut dispatcher = Dispatcher::new(sender.box_clone());

        let storage = get_storage("memory", || CircuitDirectory::new()).unwrap();
        let circuit_directory = storage.read().clone();
        let state = Arc::new(RwLock::new(SplinterState::new(
            "memory".to_string(),
            circuit_directory,
        )));
        let handler =
            ServiceConnectRequestHandler::new("123".to_string(), "127.0.0.1:0".to_string(), state);

        dispatcher.set_handler(
            CircuitMessageType::SERVICE_CONNECT_REQUEST,
            Box::new(handler),
        );
        let mut connect_request = ServiceConnectRequest::new();
        connect_request.set_circuit("alpha".into());
        connect_request.set_service_id("abc".into());
        let connect_bytes = connect_request.write_to_bytes().unwrap();

        dispatcher
            .dispatch(
                "PEER",
                &CircuitMessageType::SERVICE_CONNECT_REQUEST,
                connect_bytes.clone(),
            )
            .unwrap();
        let send_request = sender.sent().get(0).unwrap().clone();

        assert_eq!(send_request.recipient(), "PEER");

        let network_msg: NetworkMessage =
            protobuf::parse_from_bytes(send_request.payload()).unwrap();
        let circuit_msg: CircuitMessage =
            protobuf::parse_from_bytes(network_msg.get_payload()).unwrap();
        let connect_response: ServiceConnectResponse =
            protobuf::parse_from_bytes(circuit_msg.get_payload()).unwrap();

        assert_eq!(
            circuit_msg.get_message_type(),
            CircuitMessageType::SERVICE_CONNECT_RESPONSE
        );
        assert_eq!(connect_response.get_circuit(), "alpha");
        assert_eq!(connect_response.get_service_id(), "abc");
        assert_eq!(
            connect_response.get_status(),
            ServiceConnectResponse_Status::ERROR_CIRCUIT_DOES_NOT_EXIST
        );
    }

    #[test]
    fn test_service_request_handler_not_in_circuit() {
        let sender = Box::new(MockSender::default());
        let mut dispatcher = Dispatcher::new(sender.box_clone());

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
        let handler =
            ServiceConnectRequestHandler::new("123".to_string(), "127.0.0.1:0".to_string(), state);

        dispatcher.set_handler(
            CircuitMessageType::SERVICE_CONNECT_REQUEST,
            Box::new(handler),
        );
        let mut connect_request = ServiceConnectRequest::new();
        connect_request.set_circuit("alpha".into());
        connect_request.set_service_id("ABC".into());
        let connect_bytes = connect_request.write_to_bytes().unwrap();

        dispatcher
            .dispatch(
                "PEER",
                &CircuitMessageType::SERVICE_CONNECT_REQUEST,
                connect_bytes.clone(),
            )
            .unwrap();
        let send_request = sender.sent().get(0).unwrap().clone();

        assert_eq!(send_request.recipient(), "PEER");

        let network_msg: NetworkMessage =
            protobuf::parse_from_bytes(send_request.payload()).unwrap();
        let circuit_msg: CircuitMessage =
            protobuf::parse_from_bytes(network_msg.get_payload()).unwrap();
        let connect_response: ServiceConnectResponse =
            protobuf::parse_from_bytes(circuit_msg.get_payload()).unwrap();

        assert_eq!(
            circuit_msg.get_message_type(),
            CircuitMessageType::SERVICE_CONNECT_RESPONSE
        );
        assert_eq!(connect_response.get_circuit(), "alpha");
        assert_eq!(connect_response.get_service_id(), "ABC");
        assert_eq!(
            connect_response.get_status(),
            ServiceConnectResponse_Status::ERROR_SERVICE_NOT_IN_CIRCUIT_REGISTRY
        );
    }

    #[test]
    fn test_service_request_handler() {
        let sender = Box::new(MockSender::default());
        let mut dispatcher = Dispatcher::new(sender.box_clone());

        let circuit = Circuit::new(
            "alpha".into(),
            "trust".into(),
            vec!["123".into(), "345".into()],
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
        let handler = ServiceConnectRequestHandler::new(
            "123".to_string(),
            "127.0.0.1:0".to_string(),
            state.clone(),
        );

        dispatcher.set_handler(
            CircuitMessageType::SERVICE_CONNECT_REQUEST,
            Box::new(handler),
        );
        let mut connect_request = ServiceConnectRequest::new();
        connect_request.set_circuit("alpha".into());
        connect_request.set_service_id("abc".into());
        let connect_bytes = connect_request.write_to_bytes().unwrap();

        dispatcher
            .dispatch(
                "PEER",
                &CircuitMessageType::SERVICE_CONNECT_REQUEST,
                connect_bytes.clone(),
            )
            .unwrap();
        let send_requests = sender.sent();
        assert_eq!(send_requests.len(), 2);
        let send_request = send_requests.get(0).unwrap().clone();
        let network_msg: NetworkMessage =
            protobuf::parse_from_bytes(send_request.payload()).unwrap();
        let circuit_msg: CircuitMessage =
            protobuf::parse_from_bytes(network_msg.get_payload()).unwrap();
        let forward_connect: ServiceConnectForward =
            protobuf::parse_from_bytes(circuit_msg.get_payload()).unwrap();
        assert_eq!(
            circuit_msg.get_message_type(),
            CircuitMessageType::SERVICE_CONNECT_FORWARD
        );
        assert_eq!(forward_connect.get_circuit(), "alpha");
        assert_eq!(forward_connect.get_service_id(), "abc");
        assert_eq!(forward_connect.get_node_id(), "123");

        let send_request = send_requests.get(1).unwrap().clone();

        assert_eq!(send_request.recipient(), "PEER");

        let network_msg: NetworkMessage =
            protobuf::parse_from_bytes(send_request.payload()).unwrap();
        let circuit_msg: CircuitMessage =
            protobuf::parse_from_bytes(network_msg.get_payload()).unwrap();
        let connect_response: ServiceConnectResponse =
            protobuf::parse_from_bytes(circuit_msg.get_payload()).unwrap();

        assert_eq!(
            circuit_msg.get_message_type(),
            CircuitMessageType::SERVICE_CONNECT_RESPONSE
        );
        assert_eq!(connect_response.get_circuit(), "alpha");
        assert_eq!(connect_response.get_service_id(), "abc");
        assert_eq!(
            connect_response.get_status(),
            ServiceConnectResponse_Status::OK
        );

        assert!(state
            .read()
            .unwrap()
            .service_directory()
            .get("abc")
            .is_some());
    }

    #[test]
    fn test_service_request_handler_already_connected() {
        let sender = Box::new(MockSender::default());
        let mut dispatcher = Dispatcher::new(sender.box_clone());

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

        let node = SplinterNode::new("123".to_string(), vec!["123.0.0.1:0".to_string()]);
        let service = Service::new("abc".to_string(), node);
        state
            .write()
            .unwrap()
            .add_service("abc".to_string(), service);
        let handler =
            ServiceConnectRequestHandler::new("123".to_string(), "127.0.0.1:0".to_string(), state);

        dispatcher.set_handler(
            CircuitMessageType::SERVICE_CONNECT_REQUEST,
            Box::new(handler),
        );
        let mut connect_request = ServiceConnectRequest::new();
        connect_request.set_circuit("alpha".into());
        connect_request.set_service_id("abc".into());
        let connect_bytes = connect_request.write_to_bytes().unwrap();

        dispatcher
            .dispatch(
                "PEER",
                &CircuitMessageType::SERVICE_CONNECT_REQUEST,
                connect_bytes.clone(),
            )
            .unwrap();
        let send_request = sender.sent().get(0).unwrap().clone();

        assert_eq!(send_request.recipient(), "PEER");

        let network_msg: NetworkMessage =
            protobuf::parse_from_bytes(send_request.payload()).unwrap();
        let circuit_msg: CircuitMessage =
            protobuf::parse_from_bytes(network_msg.get_payload()).unwrap();
        let connect_response: ServiceConnectResponse =
            protobuf::parse_from_bytes(circuit_msg.get_payload()).unwrap();

        assert_eq!(
            circuit_msg.get_message_type(),
            CircuitMessageType::SERVICE_CONNECT_RESPONSE
        );
        assert_eq!(connect_response.get_circuit(), "alpha");
        assert_eq!(connect_response.get_service_id(), "abc");
        assert_eq!(
            connect_response.get_status(),
            ServiceConnectResponse_Status::ERROR_SERVICE_ALREADY_REGISTERED
        );
    }

    #[test]
    fn test_service_connect_forward_handler() {
        let sender = Box::new(MockSender::default());
        let mut dispatcher = Dispatcher::new(sender.box_clone());

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
        let handler = ServiceConnectForwardHandler::new(state.clone());

        dispatcher.set_handler(
            CircuitMessageType::SERVICE_CONNECT_FORWARD,
            Box::new(handler),
        );
        let mut connect_request = ServiceConnectForward::new();
        connect_request.set_circuit("alpha".into());
        connect_request.set_service_id("abc".into());
        connect_request.set_node_id("123".into());
        connect_request.set_node_endpoint("127.0.0.1:0".into());
        let connect_bytes = connect_request.write_to_bytes().unwrap();

        dispatcher
            .dispatch(
                "PEER",
                &CircuitMessageType::SERVICE_CONNECT_FORWARD,
                connect_bytes.clone(),
            )
            .unwrap();

        assert!(state
            .read()
            .unwrap()
            .service_directory()
            .get("abc")
            .is_some());
    }

    #[test]
    fn test_service_disconnect_request_handler_no_circuit() {
        let sender = Box::new(MockSender::default());
        let mut dispatcher = Dispatcher::new(sender.box_clone());

        let storage = get_storage("memory", || CircuitDirectory::new()).unwrap();
        let circuit_directory = storage.read().clone();
        let state = Arc::new(RwLock::new(SplinterState::new(
            "memory".to_string(),
            circuit_directory,
        )));
        let handler = ServiceDisconnectRequestHandler::new("123".to_string(), state);

        dispatcher.set_handler(
            CircuitMessageType::SERVICE_DISCONNECT_REQUEST,
            Box::new(handler),
        );
        let mut disconnect_request = ServiceDisconnectRequest::new();
        disconnect_request.set_circuit("alpha".into());
        disconnect_request.set_service_id("abc".into());
        let disconnect_bytes = disconnect_request.write_to_bytes().unwrap();

        dispatcher
            .dispatch(
                "PEER",
                &CircuitMessageType::SERVICE_DISCONNECT_REQUEST,
                disconnect_bytes.clone(),
            )
            .unwrap();
        let send_request = sender.sent().get(0).unwrap().clone();

        assert_eq!(send_request.recipient(), "PEER");

        let network_msg: NetworkMessage =
            protobuf::parse_from_bytes(send_request.payload()).unwrap();
        let circuit_msg: CircuitMessage =
            protobuf::parse_from_bytes(network_msg.get_payload()).unwrap();
        let disconnect_response: ServiceDisconnectResponse =
            protobuf::parse_from_bytes(circuit_msg.get_payload()).unwrap();

        assert_eq!(
            circuit_msg.get_message_type(),
            CircuitMessageType::SERVICE_DISCONNECT_RESPONSE
        );
        assert_eq!(disconnect_response.get_circuit(), "alpha");
        assert_eq!(disconnect_response.get_service_id(), "abc");
        assert_eq!(
            disconnect_response.get_status(),
            ServiceDisconnectResponse_Status::ERROR_CIRCUIT_DOES_NOT_EXIST
        );
    }

    #[test]
    fn test_service_disconnect_request_handler_not_in_circuit() {
        let sender = Box::new(MockSender::default());
        let mut dispatcher = Dispatcher::new(sender.box_clone());

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
        let handler = ServiceDisconnectRequestHandler::new("123".to_string(), state);

        dispatcher.set_handler(
            CircuitMessageType::SERVICE_DISCONNECT_REQUEST,
            Box::new(handler),
        );
        let mut disconnect_request = ServiceDisconnectRequest::new();
        disconnect_request.set_circuit("alpha".into());
        disconnect_request.set_service_id("ABC".into());
        let disconnect_bytes = disconnect_request.write_to_bytes().unwrap();

        dispatcher
            .dispatch(
                "PEER",
                &CircuitMessageType::SERVICE_DISCONNECT_REQUEST,
                disconnect_bytes.clone(),
            )
            .unwrap();
        let send_request = sender.sent().get(0).unwrap().clone();

        assert_eq!(send_request.recipient(), "PEER");

        let network_msg: NetworkMessage =
            protobuf::parse_from_bytes(send_request.payload()).unwrap();
        let circuit_msg: CircuitMessage =
            protobuf::parse_from_bytes(network_msg.get_payload()).unwrap();
        let disconnect_response: ServiceDisconnectResponse =
            protobuf::parse_from_bytes(circuit_msg.get_payload()).unwrap();

        assert_eq!(
            circuit_msg.get_message_type(),
            CircuitMessageType::SERVICE_DISCONNECT_RESPONSE
        );
        assert_eq!(disconnect_response.get_circuit(), "alpha");
        assert_eq!(disconnect_response.get_service_id(), "ABC");
        assert_eq!(
            disconnect_response.get_status(),
            ServiceDisconnectResponse_Status::ERROR_SERVICE_NOT_IN_CIRCUIT_REGISTRY
        );
    }

    #[test]
    fn test_service_disconnect_request_handler() {
        let sender = Box::new(MockSender::default());
        let mut dispatcher = Dispatcher::new(sender.box_clone());

        let circuit = Circuit::new(
            "alpha".into(),
            "trust".into(),
            vec!["123".into(), "345".into()],
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

        let node = SplinterNode::new("123".to_string(), vec!["123.0.0.1:0".to_string()]);
        let service = Service::new("abc".to_string(), node);
        state
            .write()
            .unwrap()
            .add_service("abc".to_string(), service);

        let handler = ServiceDisconnectRequestHandler::new("123".to_string(), state.clone());

        dispatcher.set_handler(
            CircuitMessageType::SERVICE_DISCONNECT_REQUEST,
            Box::new(handler),
        );
        let mut disconnect_request = ServiceDisconnectRequest::new();
        disconnect_request.set_circuit("alpha".into());
        disconnect_request.set_service_id("abc".into());
        let disconnect_bytes = disconnect_request.write_to_bytes().unwrap();

        dispatcher
            .dispatch(
                "PEER",
                &CircuitMessageType::SERVICE_DISCONNECT_REQUEST,
                disconnect_bytes.clone(),
            )
            .unwrap();
        let send_requests = sender.sent();
        assert_eq!(send_requests.len(), 2);
        let send_request = send_requests.get(0).unwrap().clone();
        let network_msg: NetworkMessage =
            protobuf::parse_from_bytes(send_request.payload()).unwrap();
        let circuit_msg: CircuitMessage =
            protobuf::parse_from_bytes(network_msg.get_payload()).unwrap();
        let forward_connect: ServiceDisconnectForward =
            protobuf::parse_from_bytes(circuit_msg.get_payload()).unwrap();
        assert_eq!(
            circuit_msg.get_message_type(),
            CircuitMessageType::SERVICE_DISCONNECT_FORWARD
        );
        assert_eq!(forward_connect.get_circuit(), "alpha");
        assert_eq!(forward_connect.get_service_id(), "abc");
        assert_eq!(forward_connect.get_node_id(), "123");

        let send_request = send_requests.get(1).unwrap().clone();

        assert_eq!(send_request.recipient(), "PEER");

        let network_msg: NetworkMessage =
            protobuf::parse_from_bytes(send_request.payload()).unwrap();
        let circuit_msg: CircuitMessage =
            protobuf::parse_from_bytes(network_msg.get_payload()).unwrap();
        let disconnect_response: ServiceDisconnectResponse =
            protobuf::parse_from_bytes(circuit_msg.get_payload()).unwrap();

        assert_eq!(
            circuit_msg.get_message_type(),
            CircuitMessageType::SERVICE_DISCONNECT_RESPONSE
        );
        assert_eq!(disconnect_response.get_circuit(), "alpha");
        assert_eq!(disconnect_response.get_service_id(), "abc");
        assert_eq!(
            disconnect_response.get_status(),
            ServiceDisconnectResponse_Status::OK
        );

        assert!(state
            .read()
            .unwrap()
            .service_directory()
            .get("abc")
            .is_none());
    }

    #[test]
    fn test_service_disconnect_request_handler_not_connected() {
        let sender = Box::new(MockSender::default());
        let mut dispatcher = Dispatcher::new(sender.box_clone());

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

        let handler = ServiceDisconnectRequestHandler::new("123".to_string(), state);

        dispatcher.set_handler(
            CircuitMessageType::SERVICE_DISCONNECT_REQUEST,
            Box::new(handler),
        );
        let mut disconnect_request = ServiceDisconnectRequest::new();
        disconnect_request.set_circuit("alpha".into());
        disconnect_request.set_service_id("abc".into());
        let disconnect_bytes = disconnect_request.write_to_bytes().unwrap();

        dispatcher
            .dispatch(
                "PEER",
                &CircuitMessageType::SERVICE_DISCONNECT_REQUEST,
                disconnect_bytes.clone(),
            )
            .unwrap();
        let send_request = sender.sent().get(0).unwrap().clone();

        assert_eq!(send_request.recipient(), "PEER");

        let network_msg: NetworkMessage =
            protobuf::parse_from_bytes(send_request.payload()).unwrap();
        let circuit_msg: CircuitMessage =
            protobuf::parse_from_bytes(network_msg.get_payload()).unwrap();
        let disconnect_response: ServiceDisconnectResponse =
            protobuf::parse_from_bytes(circuit_msg.get_payload()).unwrap();

        assert_eq!(
            circuit_msg.get_message_type(),
            CircuitMessageType::SERVICE_DISCONNECT_RESPONSE
        );
        assert_eq!(disconnect_response.get_circuit(), "alpha");
        assert_eq!(disconnect_response.get_service_id(), "abc");
        assert_eq!(
            disconnect_response.get_status(),
            ServiceDisconnectResponse_Status::ERROR_SERVICE_NOT_REGISTERED
        );
    }

    #[test]
    fn test_service_disconnect_forward_handler() {
        let sender = Box::new(MockSender::default());
        let mut dispatcher = Dispatcher::new(sender.box_clone());

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

        let node = SplinterNode::new("123".to_string(), vec!["123.0.0.1:0".to_string()]);
        let service = Service::new("abc".to_string(), node);
        state
            .write()
            .unwrap()
            .add_service("abc".to_string(), service);

        let handler = ServiceDisconnectForwardHandler::new(state.clone());

        dispatcher.set_handler(
            CircuitMessageType::SERVICE_DISCONNECT_FORWARD,
            Box::new(handler),
        );
        let mut disconnect_request = ServiceDisconnectForward::new();
        disconnect_request.set_circuit("alpha".into());
        disconnect_request.set_service_id("abc".into());
        disconnect_request.set_node_id("123".into());
        let disconnect_bytes = disconnect_request.write_to_bytes().unwrap();

        dispatcher
            .dispatch(
                "PEER",
                &CircuitMessageType::SERVICE_DISCONNECT_FORWARD,
                disconnect_bytes.clone(),
            )
            .unwrap();

        assert!(state
            .read()
            .unwrap()
            .service_directory()
            .get("abc")
            .is_none());
    }
}
