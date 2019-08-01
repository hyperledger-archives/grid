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
use std::fmt::Write;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use openssl::hash::{hash, MessageDigest};
use protobuf::{self, Message, RepeatedField};

use crate::actix_web::{web, Error as ActixError, HttpRequest, HttpResponse};
use crate::futures::{stream::Stream, Future, IntoFuture};
use crate::network::peer::PeerConnector;
use crate::protos::admin::{
    self, Circuit, CircuitCreateRequest, CircuitManagementPayload, CircuitManagementPayload_Action,
    CircuitProposal, CircuitProposal_ProposalType,
};
use crate::rest_api::{Method, Resource, RestResourceProvider};
use crate::service::{
    error::{ServiceDestroyError, ServiceError, ServiceStartError, ServiceStopError},
    Service, ServiceMessageContext, ServiceNetworkRegistry, ServiceNetworkSender,
};
use actix::prelude::*;
use actix_web_actors::ws;
use serde_json;

#[derive(Clone)]
pub struct AdminService {
    node_id: String,
    service_id: String,
    admin_service_state: Arc<Mutex<AdminServiceState>>,
}

impl AdminService {
    pub fn new(node_id: &str, peer_connector: PeerConnector) -> Self {
        Self {
            node_id: node_id.to_string(),
            service_id: admin_service_id(node_id),
            admin_service_state: Arc::new(Mutex::new(AdminServiceState {
                network_sender: None,
                open_proposals: Default::default(),
                peer_connector,
            })),
        }
    }
}

impl Service for AdminService {
    fn service_id(&self) -> &str {
        &self.service_id
    }

    fn service_type(&self) -> &str {
        "admin"
    }

    fn start(
        &mut self,
        service_registry: &dyn ServiceNetworkRegistry,
    ) -> Result<(), ServiceStartError> {
        let mut admin_service_state = self.admin_service_state.lock().map_err(|_| {
            ServiceStartError::PoisonedLock("the admin state lock was poisoned".into())
        })?;

        let network_sender = service_registry.connect(&self.service_id)?;

        admin_service_state.network_sender = Some(network_sender);

        Ok(())
    }

    fn stop(
        &mut self,
        service_registry: &dyn ServiceNetworkRegistry,
    ) -> Result<(), ServiceStopError> {
        let mut admin_service_state = self.admin_service_state.lock().map_err(|_| {
            ServiceStopError::PoisonedLock("the admin state lock was poisoned".into())
        })?;

        service_registry.disconnect(&self.service_id)?;

        admin_service_state.network_sender = None;

        Ok(())
    }

    fn destroy(self: Box<Self>) -> Result<(), ServiceDestroyError> {
        Ok(())
    }

    fn handle_message(
        &self,
        message_bytes: &[u8],
        _message_context: &ServiceMessageContext,
    ) -> Result<(), ServiceError> {
        let mut envelope: CircuitManagementPayload = protobuf::parse_from_bytes(message_bytes)
            .map_err(|err| ServiceError::InvalidMessageFormat(Box::new(err)))?;

        match envelope.action {
            CircuitManagementPayload_Action::CIRCUIT_CREATE_REQUEST => {
                let mut create_request = envelope.take_circuit_create_request();

                let proposed_circuit = create_request.take_circuit();
                let mut admin_service_state = self.admin_service_state.lock().map_err(|_| {
                    ServiceError::PoisonedLock("the admin state lock was poisoned".into())
                })?;

                if admin_service_state.has_proposal(proposed_circuit.get_circuit_id()) {
                    info!(
                        "Ignoring duplicate create proposal of circuit {}",
                        proposed_circuit.get_circuit_id()
                    );
                } else {
                    debug!("proposing {}", proposed_circuit.get_circuit_id());

                    let mut proposal = CircuitProposal::new();
                    proposal.set_proposal_type(CircuitProposal_ProposalType::CREATE);
                    proposal.set_circuit_id(proposed_circuit.get_circuit_id().into());
                    proposal.set_circuit_hash(sha256(&proposed_circuit)?);
                    proposal.set_circuit_proposal(proposed_circuit);

                    admin_service_state.add_proposal(proposal);
                }
            }
            unknown_action => {
                error!("Unable to handle {:?}", unknown_action);
            }
        }

        Ok(())
    }
}

