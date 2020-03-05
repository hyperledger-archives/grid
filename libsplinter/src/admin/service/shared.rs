// Copyright 2018-2020 Cargill Incorporated
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

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::env;
use std::iter::FromIterator;
use std::time::SystemTime;

use protobuf::{Message, RepeatedField};
use std::sync::mpsc::Sender;

use crate::circuit::SplinterState;
use crate::circuit::{
    service::SplinterNode as StateNode,
    service::{Service, ServiceId},
    AuthorizationType, Circuit as StateCircuit, DurabilityType, PersistenceType, RouteType,
    ServiceDefinition as StateServiceDefinition,
};
use crate::consensus::{Proposal, ProposalId, ProposalUpdate};
use crate::hex::to_hex;
use crate::keys::{KeyPermissionManager, KeyRegistry};
use crate::network::{
    auth::{AuthorizationCallbackError, AuthorizationInquisitor, PeerAuthorizationState},
    peer::PeerConnector,
};
use crate::orchestrator::{ServiceDefinition, ServiceOrchestrator, ShutdownServiceError};
#[cfg(feature = "service-arg-validation")]
use crate::protos::admin::SplinterService;
use crate::protos::admin::{
    AdminMessage, AdminMessage_Type, Circuit, CircuitManagementPayload,
    CircuitManagementPayload_Action, CircuitManagementPayload_Header, CircuitProposal,
    CircuitProposalVote, CircuitProposalVote_Vote, CircuitProposal_ProposalType,
    CircuitProposal_VoteRecord, Circuit_AuthorizationType, Circuit_DurabilityType,
    Circuit_PersistenceType, Circuit_RouteType, MemberReady, SplinterNode,
};
use crate::service::error::ServiceError;
#[cfg(feature = "service-arg-validation")]
use crate::service::validation::ServiceArgValidator;

use crate::service::ServiceNetworkSender;
use crate::signing::SignatureVerifier;
use crate::storage::sets::mem::DurableBTreeSet;

use super::error::{AdminSharedError, MarshallingError};
use super::mailbox::Mailbox;
use super::messages;
use super::open_proposals::OpenProposals;
#[cfg(feature = "proposal-read")]
use super::open_proposals::ProposalIter;
use super::{admin_service_id, sha256, AdminServiceEventSubscriber, AdminSubscriberError, Events};

const DEFAULT_STATE_DIR: &str = "/var/lib/splinter/";
const STATE_DIR_ENV: &str = "SPLINTER_STATE_DIR";
static VOTER_ROLE: &str = "voter";
static PROPOSER_ROLE: &str = "proposer";

const DEFAULT_IN_MEMORY_EVENT_LIMIT: usize = 100;

pub enum PayloadType {
    Circuit(CircuitManagementPayload),
    Consensus(ProposalId, (Proposal, CircuitManagementPayload)),
}

pub struct UnpeeredPendingPayload {
    pub ids: Vec<String>,
    pub payload_type: PayloadType,
    pub message_sender: String,
}

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

struct UninitializedCircuit {
    pub circuit: Option<CircuitProposal>,
    pub ready_members: HashSet<String>,
}

struct SubscriberMap {
    subscribers_by_type: RefCell<HashMap<String, Vec<Box<dyn AdminServiceEventSubscriber>>>>,
}

impl SubscriberMap {
    fn new() -> Self {
        Self {
            subscribers_by_type: RefCell::new(HashMap::new()),
        }
    }

    fn broadcast_by_type(
        &self,
        event_type: &str,
        admin_service_event: &messages::AdminServiceEvent,
        timestamp: &SystemTime,
    ) {
        let mut subscribers_by_type = self.subscribers_by_type.borrow_mut();
        if let Some(subscribers) = subscribers_by_type.get_mut(event_type) {
            subscribers.retain(|subscriber| {
                match subscriber.handle_event(admin_service_event, timestamp) {
                    Ok(()) => true,
                    Err(AdminSubscriberError::Unsubscribe) => false,
                    Err(AdminSubscriberError::UnableToHandleEvent(msg)) => {
                        error!("Unable to send event: {}", msg);
                        true
                    }
                }
            });
        }
    }

    fn add_subscriber(
        &mut self,
        event_type: String,
        listener: Box<dyn AdminServiceEventSubscriber>,
    ) {
        let mut subscribers_by_type = self.subscribers_by_type.borrow_mut();
        let subscribers = subscribers_by_type
            .entry(event_type)
            .or_insert_with(Vec::new);
        subscribers.push(listener);
    }

    fn clear(&mut self) {
        self.subscribers_by_type.borrow_mut().clear()
    }
}

pub struct AdminServiceShared {
    // the node id of the connected splinter node
    node_id: String,
    // the list of circuit proposal that are being voted on by members of a circuit
    open_proposals: OpenProposals,
    // the list of circuit that have been committed to splinter state but whose services haven't
    // been initialized
    uninitialized_circuits: HashMap<String, UninitializedCircuit>,
    // orchestrator used to initialize and shutdown services
    orchestrator: ServiceOrchestrator,
    // map of service arg validators, by service type
    #[cfg(feature = "service-arg-validation")]
    service_arg_validators: HashMap<String, Box<dyn ServiceArgValidator + Send>>,
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

    // Admin Service Event Subscribers
    event_subscribers: SubscriberMap,
    // Mailbox of AdminServiceEvent values
    event_mailbox: Mailbox,
    // copy of splinter state
    splinter_state: SplinterState,
    // signature verifier
    signature_verifier: Box<dyn SignatureVerifier + Send>,
    key_registry: Box<dyn KeyRegistry>,
    key_permission_manager: Box<dyn KeyPermissionManager>,
    proposal_sender: Option<Sender<ProposalUpdate>>,
}

impl AdminServiceShared {
    #![allow(clippy::too_many_arguments)]
    pub fn new(
        node_id: String,
        orchestrator: ServiceOrchestrator,
        #[cfg(feature = "service-arg-validation")] service_arg_validators: HashMap<
            String,
            Box<dyn ServiceArgValidator + Send>,
        >,
        peer_connector: PeerConnector,
        auth_inquisitor: Box<dyn AuthorizationInquisitor>,
        splinter_state: SplinterState,
        signature_verifier: Box<dyn SignatureVerifier + Send>,
        key_registry: Box<dyn KeyRegistry>,
        key_permission_manager: Box<dyn KeyPermissionManager>,
        storage_type: &str,
    ) -> Result<Self, ServiceError> {
        let location = {
            if let Ok(s) = env::var(STATE_DIR_ENV) {
                s
            } else {
                DEFAULT_STATE_DIR.to_string()
            }
        };

        let storage_location = match storage_type {
            "yaml" => format!("{}{}", location, "/circuit_proposals.yaml"),
            "memory" => "memory".to_string(),
            _ => panic!("Storage type is not supported: {}", storage_type),
        };

        let open_proposals = OpenProposals::new(storage_location)
            .map_err(|err| ServiceError::UnableToCreate(Box::new(err)))?;

        let event_mailbox = Mailbox::new(DurableBTreeSet::new_boxed_with_bound(
            std::num::NonZeroUsize::new(DEFAULT_IN_MEMORY_EVENT_LIMIT).unwrap(),
        ));
        Ok(AdminServiceShared {
            node_id,
            network_sender: None,
            open_proposals,
            uninitialized_circuits: Default::default(),
            orchestrator,
            #[cfg(feature = "service-arg-validation")]
            service_arg_validators,
            running_services: HashSet::new(),
            peer_connector,
            auth_inquisitor,
            unpeered_payloads: Vec::new(),
            pending_circuit_payloads: VecDeque::new(),
            pending_consensus_proposals: HashMap::new(),
            pending_changes: None,
            current_consensus_verifiers: Vec::new(),
            event_subscribers: SubscriberMap::new(),
            event_mailbox,
            splinter_state,
            signature_verifier,
            key_registry,
            key_permission_manager,
            proposal_sender: None,
        })
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

    pub fn set_proposal_sender(&mut self, proposal_sender: Option<Sender<ProposalUpdate>>) {
        self.proposal_sender = proposal_sender;
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
                let circuit_id = circuit_proposal.get_circuit_id();
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
                        self.remove_proposal(&circuit_id)?;
                        // send message about circuit acceptance

                        let circuit_proposal_proto =
                            messages::CircuitProposal::from_proto(circuit_proposal.clone())
                                .map_err(AdminSharedError::InvalidMessageFormat)?;
                        let event = messages::AdminServiceEvent::ProposalAccepted((
                            circuit_proposal_proto,
                            circuit_proposal_context.signer_public_key,
                        ));
                        self.send_event(&mgmt_type, event);

                        // send MEMBER_READY message to all other members' admin services
                        if let Some(ref network_sender) = self.network_sender {
                            let mut member_ready = MemberReady::new();
                            member_ready.set_circuit_id(circuit.circuit_id.clone());
                            member_ready.set_member_node_id(self.node_id.clone());
                            let mut msg = AdminMessage::new();
                            msg.set_message_type(AdminMessage_Type::MEMBER_READY);
                            msg.set_member_ready(member_ready);

                            let envelope_bytes =
                                msg.write_to_bytes().map_err(MarshallingError::from)?;
                            for member in circuit.members.iter() {
                                if member.get_node_id() != self.node_id {
                                    network_sender.send(
                                        &admin_service_id(member.get_node_id()),
                                        &envelope_bytes,
                                    )?;
                                }
                            }
                        }

                        // add circuit as pending initialization
                        self.add_uninitialized_circuit(circuit_proposal.clone())
                    }
                    Ok(CircuitProposalStatus::Pending) => {
                        self.add_proposal(circuit_proposal.clone())?;

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
                        self.remove_proposal(&circuit_id)?;

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
        self.validate_circuit_management_payload(&circuit_payload, &header)?;
        self.verify_signature(&circuit_payload).map_err(|_| {
            AdminSharedError::ValidationFailed(String::from("Unable to verify signature"))
        })?;
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
                    .get_proposal(proposal_vote.get_circuit_id())
                    .map_err(|err| {
                        AdminSharedError::ValidationFailed(format!(
                            "error occurred when trying to get proposal {}",
                            err
                        ))
                    })?
                    .ok_or_else(|| {
                        AdminSharedError::ValidationFailed(format!(
                            "Received vote for a proposal that does not exist: circuit id {}",
                            proposal_vote.circuit_id
                        ))
                    })?;

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
            CircuitManagementPayload_Action::ACTION_UNSET => Err(
                AdminSharedError::ValidationFailed("Action must be set".to_string()),
            ),
            unknown_action => Err(AdminSharedError::ValidationFailed(format!(
                "Unable to handle {:?}",
                unknown_action
            ))),
        }
    }

