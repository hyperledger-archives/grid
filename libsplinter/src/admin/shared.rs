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

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, RwLock};

#[cfg(feature = "ursa-compat")]
use ursa::signatures::ed25519::EcdsaSecp256k1Sha256;
#[cfg(feature = "ursa-compat")]
use ursa::signatures::SignatureScheme;

use crate::circuit::SplinterState;
use crate::consensus::{Proposal, ProposalId};
use crate::network::{
    auth::{AuthorizationInquisitor, PeerAuthorizationState},
    peer::PeerConnector,
};
use crate::orchestrator::{ServiceDefinition, ServiceOrchestrator};
use crate::protos::admin::{
    Circuit, CircuitManagementPayload, CircuitManagementPayload_Action,
    CircuitManagementPayload_Header, CircuitProposal, CircuitProposal_ProposalType,
    Circuit_AuthorizationType, Circuit_DurabilityType, Circuit_PersistenceType, Circuit_RouteType,
};
use crate::rest_api::{EventDealer, Request, Response, ResponseError};
use crate::service::error::ServiceError;
use crate::service::ServiceNetworkSender;

#[cfg(feature = "ursa-compat")]
use crate::signing::{ursa::UrsaSecp256k1SignatureVerifier, SignatureVerifier};

use super::error::{AdminSharedError, MarshallingError};
use super::messages;
use super::{admin_service_id, sha256};

type UnpeeredPendingPayload = (Vec<String>, CircuitManagementPayload);

pub struct AdminServiceShared {
    // the node id of the connected splinter node
    node_id: String,
    // the list of circuit proposal that are being voted on by members of a circuit
    open_proposals: HashMap<String, CircuitProposal>,
    // orchestrator used to initialize and shutdown services
    orchestrator: ServiceOrchestrator,
    // list of services that have been initialized using the orchestrator
    running_services: HashSet<ServiceDefinition>,
    // peer connector used to connect to new members listed in a circuit
    peer_connector: PeerConnector,
    // auth inquisitor
    auth_inquisitor: Box<dyn AuthorizationInquisitor>,
    // network sender is used to comunicated with other services on the splinter network
    network_sender: Option<Box<dyn ServiceNetworkSender>>,
    // the CircuitManagementPayloads that require peers to be fully authorized before they can go
    // through consensus
    unpeered_payloads: Vec<UnpeeredPendingPayload>,

    // CircuitManagmentPayloads that still need to go through consensus
    pending_circuit_payloads: VecDeque<CircuitManagementPayload>,
    // The pending consensus proposals
    pending_consesus_proposals: HashMap<ProposalId, (Proposal, CircuitManagementPayload)>,
    // the pending changes for the current proposal
    pending_changes: Option<CircuitProposal>,
    // the verifiers that should be broadcasted for the pending change
    current_consensus_verifiers: Vec<String>,
    // Map of event dealers, keyed by circuit management type
    event_dealers: HashMap<String, EventDealer<messages::AdminServiceEvent>>,
    // copy of splinter state
    splinter_state: Arc<RwLock<SplinterState>>,
}

impl AdminServiceShared {
    pub fn new(
        node_id: String,
        orchestrator: ServiceOrchestrator,
        peer_connector: PeerConnector,
        auth_inquisitor: Box<dyn AuthorizationInquisitor>,
        splinter_state: Arc<RwLock<SplinterState>>,
    ) -> Self {
        AdminServiceShared {
            node_id: node_id.to_string(),
            network_sender: None,
            open_proposals: Default::default(),
            orchestrator,
            running_services: HashSet::new(),
            peer_connector,
            auth_inquisitor,
            unpeered_payloads: Vec::new(),
            pending_circuit_payloads: VecDeque::new(),
            pending_consesus_proposals: HashMap::new(),
            pending_changes: None,
            current_consensus_verifiers: Vec::new(),
            event_dealers: HashMap::new(),
            splinter_state,
        }
    }

    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    pub fn network_sender(&self) -> &Option<Box<dyn ServiceNetworkSender>> {
        &self.network_sender
    }

    pub fn auth_inquisitor(&self) -> &dyn AuthorizationInquisitor {
        &*self.auth_inquisitor
    }