impl AdminService {
    /// Propose a new circuit
    ///
    /// This operation will propose a new circuit to all the member nodes of the circuit.  If there
    /// is no peer connection, a connection to the peer will also be established.
    pub fn propose_circuit(&self, proposed_circuit: Circuit) -> Result<(), ServiceError> {
        let mut admin_service_state = self
            .admin_service_state
            .lock()
            .map_err(|_| ServiceError::PoisonedLock("the admin state lock was poisoned".into()))?;

        if admin_service_state.network_sender.is_none() {
            return Err(ServiceError::NotStarted);
        }

        let mut member_node_ids = vec![];
        for node in proposed_circuit.get_members() {
            if self.node_id != node.get_node_id() {
                admin_service_state
                    .peer_connector
                    .connect_peer(node.get_node_id(), node.get_endpoint())
                    .map_err(|err| ServiceError::UnableToHandleMessage(Box::new(err)))?;

                member_node_ids.push(node.get_node_id().to_string())
            }
        }

        debug!("proposing {}", proposed_circuit.get_circuit_id());

        let mut proposal = CircuitProposal::new();
        proposal.set_proposal_type(CircuitProposal_ProposalType::CREATE);
        proposal.set_circuit_id(proposed_circuit.get_circuit_id().into());
        proposal.set_circuit_hash(sha256(&proposed_circuit)?);
        proposal.set_circuit_proposal(proposed_circuit.clone());

        admin_service_state.add_proposal(proposal);

        let mut create_request = CircuitCreateRequest::new();
        create_request.set_circuit(proposed_circuit);

        let mut envelope = CircuitManagementPayload::new();
        envelope.set_action(CircuitManagementPayload_Action::CIRCUIT_CREATE_REQUEST);
        envelope.set_circuit_create_request(create_request);

        let envelope_bytes = envelope
            .write_to_bytes()
            .map_err(|err| ServiceError::InvalidMessageFormat(Box::new(err)))?;

        for member_id in member_node_ids {
            admin_service_state
                .network_sender
                .as_ref()
                .unwrap()
                .send(&admin_service_id(&member_id), &envelope_bytes)?;
        }

        Ok(())
    }
}

fn admin_service_id(node_id: &str) -> String {
    format!("admin::{}", node_id)
}

fn sha256(circuit: &Circuit) -> Result<String, ServiceError> {
    let bytes = circuit
        .write_to_bytes()
        .map_err(|err| ServiceError::UnableToHandleMessage(Box::new(err)))?;
    hash(MessageDigest::sha256(), &bytes)
        .map(|digest| to_hex(&*digest))
        .map_err(|err| ServiceError::UnableToHandleMessage(Box::new(err)))
}

fn to_hex(bytes: &[u8]) -> String {
    let mut buf = String::new();
    for b in bytes {
        write!(&mut buf, "{:0x}", b).expect("Unable to write to string");
    }

    buf
}

struct AdminServiceState {
    open_proposals: HashMap<String, CircuitProposal>,
    peer_connector: PeerConnector,
    network_sender: Option<Box<dyn ServiceNetworkSender>>,
}

impl AdminServiceState {
    fn add_proposal(&mut self, circuit_proposal: CircuitProposal) {
        let circuit_id = circuit_proposal.get_circuit_id().to_string();

        self.open_proposals.insert(circuit_id, circuit_proposal);
    }