    pub fn has_proposal(&self, circuit_id: &str) -> bool {
        self.open_proposals.has_proposal(circuit_id)
    }

    /// Propose a new circuit
    ///
    /// This operation will propose a new circuit to all the member nodes of the circuit.  If there
    /// is no peer connection, a connection to the peer will also be established.
    pub fn propose_circuit(
        &mut self,
        payload: CircuitManagementPayload,
        message_sender: String,
    ) -> Result<(), ServiceError> {
        debug!(
            "received circuit proposal for {}",
            payload
                .get_circuit_create_request()
                .get_circuit()
                .get_circuit_id()
        );

        // get members as vec to payload can be sent to helper function as well
        let members = payload
            .get_circuit_create_request()
            .get_circuit()
            .get_members()
            .to_vec();
        self.check_connected_peers_payload(&members, payload, message_sender)
    }

    pub fn propose_vote(
        &mut self,
        payload: CircuitManagementPayload,
        message_sender: String,
    ) -> Result<(), ServiceError> {
        debug!(
            "received circuit vote for {}",
            payload.get_circuit_proposal_vote().get_circuit_id()
        );
        let circuit_id = payload.get_circuit_proposal_vote().get_circuit_id();
        let proposal = self
            .get_proposal(circuit_id)
            .map_err(|err| {
                ServiceError::UnableToHandleMessage(Box::new(AdminSharedError::ValidationFailed(
                    format!("error occurred when trying to get proposal {}", err),
                )))
            })?
            .ok_or_else(|| {
                ServiceError::UnableToHandleMessage(Box::new(AdminSharedError::ValidationFailed(
                    format!(
                        "Received vote for a proposal that does not exist: circuit id {}",
                        circuit_id
                    ),
                )))
            })?;

        self.check_connected_peers_payload(
            proposal.get_circuit_proposal().get_members(),
            payload,
            message_sender,
        )
    }

    fn check_connected_peers_payload(
        &mut self,
        members: &[SplinterNode],
        payload: CircuitManagementPayload,
        message_sender: String,
    ) -> Result<(), ServiceError> {
        let mut unauthorized_peers = vec![];
        for node in members {
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

            self.unpeered_payloads.push(UnpeeredPendingPayload {
                ids: unauthorized_peers,
                payload_type: PayloadType::Circuit(payload),
                message_sender,
            });
        }
        Ok(())
    }

    pub fn submit(&mut self, payload: CircuitManagementPayload) -> Result<(), ServiceError> {
        debug!("Payload submitted: {:?}", payload);

        let header =
            protobuf::parse_from_bytes::<CircuitManagementPayload_Header>(payload.get_header())?;
        self.validate_circuit_management_payload(&payload, &header)
            .map_err(|err| ServiceError::UnableToHandleMessage(Box::new(err)))?;
        self.verify_signature(&payload)?;

        match header.get_action() {
            CircuitManagementPayload_Action::CIRCUIT_CREATE_REQUEST => {
                let signer_public_key = header.get_requester();
                let requester_node_id = header.get_requester_node_id();
                self.validate_create_circuit(
                    payload.get_circuit_create_request().get_circuit(),
                    signer_public_key,
                    requester_node_id,
                )
                .map_err(|err| ServiceError::UnableToHandleMessage(Box::new(err)))?;

                self.propose_circuit(payload, "local".to_string())
            }
            CircuitManagementPayload_Action::CIRCUIT_PROPOSAL_VOTE => {
                let proposal_vote = payload.get_circuit_proposal_vote();

                // validate vote proposal
                // check that the circuit proposal exists
                let circuit_proposal = self
                    .get_proposal(proposal_vote.get_circuit_id())
                    .map_err(|err| {
                        ServiceError::UnableToHandleMessage(Box::new(
                            AdminSharedError::ValidationFailed(format!(
                                "error occurred when trying to get proposal {}",
                                err
                            )),
                        ))
                    })?
                    .ok_or_else(|| {
                        ServiceError::UnableToHandleMessage(Box::new(
                            AdminSharedError::ValidationFailed(format!(
                                "Received vote for a proposal that does not exist: circuit id {}",
                                proposal_vote.circuit_id
                            )),
                        ))
                    })?;

                let signer_public_key = header.get_requester();
                self.validate_circuit_vote(
                    proposal_vote,
                    signer_public_key,
                    &circuit_proposal,
                    header.get_requester_node_id(),
                )
                .map_err(|err| ServiceError::UnableToHandleMessage(Box::new(err)))?;

                self.propose_vote(payload, "local".to_string())
            }
            CircuitManagementPayload_Action::ACTION_UNSET => {
                Err(ServiceError::UnableToHandleMessage(Box::new(
                    AdminSharedError::ValidationFailed(String::from("No action specified")),
                )))
            }
            unknown_action => Err(ServiceError::UnableToHandleMessage(Box::new(
                AdminSharedError::ValidationFailed(format!(
                    "Unable to handle {:?}",
                    unknown_action
                )),
            ))),
        }
    }

