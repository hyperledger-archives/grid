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

use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::sync::{Arc, RwLock};

use protobuf::RepeatedField;

use crate::circuit::SplinterState;
use crate::circuit::{
    service::SplinterNode as StateNode, Circuit as StateCircuit,
    ServiceDefinition as StateServiceDefinition,
};
use crate::consensus::{Proposal, ProposalId};
use crate::hex::to_hex;
use crate::keys::{KeyPermissionManager, KeyRegistry};
use crate::network::{
    auth::{AuthorizationInquisitor, PeerAuthorizationState},
    peer::PeerConnector,
};
use crate::orchestrator::{ServiceDefinition, ServiceOrchestrator};
use crate::protos::admin::{
    Circuit, CircuitManagementPayload, CircuitManagementPayload_Action,
    CircuitManagementPayload_Header, CircuitProposal, CircuitProposalVote,
    CircuitProposalVote_Vote, CircuitProposal_ProposalType, CircuitProposal_VoteRecord,
    Circuit_AuthorizationType, Circuit_DurabilityType, Circuit_PersistenceType, Circuit_RouteType,
};
use crate::rest_api::{EventDealer, Request, Response, ResponseError};
use crate::service::error::ServiceError;
use crate::service::ServiceNetworkSender;
use crate::signing::SignatureVerifier;

use super::error::{AdminSharedError, MarshallingError};
use super::messages;
use super::{admin_service_id, sha256};

static VOTER_ROLE: &str = "voter";
static PROPOSER_ROLE: &str = "proposer";

type UnpeeredPendingPayload = (Vec<String>, CircuitManagementPayload);

enum CircuitProposalStatus {
    Accepted,
    Rejected,
    Pending,
}

struct CircuitProposalContext {
    pub circuit_proposal: CircuitProposal,
    pub action: CircuitManagementPayload_Action,
    pub signer_public_key: Vec<u8>,
}

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
    pending_consensus_proposals: HashMap<ProposalId, (Proposal, CircuitManagementPayload)>,
    // the pending changes for the current proposal
    pending_changes: Option<CircuitProposalContext>,
    // the verifiers that should be broadcasted for the pending change
    current_consensus_verifiers: Vec<String>,
    // Map of event dealers, keyed by circuit management type
    event_dealers: HashMap<String, EventDealer<messages::AdminServiceEvent>>,
    // copy of splinter state
    splinter_state: Arc<RwLock<SplinterState>>,
    // signature verifier
    signature_verifier: Box<dyn SignatureVerifier + Send>,
    key_registry: Box<dyn KeyRegistry>,
    key_permission_manager: Box<dyn KeyPermissionManager>,
}

