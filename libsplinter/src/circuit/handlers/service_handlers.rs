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
use crate::circuit::service::{Service, ServiceId, SplinterNode};
use crate::circuit::{ServiceDefinition, SplinterState};
use crate::network::dispatch::{DispatchError, Handler, MessageContext};
use crate::network::sender::SendRequest;
use crate::protos::circuit::{
    CircuitMessageType, ServiceConnectRequest, ServiceConnectResponse,
    ServiceConnectResponse_Status, ServiceDisconnectRequest, ServiceDisconnectResponse,
    ServiceDisconnectResponse_Status,
};
use crate::rwlock_write_unwrap;

use std::sync::{Arc, RwLock};

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
        let unique_id = ServiceId::new(circuit_name.to_string(), service_id.to_string());

        let mut response = ServiceConnectResponse::new();
        response.set_correlation_id(msg.get_correlation_id().into());
        response.set_circuit(circuit_name.into());
        response.set_service_id(service_id.into());

        // hold on to the write lock for the entirety of the function
        let mut state = rwlock_write_unwrap!(self.state);
        let circuit_result = state.circuit(circuit_name);
        if let Some(circuit) = circuit_result {
            // If the circuit has the service in its roster and the service is not yet connected
            // forward the connection to the rest of the nodes on the circuit and add the service
            // to splinter state
            if circuit.roster().contains(&service_id.to_string())
                && !state.service_directory.contains_key(&unique_id)
            {
                // This should never return None since we just checked if it exists.
                // If admin service create a service defination for the admin service
                let service = {
                    if !service_id.starts_with("admin::") {
                        circuit
                            .roster()
                            .iter()
                            .find(|service| service.service_id == service_id)
                            .expect("Cannot find service in circuit")
                            .clone()
                    } else {
                        ServiceDefinition::builder(service_id.into(), "admin".into())
                            .with_allowed_nodes(vec![self.node_id.to_string()])
                            .build()
                    }
                };

                if !service.allowed_nodes.contains(&self.node_id)
                {
                    response.set_status(ServiceConnectResponse_Status::ERROR_NOT_AN_ALLOWED_NODE);
                    response.set_error_message(format!("{} is not allowed on this node", unique_id))
                } else {
                    let node = SplinterNode::new(
                        self.node_id.to_string(),
                        vec![self.endpoint.to_string()],
                    );
                    let service = Service::new(
                        service_id.to_string(),
                        Some(context.source_peer_id().to_string()),
                        node,
                    );
                    state.add_service(unique_id, service);
                    response.set_status(ServiceConnectResponse_Status::OK);
                }
            // If the circuit exists and has the service in the roster but the service is already
            // connected, return an error response
            } else if circuit.roster().contains(&service_id.to_string())
                && state.service_directory.contains_key(&unique_id)
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
        let unique_id = ServiceId::new(circuit_name.to_string(), service_id.to_string());

        let mut response = ServiceDisconnectResponse::new();
        response.set_correlation_id(msg.get_correlation_id().into());
        response.set_circuit(circuit_name.into());
        response.set_service_id(service_id.into());

        // hold on to the write lock for the entirety of the function
        let mut state = rwlock_write_unwrap!(self.state);
        let circuit_result = state.circuit(circuit_name);
        if let Some(circuit) = circuit_result {
            // If the circuit has the service in its roster and the service is connected
            // forward the disconnection to the rest of the nodes on the circuit and remove the
            // service from splinter state
            if circuit.roster().contains(&service_id.to_string())
                && state.service_directory.contains_key(&unique_id)
            {
                state.remove_service(&unique_id);
                response.set_status(ServiceDisconnectResponse_Status::OK);
            // If the circuit exists and has the service in the roster but the service not
            // connected, return an error response
            } else if circuit.roster().contains(&service_id.to_string())
                && !state.service_directory.contains_key(&unique_id)
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
    use crate::circuit::{AuthorizationType, Circuit, DurabilityType, PersistenceType, RouteType};
    use crate::network::dispatch::Dispatcher;
    use crate::protos::circuit::CircuitMessage;
    use crate::protos::network::NetworkMessage;
    use crate::storage::get_storage;

    #[test]
    // Test that if the circuit does not exist, a ServiceConnectResponse is returned with
    // a ERROR_CIRCUIT_DOES_NOT_EXIST
    fn test_service_connect_request_handler_no_circuit() {
        let sender = Box::new(MockSender::default());
        let mut dispatcher = Dispatcher::new(sender.box_clone());

        let storage = get_storage("memory", CircuitDirectory::new).unwrap();
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
                "abc",
                &CircuitMessageType::SERVICE_CONNECT_REQUEST,
                connect_bytes.clone(),
            )
            .unwrap();
        let send_request = sender.sent().get(0).unwrap().clone();

        assert_eq!(send_request.recipient(), "abc");

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
    // Test that if the service is not in circuit, a ServiceConnectResponse is returned with
    // a ERROR_SERVICE_NOT_IN_CIRCUIT_REGISTRY
    fn test_service_connect_request_handler_not_in_circuit() {
        let sender = Box::new(MockSender::default());
        let mut dispatcher = Dispatcher::new(sender.box_clone());

        let circuit = build_circuit();

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
        connect_request.set_service_id("BAD".into());
        let connect_bytes = connect_request.write_to_bytes().unwrap();

        dispatcher
            .dispatch(
                "BAD",
                &CircuitMessageType::SERVICE_CONNECT_REQUEST,
                connect_bytes.clone(),
            )
            .unwrap();
        let send_request = sender.sent().get(0).unwrap().clone();

        assert_eq!(send_request.recipient(), "BAD");

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
        assert_eq!(connect_response.get_service_id(), "BAD");
        assert_eq!(
            connect_response.get_status(),
            ServiceConnectResponse_Status::ERROR_SERVICE_NOT_IN_CIRCUIT_REGISTRY
        );
    }

    #[test]
    // Test that if the service is in a circuit and not connected, a ServiceConnectResponse is
    // returned with an OK
    fn test_service_connect_request_handler() {
        let sender = Box::new(MockSender::default());
        let mut dispatcher = Dispatcher::new(sender.box_clone());

        let circuit = build_circuit();

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
        assert_eq!(send_requests.len(), 1);
        let send_request = send_requests.get(0).unwrap().clone();

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

        let id = ServiceId::new("alpha".into(), "abc".into());
        assert!(state.read().unwrap().service_directory().get(&id).is_some());
    }

    #[test]
    // Test that if the service is in a circuit and already connected, a ServiceConnectResponse is
    // returned with an ERROR_SERVICE_ALREADY_REGISTERED
    fn test_service_connect_request_handler_already_connected() {
        let sender = Box::new(MockSender::default());
        let mut dispatcher = Dispatcher::new(sender.box_clone());

        let circuit = build_circuit();

        let mut circuit_directory = CircuitDirectory::new();
        circuit_directory.add_circuit("alpha".to_string(), circuit);

        let state = Arc::new(RwLock::new(SplinterState::new(
            "memory".to_string(),
            circuit_directory,
        )));

        let node = SplinterNode::new("123".to_string(), vec!["123.0.0.1:0".to_string()]);
        let service = Service::new("abc".to_string(), Some("abc_network".to_string()), node);
        let id = ServiceId::new("alpha".into(), "abc".into());
        state.write().unwrap().add_service(id.clone(), service);
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
    // Test that if the circuit does not exist, a ServiceDisconnectResponse is returned with
    // a ERROR_CIRCUIT_DOES_NOT_EXIST
    fn test_service_disconnect_request_handler_no_circuit() {
        let sender = Box::new(MockSender::default());
        let mut dispatcher = Dispatcher::new(sender.box_clone());

        let storage = get_storage("memory", CircuitDirectory::new).unwrap();
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
                "abc",
                &CircuitMessageType::SERVICE_DISCONNECT_REQUEST,
                disconnect_bytes.clone(),
            )
            .unwrap();
        let send_request = sender.sent().get(0).unwrap().clone();

        assert_eq!(send_request.recipient(), "abc");

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
    // Test that if the service is not in circuit, a ServiceDisconnectResponse is returned with
    // a ERROR_SERVICE_NOT_IN_CIRCUIT_REGISTRY
    fn test_service_disconnect_request_handler_not_in_circuit() {
        let sender = Box::new(MockSender::default());
        let mut dispatcher = Dispatcher::new(sender.box_clone());

        let circuit = build_circuit();

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
        disconnect_request.set_service_id("BAD".into());
        let disconnect_bytes = disconnect_request.write_to_bytes().unwrap();

        dispatcher
            .dispatch(
                "BAD",
                &CircuitMessageType::SERVICE_DISCONNECT_REQUEST,
                disconnect_bytes.clone(),
            )
            .unwrap();
        let send_request = sender.sent().get(0).unwrap().clone();

        assert_eq!(send_request.recipient(), "BAD");

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
        assert_eq!(disconnect_response.get_service_id(), "BAD");
        assert_eq!(
            disconnect_response.get_status(),
            ServiceDisconnectResponse_Status::ERROR_SERVICE_NOT_IN_CIRCUIT_REGISTRY
        );
    }

    #[test]
    // Test that if the service is in a circuit and already connected, a ServiceDisconnectResponse
    // is returned with an OK.
    fn test_service_disconnect_request_handler() {
        let sender = Box::new(MockSender::default());
        let mut dispatcher = Dispatcher::new(sender.box_clone());

        let circuit = build_circuit();

        let mut circuit_directory = CircuitDirectory::new();
        circuit_directory.add_circuit("alpha".to_string(), circuit);

        let state = Arc::new(RwLock::new(SplinterState::new(
            "memory".to_string(),
            circuit_directory,
        )));

        let node = SplinterNode::new("123".to_string(), vec!["123.0.0.1:0".to_string()]);
        let service = Service::new("abc".to_string(), Some("abc_network".to_string()), node);
        let id = ServiceId::new("alpha".into(), "abc".into());
        state.write().unwrap().add_service(id.clone(), service);

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
        assert_eq!(send_requests.len(), 1);
        let send_request = send_requests.get(0).unwrap().clone();

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

        assert!(state.read().unwrap().service_directory().get(&id).is_none());
    }

    #[test]
    // Test that if the service is in a circuit and not connected, a ServiceDisconnectResponse
    // is returned with an ERROR_SERVICE_NOT_REGISTERED
    fn test_service_disconnect_request_handler_not_connected() {
        let sender = Box::new(MockSender::default());
        let mut dispatcher = Dispatcher::new(sender.box_clone());

        let circuit = build_circuit();

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

    fn build_circuit() -> Circuit {
        let service_abc = ServiceDefinition::builder("abc".into(), "test".into())
            .with_allowed_nodes(vec!["123".to_string()])
            .build();

        let service_def = ServiceDefinition::builder("def".into(), "test".into())
            .with_allowed_nodes(vec!["345".to_string()])
            .build();

        let circuit = Circuit::builder()
            .with_id("alpha".into())
            .with_auth(AuthorizationType::Trust)
            .with_members(vec!["123".into(), "345".into()])
            .with_roster(vec![service_abc, service_def])
            .with_persistence(PersistenceType::Any)
            .with_durability(DurabilityType::NoDurabilty)
            .with_routes(RouteType::Any)
            .with_circuit_management_type("service_connect_test_app".into())
            .build()
            .expect("Should have built a correct circuit");

        circuit
    }
}