    /// Handle a new circuit proposal
    ///
    /// This operation will accept a new circuit proposal.  If there is no peer connection, a
    /// connection to the peer will also be established.
    pub fn handle_proposed_circuit(
        &mut self,
        proposal: Proposal,
        payload: CircuitManagementPayload,
        message_sender: String,
    ) -> Result<(), ServiceError> {
        let mut unauthorized_peers = vec![];
        for node in payload
            .get_circuit_create_request()
            .get_circuit()
            .get_members()
        {
            if self.node_id() != node.get_node_id() {
                if self.auth_inquisitor().is_authorized(node.get_node_id()) {
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
            self.add_pending_consensus_proposal(proposal.id.clone(), (proposal.clone(), payload));
            self.proposal_sender
                .as_ref()
                .ok_or_else(|| ServiceError::NotStarted)?
                .send(ProposalUpdate::ProposalReceived(
                    proposal,
                    message_sender.as_bytes().into(),
                ))
                .map_err(|err| ServiceError::UnableToHandleMessage(Box::new(err)))
        } else {
            debug!(
                "Members {:?} added; awaiting network authorization before proceeding",
                &unauthorized_peers
            );

            self.unpeered_payloads.push(UnpeeredPendingPayload {
                ids: unauthorized_peers,
                payload_type: PayloadType::Consensus(proposal.id.clone(), (proposal, payload)),
                message_sender,
            });
            Ok(())
        }
    }

    pub fn get_events_since(
        &self,
        since_timestamp: &SystemTime,
        circuit_management_type: &str,
    ) -> Result<Events, AdminSharedError> {
        let events = self
            .event_mailbox
            .iter_since(*since_timestamp)
            .map_err(|err| AdminSharedError::UnableToAddSubscriber(err.to_string()))?;

        let circuit_management_type = circuit_management_type.to_string();
        Ok(Events {
            inner: Box::new(events.filter(move |(_, evt)| {
                evt.proposal().circuit.circuit_management_type == circuit_management_type
            })),
        })
    }

    pub fn add_subscriber(
        &mut self,
        circuit_management_type: String,
        subscriber: Box<dyn AdminServiceEventSubscriber>,
    ) -> Result<(), AdminSharedError> {
        self.event_subscribers
            .add_subscriber(circuit_management_type, subscriber);

        Ok(())
    }

    pub fn send_event(
        &mut self,
        circuit_management_type: &str,
        event: messages::AdminServiceEvent,
    ) {
        let (ts, event) = match self.event_mailbox.add(event) {
            Ok((ts, event)) => (ts, event),
            Err(err) => {
                error!("Unable to store admin event: {}", err);
                return;
            }
        };

        self.event_subscribers
            .broadcast_by_type(&circuit_management_type, &event, &ts);
    }

    pub fn remove_all_event_subscribers(&mut self) {
        self.event_subscribers.clear();
    }

    pub fn on_authorization_change(
        &mut self,
        peer_id: &str,
        state: PeerAuthorizationState,
    ) -> Result<(), AuthorizationCallbackError> {
        let mut unpeered_payloads = std::mem::replace(&mut self.unpeered_payloads, vec![]);
        for unpeered_payload in unpeered_payloads.iter_mut() {
            match state {
                PeerAuthorizationState::Authorized => {
                    unpeered_payload
                        .ids
                        .retain(|unpeered_id| unpeered_id != peer_id);
                }
                PeerAuthorizationState::Unauthorized => {
                    if unpeered_payload
                        .ids
                        .iter()
                        .any(|unpeered_id| unpeered_id == peer_id)
                    {
                        warn!(
                            "Dropping circuit request including peer {}, \
                             due to authorization failure",
                            peer_id
                        );
                        unpeered_payload.ids.clear();
                    }
                }
            }
        }

        let (fully_peered, still_unpeered): (
            Vec<UnpeeredPendingPayload>,
            Vec<UnpeeredPendingPayload>,
        ) = unpeered_payloads
            .into_iter()
            .partition(|unpeered_payload| unpeered_payload.ids.is_empty());

        std::mem::replace(&mut self.unpeered_payloads, still_unpeered);
        if state == PeerAuthorizationState::Authorized {
            for peered_payload in fully_peered {
                match peered_payload.payload_type {
                    PayloadType::Circuit(payload) => {
                        self.pending_circuit_payloads.push_back(payload)
                    }
                    PayloadType::Consensus(id, (proposal, payload)) => {
                        self.add_pending_consensus_proposal(id, (proposal.clone(), payload));

                        if let Some(proposal_sender) = &self.proposal_sender {
                            proposal_sender
                                .send(ProposalUpdate::ProposalReceived(
                                    proposal,
                                    peered_payload.message_sender.as_bytes().into(),
                                ))
                                .map_err(|err| {
                                    AuthorizationCallbackError(format!(
                                        "Unable to send consensus proposal update: {}",
                                        err
                                    ))
                                })?;
                        } else {
                            return Err(AuthorizationCallbackError(
                                "AdminService is not started".into(),
                            ));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn get_proposal(
        &self,
        circuit_id: &str,
    ) -> Result<Option<CircuitProposal>, AdminSharedError> {
        Ok(self.open_proposals.get_proposal(circuit_id)?)
    }

    #[cfg(feature = "proposal-read")]
    pub fn get_proposals(&self) -> ProposalIter {
        self.open_proposals.get_proposals()
    }

    pub fn remove_proposal(
        &mut self,
        circuit_id: &str,
    ) -> Result<Option<CircuitProposal>, AdminSharedError> {
        Ok(self.open_proposals.remove_proposal(circuit_id)?)
    }

    pub fn add_proposal(
        &mut self,
        circuit_proposal: CircuitProposal,
    ) -> Result<Option<CircuitProposal>, AdminSharedError> {
        Ok(self.open_proposals.add_proposal(circuit_proposal)?)
    }

    /// Add a circuit definition as an uninitialized circuit. If all members are ready, initialize
    /// services.
    fn add_uninitialized_circuit(
        &mut self,
        circuit: CircuitProposal,
    ) -> Result<(), AdminSharedError> {
        let circuit_id = circuit.get_circuit_id().to_string();

        // If uninitialized circuit already exists, add the circuit definition; if not, create the
        // uninitialized circuit.
        match self.uninitialized_circuits.get_mut(&circuit_id) {
            Some(uninit_circuit) => uninit_circuit.circuit = Some(circuit),
            None => {
                self.uninitialized_circuits.insert(
                    circuit_id.to_string(),
                    UninitializedCircuit {
                        circuit: Some(circuit),
                        ready_members: HashSet::new(),
                    },
                );
            }
        }

        // Add self as ready
        self.uninitialized_circuits
            .get_mut(&circuit_id)
            .expect("Uninitialized circuit not set")
            .ready_members
            .insert(self.node_id.clone());

        self.initialize_services_if_members_ready(&circuit_id)
    }

    /// Mark member node as ready to initialize services on the given circuit. If all members are
    /// now ready, initialize services.
    pub fn add_ready_member(
        &mut self,
        circuit_id: &str,
        member_node_id: String,
    ) -> Result<(), AdminSharedError> {
        // If uninitialized circuit does not already exist, create it
        if self.uninitialized_circuits.get(circuit_id).is_none() {
            self.uninitialized_circuits.insert(
                circuit_id.to_string(),
                UninitializedCircuit {
                    circuit: None,
                    ready_members: HashSet::new(),
                },
            );
        }

        self.uninitialized_circuits
            .get_mut(circuit_id)
            .expect("Uninitialized circuit not set")
            .ready_members
            .insert(member_node_id);

        self.initialize_services_if_members_ready(circuit_id)
    }

    /// If all members of an uninitialized circuit are ready, initialize services. Also send
    /// CircuitReady notification to application authorization handler.
    fn initialize_services_if_members_ready(
        &mut self,
        circuit_id: &str,
    ) -> Result<(), AdminSharedError> {
        let ready = {
            if let Some(uninitialized_circuit) = self.uninitialized_circuits.get(circuit_id) {
                if let Some(ref circuit_proposal) = uninitialized_circuit.circuit {
                    let all_members = HashSet::from_iter(
                        circuit_proposal
                            .get_circuit_proposal()
                            .members
                            .iter()
                            .map(|node| node.node_id.clone()),
                    );
                    all_members.is_subset(&uninitialized_circuit.ready_members)
                } else {
                    false
                }
            } else {
                false
            }
        };

        if ready {
            let circuit_proposal = self
                .uninitialized_circuits
                .remove(circuit_id)
                .expect("Uninitialized circuit not set")
                .circuit
                .expect("Uninitialized circuit's circuit proposal not set");
            self.initialize_services(circuit_proposal.get_circuit_proposal())?;

            let mgmt_type = circuit_proposal
                .get_circuit_proposal()
                .circuit_management_type
                .clone();
            let event = messages::AdminServiceEvent::CircuitReady(
                messages::CircuitProposal::from_proto(circuit_proposal)?,
            );
            self.send_event(&mgmt_type, event);
        }

        Ok(())
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

        self.validate_key(signer_public_key)?;

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

        if self.splinter_state.has_circuit(circuit.get_circuit_id())? {
            return Err(AdminSharedError::ValidationFailed(format!(
                "Circuit with circuit id {} already exists",
                circuit.get_circuit_id()
            )));
        }

        self.validate_circuit(circuit)?;
        Ok(())
    }

    fn validate_key(&self, public_key: &[u8]) -> Result<(), AdminSharedError> {
        if public_key.len() != 33 {
            return Err(AdminSharedError::ValidationFailed(format!(
                "{} is not a valid public key: invalid length",
                to_hex(public_key)
            )));
        }

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

        let mut members: Vec<String> = Vec::new();
        let mut endpoints: Vec<String> = Vec::new();
        for member in circuit.get_members() {
            let node_id = member.get_node_id().to_string();
            if node_id.is_empty() {
                return Err(AdminSharedError::ValidationFailed(
                    "Member node id cannot be empty".to_string(),
                ));
            } else if members.contains(&node_id) {
                return Err(AdminSharedError::ValidationFailed(
                    "Every member must be unique in the circuit.".to_string(),
                ));
            } else {
                members.push(node_id);
            }

            let endpoint = member.get_endpoint().to_string();
            if endpoint.is_empty() {
                return Err(AdminSharedError::ValidationFailed(
                    "Member endpoint cannot be empty".to_string(),
                ));
            } else if endpoints.contains(&endpoint) {
                return Err(AdminSharedError::ValidationFailed(
                    "Every member endpoint must be unique in the circuit.".to_string(),
                ));
            } else {
                endpoints.push(endpoint);
            }
        }

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

        let mut services: Vec<String> = Vec::new();
        // check that all services' allowed nodes are in members
        for service in circuit.get_roster() {
            if service.get_allowed_nodes().is_empty() {
                return Err(AdminSharedError::ValidationFailed(
                    "Service cannot have an empty allowed nodes list".to_string(),
                ));
            }

            if service.get_allowed_nodes().len() > 1 {
                return Err(AdminSharedError::ValidationFailed(
                    "Only one allowed node for a service is supported".to_string(),
                ));
            }

            for node in service.get_allowed_nodes() {
                if !members.contains(node) {
                    return Err(AdminSharedError::ValidationFailed(format!(
                        "Service cannot have an allowed node that is not in members: {}",
                        node
                    )));
                }
            }

            let service_id = service.get_service_id().to_string();
            if service_id.is_empty() {
                return Err(AdminSharedError::ValidationFailed(
                    "Service id cannot be empty".to_string(),
                ));
            } else if services.contains(&service_id) {
                return Err(AdminSharedError::ValidationFailed(
                    "Every service must be unique in the circuit.".to_string(),
                ));
            } else {
                services.push(service_id)
            }

            #[cfg(feature = "service-arg-validation")]
            {
                self.validate_service_args(&service)?;
            }
        }

        if circuit.get_circuit_management_type().is_empty() {
            return Err(AdminSharedError::ValidationFailed(
                "The circuit must have a mangement type".to_string(),
            ));
        }

        Ok(())
    }

    #[cfg(feature = "service-arg-validation")]
    fn validate_service_args(&self, service: &SplinterService) -> Result<(), AdminSharedError> {
        if let Some(validator) = self.service_arg_validators.get(service.get_service_type()) {
            let args: HashMap<String, String> = service
                .get_arguments()
                .iter()
                .map(|arg| (arg.get_key().into(), arg.get_value().into()))
                .collect();

            validator
                .validate(&args)
                .map_err(|err| AdminSharedError::ValidationFailed(err.to_string()))
        } else {
            Ok(())
        }
    }

    fn validate_circuit_vote(
        &self,
        proposal_vote: &CircuitProposalVote,
        signer_public_key: &[u8],
        circuit_proposal: &CircuitProposal,
        node_id: &str,
    ) -> Result<(), AdminSharedError> {
        let circuit_hash = proposal_vote.get_circuit_hash();

        self.validate_key(signer_public_key)?;

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
                "Payload requester {} is not registered to node {}",
                to_hex(circuit_proposal.get_requester()),
                node_id
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

    fn validate_circuit_management_payload(
        &self,
        payload: &CircuitManagementPayload,
        header: &CircuitManagementPayload_Header,
    ) -> Result<(), AdminSharedError> {
        // Validate payload signature
        if payload.get_signature().is_empty() {
            return Err(AdminSharedError::ValidationFailed(
                "CircuitManagementPayload signature must be set".to_string(),
            ));
        };

        // Validate the payload header
        if payload.get_header().is_empty() {
            return Err(AdminSharedError::ValidationFailed(
                "CircuitManagementPayload header must be set".to_string(),
            ));
        };

        // Validate the header, requester field is set
        if header.get_requester().is_empty() {
            return Err(AdminSharedError::ValidationFailed(
                "CircuitManagementPayload must have a requester".to_string(),
            ));
        };

        self.validate_key(header.get_requester())?;

        // Validate the header, requester_node_id is set
        if header.get_requester_node_id().is_empty() {
            return Err(AdminSharedError::ValidationFailed(
                "CircuitManagementPayload must have a requester node id".to_string(),
            ));
        };

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
    /// orchestrator. This may not include all services if they are not supported locally. It is
    /// expected that some services will be started externally.
    pub fn initialize_services(&mut self, circuit: &Circuit) -> Result<(), AdminSharedError> {
        // Get all services this node is allowed to run
        let services = circuit
            .get_roster()
            .iter()
            .filter(|service| {
                service.allowed_nodes.contains(&self.node_id)
                    && self
                        .orchestrator
                        .supported_service_types()
                        .contains(&service.get_service_type().to_string())
            })
            .collect::<Vec<_>>();

        // Start all services the orchestrator has a factory for
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

    /// Stops all running services
    pub fn stop_services(&mut self) -> Result<(), AdminSharedError> {
        let shutdown_errors = self
            .running_services
            .iter()
            .map(|service| {
                debug!(
                    "Stopping service {} in circuit {}",
                    service.service_type, service.circuit
                );
                self.orchestrator.shutdown_service(&service)
            })
            .filter_map(Result::err)
            .collect::<Vec<ShutdownServiceError>>();

        self.running_services = HashSet::new();

        if shutdown_errors.is_empty() {
            Ok(())
        } else {
            Err(AdminSharedError::ServiceShutdownFailed(shutdown_errors))
        }
    }

    /// On restart of a splinter node, all services that this node should run on the existing
    /// circuits should be intialized using the service orchestrator. This may not include all
    /// services if they are not supported locally. It is expected that some services will be
    /// started externally.
    pub fn restart_services(&mut self) -> Result<(), AdminSharedError> {
        let circuits = self.splinter_state.circuits()?;

        // start all services of the supported types
        for (circuit_name, circuit) in circuits.iter() {
            // Get all services this node is allowed to run and the orchestrator has a factory for
            let services = circuit
                .roster()
                .iter()
                .filter(|service| {
                    service.allowed_nodes().contains(&self.node_id)
                        && self
                            .orchestrator
                            .supported_service_types()
                            .contains(&service.service_type().to_string())
                })
                .collect::<Vec<_>>();

            // Start all services
            for service in services {
                let service_definition = ServiceDefinition {
                    circuit: circuit_name.into(),
                    service_id: service.service_id().into(),
                    service_type: service.service_type().into(),
                };

                let service_arguments = service
                    .arguments()
                    .iter()
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect();

                self.orchestrator
                    .initialize_service(service_definition.clone(), service_arguments)?;

                self.running_services.insert(service_definition);
            }
        }
        Ok(())
    }

    fn update_splinter_state(&mut self, circuit: &Circuit) -> Result<(), AdminSharedError> {
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
            Circuit_AuthorizationType::TRUST_AUTHORIZATION => AuthorizationType::Trust,
            // This should never happen
            Circuit_AuthorizationType::UNSET_AUTHORIZATION_TYPE => {
                return Err(AdminSharedError::CommitError(
                    "Missing authorization type on circuit commit".to_string(),
                ))
            }
        };

        let persistence = match circuit.get_persistence() {
            Circuit_PersistenceType::ANY_PERSISTENCE => PersistenceType::Any,
            // This should never happen
            Circuit_PersistenceType::UNSET_PERSISTENCE_TYPE => {
                return Err(AdminSharedError::CommitError(
                    "Missing persistence type on circuit commit".to_string(),
                ))
            }
        };

        let durability = match circuit.get_durability() {
            Circuit_DurabilityType::NO_DURABILITY => DurabilityType::NoDurability,
            // This should never happen
            Circuit_DurabilityType::UNSET_DURABILITY_TYPE => {
                return Err(AdminSharedError::CommitError(
                    "Missing durabilty type on circuit commit".to_string(),
                ))
            }
        };

        let routes = match circuit.get_routes() {
            Circuit_RouteType::ANY_ROUTE => RouteType::Any,
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
            .with_roster(roster.clone())
            .with_auth(auth)
            .with_persistence(persistence)
            .with_durability(durability)
            .with_routes(routes)
            .with_circuit_management_type(circuit.get_circuit_management_type().to_string())
            .build()
            .map_err(|err| {
                AdminSharedError::CommitError(format!("Unable build new circuit: {}", err))
            })?;

        for member in members {
            self.splinter_state
                .add_node(member.id().to_string(), member)?;
        }

        self.splinter_state
            .add_circuit(new_circuit.id().to_string(), new_circuit)?;

        for service in roster {
            if service.allowed_nodes().contains(&self.node_id) {
                continue;
            }

            let unique_id = ServiceId::new(
                circuit.circuit_id.to_string(),
                service.service_id().to_string(),
            );

            let allowed_node = &service.allowed_nodes()[0];
            if let Some(member) = self.splinter_state.node(&allowed_node)? {
                let service = Service::new(service.service_id().to_string(), None, member.clone());
                self.splinter_state.add_service(unique_id, service)?;
            } else {
                return Err(AdminSharedError::CommitError(format!(
                    "Unable to find allowed node {} when adding service {} to directory",
                    allowed_node,
                    service.service_id()
                )));
            }
        }

        Ok(())
    }

    pub fn add_services_to_directory(&mut self) -> Result<(), AdminSharedError> {
        let circuits = self.splinter_state.circuits()?;
        for (id, circuit) in circuits {
            for service in circuit.roster() {
                if service.allowed_nodes().contains(&self.node_id) {
                    continue;
                }
                let unique_id = ServiceId::new(id.to_string(), service.service_id().to_string());

                let allowed_node = &service.allowed_nodes()[0];
                if let Some(member) = self.splinter_state.node(&allowed_node)? {
                    // rebuild Node with id
                    let node =
                        StateNode::new(allowed_node.to_string(), member.endpoints().to_vec());
                    let service = Service::new(service.service_id().to_string(), None, node);
                    self.splinter_state.add_service(unique_id, service)?
                } else {
                    return Err(AdminSharedError::CommitError(format!(
                        "Unable to find allowed node {} when adding service {} to directory",
                        allowed_node,
                        service.service_id()
                    )));
                }
            }
        }

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
    use crate::signing::{
        hash::{HashSigner, HashVerifier},
        Signer,
    };
    use crate::storage::get_storage;
    use crate::transport::{
        ConnectError, Connection, DisconnectError, RecvError, SendError, Transport,
    };

    /// Test that the CircuitManagementPayload is moved to the pending payloads when the peers are
    /// fully authorized.
    #[test]
    fn test_auth_change() {
        let mesh = Mesh::new(4, 16);
        let network = Network::new(mesh.clone(), 0).unwrap();
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
            #[cfg(feature = "service-arg-validation")]
            HashMap::new(),
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
            Box::new(HashVerifier),
            Box::new(key_registry),
            Box::new(AllowAllKeyPermissionManager),
            "memory",
        )
        .unwrap();

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
            .propose_circuit(payload, "test".to_string())
            .expect("Proposal not accepted");

        // None of the proposed members are peered
        assert_eq!(0, shared.pending_circuit_payloads.len());
        shared
            .on_authorization_change("test-node", PeerAuthorizationState::Authorized)
            .expect("received unexpected error");

        // One node is still unpeered
        assert_eq!(0, shared.pending_circuit_payloads.len());
        shared
            .on_authorization_change("other-node", PeerAuthorizationState::Authorized)
            .expect("received unexpected error");

        // We're fully peered, so the pending payload is now available
        assert_eq!(1, shared.pending_circuit_payloads.len());
    }

    /// Test that the CircuitManagementPayload message is dropped, if a node fails authorization.
    #[test]
    fn test_unauth_change() {
        let mesh = Mesh::new(4, 16);
        let network = Network::new(mesh.clone(), 0).unwrap();
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
            #[cfg(feature = "service-arg-validation")]
            HashMap::new(),
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
            Box::new(HashVerifier),
            Box::new(key_registry),
            Box::new(AllowAllKeyPermissionManager),
            "memory",
        )
        .unwrap();

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
            .propose_circuit(payload, "local".to_string())
            .expect("Proposal not accepted");

        // None of the proposed members are peered
        assert_eq!(1, shared.unpeered_payloads.len());
        assert_eq!(0, shared.pending_circuit_payloads.len());
        shared
            .on_authorization_change("test-node", PeerAuthorizationState::Unauthorized)
            .expect("received unexpected error");

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
        let pub_key = (0u8..33).collect::<Vec<_>>();
        let mut key_registry = StorageKeyRegistry::new("memory".to_string()).unwrap();
        let key_info = KeyInfo::builder(pub_key.clone(), "node_a".to_string()).build();
        key_registry.save_key(key_info).unwrap();

        let admin_shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            #[cfg(feature = "service-arg-validation")]
            HashMap::new(),
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
            Box::new(HashVerifier),
            Box::new(key_registry),
            Box::new(AllowAllKeyPermissionManager),
            "memory",
        )
        .unwrap();
        let circuit = setup_test_circuit();

        if let Err(err) = admin_shared.validate_create_circuit(&circuit, &pub_key, "node_a") {
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
            #[cfg(feature = "service-arg-validation")]
            HashMap::new(),
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
            Box::new(HashVerifier),
            Box::new(key_registry),
            Box::new(AllowAllKeyPermissionManager),
            "memory",
        )
        .unwrap();
        let circuit = setup_test_circuit();

        let pub_key = (0u8..33).collect::<Vec<_>>();
        if let Ok(_) = admin_shared.validate_create_circuit(&circuit, &pub_key, "node_a") {
            panic!("Should have been invalid due to signer not being registered to a node");
        }
    }

    #[test]
    // test that if a circuit is proposed by a signer key is not a valid public key the proposal is
    // invalid
    fn test_validate_circuit_signer_key_invalid() {
        let state = setup_splinter_state();
        let peer_connector = setup_peer_connector();
        let orchestrator = setup_orchestrator();
        let key_registry = StorageKeyRegistry::new("memory".to_string()).unwrap();

        let admin_shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            #[cfg(feature = "service-arg-validation")]
            HashMap::new(),
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
            Box::new(HashVerifier),
            Box::new(key_registry),
            Box::new(AllowAllKeyPermissionManager),
            "memory",
        )
        .unwrap();
        let circuit = setup_test_circuit();

        let pub_key = (0u8..50).collect::<Vec<_>>();
        // too short
        if let Ok(_) = admin_shared.validate_create_circuit(&circuit, &pub_key[0..10], "node_a") {
            panic!("Should have been invalid due to key being too short");
        }
        // too long
        if let Ok(_) = admin_shared.validate_create_circuit(&circuit, &pub_key, "node_a") {
            panic!("Should have been invalid due to key being too long");
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
        let pub_key = (0u8..33).collect::<Vec<_>>();
        let mut key_registry = StorageKeyRegistry::new("memory".to_string()).unwrap();
        let key_info = KeyInfo::builder(pub_key.clone(), "node_a".to_string()).build();
        key_registry.save_key(key_info).unwrap();

        let admin_shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            #[cfg(feature = "service-arg-validation")]
            HashMap::new(),
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
            Box::new(HashVerifier),
            Box::new(key_registry),
            Box::new(AllowAllKeyPermissionManager),
            "memory",
        )
        .unwrap();
        let mut circuit = setup_test_circuit();

        let mut service_bad = SplinterService::new();
        service_bad.set_service_id("service_b".to_string());
        service_bad.set_service_type("type_a".to_string());
        service_bad.set_allowed_nodes(RepeatedField::from_vec(vec!["node_bad".to_string()]));

        circuit.set_roster(RepeatedField::from_vec(vec![service_bad]));

        if let Ok(_) = admin_shared.validate_create_circuit(&circuit, &pub_key, "node_a") {
            panic!("Should have been invalid due to service having an allowed node not in members");
        }
    }

    #[test]
    // test that if a circuit has a service in its roster with too many allowed nodes
    fn test_validate_circuit_too_many_allowed_nodes() {
        let state = setup_splinter_state();
        let peer_connector = setup_peer_connector();
        let orchestrator = setup_orchestrator();
        // set up key registry
        let pub_key = (0u8..33).collect::<Vec<_>>();
        let mut key_registry = StorageKeyRegistry::new("memory".to_string()).unwrap();
        let key_info = KeyInfo::builder(pub_key.clone(), "node_a".to_string()).build();
        key_registry.save_key(key_info).unwrap();

        let admin_shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            #[cfg(feature = "service-arg-validation")]
            HashMap::new(),
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
            Box::new(HashVerifier),
            Box::new(key_registry),
            Box::new(AllowAllKeyPermissionManager),
            "memory",
        )
        .unwrap();
        let mut circuit = setup_test_circuit();

        let mut service_bad = SplinterService::new();
        service_bad.set_service_id("service_b".to_string());
        service_bad.set_service_type("type_a".to_string());
        service_bad.set_allowed_nodes(RepeatedField::from_vec(vec![
            "node_b".to_string(),
            "extra".to_string(),
        ]));

        circuit.set_roster(RepeatedField::from_vec(vec![service_bad]));

        if let Ok(_) = admin_shared.validate_create_circuit(&circuit, &pub_key, "node_a") {
            panic!("Should have been invalid due to service having too many allowed nodes");
        }
    }

    #[test]
    // test that if a circuit has a service with "" for a service id an error is returned
    fn test_validate_circuit_empty_service_id() {
        let state = setup_splinter_state();
        let peer_connector = setup_peer_connector();
        let orchestrator = setup_orchestrator();
        // set up key registry
        let pub_key = (0u8..33).collect::<Vec<_>>();
        let mut key_registry = StorageKeyRegistry::new("memory".to_string()).unwrap();
        let key_info = KeyInfo::builder(pub_key.clone(), "node_a".to_string()).build();
        key_registry.save_key(key_info).unwrap();

        let admin_shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            #[cfg(feature = "service-arg-validation")]
            HashMap::new(),
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
            Box::new(HashVerifier),
            Box::new(key_registry),
            Box::new(AllowAllKeyPermissionManager),
            "memory",
        )
        .unwrap();
        let mut circuit = setup_test_circuit();

        let mut service_ = SplinterService::new();
        service_.set_service_id("".to_string());
        service_.set_service_type("type_a".to_string());
        service_.set_allowed_nodes(RepeatedField::from_vec(vec!["node_a".to_string()]));

        circuit.set_roster(RepeatedField::from_vec(vec![service_]));

        if let Ok(_) = admin_shared.validate_create_circuit(&circuit, &pub_key, "node_a") {
            panic!("Should have been invalid due to service's id being empty");
        }
    }

    #[test]
    // test that if a circuit has a service with duplicate service ids an error is returned
    fn test_validate_circuit_duplicate_service_id() {
        let state = setup_splinter_state();
        let peer_connector = setup_peer_connector();
        let orchestrator = setup_orchestrator();
        // set up key registry
        let pub_key = (0u8..33).collect::<Vec<_>>();
        let mut key_registry = StorageKeyRegistry::new("memory".to_string()).unwrap();
        let key_info = KeyInfo::builder(pub_key.clone(), "node_a".to_string()).build();
        key_registry.save_key(key_info).unwrap();

        let admin_shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            #[cfg(feature = "service-arg-validation")]
            HashMap::new(),
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
            Box::new(HashVerifier),
            Box::new(key_registry),
            Box::new(AllowAllKeyPermissionManager),
            "memory",
        )
        .unwrap();
        let mut circuit = setup_test_circuit();

        let mut service_a = SplinterService::new();
        service_a.set_service_id("service_a".to_string());
        service_a.set_service_type("type_a".to_string());
        service_a.set_allowed_nodes(RepeatedField::from_vec(vec!["node_a".to_string()]));

        let mut service_a2 = SplinterService::new();
        service_a2.set_service_id("service_a".to_string());
        service_a2.set_service_type("type_a".to_string());
        service_a2.set_allowed_nodes(RepeatedField::from_vec(vec!["node_b".to_string()]));

        circuit.set_roster(RepeatedField::from_vec(vec![service_a, service_a2]));

        if let Ok(_) = admin_shared.validate_create_circuit(&circuit, &pub_key, "node_a") {
            panic!("Should have been invalid due to service's id being a duplicate");
        }
    }

    #[test]
    // test that if a circuit does not have any services in its roster an error is returned
    fn test_validate_circuit_empty_roster() {
        let state = setup_splinter_state();
        let peer_connector = setup_peer_connector();
        let orchestrator = setup_orchestrator();
        // set up key registry
        let pub_key = (0u8..33).collect::<Vec<_>>();
        let mut key_registry = StorageKeyRegistry::new("memory".to_string()).unwrap();
        let key_info = KeyInfo::builder(pub_key.clone(), "node_a".to_string()).build();
        key_registry.save_key(key_info).unwrap();

        let admin_shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            #[cfg(feature = "service-arg-validation")]
            HashMap::new(),
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
            Box::new(HashVerifier),
            Box::new(key_registry),
            Box::new(AllowAllKeyPermissionManager),
            "memory",
        )
        .unwrap();
        let mut circuit = setup_test_circuit();
        circuit.set_roster(RepeatedField::from_vec(vec![]));

        if let Ok(_) = admin_shared.validate_create_circuit(&circuit, &pub_key, "node_a") {
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
        let pub_key = (0u8..33).collect::<Vec<_>>();
        let mut key_registry = StorageKeyRegistry::new("memory".to_string()).unwrap();
        let key_info = KeyInfo::builder(pub_key.clone(), "node_a".to_string()).build();
        key_registry.save_key(key_info).unwrap();

        let admin_shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            #[cfg(feature = "service-arg-validation")]
            HashMap::new(),
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
            Box::new(HashVerifier),
            Box::new(key_registry),
            Box::new(AllowAllKeyPermissionManager),
            "memory",
        )
        .unwrap();
        let mut circuit = setup_test_circuit();

        circuit.set_members(RepeatedField::from_vec(vec![]));

        if let Ok(_) = admin_shared.validate_create_circuit(&circuit, &pub_key, "node_a") {
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
        let pub_key = (0u8..33).collect::<Vec<_>>();
        let mut key_registry = StorageKeyRegistry::new("memory".to_string()).unwrap();
        let key_info = KeyInfo::builder(pub_key.clone(), "node_a".to_string()).build();
        key_registry.save_key(key_info).unwrap();

        let admin_shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            #[cfg(feature = "service-arg-validation")]
            HashMap::new(),
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
            Box::new(HashVerifier),
            Box::new(key_registry),
            Box::new(AllowAllKeyPermissionManager),
            "memory",
        )
        .unwrap();
        let mut circuit = setup_test_circuit();

        let mut node_b = SplinterNode::new();
        node_b.set_node_id("node_b".to_string());
        node_b.set_endpoint("test://endpoint_b:0".to_string());

        circuit.set_members(RepeatedField::from_vec(vec![node_b]));

        if let Ok(_) = admin_shared.validate_create_circuit(&circuit, &pub_key, "node_a") {
            panic!("Should have been invalid because node_a is not in members");
        }
    }

    #[test]
    // test that if a circuit has a member with node id of "" an error is
    // returned
    fn test_validate_circuit_empty_node_id() {
        let state = setup_splinter_state();
        let peer_connector = setup_peer_connector();
        let orchestrator = setup_orchestrator();
        // set up key registry
        let pub_key = (0u8..33).collect::<Vec<_>>();
        let mut key_registry = StorageKeyRegistry::new("memory".to_string()).unwrap();
        let key_info = KeyInfo::builder(pub_key.clone(), "node_a".to_string()).build();
        key_registry.save_key(key_info).unwrap();

        let admin_shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            #[cfg(feature = "service-arg-validation")]
            HashMap::new(),
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
            Box::new(HashVerifier),
            Box::new(key_registry),
            Box::new(AllowAllKeyPermissionManager),
            "memory",
        )
        .unwrap();
        let mut circuit = setup_test_circuit();

        let mut node_a = SplinterNode::new();
        node_a.set_node_id("node_a".to_string());
        node_a.set_endpoint("test://endpoint_a:0".to_string());

        let mut node_b = SplinterNode::new();
        node_b.set_node_id("node_b".to_string());
        node_b.set_endpoint("test://endpoint_b:0".to_string());

        let mut node_ = SplinterNode::new();
        node_.set_node_id("".to_string());
        node_.set_endpoint("test://endpoint_:0".to_string());

        circuit.set_members(RepeatedField::from_vec(vec![node_a, node_b, node_]));

        if let Ok(_) = admin_shared.validate_create_circuit(&circuit, &pub_key, "node_a") {
            panic!("Should have been invalid because node_ is has an empty node id");
        }
    }

    #[test]
    // test that if a circuit has duplicate members an error is returned
    fn test_validate_circuit_duplicate_members() {
        let state = setup_splinter_state();
        let peer_connector = setup_peer_connector();
        let orchestrator = setup_orchestrator();
        // set up key registry
        let pub_key = (0u8..33).collect::<Vec<_>>();
        let mut key_registry = StorageKeyRegistry::new("memory".to_string()).unwrap();
        let key_info = KeyInfo::builder(pub_key.clone(), "node_a".to_string()).build();
        key_registry.save_key(key_info).unwrap();

        let admin_shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            #[cfg(feature = "service-arg-validation")]
            HashMap::new(),
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
            Box::new(HashVerifier),
            Box::new(key_registry),
            Box::new(AllowAllKeyPermissionManager),
            "memory",
        )
        .unwrap();
        let mut circuit = setup_test_circuit();

        let mut node_a = SplinterNode::new();
        node_a.set_node_id("node_a".to_string());
        node_a.set_endpoint("test://endpoint_a:0".to_string());

        let mut node_b = SplinterNode::new();
        node_b.set_node_id("node_b".to_string());
        node_b.set_endpoint("test://endpoint_b:0".to_string());

        let mut node_b2 = SplinterNode::new();
        node_b2.set_node_id("node_b".to_string());
        node_b2.set_endpoint("test://endpoint_b2:0".to_string());

        circuit.set_members(RepeatedField::from_vec(vec![node_a, node_b, node_b2]));

        if let Ok(_) = admin_shared.validate_create_circuit(&circuit, &pub_key, "node_a") {
            panic!("Should have been invalid because there are duplicate members");
        }
    }

    #[test]
    // test that if a circuit has a member with an empty endpoint an error is returned
    fn test_validate_circuit_empty_endpoint() {
        let state = setup_splinter_state();
        let peer_connector = setup_peer_connector();
        let orchestrator = setup_orchestrator();
        // set up key registry
        let pub_key = (0u8..33).collect::<Vec<_>>();
        let mut key_registry = StorageKeyRegistry::new("memory".to_string()).unwrap();
        let key_info = KeyInfo::builder(pub_key.clone(), "node_a".to_string()).build();
        key_registry.save_key(key_info).unwrap();

        let admin_shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            #[cfg(feature = "service-arg-validation")]
            HashMap::new(),
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
            Box::new(HashVerifier),
            Box::new(key_registry),
            Box::new(AllowAllKeyPermissionManager),
            "memory",
        )
        .unwrap();
        let mut circuit = setup_test_circuit();

        let mut node_a = SplinterNode::new();
        node_a.set_node_id("node_a".to_string());
        node_a.set_endpoint("test://endpoint_a:0".to_string());

        let mut node_b = SplinterNode::new();
        node_b.set_node_id("node_b".to_string());
        node_b.set_endpoint("".to_string());

        circuit.set_members(RepeatedField::from_vec(vec![node_a, node_b]));

        if let Ok(_) = admin_shared.validate_create_circuit(&circuit, &pub_key, "node_a") {
            panic!("Should have been invalid because a member has an empty endpoint");
        }
    }

    #[test]
    // test that if a circuit has a member with a duplicate endpoint an error is returned
    fn test_validate_circuit_duplicate_endpoint() {
        let state = setup_splinter_state();
        let peer_connector = setup_peer_connector();
        let orchestrator = setup_orchestrator();
        // set up key registry
        let pub_key = (0u8..33).collect::<Vec<_>>();
        let mut key_registry = StorageKeyRegistry::new("memory".to_string()).unwrap();
        let key_info = KeyInfo::builder(pub_key.clone(), "node_a".to_string()).build();
        key_registry.save_key(key_info).unwrap();

        let admin_shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            #[cfg(feature = "service-arg-validation")]
            HashMap::new(),
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
            Box::new(HashVerifier),
            Box::new(key_registry),
            Box::new(AllowAllKeyPermissionManager),
            "memory",
        )
        .unwrap();
        let mut circuit = setup_test_circuit();

        let mut node_a = SplinterNode::new();
        node_a.set_node_id("node_a".to_string());
        node_a.set_endpoint("test://endpoint_a:0".to_string());

        let mut node_b = SplinterNode::new();
        node_b.set_node_id("node_b".to_string());
        node_b.set_endpoint("test://endpoint_a:0".to_string());

        circuit.set_members(RepeatedField::from_vec(vec![node_a, node_b]));

        if let Ok(_) = admin_shared.validate_create_circuit(&circuit, &pub_key, "node_a") {
            panic!("Should have been invalid because a member has a duplicate endpoint");
        }
    }