    pub fn set_network_sender(&mut self, network_sender: Option<Box<dyn ServiceNetworkSender>>) {
        self.network_sender = network_sender;
    }

    pub fn pop_pending_circuit_payload(&mut self) -> Option<CircuitManagementPayload> {
        self.pending_circuit_payloads.pop_front()
    }

    pub fn pending_consesus_proposals(
        &self,
        id: &ProposalId,
    ) -> Option<&(Proposal, CircuitManagementPayload)> {
        self.pending_consesus_proposals.get(id)
    }

    pub fn remove_pending_consesus_proposals(
        &mut self,
        id: &ProposalId,
    ) -> Option<(Proposal, CircuitManagementPayload)> {
        self.pending_consesus_proposals.remove(id)
    }

    pub fn add_pending_consesus_proposal(
        &mut self,
        id: ProposalId,
        proposal: (Proposal, CircuitManagementPayload),
    ) {
        self.pending_consesus_proposals.insert(id, proposal);
    }

    pub fn current_consensus_verifiers(&self) -> &Vec<String> {
        &self.current_consensus_verifiers
    }

    pub fn commit(&mut self) -> Result<(), AdminSharedError> {
        match self.pending_changes.take() {
            Some(circuit_proposal) => {
                let circuit_id = circuit_proposal.get_circuit_id().to_string();
                let mgmt_type = circuit_proposal
                    .get_circuit_proposal()
                    .circuit_management_type
                    .clone();

                self.add_proposal(circuit_proposal.clone());

                // notify registered authorization application handlers of the commited circuit
                // proposal
                let event = messages::AdminServiceEvent::ProposalSubmitted(
                    messages::CircuitProposal::from_proto(circuit_proposal.clone())
                        .map_err(AdminSharedError::InvalidMessageFormat)?,
                );
                self.send_event(&mgmt_type, event);

                info!("committed change for circuit proposal {}", circuit_id,);

                Ok(())
            }
            None => Err(AdminSharedError::NoPendingChanges),
        }
    }

    pub fn rollback(&mut self) -> Result<(), AdminSharedError> {
        match self.pending_changes.take() {
            Some(circuit_proposal) => {
                info!("discarded change for {}", circuit_proposal.get_circuit_id())
            }
            None => debug!("no changes to rollback"),
        }

        Ok(())
    }

    pub fn propose_change(
        &mut self,
        mut circuit_payload: CircuitManagementPayload,
    ) -> Result<(String, CircuitProposal), AdminSharedError> {
        let header = protobuf::parse_from_bytes::<CircuitManagementPayload_Header>(
            circuit_payload.get_header(),
        )
        .map_err(MarshallingError::from)?;

        match header.get_action() {
            CircuitManagementPayload_Action::CIRCUIT_CREATE_REQUEST => {
                let mut create_request = circuit_payload.take_circuit_create_request();
                let proposed_circuit = create_request.take_circuit();
                let mut verifiers = vec![];
                for member in proposed_circuit.get_members() {
                    verifiers.push(admin_service_id(member.get_node_id()));
                }

                self.validate_create_circuit(&proposed_circuit)?;
                debug!("proposing {}", proposed_circuit.get_circuit_id());

                let mut circuit_proposal = CircuitProposal::new();
                circuit_proposal.set_proposal_type(CircuitProposal_ProposalType::CREATE);
                circuit_proposal.set_circuit_id(proposed_circuit.get_circuit_id().into());
                circuit_proposal.set_circuit_hash(sha256(&proposed_circuit)?);
                circuit_proposal.set_circuit_proposal(proposed_circuit);

                let expected_hash = sha256(&circuit_proposal)?;
                self.pending_changes = Some(circuit_proposal.clone());
                self.current_consensus_verifiers = verifiers;

                Ok((expected_hash, circuit_proposal))
            }
            unknown_action => Err(AdminSharedError::UnknownAction(format!(
                "{:?}",
                unknown_action
            ))),
        }
    }

    pub fn has_proposal(&self, circuit_id: &str) -> bool {
        self.open_proposals.contains_key(circuit_id)
    }