impl AdminServiceShared {
    #![allow(clippy::too_many_arguments)]
    pub fn new(
        node_id: String,
        orchestrator: ServiceOrchestrator,
        peer_connector: PeerConnector,
        auth_inquisitor: Box<dyn AuthorizationInquisitor>,
        splinter_state: Arc<RwLock<SplinterState>>,
        signature_verifier: Box<dyn SignatureVerifier + Send>,
        key_registry: Box<dyn KeyRegistry>,
        key_permission_manager: Box<dyn KeyPermissionManager>,
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
            pending_consensus_proposals: HashMap::new(),
            pending_changes: None,
            current_consensus_verifiers: Vec::new(),
            event_dealers: HashMap::new(),
            splinter_state,
            signature_verifier,
            key_registry,
            key_permission_manager,
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

    pub fn pending_consensus_proposals(
        &self,
        id: &ProposalId,
    ) -> Option<&(Proposal, CircuitManagementPayload)> {
        self.pending_consensus_proposals.get(id)
    }

    pub fn remove_pending_consensus_proposals(
        &mut self,
        id: &ProposalId,
    ) -> Option<(Proposal, CircuitManagementPayload)> {
        self.pending_consensus_proposals.remove(id)
    }

    pub fn add_pending_consensus_proposal(
        &mut self,
        id: ProposalId,
        proposal: (Proposal, CircuitManagementPayload),
    ) {
        self.pending_consensus_proposals.insert(id, proposal);
    }

    pub fn current_consensus_verifiers(&self) -> &Vec<String> {
        &self.current_consensus_verifiers
    }

    pub fn commit(&mut self) -> Result<(), AdminSharedError> {
        match self.pending_changes.take() {
            Some(circuit_proposal_context) => {
                let circuit_proposal = circuit_proposal_context.circuit_proposal;
                let action = circuit_proposal_context.action;
                let circuit_id = circuit_proposal.get_circuit_id().to_string();
                let mgmt_type = circuit_proposal
                    .get_circuit_proposal()
                    .circuit_management_type
                    .clone();

                match self.check_approved(&circuit_proposal) {
                    Ok(CircuitProposalStatus::Accepted) => {
                        // commit new circuit
                        let circuit = circuit_proposal.get_circuit_proposal();
                        self.update_splinter_state(circuit)?;
                        // remove approved proposal
                        self.remove_proposal(&circuit_id);
                        // send message about circuit acceptance

                        let circuit_proposal_proto =
                            messages::CircuitProposal::from_proto(circuit_proposal.clone())
                                .map_err(AdminSharedError::InvalidMessageFormat)?;
                        let event = messages::AdminServiceEvent::ProposalAccepted((
                            circuit_proposal_proto,
                            circuit_proposal_context.signer_public_key,
                        ));
                        self.send_event(&mgmt_type, event);

                        Ok(())
                    }
                    Ok(CircuitProposalStatus::Pending) => {
                        self.add_proposal(circuit_proposal.clone());

                        match action {
                            CircuitManagementPayload_Action::CIRCUIT_CREATE_REQUEST => {
                                // notify registered application authorization handlers of the
                                // committed circuit proposal
                                let event = messages::AdminServiceEvent::ProposalSubmitted(
                                    messages::CircuitProposal::from_proto(circuit_proposal.clone())
                                        .map_err(AdminSharedError::InvalidMessageFormat)?,
                                );
                                self.send_event(&mgmt_type, event);

                                info!("committed changes for new circuit proposal {}", circuit_id);
                                Ok(())
                            }

                            CircuitManagementPayload_Action::CIRCUIT_PROPOSAL_VOTE => {
                                // notify registered application authorization handlers of the
                                // committed circuit proposal
                                let circuit_proposal_proto =
                                    messages::CircuitProposal::from_proto(circuit_proposal.clone())
                                        .map_err(AdminSharedError::InvalidMessageFormat)?;
                                let event = messages::AdminServiceEvent::ProposalVote((
                                    circuit_proposal_proto,
                                    circuit_proposal_context.signer_public_key,
                                ));
                                self.send_event(&mgmt_type, event);

                                info!("committed vote for circuit proposal {}", circuit_id);
                                Ok(())
                            }
                            _ => Err(AdminSharedError::UnknownAction(format!(
                                "Received unknown action: {:?}",
                                action
                            ))),
                        }
                    }
                    Ok(CircuitProposalStatus::Rejected) => {
                        // remove circuit
                        self.remove_proposal(&circuit_id);

                        let circuit_proposal_proto =
                            messages::CircuitProposal::from_proto(circuit_proposal.clone())
                                .map_err(AdminSharedError::InvalidMessageFormat)?;
                        let event = messages::AdminServiceEvent::ProposalRejected((
                            circuit_proposal_proto,
                            circuit_proposal_context.signer_public_key,
                        ));
                        self.send_event(&mgmt_type, event);

                        info!("circuit proposal for {} has been rejected", circuit_id);
                        Ok(())
                    }
                    Err(err) => Err(err),
                }
            }
            None => Err(AdminSharedError::NoPendingChanges),
        }
    }

    pub fn rollback(&mut self) -> Result<(), AdminSharedError> {
        match self.pending_changes.take() {
            Some(circuit_proposal_context) => info!(
                "discarded change for {}",
                circuit_proposal_context.circuit_proposal.get_circuit_id()
            ),
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

                let signer_public_key = header.get_requester();
                let requester_node_id = header.get_requester_node_id();

                self.validate_create_circuit(
                    &proposed_circuit,
                    signer_public_key,
                    requester_node_id,
                )?;
                debug!("proposing {}", proposed_circuit.get_circuit_id());

                let mut circuit_proposal = CircuitProposal::new();
                circuit_proposal.set_proposal_type(CircuitProposal_ProposalType::CREATE);
                circuit_proposal.set_circuit_id(proposed_circuit.get_circuit_id().into());
                circuit_proposal.set_circuit_hash(sha256(&proposed_circuit)?);
                circuit_proposal.set_circuit_proposal(proposed_circuit);
                circuit_proposal.set_requester(header.get_requester().to_vec());
                circuit_proposal.set_requester_node_id(header.get_requester_node_id().to_string());

                let expected_hash = sha256(&circuit_proposal)?;
                self.pending_changes = Some(CircuitProposalContext {
                    circuit_proposal: circuit_proposal.clone(),
                    signer_public_key: header.get_requester().to_vec(),
                    action: CircuitManagementPayload_Action::CIRCUIT_CREATE_REQUEST,
                });
                self.current_consensus_verifiers = verifiers;

                Ok((expected_hash, circuit_proposal))
            }
            CircuitManagementPayload_Action::CIRCUIT_PROPOSAL_VOTE => {
                let proposal_vote = circuit_payload.get_circuit_proposal_vote();

                // validate vote proposal
                // check that the circuit proposal exists
                let mut circuit_proposal = self
                    .open_proposals
                    .get(proposal_vote.get_circuit_id())
                    .ok_or_else(|| {
                        AdminSharedError::ValidationFailed(format!(
                            "Received vote for a proposal that does not exist: circuit id {}",
                            proposal_vote.circuit_id
                        ))
                    })?
                    .clone();

                let mut verifiers = vec![];
                for member in circuit_proposal.get_circuit_proposal().get_members() {
                    verifiers.push(admin_service_id(member.get_node_id()));
                }
                let signer_public_key = header.get_requester();

                self.validate_circuit_vote(
                    proposal_vote,
                    signer_public_key,
                    &circuit_proposal,
                    header.get_requester_node_id(),
                )?;
                // add vote to circuit_proposal
                let mut vote_record = CircuitProposal_VoteRecord::new();
                vote_record.set_public_key(signer_public_key.to_vec());
                vote_record.set_vote(proposal_vote.get_vote());
                vote_record.set_voter_node_id(header.get_requester_node_id().to_string());

                let mut votes = circuit_proposal.get_votes().to_vec();
                votes.push(vote_record);
                circuit_proposal.set_votes(RepeatedField::from_vec(votes));

                let expected_hash = sha256(&circuit_proposal)?;
                self.pending_changes = Some(CircuitProposalContext {
                    circuit_proposal: circuit_proposal.clone(),
                    signer_public_key: header.get_requester().to_vec(),
                    action: CircuitManagementPayload_Action::CIRCUIT_PROPOSAL_VOTE,
                });
                self.current_consensus_verifiers = verifiers;
                Ok((expected_hash, circuit_proposal))
            }
            unknown_action => Err(AdminSharedError::ValidationFailed(format!(
                "Unable to handle {:?}",
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

    pub fn propose_vote(&mut self, payload: CircuitManagementPayload) -> Result<(), ServiceError> {
        debug!(
            "received circuit vote for {}",
            payload.get_circuit_proposal_vote().get_circuit_id()
        );

        self.pending_circuit_payloads.push_back(payload);
        Ok(())
    }

    pub fn submit(&mut self, payload: CircuitManagementPayload) -> Result<(), ServiceError> {
        debug!("Payload submitted: {:?}", payload);

        match self.verify_signature(&payload) {
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
            CircuitManagementPayload_Action::CIRCUIT_PROPOSAL_VOTE => self.propose_vote(payload),
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

    fn remove_proposal(&mut self, circuit_id: &str) {
        self.open_proposals.remove(circuit_id);
    }

    fn validate_create_circuit(
        &self,
        circuit: &Circuit,
        signer_public_key: &[u8],
        requester_node_id: &str,
    ) -> Result<(), AdminSharedError> {
        if requester_node_id.is_empty() {
            return Err(AdminSharedError::ValidationFailed(
                "requester_node_id is empty".to_string(),
            ));
        }

        let key_info = self
            .key_registry
            .get_key(signer_public_key)
            .map_err(|err| AdminSharedError::ValidationFailed(err.to_string()))?
            .ok_or_else(|| {
                AdminSharedError::ValidationFailed(format!(
                    "{} is not registered for a node",
                    to_hex(signer_public_key)
                ))
            })?;

        if key_info.associated_node_id() != requester_node_id {
            return Err(AdminSharedError::ValidationFailed(format!(
                "{} is not registered for the node in header",
                to_hex(signer_public_key)
            )));
        };

        self.key_permission_manager
            .is_permitted(signer_public_key, PROPOSER_ROLE)
            .map_err(|_| {
                AdminSharedError::ValidationFailed(format!(
                    "{} is not permitted to vote for node {}",
                    to_hex(signer_public_key),
                    key_info.associated_node_id()
                ))
            })?;

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

    fn validate_circuit_vote(
        &self,
        proposal_vote: &CircuitProposalVote,
        signer_public_key: &[u8],
        circuit_proposal: &CircuitProposal,
        node_id: &str,
    ) -> Result<(), AdminSharedError> {
        let circuit_hash = proposal_vote.get_circuit_hash();

        let key_info = self
            .key_registry
            .get_key(signer_public_key)
            .map_err(|err| AdminSharedError::ValidationFailed(err.to_string()))?
            .ok_or_else(|| {
                AdminSharedError::ValidationFailed(format!(
                    "{} is not registered for a node",
                    to_hex(signer_public_key)
                ))
            })?;

        let signer_node = key_info.associated_node_id().to_string();

        if signer_node != node_id {
            return Err(AdminSharedError::ValidationFailed(format!(
                "Payload requester node id does not match the node the key is registered to: {}",
                to_hex(circuit_proposal.get_requester())
            )));
        }

        if circuit_proposal.get_requester_node_id() == signer_node {
            return Err(AdminSharedError::ValidationFailed(format!(
                "Received vote from requester node: {}",
                to_hex(circuit_proposal.get_requester())
            )));
        }

        let voted_nodes: Vec<String> = circuit_proposal
            .get_votes()
            .iter()
            .map(|vote| vote.get_voter_node_id().to_string())
            .collect();

        if voted_nodes.iter().any(|node| *node == signer_node) {
            return Err(AdminSharedError::ValidationFailed(format!(
                "Received duplicate vote from {} for {}",
                signer_node, proposal_vote.circuit_id
            )));
        }

        self.key_permission_manager
            .is_permitted(signer_public_key, VOTER_ROLE)
            .map_err(|_| {
                AdminSharedError::ValidationFailed(format!(
                    "{} is not permitted to vote for node {}",
                    to_hex(signer_public_key),
                    signer_node
                ))
            })?;

        // validate hash of circuit
        if circuit_proposal.get_circuit_hash() != circuit_hash {
            return Err(AdminSharedError::ValidationFailed(format!(
                "Hash of circuit does not match circuit proposal: {}",
                proposal_vote.circuit_id
            )));
        }

        Ok(())
    }

    fn check_approved(
        &self,
        proposal: &CircuitProposal,
    ) -> Result<CircuitProposalStatus, AdminSharedError> {
        let mut received_votes = HashSet::new();
        for vote in proposal.get_votes() {
            if vote.get_vote() == CircuitProposalVote_Vote::REJECT {
                return Ok(CircuitProposalStatus::Rejected);
            }
            received_votes.insert(vote.get_voter_node_id().to_string());
        }

        let mut required_votes = proposal
            .get_circuit_proposal()
            .get_members()
            .to_vec()
            .iter()
            .map(|member| member.get_node_id().to_string())
            .collect::<HashSet<String>>();

        required_votes.remove(proposal.get_requester_node_id());

        if required_votes == received_votes {
            Ok(CircuitProposalStatus::Accepted)
        } else {
            Ok(CircuitProposalStatus::Pending)
        }
    }

    /// Initialize all services that this node should run on the created circuit using the service
    /// orchestrator.
    pub fn initialize_services(&mut self, circuit: &Circuit) -> Result<(), AdminSharedError> {
        // Get all services this node is allowed to run
        let services = circuit
            .get_roster()
            .iter()
            .filter(|service| service.allowed_nodes.contains(&self.node_id))
            .collect::<Vec<_>>();

        // Start all services
        for service in services {
            let service_definition = ServiceDefinition {
                circuit: circuit.circuit_id.clone(),
                service_id: service.service_id.clone(),
                service_type: service.service_type.clone(),
            };

            let service_arguments = service
                .arguments
                .iter()
                .map(|arg| (arg.key.clone(), arg.value.clone()))
                .collect();

            self.orchestrator
                .initialize_service(service_definition.clone(), service_arguments)?;

            self.running_services.insert(service_definition);
        }

        Ok(())
    }

    fn update_splinter_state(&self, circuit: &Circuit) -> Result<(), AdminSharedError> {
        let members: Vec<StateNode> = circuit
            .get_members()
            .iter()
            .map(|node| {
                StateNode::new(
                    node.get_node_id().to_string(),
                    vec![node.get_endpoint().to_string()],
                )
            })
            .collect();

        let roster = circuit.get_roster().iter().map(|service| {
            StateServiceDefinition::builder(
                service.get_service_id().to_string(),
                service.get_service_type().to_string(),
            )
            .with_allowed_nodes(service.get_allowed_nodes().to_vec())
            .with_arguments(
                service
                    .get_arguments()
                    .iter()
                    .map(|argument| {
                        (
                            argument.get_key().to_string(),
                            argument.get_value().to_string(),
                        )
                    })
                    .collect::<BTreeMap<String, String>>(),
            )
            .build()
        });

        let auth = match circuit.get_authorization_type() {
            Circuit_AuthorizationType::TRUST_AUTHORIZATION => "trust".to_string(),
            // This should never happen
            Circuit_AuthorizationType::UNSET_AUTHORIZATION_TYPE => {
                return Err(AdminSharedError::CommitError(
                    "Missing authorization type on circuit commit".to_string(),
                ))
            }
        };

        let persistence = match circuit.get_persistence() {
            Circuit_PersistenceType::ANY_PERSISTENCE => "any".to_string(),
            // This should never happen
            Circuit_PersistenceType::UNSET_PERSISTENCE_TYPE => {
                return Err(AdminSharedError::CommitError(
                    "Missing persistence type on circuit commit".to_string(),
                ))
            }
        };

        let durability = match circuit.get_durability() {
            Circuit_DurabilityType::NO_DURABILITY => "none".to_string(),
            // This should never happen
            Circuit_DurabilityType::UNSET_DURABILITY_TYPE => {
                return Err(AdminSharedError::CommitError(
                    "Missing durabilty type on circuit commit".to_string(),
                ))
            }
        };

        let routes = match circuit.get_routes() {
            Circuit_RouteType::ANY_ROUTE => "any".to_string(),
            // This should never happen
            Circuit_RouteType::UNSET_ROUTE_TYPE => {
                return Err(AdminSharedError::CommitError(
                    "Missing route type on circuit commit".to_string(),
                ))
            }
        };

        let new_circuit = StateCircuit::builder()
            .with_id(circuit.get_circuit_id().to_string())
            .with_members(
                members
                    .iter()
                    .map(|node| node.id().to_string())
                    .collect::<Vec<String>>(),
            )
            .with_roster(roster)
            .with_auth(auth)
            .with_persistence(persistence)
            .with_durability(durability)
            .with_routes(routes)
            .with_circuit_management_type(circuit.get_circuit_management_type().to_string())
            .build()
            .map_err(|err| {
                AdminSharedError::CommitError(format!("Unable build new circuit: {}", err))
            })?;

        let mut splinter_state = self.splinter_state.write().map_err(|err| {
            AdminSharedError::CommitError(format!("Unable to unlock splinter state: {}", err))
        })?;
        for member in members {
            splinter_state
                .add_node(member.id().to_string(), member)
                .map_err(|err| {
                    AdminSharedError::CommitError(format!(
                        "Unable to add node to splinter state: {}",
                        err
                    ))
                })?;
        }
        splinter_state
            .add_circuit(new_circuit.id().to_string(), new_circuit)
            .map_err(|err| {
                AdminSharedError::CommitError(format!(
                    "Unable to add circuit to splinter state: {}",
                    err
                ))
            })?;
        Ok(())
    }

    fn verify_signature(&self, payload: &CircuitManagementPayload) -> Result<bool, ServiceError> {
        let header =
            protobuf::parse_from_bytes::<CircuitManagementPayload_Header>(payload.get_header())?;

        let signature = payload.get_signature();
        let public_key = header.get_requester();

        self.signature_verifier
            .verify(&payload.get_header(), &signature, &public_key)
            .map_err(|err| ServiceError::UnableToHandleMessage(Box::new(err)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use protobuf::{Message, RepeatedField};

    use crate::circuit::directory::CircuitDirectory;
    use crate::keys::{
        insecure::AllowAllKeyPermissionManager, storage::StorageKeyRegistry, KeyInfo,
    };
    use crate::mesh::Mesh;
    use crate::network::{
        auth::{AuthorizationCallback, AuthorizationCallbackError},
        Network,
    };
    use crate::protos::admin;
    use crate::protos::admin::{SplinterNode, SplinterService};
    use crate::protos::authorization::{
        AuthorizationMessage, AuthorizationMessageType, AuthorizedMessage,
    };
    use crate::protos::network::{NetworkMessage, NetworkMessageType};
    use crate::signing::hash::HashVerifier;
    use crate::storage::get_storage;
    use crate::transport::{
        ConnectError, Connection, DisconnectError, RecvError, SendError, Transport,
    };

    /// Test that the CircuitManagementPayload is moved to the pending payloads when the peers are
    /// fully authorized.
    #[test]
    fn test_auth_change() {
        let mesh = Mesh::new(4, 16);
        let network = Network::new(mesh.clone());
        let mut transport = MockConnectingTransport::expect_connections(vec![
            Ok(Box::new(MockConnection::new())),
            Ok(Box::new(MockConnection::new())),
            Ok(Box::new(MockConnection::new())),
        ]);
        let orchestrator_connection = transport
            .connect("inproc://admin-service")
            .expect("failed to create connection");
        let orchestrator = ServiceOrchestrator::new(vec![], orchestrator_connection, 1, 1, 1)
            .expect("failed to create orchestrator");
        let peer_connector = PeerConnector::new(network.clone(), Box::new(transport));
        let state = setup_splinter_state();
        let key_registry = StorageKeyRegistry::new("memory".to_string()).unwrap();
        let mut shared = AdminServiceShared::new(
            "my_peer_id".into(),
            orchestrator,
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
            Box::new(HashVerifier),
            Box::new(key_registry),
            Box::new(AllowAllKeyPermissionManager),
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
        let mut transport = MockConnectingTransport::expect_connections(vec![
            Ok(Box::new(MockConnection::new())),
            Ok(Box::new(MockConnection::new())),
            Ok(Box::new(MockConnection::new())),
        ]);
        let orchestrator_connection = transport
            .connect("inproc://admin-service")
            .expect("failed to create connection");
        let orchestrator = ServiceOrchestrator::new(vec![], orchestrator_connection, 1, 1, 1)
            .expect("failed to create orchestrator");
        let peer_connector = PeerConnector::new(network.clone(), Box::new(transport));
        let state = setup_splinter_state();
        let key_registry = StorageKeyRegistry::new("memory".to_string()).unwrap();
        let mut shared = AdminServiceShared::new(
            "my_peer_id".into(),
            orchestrator,
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
            Box::new(HashVerifier),
            Box::new(key_registry),
            Box::new(AllowAllKeyPermissionManager),
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
        let orchestrator = setup_orchestrator();
        // set up key registry
        let mut key_registry = StorageKeyRegistry::new("memory".to_string()).unwrap();
        let key_info = KeyInfo::builder(b"test_signer_a".to_vec(), "node_a".to_string()).build();
        key_registry.save_key(key_info).unwrap();

        let admin_shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
            Box::new(HashVerifier),
            Box::new(key_registry),
            Box::new(AllowAllKeyPermissionManager),
        );
        let circuit = setup_test_circuit();

        if let Err(err) = admin_shared.validate_create_circuit(&circuit, b"test_signer_a", "node_a")
        {
            panic!("Should have been valid: {}", err);
        }
    }

    #[test]
    // test that if a circuit is proposed by a signer key that is not registered the proposal is
    // invalid
    fn test_validate_circuit_signer_key_not_registered() {
        let state = setup_splinter_state();
        let peer_connector = setup_peer_connector();
        let orchestrator = setup_orchestrator();
        let key_registry = StorageKeyRegistry::new("memory".to_string()).unwrap();

        let admin_shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
            Box::new(HashVerifier),
            Box::new(key_registry),
            Box::new(AllowAllKeyPermissionManager),
        );
        let mut circuit = setup_test_circuit();

        let mut service_bad = SplinterService::new();
        service_bad.set_service_id("service_b".to_string());
        service_bad.set_service_type("type_a".to_string());
        service_bad.set_allowed_nodes(RepeatedField::from_vec(vec!["node_b".to_string()]));

        circuit.set_roster(RepeatedField::from_vec(vec![service_bad]));

        if let Ok(_) = admin_shared.validate_create_circuit(&circuit, b"test_signer_a", "node_a") {
            panic!("Should have been invalid due to signer not being registered to a node");
        }
    }

    #[test]
    // test that if a circuit has a service in its roster with an allowed node that is not in
    // members an error is returned
    fn test_validate_circuit_bad_node() {
        let state = setup_splinter_state();
        let peer_connector = setup_peer_connector();
        let orchestrator = setup_orchestrator();
        // set up key registry
        let mut key_registry = StorageKeyRegistry::new("memory".to_string()).unwrap();
        let key_info = KeyInfo::builder(b"test_signer_a".to_vec(), "node_a".to_string()).build();
        key_registry.save_key(key_info).unwrap();

        let admin_shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
            Box::new(HashVerifier),
            Box::new(key_registry),
            Box::new(AllowAllKeyPermissionManager),
        );
        let mut circuit = setup_test_circuit();

        let mut service_bad = SplinterService::new();
        service_bad.set_service_id("service_b".to_string());
        service_bad.set_service_type("type_a".to_string());
        service_bad.set_allowed_nodes(RepeatedField::from_vec(vec!["node_bad".to_string()]));

        circuit.set_roster(RepeatedField::from_vec(vec![service_bad]));

        if let Ok(_) = admin_shared.validate_create_circuit(&circuit, b"test_signer_a", "node_a") {
            panic!("Should have been invalid due to service having an allowed node not in members");
        }
    }

    #[test]
    // test that if a circuit does not have any services in its roster an error is returned
    fn test_validate_circuit_empty_roster() {
        let state = setup_splinter_state();
        let peer_connector = setup_peer_connector();
        let orchestrator = setup_orchestrator();
        // set up key registry
        let mut key_registry = StorageKeyRegistry::new("memory".to_string()).unwrap();
        let key_info = KeyInfo::builder(b"test_signer_a".to_vec(), "node_a".to_string()).build();
        key_registry.save_key(key_info).unwrap();

        let admin_shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
            Box::new(HashVerifier),
            Box::new(key_registry),
            Box::new(AllowAllKeyPermissionManager),
        );
        let mut circuit = setup_test_circuit();
        circuit.set_roster(RepeatedField::from_vec(vec![]));

        if let Ok(_) = admin_shared.validate_create_circuit(&circuit, b"test_signer_a", "node_a") {
            panic!("Should have been invalid due to empty roster");
        }
    }

    #[test]
    // test that if a circuit does not have any nodes in its members an error is returned
    fn test_validate_circuit_empty_members() {
        let state = setup_splinter_state();
        let peer_connector = setup_peer_connector();
        let orchestrator = setup_orchestrator();
        // set up key registry
        let mut key_registry = StorageKeyRegistry::new("memory".to_string()).unwrap();
        let key_info = KeyInfo::builder(b"test_signer_a".to_vec(), "node_a".to_string()).build();
        key_registry.save_key(key_info).unwrap();

        let admin_shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
            Box::new(HashVerifier),
            Box::new(key_registry),
            Box::new(AllowAllKeyPermissionManager),
        );
        let mut circuit = setup_test_circuit();

        circuit.set_members(RepeatedField::from_vec(vec![]));

        if let Ok(_) = admin_shared.validate_create_circuit(&circuit, b"test_signer_a", "node_a") {
            panic!("Should have been invalid empty members");
        }
    }

    #[test]
    // test that if a circuit does not have the local node in the member list an error is
    // returned
    fn test_validate_circuit_missing_local_node() {
        let state = setup_splinter_state();
        let peer_connector = setup_peer_connector();
        let orchestrator = setup_orchestrator();
        // set up key registry
        let mut key_registry = StorageKeyRegistry::new("memory".to_string()).unwrap();
        let key_info = KeyInfo::builder(b"test_signer_a".to_vec(), "node_a".to_string()).build();
        key_registry.save_key(key_info).unwrap();

        let admin_shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
            Box::new(HashVerifier),
            Box::new(key_registry),
            Box::new(AllowAllKeyPermissionManager),
        );
        let mut circuit = setup_test_circuit();

        let mut node_b = SplinterNode::new();
        node_b.set_node_id("node_b".to_string());
        node_b.set_endpoint("test://endpoint_b:0".to_string());

        circuit.set_members(RepeatedField::from_vec(vec![node_b]));

        if let Ok(_) = admin_shared.validate_create_circuit(&circuit, b"test_signer_a", "node_a") {
            panic!("Should have been invalid because node_a is not in members");
        }
    }

    #[test]
    // test that if a circuit does not have authorization set an error is returned
    fn test_validate_circuit_no_authorization() {
        let state = setup_splinter_state();
        let peer_connector = setup_peer_connector();
        let orchestrator = setup_orchestrator();
        // set up key registry
        let mut key_registry = StorageKeyRegistry::new("memory".to_string()).unwrap();
        let key_info = KeyInfo::builder(b"test_signer_a".to_vec(), "node_a".to_string()).build();
        key_registry.save_key(key_info).unwrap();

        let admin_shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
            Box::new(HashVerifier),
            Box::new(key_registry),
            Box::new(AllowAllKeyPermissionManager),
        );
        let mut circuit = setup_test_circuit();

        circuit.set_authorization_type(Circuit_AuthorizationType::UNSET_AUTHORIZATION_TYPE);

        if let Ok(_) = admin_shared.validate_create_circuit(&circuit, b"test_signer_a", "node_a") {
            panic!("Should have been invalid because authorizaiton type is unset");
        }
    }

    #[test]
    // test that if a circuit does not have persistence set an error is returned
    fn test_validate_circuit_no_persitance() {
        let state = setup_splinter_state();
        let peer_connector = setup_peer_connector();
        let orchestrator = setup_orchestrator();
        // set up key registry
        let mut key_registry = StorageKeyRegistry::new("memory".to_string()).unwrap();
        let key_info = KeyInfo::builder(b"test_signer_a".to_vec(), "node_a".to_string()).build();
        key_registry.save_key(key_info).unwrap();

        let admin_shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
            Box::new(HashVerifier),
            Box::new(key_registry),
            Box::new(AllowAllKeyPermissionManager),
        );
        let mut circuit = setup_test_circuit();

        circuit.set_persistence(Circuit_PersistenceType::UNSET_PERSISTENCE_TYPE);

        if let Ok(_) = admin_shared.validate_create_circuit(&circuit, b"test_signer_a", "node_a") {
            panic!("Should have been invalid because persistence type is unset");
        }
    }

    #[test]
    // test that if a circuit does not have durability set an error is returned
    fn test_validate_circuit_unset_durability() {
        let state = setup_splinter_state();
        let peer_connector = setup_peer_connector();
        let orchestrator = setup_orchestrator();
        // set up key registry
        let mut key_registry = StorageKeyRegistry::new("memory".to_string()).unwrap();
        let key_info = KeyInfo::builder(b"test_signer_a".to_vec(), "node_a".to_string()).build();
        key_registry.save_key(key_info).unwrap();

        let admin_shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
            Box::new(HashVerifier),
            Box::new(key_registry),
            Box::new(AllowAllKeyPermissionManager),
        );
        let mut circuit = setup_test_circuit();

        circuit.set_durability(Circuit_DurabilityType::UNSET_DURABILITY_TYPE);

        if let Ok(_) = admin_shared.validate_create_circuit(&circuit, b"test_signer_a", "node_a") {
            panic!("Should have been invalid because durabilty type is unset");
        }
    }

    #[test]
    // test that if a circuit does not have route type set an error is returned
    fn test_validate_circuit_no_routes() {
        let state = setup_splinter_state();
        let peer_connector = setup_peer_connector();
        let orchestrator = setup_orchestrator();
        // set up key registry
        let mut key_registry = StorageKeyRegistry::new("memory".to_string()).unwrap();
        let key_info = KeyInfo::builder(b"test_signer_a".to_vec(), "node_a".to_string()).build();
        key_registry.save_key(key_info).unwrap();

        let admin_shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
            Box::new(HashVerifier),
            Box::new(key_registry),
            Box::new(AllowAllKeyPermissionManager),
        );
        let mut circuit = setup_test_circuit();

        circuit.set_routes(Circuit_RouteType::UNSET_ROUTE_TYPE);

        if let Ok(_) = admin_shared.validate_create_circuit(&circuit, b"test_signer_a", "node_a") {
            panic!("Should have been invalid because route type is unset");
        }
    }

    #[test]
    // test that if a circuit does not have circuit_management_type set an error is returned
    fn test_validate_circuit_no_management_type() {
        let state = setup_splinter_state();
        let peer_connector = setup_peer_connector();
        let orchestrator = setup_orchestrator();
        // set up key registry
        let mut key_registry = StorageKeyRegistry::new("memory".to_string()).unwrap();
        let key_info = KeyInfo::builder(b"test_signer_a".to_vec(), "node_a".to_string()).build();
        key_registry.save_key(key_info).unwrap();

        let admin_shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
            Box::new(HashVerifier),
            Box::new(key_registry),
            Box::new(AllowAllKeyPermissionManager),
        );
        let mut circuit = setup_test_circuit();

        circuit.set_circuit_management_type("".to_string());

        if let Ok(_) = admin_shared.validate_create_circuit(&circuit, b"test_signer_a", "node_a") {
            panic!("Should have been invalid because route type is unset");
        }
    }

    #[test]
    // test that a valid circuit proposal comes back as valid
    fn test_validate_proposal_vote_valid() {
        let state = setup_splinter_state();
        let peer_connector = setup_peer_connector();
        let orchestrator = setup_orchestrator();

        // set up key registry
        let mut key_registry = StorageKeyRegistry::new("memory".to_string()).unwrap();
        let key_info = KeyInfo::builder(b"test_signer_a".to_vec(), "node_a".to_string()).build();
        key_registry.save_key(key_info).unwrap();

        let admin_shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
            Box::new(HashVerifier),
            Box::new(key_registry),
            Box::new(AllowAllKeyPermissionManager),
        );
        let circuit = setup_test_circuit();
        let vote = setup_test_vote(&circuit);
        let proposal = setup_test_proposal(&circuit);

        if let Err(err) =
            admin_shared.validate_circuit_vote(&vote, b"test_signer_a", &proposal, "node_a")
        {
            panic!("Should have been valid: {}", err);
        }
    }

    #[test]
    // test that if the signer of the vote is not registered to a node the vote is invalid
    fn test_validate_proposal_vote_node_not_registered() {
        let state = setup_splinter_state();
        let peer_connector = setup_peer_connector();
        let orchestrator = setup_orchestrator();

        // set up key registry
        let key_registry = StorageKeyRegistry::new("memory".to_string()).unwrap();

        let admin_shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
            Box::new(HashVerifier),
            Box::new(key_registry),
            Box::new(AllowAllKeyPermissionManager),
        );
        let circuit = setup_test_circuit();
        let vote = setup_test_vote(&circuit);
        let proposal = setup_test_proposal(&circuit);

        if let Ok(_) =
            admin_shared.validate_circuit_vote(&vote, b"test_signer_a", &proposal, "node_a")
        {
            panic!("Should have been invalid because signer is not registered for a node");
        }
    }

