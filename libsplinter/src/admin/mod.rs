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

use openssl::hash::{hash, MessageDigest};
use protobuf::{self, Message};

use crate::actix_web::{web, Error as ActixError, HttpRequest, HttpResponse};
use crate::futures::{stream::Stream, Future, IntoFuture};
use crate::network::peer::PeerConnector;
use crate::protos::admin::{
    Circuit, CircuitManagementPayload, CircuitManagementPayload_Action, CircuitProposal,
    CircuitProposal_ProposalType,
};
use crate::rest_api::{Method, Resource, RestResourceProvider};
use crate::service::{
    error::{ServiceDestroyError, ServiceError, ServiceStartError, ServiceStopError},
    Service, ServiceMessageContext, ServiceNetworkRegistry, ServiceNetworkSender,
};
use serde_json;

#[derive(Clone)]
pub struct AdminService {
    service_id: String,
    network_sender: Option<Box<dyn ServiceNetworkSender>>,
    admin_service_state: Arc<Mutex<AdminServiceState>>,
}

impl AdminService {
    pub fn new(node_id: &str, peer_connector: PeerConnector) -> Self {
        Self {
            service_id: format!("admin::{}", node_id),
            network_sender: None,
            admin_service_state: Arc::new(Mutex::new(AdminServiceState {
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
        let network_sender = service_registry
            .connect(&self.service_id)
            .map_err(|err| ServiceStartError(Box::new(err)))?;

        self.network_sender = Some(network_sender);

        Ok(())
    }

    fn stop(
        &mut self,
        service_registry: &dyn ServiceNetworkRegistry,
    ) -> Result<(), ServiceStopError> {
        service_registry
            .disconnect(&self.service_id)
            .map_err(|err| ServiceStopError(Box::new(err)))?;

        self.network_sender = None;

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
        if self.network_sender.is_none() {
            return Err(ServiceError::NotStarted);
        }

        let mut envelope: CircuitManagementPayload = protobuf::parse_from_bytes(message_bytes)
            .map_err(|err| ServiceError::InvalidMessageFormat(Box::new(err)))?;

        match envelope.action {
            CircuitManagementPayload_Action::CIRCUIT_CREATE_REQUEST => {
                let mut create_request = envelope.take_circuit_create_request();

                let proposed_circuit = create_request.take_circuit();
                let mut admin_service_state = self
                    .admin_service_state
                    .lock()
                    .expect("the admin state lock was poisoned");

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
}

#[derive(Serialize, Deserialize, Debug)]
enum AuthorizationType {
    TRUST_AUTHORIZATION,
}

#[derive(Serialize, Deserialize, Debug)]
enum PersistenceType {
    ANY_PERSISTENCE,
}

#[derive(Serialize, Deserialize, Debug)]
enum RouteType {
    ANY_ROUTE,
}

enum ProposalMarshallingError {
    InvalidAuthorizationType,
    InvalidRouteType,
    InvalidPersistenceType,
    InvalidDurabilityType,
    ServiceError(ServiceError),
}

impl From<ServiceError> for ProposalMarshallingError {
    fn from(err: ServiceError) -> Self {
        ProposalMarshallingError::ServiceError(err)
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct SplinterNode {
    node_id: String,
    endpoint: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct SplinterService {
    service_id: String,
    service_type: String,
    allowed_nodes: Vec<String>,
}

impl RestResourceProvider for AdminService {
    fn resources(&self) -> Vec<Resource> {
        vec![
            make_create_circuit_route(),
            make_application_handler_registration_route(),
        ]
    }
}

fn make_create_circuit_route() -> Resource {
    Resource::new(Method::Post, "/auth/circuit", move |r, p| {
        create_circuit(r, p)
    })
}

fn make_application_handler_registration_route() -> Resource {
    Resource::new(Method::Put, "/auth/register/{type}", move |r, _| {
        let circuit_management_type = if let Some(t) = r.match_info().get("type") {
            t
        } else {
            return Box::new(HttpResponse::BadRequest().finish().into_future());
        };

        debug!("circuit management type {}", circuit_management_type);
        Box::new(HttpResponse::Ok().finish().into_future())
    })
}

fn create_circuit(
    req: HttpRequest,
    payload: web::Payload,
) -> Box<Future<Item = HttpResponse, Error = ActixError>> {
    Box::new(CreateCircuit::from_payload(payload).and_then(|circuit| {
        debug!("Circuit: {:#?}", circuit);
        Ok(HttpResponse::Accepted().finish())
    }))
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