    /// Propose a new circuit
    ///
    /// This operation will propose a new circuit to all the member nodes of the circuit.  If there
    /// is no peer connection, a connection to the peer will also be established.
    pub fn propose_circuit(
        &mut self,
        payload: CircuitManagementPayload,
    ) -> Result<(), ServiceError> {
        debug!(
            "received circuit proposal for {}",
            payload
                .get_circuit_create_request()
                .get_circuit()
                .get_circuit_id()
        );

        let mut unauthorized_peers = vec![];
        for node in payload
            .get_circuit_create_request()
            .get_circuit()
            .get_members()
        {
            if self.node_id() != node.get_node_id() {
                if self.auth_inquisitor.is_authorized(node.get_node_id()) {
                    continue;
                }

                debug!("Connecting to node {:?}", node);
                self.peer_connector
                    .connect_peer(node.get_node_id(), node.get_endpoint())
                    .map_err(|err| ServiceError::UnableToHandleMessage(Box::new(err)))?;

                unauthorized_peers.push(node.get_node_id().into());
            }
        }

        if unauthorized_peers.is_empty() {
            self.pending_circuit_payloads.push_back(payload);
        } else {
            debug!(
                "Members {:?} added; awaiting network authorization before proceeding",
                &unauthorized_peers
            );

            self.unpeered_payloads.push((unauthorized_peers, payload));
        }
        Ok(())
    }

    pub fn submit(&mut self, payload: CircuitManagementPayload) -> Result<(), ServiceError> {
        debug!("Payload submitted: {:?}", payload);

        match verify_signature(&payload) {
            Ok(_) => (),
            Err(ServiceError::UnableToHandleMessage(_)) => (),
            Err(err) => return Err(err),
        };

        let header =
            protobuf::parse_from_bytes::<CircuitManagementPayload_Header>(payload.get_header())?;

        match header.get_action() {
            CircuitManagementPayload_Action::CIRCUIT_CREATE_REQUEST => {
                self.propose_circuit(payload)
            }
            _ => {
                debug!("Unhandled action: {:?}", header.get_action());
                Ok(())
            }
        }
    }

    pub fn add_subscriber(
        &mut self,
        circuit_management_type: String,
        request: Request,
    ) -> Result<Response, ResponseError> {
        if let Some(dealer) = self.event_dealers.get_mut(&circuit_management_type) {
            dealer.subscribe(request)
        } else {
            let mut dealer = EventDealer::new();
            let res = dealer.subscribe(request)?;

            self.event_dealers.insert(circuit_management_type, dealer);

            Ok(res)
        }
    }

    pub fn send_event(
        &mut self,
        circuit_management_type: &str,
        event: messages::AdminServiceEvent,
    ) {
        if let Some(dealer) = self.event_dealers.get_mut(circuit_management_type) {
            dealer.dispatch(event);
        } else {
            warn!(
                "No event dealer for circuit management type {}",
                circuit_management_type
            );
        }
    }

    pub fn on_authorization_change(&mut self, peer_id: &str, state: PeerAuthorizationState) {
        let mut unpeered_payloads = std::mem::replace(&mut self.unpeered_payloads, vec![]);
        for (ref mut peers, _) in unpeered_payloads.iter_mut() {
            match state {
                PeerAuthorizationState::Authorized => {
                    peers.retain(|unpeered_id| unpeered_id != peer_id);
                }
                PeerAuthorizationState::Unauthorized => {
                    if peers.iter().any(|unpeered_id| unpeered_id == peer_id) {
                        warn!("Dropping circuit request including peer {}, due to authorization failure", peer_id);
                        peers.clear();
                    }
                }
            }
        }

        let (fully_peered, still_unpeered): (
            Vec<UnpeeredPendingPayload>,
            Vec<UnpeeredPendingPayload>,
        ) = unpeered_payloads
            .into_iter()
            .partition(|(peers, _)| peers.is_empty());

        std::mem::replace(&mut self.unpeered_payloads, still_unpeered);
        if state == PeerAuthorizationState::Authorized {
            self.pending_circuit_payloads
                .extend(fully_peered.into_iter().map(|(_, payload)| payload));
        }
    }

    fn add_proposal(&mut self, circuit_proposal: CircuitProposal) {
        let circuit_id = circuit_proposal.get_circuit_id().to_string();

        self.open_proposals.insert(circuit_id, circuit_proposal);
    }