    #[test]
    // test if the voter is registered to the original requester node the vote is invalid
    fn test_validate_proposal_vote_requester() {
        let state = setup_splinter_state();
        let peer_connector = setup_peer_connector();
        let orchestrator = setup_orchestrator();

        // set up key registry
        let mut key_registry = StorageKeyRegistry::new("memory".to_string()).unwrap();
        let key_info = KeyInfo::builder(b"test_signer_a".to_vec(), "node_b".to_string()).build();
        key_registry.save_key(key_info).unwrap();

        let admin_shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
            Box::new(HashVerifier),
            Box::new(key_registry),
            Box::new(AllowAllKeyPermissionManager),
        );
        let circuit = setup_test_circuit();
        let vote = setup_test_vote(&circuit);
        let proposal = setup_test_proposal(&circuit);
        if let Ok(_) =
            admin_shared.validate_circuit_vote(&vote, b"test_signer_a", &proposal, "node_a")
        {
            panic!("Should have been invalid because signer registered for the requester node");
        }
    }

    #[test]
    // test if a voter has aleady voted on a proposal the new vote is invalid
    fn test_validate_proposal_vote_duplicate_vote() {
        let state = setup_splinter_state();
        let peer_connector = setup_peer_connector();
        let orchestrator = setup_orchestrator();

        // set up key registry
        let mut key_registry = StorageKeyRegistry::new("memory".to_string()).unwrap();
        let key_info = KeyInfo::builder(b"test_signer_a".to_vec(), "node_a".to_string()).build();
        key_registry.save_key(key_info).unwrap();

        let admin_shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
            Box::new(HashVerifier),
            Box::new(key_registry),
            Box::new(AllowAllKeyPermissionManager),
        );
        let circuit = setup_test_circuit();
        let vote = setup_test_vote(&circuit);
        let mut proposal = setup_test_proposal(&circuit);

        let mut vote_record = CircuitProposal_VoteRecord::new();
        vote_record.set_vote(CircuitProposalVote_Vote::ACCEPT);
        vote_record.set_public_key(b"test_signer_a".to_vec());
        vote_record.set_voter_node_id("node_a".to_string());

        proposal.set_votes(RepeatedField::from_vec(vec![vote_record]));

        if let Ok(_) =
            admin_shared.validate_circuit_vote(&vote, b"test_signer_a", &proposal, "node_a")
        {
            panic!("Should have been invalid because node as already submited a vote");
        }
    }

    #[test]
    // test that if the circuit hash in the circuit proposal does not match the cirucit hash on
    // the vote, the vote is invalid
    fn test_validate_proposal_vote_circuit_hash_mismatch() {
        let state = setup_splinter_state();
        let peer_connector = setup_peer_connector();
        let orchestrator = setup_orchestrator();

        // set up key registry
        let mut key_registry = StorageKeyRegistry::new("memory".to_string()).unwrap();
        let key_info = KeyInfo::builder(b"test_signer_a".to_vec(), "node_a".to_string()).build();
        key_registry.save_key(key_info).unwrap();

        let admin_shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
            Box::new(HashVerifier),
            Box::new(key_registry),
            Box::new(AllowAllKeyPermissionManager),
        );
        let circuit = setup_test_circuit();
        let vote = setup_test_vote(&circuit);
        let mut proposal = setup_test_proposal(&circuit);

        proposal.set_circuit_hash("bad_hash".to_string());

        if let Ok(_) =
            admin_shared.validate_circuit_vote(&vote, b"test_signer_a", &proposal, "node_a")
        {
            panic!("Should have been invalid becasue the circuit hash does not match");
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

    fn setup_test_vote(circuit: &Circuit) -> CircuitProposalVote {
        let mut circuit_vote = CircuitProposalVote::new();
        circuit_vote.set_vote(CircuitProposalVote_Vote::ACCEPT);
        circuit_vote.set_circuit_id(circuit.get_circuit_id().to_string());
        let circuit_hash = sha256(circuit).unwrap();
        circuit_vote.set_circuit_hash(circuit_hash);

        circuit_vote
    }

    fn setup_test_proposal(proposed_circuit: &Circuit) -> CircuitProposal {
        let mut circuit_proposal = CircuitProposal::new();
        circuit_proposal.set_proposal_type(CircuitProposal_ProposalType::CREATE);
        circuit_proposal.set_circuit_id(proposed_circuit.get_circuit_id().into());
        circuit_proposal.set_circuit_hash(sha256(proposed_circuit).unwrap());
        circuit_proposal.set_circuit_proposal(proposed_circuit.clone());
        circuit_proposal.set_requester(b"test_signer_b".to_vec());
        circuit_proposal.set_requester_node_id("node_b".to_string());

        circuit_proposal
    }

    fn setup_splinter_state() -> Arc<RwLock<SplinterState>> {
        let mut storage = get_storage("memory", CircuitDirectory::new).unwrap();
        let circuit_directory = storage.write().clone();
        let state = Arc::new(RwLock::new(SplinterState::new(
            "memory".to_string(),
            circuit_directory,
        )));
        state
    }

    fn setup_peer_connector() -> PeerConnector {
        let mesh = Mesh::new(4, 16);
        let network = Network::new(mesh.clone());
        let transport = MockConnectingTransport::expect_connections(vec![
            Ok(Box::new(MockConnection::new())),
            Ok(Box::new(MockConnection::new())),
        ]);
        let peer_connector = PeerConnector::new(network.clone(), Box::new(transport));
        peer_connector
    }

    fn setup_orchestrator() -> ServiceOrchestrator {
        let mut transport =
            MockConnectingTransport::expect_connections(vec![Ok(Box::new(MockConnection::new()))]);
        let orchestrator_connection = transport
            .connect("inproc://admin-service")
            .expect("failed to create connection");
        ServiceOrchestrator::new(vec![], orchestrator_connection, 1, 1, 1)
            .expect("failed to create orchestrator")
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

    struct MockConnection {
        auth_response: Option<Vec<u8>>,
        evented: MockEvented,
    }

    impl MockConnection {
        fn new() -> Self {
            Self {
                auth_response: Some(authorized_response()),
                evented: MockEvented::new(),
            }
        }
    }

    impl Connection for MockConnection {
        fn send(&mut self, _message: &[u8]) -> Result<(), SendError> {
            Ok(())
        }

        fn recv(&mut self) -> Result<Vec<u8>, RecvError> {
            Ok(self.auth_response.take().unwrap_or_else(|| vec![]))
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
            &self.evented
        }
    }

    struct MockEvented {
        registration: mio::Registration,
        set_readiness: mio::SetReadiness,
    }

    impl MockEvented {
        fn new() -> Self {
            let (registration, set_readiness) = mio::Registration::new2();

            Self {
                registration,
                set_readiness,
            }
        }
    }

    impl mio::Evented for MockEvented {
        fn register(
            &self,
            poll: &mio::Poll,
            token: mio::Token,
            interest: mio::Ready,
            opts: mio::PollOpt,
        ) -> std::io::Result<()> {
            self.registration.register(poll, token, interest, opts)?;
            self.set_readiness.set_readiness(mio::Ready::readable())?;

            Ok(())
        }

        fn reregister(
            &self,
            poll: &mio::Poll,
            token: mio::Token,
            interest: mio::Ready,
            opts: mio::PollOpt,
        ) -> std::io::Result<()> {
            self.registration.reregister(poll, token, interest, opts)
        }

        fn deregister(&self, poll: &mio::Poll) -> std::io::Result<()> {
            poll.deregister(&self.registration)
        }
    }

    fn authorized_response() -> Vec<u8> {
        let msg_type = AuthorizationMessageType::AUTHORIZE;
        let auth_msg = AuthorizedMessage::new();
        let mut auth_msg_env = AuthorizationMessage::new();
        auth_msg_env.set_message_type(msg_type);
        auth_msg_env.set_payload(auth_msg.write_to_bytes().expect("unable to write to bytes"));

        let mut network_msg = NetworkMessage::new();
        network_msg.set_message_type(NetworkMessageType::AUTHORIZATION);
        network_msg.set_payload(
            auth_msg_env
                .write_to_bytes()
                .expect("unable to write to bytes"),
        );

        network_msg
            .write_to_bytes()
            .expect("unable to write to bytes")
    }
}