    #[test]
    // test that if a circuit does not have authorization set an error is returned
    fn test_validate_circuit_no_authorization() {
        let state = setup_splinter_state();
        let peer_connector = setup_peer_connector();
        let orchestrator = setup_orchestrator();
        // set up key registry
        let pub_key = (0u8..33).collect::<Vec<_>>();
        let mut key_registry = StorageKeyRegistry::new("memory".to_string()).unwrap();
        let key_info = KeyInfo::builder(pub_key.clone(), "node_a".to_string()).build();
        key_registry.save_key(key_info).unwrap();

        let admin_shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            #[cfg(feature = "service-arg-validation")]
            HashMap::new(),
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
            Box::new(HashVerifier),
            Box::new(key_registry),
            Box::new(AllowAllKeyPermissionManager),
            "memory",
        )
        .unwrap();
        let mut circuit = setup_test_circuit();

        circuit.set_authorization_type(Circuit_AuthorizationType::UNSET_AUTHORIZATION_TYPE);

        if let Ok(_) = admin_shared.validate_create_circuit(&circuit, &pub_key, "node_a") {
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
        let pub_key = (0u8..33).collect::<Vec<_>>();
        let mut key_registry = StorageKeyRegistry::new("memory".to_string()).unwrap();
        let key_info = KeyInfo::builder(pub_key.clone(), "node_a".to_string()).build();
        key_registry.save_key(key_info).unwrap();

        let admin_shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            #[cfg(feature = "service-arg-validation")]
            HashMap::new(),
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
            Box::new(HashVerifier),
            Box::new(key_registry),
            Box::new(AllowAllKeyPermissionManager),
            "memory",
        )
        .unwrap();
        let mut circuit = setup_test_circuit();