    fn has_proposal(&self, circuit_id: &str) -> bool {
        self.open_proposals.contains_key(circuit_id)
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct CreateCircuit {
    circuit_id: String,
    roster: Vec<SplinterService>,
    members: Vec<SplinterNode>,
    authorization_type: AuthorizationType,
    persistence: PersistenceType,
    routes: RouteType,
    circuit_management_type: String,
    application_metadata: Vec<u8>,
}

impl CreateCircuit {
    fn from_payload(payload: web::Payload) -> impl Future<Item = Self, Error = ActixError> {
        payload
            .from_err()
            .fold(web::BytesMut::new(), move |mut body, chunk| {
                body.extend_from_slice(&chunk);
                Ok::<_, ActixError>(body)
            })
            .and_then(|body| {
                let proposal = serde_json::from_slice::<CreateCircuit>(&body).unwrap();
                Ok(proposal)
            })
            .into_future()
    }

    fn into_proto(self) -> Result<CircuitCreateRequest, ProposalMarshallingError> {
        let mut circuit = Circuit::new();

        circuit.set_circuit_id(self.circuit_id);
        circuit.set_roster(RepeatedField::from_vec(
            self.roster
                .into_iter()
                .map(SplinterService::into_proto)
                .collect(),
        ));
        circuit.set_members(RepeatedField::from_vec(
            self.members
                .into_iter()
                .map(SplinterNode::into_proto)
                .collect(),
        ));

        circuit.set_circuit_management_type(self.circuit_management_type);
        circuit.set_application_metadata(self.application_metadata);

        match self.authorization_type {
            AuthorizationType::Trust => {
                circuit
                    .set_authorization_type(admin::Circuit_AuthorizationType::TRUST_AUTHORIZATION);
            }
        };

        match self.persistence {
            PersistenceType::Any => {
                circuit.set_persistence(admin::Circuit_PersistenceType::ANY_PERSISTENCE);
            }
        };

        match self.routes {
            RouteType::Any => circuit.set_routes(admin::Circuit_RouteType::ANY_ROUTE),
        };

        let mut create_request = CircuitCreateRequest::new();
        create_request.set_circuit(circuit);

        Ok(create_request)
    }
}

#[derive(Serialize, Deserialize, Debug)]
enum AuthorizationType {
    Trust,
}

#[derive(Serialize, Deserialize, Debug)]
enum PersistenceType {
    Any,
}

#[derive(Serialize, Deserialize, Debug)]
enum RouteType {
    Any,
}

#[derive(Debug)]
struct ProposalMarshallingError;

impl std::error::Error for ProposalMarshallingError {}

impl std::fmt::Display for ProposalMarshallingError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("unable to marshal object")
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct SplinterNode {
    node_id: String,
    endpoint: String,
}

impl SplinterNode {
    fn into_proto(self) -> admin::SplinterNode {
        let mut proto = admin::SplinterNode::new();

        proto.set_node_id(self.node_id);
        proto.set_endpoint(self.endpoint);

        proto
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct SplinterService {
    service_id: String,
    service_type: String,
    allowed_nodes: Vec<String>,
}

impl SplinterService {
    fn into_proto(self) -> admin::SplinterService {
        let mut proto = admin::SplinterService::new();
        proto.set_service_id(self.service_id);
        proto.set_service_type(self.service_type);
        proto.set_allowed_nodes(RepeatedField::from_vec(self.allowed_nodes));

        proto
    }
}

impl RestResourceProvider for AdminService {
    fn resources(&self) -> Vec<Resource> {
        vec![
            make_create_circuit_route(self.clone()),
            make_application_handler_registration_route(),
        ]
    }
}

fn make_create_circuit_route(admin_service: AdminService) -> Resource {
    Resource::new(Method::Post, "/auth/circuit", move |r, p| {
        create_circuit(r, p, admin_service.clone())
    })
}

fn make_application_handler_registration_route() -> Resource {
    Resource::new(Method::Get, "/ws/auth/register/{type}", move |r, p| {
        let circuit_management_type = if let Some(t) = r.match_info().get("type") {
            t
        } else {
            return Box::new(HttpResponse::BadRequest().finish().into_future());
        };

        let res = ws::start(AdminServiceWebSocket::new(), &r, p);

        debug!("circuit management type {}", circuit_management_type);
        debug!("Websocket response: {:?}", res);
        Box::new(res.into_future())
    })
}

fn create_circuit(
    _req: HttpRequest,
    payload: web::Payload,
    admin_service: AdminService,
) -> Box<Future<Item = HttpResponse, Error = ActixError>> {
    Box::new(
        CreateCircuit::from_payload(payload).and_then(move |create_circuit| {
            let mut circuit_create_request = match create_circuit.into_proto() {
                Ok(request) => request,
                Err(_) => return Ok(HttpResponse::BadRequest().finish()),
            };
            let circuit = circuit_create_request.take_circuit();
            let circuit_id = circuit.circuit_id.clone();
            if let Err(err) = admin_service.propose_circuit(circuit) {
                error!("Unable to submit circuit {} proposal: {}", circuit_id, err);
                Ok(HttpResponse::BadRequest().finish())
            } else {
                debug!("Circuit {} proposed", circuit_id);
                Ok(HttpResponse::Accepted().finish())
            }
        }),
    )
}

pub struct AdminServiceWebSocket {}

impl AdminServiceWebSocket {
    fn new() -> Self {
        Self {}
    }

    fn push_updates(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(Duration::from_secs(3), |_, _| {
            debug!("Admin Service gonna send cool stuff");
        });
    }
}

impl Actor for AdminServiceWebSocket {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        debug!("Starting Admin Service");
        self.push_updates(ctx)
    }
}

impl StreamHandler<ws::Message, ws::ProtocolError> for AdminServiceWebSocket {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        debug!("WS: {:?}", msg);
        match msg {
            ws::Message::Ping(msg) => ctx.ping(&msg),
            ws::Message::Pong(msg) => ctx.pong(&msg),
            ws::Message::Text(text) => ctx.text(text),
            ws::Message::Binary(bin) => ctx.binary(bin),
            ws::Message::Close(_) => ctx.stop(),
            ws::Message::Nop => (),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::VecDeque;
    use std::sync::mpsc::{channel, Sender};

    use crate::mesh::Mesh;
    use crate::network::Network;
    use crate::protos::admin;
    use crate::service::{error, ServiceNetworkRegistry, ServiceNetworkSender};
    use crate::transport::{
        ConnectError, Connection, DisconnectError, RecvError, SendError, Transport,
    };

    /// Test that a circuit creation creates the correct connections and sends the appropriate
    /// messages.
    #[test]
    fn test_propose_circuit() {
        let mesh = Mesh::new(4, 16);
        let network = Network::new(mesh.clone());
        let transport =
            MockConnectingTransport::expect_connections(vec![Ok(Box::new(MockConnection))]);

        let peer_connector = PeerConnector::new(network.clone(), Box::new(transport));
        let mut admin_service = AdminService::new("test-node".into(), peer_connector);

        let (tx, rx) = channel();
        admin_service
            .start(&MockNetworkRegistry { tx })
            .expect("Service should have started correctly");

        let mut proposed_circuit = Circuit::new();
        proposed_circuit.set_circuit_id("test_propose_circuit".into());
        proposed_circuit
            .set_authorization_type(admin::Circuit_AuthorizationType::TRUST_AUTHORIZATION);
        proposed_circuit.set_persistence(admin::Circuit_PersistenceType::ANY_PERSISTENCE);
        proposed_circuit.set_routes(admin::Circuit_RouteType::ANY_ROUTE);
        proposed_circuit.set_circuit_management_type("test app auth handler".into());

        proposed_circuit.set_members(protobuf::RepeatedField::from_vec(vec![
            splinter_node("test-node", "tcp://someplace:8000"),
            splinter_node("other-node", "tcp://otherplace:8000"),
        ]));
        proposed_circuit.set_roster(protobuf::RepeatedField::from_vec(vec![
            splinter_service("service-a", "sabre"),
            splinter_service("service-b", "sabre"),
        ]));

        admin_service
            .propose_circuit(proposed_circuit.clone())
            .expect("The proposal was not handled correctly");

        let (recipient, message) = rx.try_recv().expect("A message should have been sent");
        assert_eq!("admin::other-node".to_string(), recipient);

        let mut envelope: CircuitManagementPayload =
            protobuf::parse_from_bytes(&message).expect("The message could not be parsed");
        assert_eq!(
            CircuitManagementPayload_Action::CIRCUIT_CREATE_REQUEST,
            envelope.get_action()
        );
        assert_eq!(
            proposed_circuit,
            envelope.take_circuit_create_request().take_circuit()
        );

        assert_eq!(Some(&"other-node".to_string()), network.peer_ids().get(0));
    }

    fn splinter_node(node_id: &str, endpoint: &str) -> admin::SplinterNode {
        let mut node = admin::SplinterNode::new();
        node.set_node_id(node_id.into());
        node.set_endpoint(endpoint.into());
        node
    }

    fn splinter_service(service_id: &str, service_type: &str) -> admin::SplinterService {
        let mut service = admin::SplinterService::new();
        service.set_service_id(service_id.into());
        service.set_service_type(service_type.into());
        service
    }

    struct MockNetworkRegistry {
        tx: Sender<(String, Vec<u8>)>,
    }

    impl ServiceNetworkRegistry for MockNetworkRegistry {
        fn connect(
            &self,
            _service_id: &str,
        ) -> Result<Box<dyn ServiceNetworkSender>, error::ServiceConnectionError> {
            Ok(Box::new(MockNetworkSender {
                tx: self.tx.clone(),
            }))
        }

        fn disconnect(&self, _service_id: &str) -> Result<(), error::ServiceDisconnectionError> {
            Ok(())
        }
    }

    struct MockNetworkSender {
        tx: Sender<(String, Vec<u8>)>,
    }

    impl ServiceNetworkSender for MockNetworkSender {
        fn send(&self, recipient: &str, message: &[u8]) -> Result<(), error::ServiceSendError> {
            self.tx
                .send((recipient.to_string(), message.to_vec()))
                .expect("Unable to send test message");

            Ok(())
        }

        fn send_and_await(
            &self,
            _recipient: &str,
            _message: &[u8],
        ) -> Result<Vec<u8>, error::ServiceSendError> {
            unimplemented!()
        }

        fn reply(
            &self,
            _message_origin: &ServiceMessageContext,
            _message: &[u8],
        ) -> Result<(), error::ServiceSendError> {
            unimplemented!()
        }

        fn clone_box(&self) -> Box<dyn ServiceNetworkSender> {
            unimplemented!()
        }
    }

    struct MockConnectingTransport {
        connection_results: VecDeque<Result<Box<dyn Connection>, ConnectError>>,
    }

    impl MockConnectingTransport {
        fn expect_connections(results: Vec<Result<Box<dyn Connection>, ConnectError>>) -> Self {
            Self {
                connection_results: results.into_iter().collect(),
            }
        }
    }

    impl Transport for MockConnectingTransport {
        fn accepts(&self, _: &str) -> bool {
            true
        }

        fn connect(&mut self, _: &str) -> Result<Box<dyn Connection>, ConnectError> {
            self.connection_results
                .pop_front()
                .expect("No test result added to mock")
        }

        fn listen(
            &mut self,
            _: &str,
        ) -> Result<Box<dyn crate::transport::Listener>, crate::transport::ListenError> {
            unimplemented!()
        }
    }

    struct MockConnection;

    impl Connection for MockConnection {
        fn send(&mut self, _message: &[u8]) -> Result<(), SendError> {
            Ok(())
        }

        fn recv(&mut self) -> Result<Vec<u8>, RecvError> {
            unimplemented!()
        }

        fn remote_endpoint(&self) -> String {
            String::from("MockConnection")
        }

        fn local_endpoint(&self) -> String {
            String::from("MockConnection")
        }

        fn disconnect(&mut self) -> Result<(), DisconnectError> {
            Ok(())
        }

        fn evented(&self) -> &dyn mio::Evented {
            &MockEvented
        }
    }

    struct MockEvented;

    impl mio::Evented for MockEvented {
        fn register(
            &self,
            _poll: &mio::Poll,
            _token: mio::Token,
            _interest: mio::Ready,
            _opts: mio::PollOpt,
        ) -> std::io::Result<()> {
            Ok(())
        }

        fn reregister(
            &self,
            _poll: &mio::Poll,
            _token: mio::Token,
            _interest: mio::Ready,
            _opts: mio::PollOpt,
        ) -> std::io::Result<()> {
            Ok(())
        }

        fn deregister(&self, _poll: &mio::Poll) -> std::io::Result<()> {
            Ok(())
        }
    }
}