    fn validate_create_circuit(&self, circuit: &Circuit) -> Result<(), AdminSharedError> {
        if self.has_proposal(circuit.get_circuit_id()) {
            return Err(AdminSharedError::ValidationFailed(format!(
                "Ignoring duplicate create proposal of circuit {}",
                circuit.get_circuit_id()
            )));
        }

        if self
            .splinter_state
            .read()
            .expect("splinter state lock poisoned")
            .has_circuit(circuit.get_circuit_id())
        {
            return Err(AdminSharedError::ValidationFailed(format!(
                "Circuit with circuit id {} already exists",
                circuit.get_circuit_id()
            )));
        }

        self.validate_circuit(circuit)?;
        Ok(())
    }

    fn validate_circuit(&self, circuit: &Circuit) -> Result<(), AdminSharedError> {
        if circuit.get_authorization_type() == Circuit_AuthorizationType::UNSET_AUTHORIZATION_TYPE {
            return Err(AdminSharedError::ValidationFailed(
                "authorization_type cannot be unset".to_string(),
            ));
        }

        if circuit.get_persistence() == Circuit_PersistenceType::UNSET_PERSISTENCE_TYPE {
            return Err(AdminSharedError::ValidationFailed(
                "persistence_type cannot be unset".to_string(),
            ));
        }

        if circuit.get_durability() == Circuit_DurabilityType::UNSET_DURABILITY_TYPE {
            return Err(AdminSharedError::ValidationFailed(
                "durability_type cannot be unset".to_string(),
            ));
        }

        if circuit.get_routes() == Circuit_RouteType::UNSET_ROUTE_TYPE {
            return Err(AdminSharedError::ValidationFailed(
                "route_type cannot be unset".to_string(),
            ));
        }

        if circuit.get_circuit_id().is_empty() {
            return Err(AdminSharedError::ValidationFailed(
                "circuit_id must be set".to_string(),
            ));
        }

        if circuit.get_circuit_management_type().is_empty() {
            return Err(AdminSharedError::ValidationFailed(
                "circuit_management_type must be set".to_string(),
            ));
        }

        let members: Vec<String> = circuit
            .get_members()
            .iter()
            .map(|node| node.get_node_id().to_string())
            .collect();

        if members.is_empty() {
            return Err(AdminSharedError::ValidationFailed(
                "The circuit must have members".to_string(),
            ));
        }

        // check this node is in members
        if !members.contains(&self.node_id) {
            return Err(AdminSharedError::ValidationFailed(format!(
                "Circuit does not contain this node: {}",
                self.node_id
            )));
        }

        if circuit.get_roster().is_empty() {
            return Err(AdminSharedError::ValidationFailed(
                "The circuit must have services".to_string(),
            ));
        }

        // check that all services' allowed nodes are in members
        for service in circuit.get_roster() {
            for node in service.get_allowed_nodes() {
                if !members.contains(node) {
                    return Err(AdminSharedError::ValidationFailed(format!(
                        "Service cannot have an allowed node that is not in members: {}",
                        self.node_id
                    )));
                }
            }
        }

        if circuit.get_circuit_management_type().is_empty() {
            return Err(AdminSharedError::ValidationFailed(
                "The circuit must have a mangement type".to_string(),
            ));
        }

        Ok(())
    }

    /// Initialize all services that this node should run on the created circuit using the service
    /// orchestrator.
    pub fn initialize_services(
        &mut self,
        create_circuit: &messages::CreateCircuit,
    ) -> Result<(), AdminSharedError> {
        // Get all services this node is allowed to run
        let services = create_circuit
            .roster
            .iter()
            .filter(|service| service.allowed_nodes.contains(&self.node_id))
            .collect::<Vec<_>>();

        // Start all services
        for service in services {
            let service_definition = ServiceDefinition {
                circuit: create_circuit.circuit_id.clone(),
                service_id: service.service_id.clone(),
                service_type: service.service_type.clone(),
            };

            self.orchestrator
                .initialize_service(service_definition.clone(), service.arguments.clone())?;

            self.running_services.insert(service_definition);
        }

        Ok(())
    }
}