        circuit.set_persistence(Circuit_PersistenceType::UNSET_PERSISTENCE_TYPE);

        if let Ok(_) = admin_shared.validate_create_circuit(&circuit, &pub_key, "node_a") {
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
        let pub_key = (0u8..33).collect::<Vec<_>>();
        let mut key_registry = StorageKeyRegistry::new("memory".to_string()).unwrap();
        let key_info = KeyInfo::builder(pub_key.clone(), "node_a".to_string()).build();
        key_registry.save_key(key_info).unwrap();

        let admin_shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            #[cfg(feature = "service-arg-validation")]
            HashMap::new(),
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
            Box::new(HashVerifier),
            Box::new(key_registry),
            Box::new(AllowAllKeyPermissionManager),
            "memory",
        )
        .unwrap();
        let mut circuit = setup_test_circuit();

        circuit.set_durability(Circuit_DurabilityType::UNSET_DURABILITY_TYPE);

        if let Ok(_) = admin_shared.validate_create_circuit(&circuit, &pub_key, "node_a") {
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
        let pub_key = (0u8..33).collect::<Vec<_>>();
        let mut key_registry = StorageKeyRegistry::new("memory".to_string()).unwrap();
        let key_info = KeyInfo::builder(pub_key.clone(), "node_a".to_string()).build();
        key_registry.save_key(key_info).unwrap();

        let admin_shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            #[cfg(feature = "service-arg-validation")]
            HashMap::new(),
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
            Box::new(HashVerifier),
            Box::new(key_registry),
            Box::new(AllowAllKeyPermissionManager),
            "memory",
        )
        .unwrap();
        let mut circuit = setup_test_circuit();