#[cfg(feature = "ursa-compat")]
fn verify_signature(payload: &CircuitManagementPayload) -> Result<bool, ServiceError> {
    let scheme = EcdsaSecp256k1Sha256::new();
    let ursa_signature_verifier = UrsaSecp256k1SignatureVerifier::new(&scheme);

    let header = protobuf::parse_from_bytes::<CircuitManagementPayload_Header>(payload.header())?;

    let signature = payload.get_signature();
    let public_key = header.get_requester();

    ursa_signature_verifier
        .verify(&payload.get_header(), &signature, &public_key)
        .map_err(AdminShared::from)
        .map_err(Box::new)
        .map_err(ServiceError::UnableToHandleMessage)
}

#[cfg(not(feature = "ursa-compat"))]
fn verify_signature(_: &CircuitManagementPayload) -> Result<bool, ServiceError> {
    Err(ServiceError::UnableToHandleMessage(Box::new(
        AdminSharedError::UndefinedSigner,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use protobuf::RepeatedField;
    use tempdir::TempDir;

    use crate::circuit::directory::CircuitDirectory;
    use crate::mesh::Mesh;
    use crate::network::{
        auth::{AuthorizationCallback, AuthorizationCallbackError},
        Network,
    };
    use crate::protos::admin;
    use crate::protos::admin::{SplinterNode, SplinterService};
    use crate::storage::get_storage;
    use crate::transport::{
        inproc::InprocTransport, ConnectError, Connection, DisconnectError, RecvError, SendError,
        Transport,
    };

    /// Test that the CircuitManagementPayload is moved to the pending payloads when the peers are
    /// fully authorized.
    #[test]
    fn test_auth_change() {
        let mesh = Mesh::new(4, 16);
        let network = Network::new(mesh.clone());
        let transport = MockConnectingTransport::expect_connections(vec![
            Ok(Box::new(MockConnection)),
            Ok(Box::new(MockConnection)),
        ]);

        let orchestrator =
            ServiceOrchestrator::new(vec![], "".to_string(), InprocTransport::default());
        let peer_connector = PeerConnector::new(network.clone(), Box::new(transport));
        let state = setup_splinter_state();
        let mut shared = AdminServiceShared::new(
            "my_peer_id".into(),
            orchestrator,
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
        );

        let mut circuit = admin::Circuit::new();
        circuit.set_circuit_id("test_propose_circuit".into());
        circuit.set_authorization_type(admin::Circuit_AuthorizationType::TRUST_AUTHORIZATION);
        circuit.set_persistence(admin::Circuit_PersistenceType::ANY_PERSISTENCE);
        circuit.set_routes(admin::Circuit_RouteType::ANY_ROUTE);
        circuit.set_durability(admin::Circuit_DurabilityType::NO_DURABILITY);
        circuit.set_circuit_management_type("test app auth handler".into());

        circuit.set_members(protobuf::RepeatedField::from_vec(vec![
            splinter_node("test-node", "tcp://someplace:8000"),
            splinter_node("other-node", "tcp://otherplace:8000"),
        ]));
        circuit.set_roster(protobuf::RepeatedField::from_vec(vec![
            splinter_service("service-a", "sabre"),
            splinter_service("service-b", "sabre"),
        ]));

        let mut request = admin::CircuitCreateRequest::new();
        request.set_circuit(circuit);

        let mut header = admin::CircuitManagementPayload_Header::new();
        header.set_action(admin::CircuitManagementPayload_Action::CIRCUIT_CREATE_REQUEST);

        let mut payload = admin::CircuitManagementPayload::new();

        payload.set_signature(Vec::new());
        payload.set_header(protobuf::Message::write_to_bytes(&header).unwrap());
        payload.set_circuit_create_request(request);

        shared
            .propose_circuit(payload)
            .expect("Proposal not accepted");

        // None of the proposed members are peered
        assert_eq!(0, shared.pending_circuit_payloads.len());
        shared.on_authorization_change("test-node", PeerAuthorizationState::Authorized);

        // One node is still unpeered
        assert_eq!(0, shared.pending_circuit_payloads.len());
        shared.on_authorization_change("other-node", PeerAuthorizationState::Authorized);

        // We're fully peered, so the pending payload is now available
        assert_eq!(1, shared.pending_circuit_payloads.len());
    }

    /// Test that the CircuitManagementPayload message is dropped, if a node fails authorization.
    #[test]
    fn test_unauth_change() {
        let mesh = Mesh::new(4, 16);
        let network = Network::new(mesh.clone());
        let transport = MockConnectingTransport::expect_connections(vec![
            Ok(Box::new(MockConnection)),
            Ok(Box::new(MockConnection)),
        ]);

        let orchestrator =
            ServiceOrchestrator::new(vec![], "".to_string(), InprocTransport::default());
        let peer_connector = PeerConnector::new(network.clone(), Box::new(transport));
        let state = setup_splinter_state();
        let mut shared = AdminServiceShared::new(
            "my_peer_id".into(),
            orchestrator,
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
        );

        let mut circuit = admin::Circuit::new();
        circuit.set_circuit_id("test_propose_circuit".into());
        circuit.set_authorization_type(admin::Circuit_AuthorizationType::TRUST_AUTHORIZATION);
        circuit.set_persistence(admin::Circuit_PersistenceType::ANY_PERSISTENCE);
        circuit.set_routes(admin::Circuit_RouteType::ANY_ROUTE);
        circuit.set_circuit_management_type("test app auth handler".into());

        circuit.set_members(protobuf::RepeatedField::from_vec(vec![
            splinter_node("test-node", "tcp://someplace:8000"),
            splinter_node("other-node", "tcp://otherplace:8000"),
        ]));
        circuit.set_roster(protobuf::RepeatedField::from_vec(vec![
            splinter_service("service-a", "sabre"),
            splinter_service("service-b", "sabre"),
        ]));

        let mut request = admin::CircuitCreateRequest::new();
        request.set_circuit(circuit);

        let mut header = admin::CircuitManagementPayload_Header::new();
        header.set_action(admin::CircuitManagementPayload_Action::CIRCUIT_CREATE_REQUEST);

        let mut payload = admin::CircuitManagementPayload::new();

        payload.set_signature(Vec::new());
        payload.set_header(protobuf::Message::write_to_bytes(&header).unwrap());
        payload.set_circuit_create_request(request);

        shared
            .propose_circuit(payload)
            .expect("Proposal not accepted");

        // None of the proposed members are peered
        assert_eq!(1, shared.unpeered_payloads.len());
        assert_eq!(0, shared.pending_circuit_payloads.len());
        shared.on_authorization_change("test-node", PeerAuthorizationState::Unauthorized);

        // The message should be dropped
        assert_eq!(0, shared.pending_circuit_payloads.len());
        assert_eq!(0, shared.unpeered_payloads.len());
    }

    #[test]
    // test that a valid circuit is validated correctly
    fn test_validate_circuit_valid() {
        let state = setup_splinter_state();
        let peer_connector = setup_peer_connector();
        let orchestrator =
            ServiceOrchestrator::new(vec![], "".to_string(), InprocTransport::default());
        let admin_shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
        );
        let circuit = setup_test_circuit();

        if let Err(err) = admin_shared.validate_create_circuit(&circuit) {
            panic!("Should have been valid: {}", err);
        }
    }

    #[test]
    // test that if a circuit has a service in its roster with an allowed node that is not in
    // members an error is returned
    fn test_validate_circuit_bad_node() {
        let state = setup_splinter_state();
        let peer_connector = setup_peer_connector();
        let orchestrator =
            ServiceOrchestrator::new(vec![], "".to_string(), InprocTransport::default());
        let admin_shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
        );
        let mut circuit = setup_test_circuit();

        let mut service_bad = SplinterService::new();
        service_bad.set_service_id("service_b".to_string());
        service_bad.set_service_type("type_a".to_string());
        service_bad.set_allowed_nodes(RepeatedField::from_vec(vec!["node_bad".to_string()]));

        circuit.set_roster(RepeatedField::from_vec(vec![service_bad]));

        if let Ok(_) = admin_shared.validate_create_circuit(&circuit) {
            panic!("Should have been invalid due to service having an allowed node not in members");
        }
    }

    #[test]
    // test that if a circuit does not have any services in its roster an error is returned
    fn test_validate_circuit_empty_roster() {
        let state = setup_splinter_state();
        let peer_connector = setup_peer_connector();
        let orchestrator =
            ServiceOrchestrator::new(vec![], "".to_string(), InprocTransport::default());
        let admin_shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
        );
        let mut circuit = setup_test_circuit();
        circuit.set_roster(RepeatedField::from_vec(vec![]));

        if let Ok(_) = admin_shared.validate_create_circuit(&circuit) {
            panic!("Should have been invalid due to empty roster");
        }
    }

    #[test]
    // test that if a circuit does not have any nodes in its members an error is returned
    fn test_validate_circuit_empty_members() {
        let state = setup_splinter_state();
        let peer_connector = setup_peer_connector();
        let orchestrator =
            ServiceOrchestrator::new(vec![], "".to_string(), InprocTransport::default());
        let admin_shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
        );
        let mut circuit = setup_test_circuit();

        circuit.set_members(RepeatedField::from_vec(vec![]));

        if let Ok(_) = admin_shared.validate_create_circuit(&circuit) {
            panic!("Should have been invalid empty members");
        }
    }

    #[test]
    // test that if a circuit does not have the local node in the member list an error is
    // returned
    fn test_validate_circuit_missing_local_node() {
        let state = setup_splinter_state();
        let peer_connector = setup_peer_connector();
        let orchestrator =
            ServiceOrchestrator::new(vec![], "".to_string(), InprocTransport::default());
        let admin_shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
        );
        let mut circuit = setup_test_circuit();

        let mut node_b = SplinterNode::new();
        node_b.set_node_id("node_b".to_string());
        node_b.set_endpoint("test://endpoint_b:0".to_string());

        circuit.set_members(RepeatedField::from_vec(vec![node_b]));

        if let Ok(_) = admin_shared.validate_create_circuit(&circuit) {
            panic!("Should have been invalid because node_a is not in members");
        }
    }

    #[test]
    // test that if a circuit does not have authorization set an error is returned
    fn test_validate_circuit_no_authorization() {
        let state = setup_splinter_state();
        let peer_connector = setup_peer_connector();
        let orchestrator =
            ServiceOrchestrator::new(vec![], "".to_string(), InprocTransport::default());
        let admin_shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
        );
        let mut circuit = setup_test_circuit();

        circuit.set_authorization_type(Circuit_AuthorizationType::UNSET_AUTHORIZATION_TYPE);

        if let Ok(_) = admin_shared.validate_create_circuit(&circuit) {
            panic!("Should have been invalid because authorizaiton type is unset");
        }
    }

    #[test]
    // test that if a circuit does not have persistence set an error is returned
    fn test_validate_circuit_no_persitance() {
        let state = setup_splinter_state();
        let peer_connector = setup_peer_connector();
        let orchestrator =
            ServiceOrchestrator::new(vec![], "".to_string(), InprocTransport::default());
        let admin_shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
        );
        let mut circuit = setup_test_circuit();

        circuit.set_persistence(Circuit_PersistenceType::UNSET_PERSISTENCE_TYPE);

        if let Ok(_) = admin_shared.validate_create_circuit(&circuit) {
            panic!("Should have been invalid because persistence type is unset");
        }
    }

    #[test]
    // test that if a circuit does not have durability set an error is returned
    fn test_validate_circuit_unset_durability() {
        let state = setup_splinter_state();
        let peer_connector = setup_peer_connector();
        let orchestrator =
            ServiceOrchestrator::new(vec![], "".to_string(), InprocTransport::default());
        let admin_shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
        );
        let mut circuit = setup_test_circuit();

        circuit.set_durability(Circuit_DurabilityType::UNSET_DURABILITY_TYPE);

        if let Ok(_) = admin_shared.validate_create_circuit(&circuit) {
            panic!("Should have been invalid because durabilty type is unset");
        }
    }

    #[test]
    // test that if a circuit does not have route type set an error is returned
    fn test_validate_circuit_no_routes() {
        let state = setup_splinter_state();
        let peer_connector = setup_peer_connector();
        let orchestrator =
            ServiceOrchestrator::new(vec![], "".to_string(), InprocTransport::default());
        let admin_shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
        );
        let mut circuit = setup_test_circuit();

        circuit.set_routes(Circuit_RouteType::UNSET_ROUTE_TYPE);

        if let Ok(_) = admin_shared.validate_create_circuit(&circuit) {
            panic!("Should have been invalid because route type is unset");
        }
    }

    #[test]
    // test that if a circuit does not have circuit_management_type set an error is returned
    fn test_validate_circuit_no_management_type() {
        let state = setup_splinter_state();
        let peer_connector = setup_peer_connector();
        let orchestrator =
            ServiceOrchestrator::new(vec![], "".to_string(), InprocTransport::default());
        let admin_shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
        );
        let mut circuit = setup_test_circuit();

        circuit.set_circuit_management_type("".to_string());

        if let Ok(_) = admin_shared.validate_create_circuit(&circuit) {
            panic!("Should have been invalid because route type is unset");
        }
    }

    pub fn setup_test_circuit() -> Circuit {
        let mut service_a = SplinterService::new();
        service_a.set_service_id("service_a".to_string());
        service_a.set_service_type("type_a".to_string());
        service_a.set_allowed_nodes(RepeatedField::from_vec(vec!["node_a".to_string()]));

        let mut service_b = SplinterService::new();
        service_b.set_service_id("service_b".to_string());
        service_b.set_service_type("type_a".to_string());
        service_b.set_allowed_nodes(RepeatedField::from_vec(vec!["node_b".to_string()]));

        let mut node_a = SplinterNode::new();
        node_a.set_node_id("node_a".to_string());
        node_a.set_endpoint("test://endpoint_a:0".to_string());

        let mut node_b = SplinterNode::new();
        node_b.set_node_id("node_b".to_string());
        node_b.set_endpoint("test://endpoint_b:0".to_string());

        let mut circuit = Circuit::new();
        circuit.set_circuit_id("alpha".to_string());
        circuit.set_members(RepeatedField::from_vec(vec![node_a, node_b]));
        circuit.set_roster(RepeatedField::from_vec(vec![service_b, service_a]));
        circuit.set_authorization_type(Circuit_AuthorizationType::TRUST_AUTHORIZATION);
        circuit.set_persistence(Circuit_PersistenceType::ANY_PERSISTENCE);
        circuit.set_durability(Circuit_DurabilityType::NO_DURABILITY);
        circuit.set_routes(Circuit_RouteType::ANY_ROUTE);
        circuit.set_circuit_management_type("test_circuit".to_string());
        circuit.set_application_metadata(b"test_data".to_vec());

        circuit
    }

    fn setup_splinter_state() -> Arc<RwLock<SplinterState>> {
        // create temp directoy
        let temp_dir = TempDir::new("test_circuit_write_file").unwrap();
        let temp_dir = temp_dir.path().to_path_buf();

        // setup empty state filename
        let path = setup_storage(temp_dir);
        let mut storage = get_storage(&path, CircuitDirectory::new).unwrap();
        let circuit_directory = storage.write().clone();
        let state = Arc::new(RwLock::new(SplinterState::new(
            path.to_string(),
            circuit_directory,
        )));
        state
    }

    fn setup_peer_connector() -> PeerConnector {
        let mesh = Mesh::new(4, 16);
        let network = Network::new(mesh.clone());
        let transport = MockConnectingTransport::expect_connections(vec![
            Ok(Box::new(MockConnection)),
            Ok(Box::new(MockConnection)),
        ]);
        let peer_connector = PeerConnector::new(network.clone(), Box::new(transport));
        peer_connector
    }

    fn setup_storage(mut temp_dir: PathBuf) -> String {
        // Creat the temp file
        temp_dir.push("circuits.yaml");
        let path = temp_dir.to_str().unwrap().to_string();

        // Write out the mock state file to the temp directory
        path
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

    struct MockAuthInquisitor;

    impl AuthorizationInquisitor for MockAuthInquisitor {
        fn is_authorized(&self, _: &str) -> bool {
            false
        }

        fn register_callback(
            &self,
            _: Box<dyn AuthorizationCallback>,
        ) -> Result<(), AuthorizationCallbackError> {
            unimplemented!();
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
            panic!("MockConnectingTransport.listen unexpectedly called")
        }
    }

    struct MockConnection;

    impl Connection for MockConnection {
        fn send(&mut self, _message: &[u8]) -> Result<(), SendError> {
            Ok(())
        }

        fn recv(&mut self) -> Result<Vec<u8>, RecvError> {
            panic!("MockConnection.recv unexpectedly called")
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