        circuit.set_routes(Circuit_RouteType::UNSET_ROUTE_TYPE);

        if let Ok(_) = admin_shared.validate_create_circuit(&circuit, &pub_key, "node_a") {
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
        let pub_key = (0u8..33).collect::<Vec<_>>();
        let mut key_registry = StorageKeyRegistry::new("memory".to_string()).unwrap();
        let key_info = KeyInfo::builder(pub_key.clone(), "node_a".to_string()).build();
        key_registry.save_key(key_info).unwrap();

        let admin_shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            #[cfg(feature = "service-arg-validation")]
            HashMap::new(),
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
            Box::new(HashVerifier),
            Box::new(key_registry),
            Box::new(AllowAllKeyPermissionManager),
            "memory",
        )
        .unwrap();
        let mut circuit = setup_test_circuit();

        circuit.set_circuit_management_type("".to_string());

        if let Ok(_) = admin_shared.validate_create_circuit(&circuit, &pub_key, "node_a") {
            panic!("Should have been invalid because route type is unset");
        }
    }

    #[test]
    // test that a valid circuit proposal vote comes back as valid
    fn test_validate_proposal_vote_valid() {
        let state = setup_splinter_state();
        let peer_connector = setup_peer_connector();
        let orchestrator = setup_orchestrator();

        // set up key registry
        let pub_key = (0u8..33).collect::<Vec<_>>();
        let mut key_registry = StorageKeyRegistry::new("memory".to_string()).unwrap();
        let key_info = KeyInfo::builder(pub_key.clone(), "node_a".to_string()).build();
        key_registry.save_key(key_info).unwrap();

        let admin_shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            #[cfg(feature = "service-arg-validation")]
            HashMap::new(),
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
            Box::new(HashVerifier),
            Box::new(key_registry),
            Box::new(AllowAllKeyPermissionManager),
            "memory",
        )
        .unwrap();
        let circuit = setup_test_circuit();
        let vote = setup_test_vote(&circuit);
        let proposal = setup_test_proposal(&circuit);

        if let Err(err) = admin_shared.validate_circuit_vote(&vote, &pub_key, &proposal, "node_a") {
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
            #[cfg(feature = "service-arg-validation")]
            HashMap::new(),
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
            Box::new(HashVerifier),
            Box::new(key_registry),
            Box::new(AllowAllKeyPermissionManager),
            "memory",
        )
        .unwrap();
        let circuit = setup_test_circuit();
        let vote = setup_test_vote(&circuit);
        let proposal = setup_test_proposal(&circuit);

        let pub_key = (0u8..33).collect::<Vec<_>>();
        if let Ok(_) = admin_shared.validate_circuit_vote(&vote, &pub_key, &proposal, "node_a") {
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
        let pub_key = (0u8..33).collect::<Vec<_>>();
        let mut key_registry = StorageKeyRegistry::new("memory".to_string()).unwrap();
        let key_info = KeyInfo::builder(pub_key.clone(), "node_b".to_string()).build();
        key_registry.save_key(key_info).unwrap();

        let admin_shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            #[cfg(feature = "service-arg-validation")]
            HashMap::new(),
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
            Box::new(HashVerifier),
            Box::new(key_registry),
            Box::new(AllowAllKeyPermissionManager),
            "memory",
        )
        .unwrap();
        let circuit = setup_test_circuit();
        let vote = setup_test_vote(&circuit);
        let proposal = setup_test_proposal(&circuit);
        if let Ok(_) = admin_shared.validate_circuit_vote(&vote, &pub_key, &proposal, "node_a") {
            panic!("Should have been invalid because signer registered for the requester node");
        }
    }

    #[test]
    // test if a voter has already voted on a proposal the new vote is invalid
    fn test_validate_proposal_vote_duplicate_vote() {
        let state = setup_splinter_state();
        let peer_connector = setup_peer_connector();
        let orchestrator = setup_orchestrator();

        // set up key registry
        let pub_key = (0u8..33).collect::<Vec<_>>();
        let mut key_registry = StorageKeyRegistry::new("memory".to_string()).unwrap();
        let key_info = KeyInfo::builder(pub_key.clone(), "node_a".to_string()).build();
        key_registry.save_key(key_info).unwrap();

        let admin_shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            #[cfg(feature = "service-arg-validation")]
            HashMap::new(),
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
            Box::new(HashVerifier),
            Box::new(key_registry),
            Box::new(AllowAllKeyPermissionManager),
            "memory",
        )
        .unwrap();
        let circuit = setup_test_circuit();
        let vote = setup_test_vote(&circuit);
        let mut proposal = setup_test_proposal(&circuit);

        let mut vote_record = CircuitProposal_VoteRecord::new();
        vote_record.set_vote(CircuitProposalVote_Vote::ACCEPT);
        vote_record.set_public_key(b"test_signer_a".to_vec());
        vote_record.set_voter_node_id("node_a".to_string());

        proposal.set_votes(RepeatedField::from_vec(vec![vote_record]));

        if let Ok(_) = admin_shared.validate_circuit_vote(&vote, &pub_key, &proposal, "node_a") {
            panic!("Should have been invalid because node as already submited a vote");
        }
    }

    #[test]
    // test that if the circuit hash in the circuit proposal does not match the circuit hash on
    // the vote, the vote is invalid
    fn test_validate_proposal_vote_circuit_hash_mismatch() {
        let state = setup_splinter_state();
        let peer_connector = setup_peer_connector();
        let orchestrator = setup_orchestrator();

        // set up key registry
        let pub_key = (0u8..33).collect::<Vec<_>>();
        let mut key_registry = StorageKeyRegistry::new("memory".to_string()).unwrap();
        let key_info = KeyInfo::builder(pub_key.clone(), "node_a".to_string()).build();
        key_registry.save_key(key_info).unwrap();

        let admin_shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            #[cfg(feature = "service-arg-validation")]
            HashMap::new(),
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
            Box::new(HashVerifier),
            Box::new(key_registry),
            Box::new(AllowAllKeyPermissionManager),
            "memory",
        )
        .unwrap();
        let circuit = setup_test_circuit();
        let vote = setup_test_vote(&circuit);
        let mut proposal = setup_test_proposal(&circuit);

        proposal.set_circuit_hash("bad_hash".to_string());

        if let Ok(_) = admin_shared.validate_circuit_vote(&vote, &pub_key, &proposal, "node_a") {
            panic!("Should have been invalid because the circuit hash does not match");
        }
    }

    #[test]
    // test that the validate_circuit_management_payload method returns an error in case the
    // signature is empty.
    fn test_validate_circuit_management_payload_signature() {
        let state = setup_splinter_state();
        let peer_connector = setup_peer_connector();
        let orchestrator = setup_orchestrator();

        // set up key registry
        let pub_key = (0u8..33).collect::<Vec<_>>();
        let mut key_registry = StorageKeyRegistry::new("memory".to_string()).unwrap();
        let key_info = KeyInfo::builder(pub_key.clone(), "node_a".to_string()).build();
        key_registry.save_key(key_info).unwrap();

        let shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            #[cfg(feature = "service-arg-validation")]
            HashMap::new(),
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
            Box::new(HashVerifier),
            Box::new(key_registry),
            Box::new(AllowAllKeyPermissionManager),
            "memory",
        )
        .unwrap();

        let circuit = setup_test_circuit();

        let mut request = admin::CircuitCreateRequest::new();
        request.set_circuit(circuit);

        let mut header = admin::CircuitManagementPayload_Header::new();
        header.set_action(admin::CircuitManagementPayload_Action::CIRCUIT_CREATE_REQUEST);
        header.set_requester(pub_key);
        header.set_requester_node_id("node_b".to_string());
        let mut payload = admin::CircuitManagementPayload::new();
        payload.set_signature(Vec::new());
        payload.set_header(protobuf::Message::write_to_bytes(&header).unwrap());
        payload.set_circuit_create_request(request);

        // Asserting the payload will be deemed invalid as the signature is an empty vec.
        if let Ok(_) = shared.validate_circuit_management_payload(&payload, &header) {
            panic!("Should have been invalid due to empty signature");
        }

        payload.set_signature(HashSigner.sign(&payload.header).unwrap());
        // Asserting the payload passed through validation.
        if let Err(_) = shared.validate_circuit_management_payload(&payload, &header) {
            panic!("Should have been valid");
        }
    }

    #[test]
    // test that the validate_circuit_management_payload method returns an error in case the header is empty.
    fn test_validate_circuit_management_payload_header() {
        let state = setup_splinter_state();
        let peer_connector = setup_peer_connector();
        let orchestrator = setup_orchestrator();

        // set up key registry
        let pub_key = (0u8..33).collect::<Vec<_>>();
        let mut key_registry = StorageKeyRegistry::new("memory".to_string()).unwrap();
        let key_info = KeyInfo::builder(pub_key.clone(), "node_a".to_string()).build();
        key_registry.save_key(key_info).unwrap();

        let shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            #[cfg(feature = "service-arg-validation")]
            HashMap::new(),
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
            Box::new(HashVerifier),
            Box::new(key_registry),
            Box::new(AllowAllKeyPermissionManager),
            "memory",
        )
        .unwrap();

        let circuit = setup_test_circuit();

        let mut request = admin::CircuitCreateRequest::new();
        request.set_circuit(circuit);

        let mut header = admin::CircuitManagementPayload_Header::new();
        header.set_action(admin::CircuitManagementPayload_Action::CIRCUIT_CREATE_REQUEST);
        header.set_requester(pub_key);
        header.set_requester_node_id("node_b".to_string());
        let mut payload = admin::CircuitManagementPayload::new();
        payload.set_signature(HashSigner.sign(&payload.header).unwrap());
        payload.set_circuit_create_request(request);

        // Asserting the payload will be deemed invalid as the header is empty.
        match shared.validate_circuit_management_payload(&payload, &header) {
            Err(err) => assert!(err
                .to_string()
                .contains("CircuitManagementPayload header must be set")),
            _ => panic!("Should have been invalid because empty requester field"),
        }
        payload.set_header(protobuf::Message::write_to_bytes(&header).unwrap());
        // Asserting the payload passed through validation, and failed at a further step.
        if let Err(_) = shared.validate_circuit_management_payload(&payload, &header) {
            panic!("Should have been valid");
        }
    }

    #[test]
    // test that the validate_circuit_management_payload method returns an error in case the header
    // requester field is empty.
    fn test_validate_circuit_management_header_requester() {
        let state = setup_splinter_state();
        let peer_connector = setup_peer_connector();
        let orchestrator = setup_orchestrator();

        // set up key registry
        let pub_key = (0u8..33).collect::<Vec<_>>();
        let mut key_registry = StorageKeyRegistry::new("memory".to_string()).unwrap();
        let key_info = KeyInfo::builder(pub_key.clone(), "node_a".to_string()).build();
        key_registry.save_key(key_info).unwrap();

        let shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            #[cfg(feature = "service-arg-validation")]
            HashMap::new(),
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
            Box::new(HashVerifier),
            Box::new(key_registry),
            Box::new(AllowAllKeyPermissionManager),
            "memory",
        )
        .unwrap();

        let circuit = setup_test_circuit();

        let mut request = admin::CircuitCreateRequest::new();
        request.set_circuit(circuit);

        let mut header = admin::CircuitManagementPayload_Header::new();
        header.set_action(admin::CircuitManagementPayload_Action::CIRCUIT_CREATE_REQUEST);
        header.set_requester_node_id("node_b".to_string());
        let mut payload = admin::CircuitManagementPayload::new();
        payload.set_signature(HashSigner.sign(&payload.header).unwrap());
        payload.set_circuit_create_request(request);

        payload.set_header(protobuf::Message::write_to_bytes(&header).unwrap());
        // Asserting the payload will be deemed invalid as the header is empty.
        match shared.validate_circuit_management_payload(&payload, &header) {
            Err(err) => assert!(err
                .to_string()
                .contains("CircuitManagementPayload must have a requester")),
            _ => panic!("Should have been invalid because empty requester field"),
        }

        header.set_requester(pub_key);
        payload.set_header(protobuf::Message::write_to_bytes(&header).unwrap());
        // Asserting the payload passed through validation, and failed at a further step.
        if let Err(_) = shared.validate_circuit_management_payload(&payload, &header) {
            panic!("Should have been valid");
        }
    }

    #[test]
    // test that the CircuitManagementPayload returns an error in case the header requester_node_id
    // field is empty.
    fn test_validate_circuit_management_header_requester_node_id() {
        let state = setup_splinter_state();
        let peer_connector = setup_peer_connector();
        let orchestrator = setup_orchestrator();

        // set up key registry
        let pub_key = (0u8..33).collect::<Vec<_>>();
        let mut key_registry = StorageKeyRegistry::new("memory".to_string()).unwrap();
        let key_info = KeyInfo::builder(pub_key.clone(), "node_a".to_string()).build();
        key_registry.save_key(key_info).unwrap();

        let shared = AdminServiceShared::new(
            "node_a".into(),
            orchestrator,
            #[cfg(feature = "service-arg-validation")]
            HashMap::new(),
            peer_connector,
            Box::new(MockAuthInquisitor),
            state,
            Box::new(HashVerifier),
            Box::new(key_registry),
            Box::new(AllowAllKeyPermissionManager),
            "memory",
        )
        .unwrap();

        let circuit = setup_test_circuit();

        let mut request = admin::CircuitCreateRequest::new();
        request.set_circuit(circuit);

        let mut header = admin::CircuitManagementPayload_Header::new();
        header.set_action(admin::CircuitManagementPayload_Action::CIRCUIT_CREATE_REQUEST);
        header.set_requester(pub_key);
        let mut payload = admin::CircuitManagementPayload::new();
        payload.set_signature(HashSigner.sign(&payload.header).unwrap());
        payload.set_circuit_create_request(request);

        payload.set_header(protobuf::Message::write_to_bytes(&header).unwrap());
        // Asserting the payload will be deemed invalid as the header is empty.
        match shared.validate_circuit_management_payload(&payload, &header) {
            Err(err) => assert!(err
                .to_string()
                .contains("CircuitManagementPayload must have a requester node id")),
            _ => panic!("Should have been invalid because empty requester field"),
        }

        header.set_requester_node_id("node_b".to_string());
        payload.set_header(protobuf::Message::write_to_bytes(&header).unwrap());
        // Asserting the payload passed through validation, and failed at a further step.
        if let Err(_) = shared.validate_circuit_management_payload(&payload, &header) {
            panic!("Should have been valid");
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

    fn setup_splinter_state() -> SplinterState {
        let mut storage = get_storage("memory", CircuitDirectory::new).unwrap();
        let circuit_directory = storage.write().clone();

        SplinterState::new("memory".to_string(), circuit_directory)
    }

    fn setup_peer_connector() -> PeerConnector {
        let mesh = Mesh::new(4, 16);
        let network = Network::new(mesh.clone(), 0).unwrap();
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
