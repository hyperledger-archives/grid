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
use std::collections::VecDeque;

use crossbeam_channel::Sender;

use crate::consensus::{Proposal, ProposalId};
use crate::network::peer::PeerConnector;
use crate::protos::admin::{
    Circuit, CircuitCreateRequest, CircuitManagementPayload, CircuitManagementPayload_Action,
    CircuitProposal, CircuitProposal_ProposalType,
};
use crate::service::error::ServiceError;
use crate::service::ServiceNetworkSender;

use super::error::AdminStateError;
use super::messages;
use super::sha256;

pub struct AdminServiceShared {
    // the node id of the connected splinter node
    node_id: String,
    // the list of circuit proposal that are being voted on by members of a circuit
    open_proposals: HashMap<String, CircuitProposal>,
    // peer connector used to connect to new members listed in a circuit
    peer_connector: PeerConnector,
    // network sender is used to comunicated with other services on the splinter network
    network_sender: Option<Box<dyn ServiceNetworkSender>>,
    // CircuitManagmentPayloads that still need to go through consensus
    pending_circuit_payloads: VecDeque<CircuitManagementPayload>,
    // The pending consensus proposals
    pending_consesus_proposals: HashMap<ProposalId, (Proposal, CircuitManagementPayload)>,
    // the pending changes for the current proposal
    pending_changes: Option<CircuitProposal>,
    socket_senders: Vec<(String, Sender<messages::AdminServiceEvent>)>,
}

impl AdminServiceShared {
    pub fn new(node_id: String, peer_connector: PeerConnector) -> Self {
        AdminServiceShared {
            node_id: node_id.to_string(),
            network_sender: None,
            open_proposals: Default::default(),
            peer_connector,
            pending_circuit_payloads: VecDeque::new(),
            pending_consesus_proposals: HashMap::new(),
            pending_changes: None,
            socket_senders: Vec::new(),
        }
    }

    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    pub fn network_sender(&self) -> &Option<Box<dyn ServiceNetworkSender>> {
        &self.network_sender
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

    pub fn pending_changes(&self) -> &Option<CircuitProposal> {
        &self.pending_changes
    }

    pub fn commit(&mut self) -> Result<(), AdminStateError> {
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
                    messages::CircuitProposal::from_proto(circuit_proposal.clone()).map_err(
                        |err| AdminStateError(format!("invalid message format {}", err)),
                    )?,
                );
                self.send_event(&mgmt_type, event);

                info!("committed change for circuit proposal {}", circuit_id,);

                Ok(())
            }
            None => Err(AdminStateError("no pending changes to commit".into())),
        }
    }

    pub fn rollback(&mut self) -> Result<(), AdminStateError> {
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
    ) -> Result<(String, CircuitProposal), AdminStateError> {
        match circuit_payload.get_action() {
            CircuitManagementPayload_Action::CIRCUIT_CREATE_REQUEST => {
                let mut create_request = circuit_payload.take_circuit_create_request();
                let proposed_circuit = create_request.take_circuit();

                if self.has_proposal(proposed_circuit.get_circuit_id()) {
                    Err(AdminStateError(format!(
                        "Ignoring duplicate create proposal of circuit {}",
                        proposed_circuit.get_circuit_id()
                    )))
                } else {
                    debug!("proposing {}", proposed_circuit.get_circuit_id());

                    let mut circuit_proposal = CircuitProposal::new();
                    circuit_proposal.set_proposal_type(CircuitProposal_ProposalType::CREATE);
                    circuit_proposal.set_circuit_id(proposed_circuit.get_circuit_id().into());
                    circuit_proposal.set_circuit_hash(sha256(&proposed_circuit)?);
                    circuit_proposal.set_circuit_proposal(proposed_circuit);

                    let expected_hash = sha256(&circuit_proposal)?;
                    self.pending_changes = Some(circuit_proposal.clone());

                    Ok((expected_hash, circuit_proposal))
                }
            }
            unknown_action => Err(AdminStateError(format!(
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
    pub fn propose_circuit(&mut self, proposed_circuit: Circuit) -> Result<(), ServiceError> {
        debug!("Adding members as peers");

        for node in proposed_circuit.get_members() {
            if self.node_id() != node.get_node_id() {
                debug!("Connecting to node {:?}", node);
                self.peer_connector
                    .connect_peer(node.get_node_id(), node.get_endpoint())
                    .map_err(|err| ServiceError::UnableToHandleMessage(Box::new(err)))?;
            }
        }
        debug!("Members added");

        debug!(
            "recieved circuit proposal for {}",
            proposed_circuit.get_circuit_id()
        );
        let mut create_request = CircuitCreateRequest::new();
        create_request.set_circuit(proposed_circuit);

        let mut envelope = CircuitManagementPayload::new();
        envelope.set_action(CircuitManagementPayload_Action::CIRCUIT_CREATE_REQUEST);
        envelope.set_circuit_create_request(create_request);

        self.pending_circuit_payloads.push_back(envelope);
        Ok(())
    }

    fn add_proposal(&mut self, circuit_proposal: CircuitProposal) {
        let circuit_id = circuit_proposal.get_circuit_id().to_string();

        self.open_proposals.insert(circuit_id, circuit_proposal);
    }

    pub fn add_socket_sender(
        &mut self,
        circuit_management_type: String,
        sender: Sender<messages::AdminServiceEvent>,
    ) {
        self.socket_senders.push((circuit_management_type, sender));
    }

    pub fn send_event(
        &mut self,
        circuit_management_type: &str,
        event: messages::AdminServiceEvent,
    ) {
        // The use of retain allows us to drop any senders that are no longer valid.
        self.socket_senders.retain(|(mgmt_type, sender)| {
            if mgmt_type != circuit_management_type {
                return true;
            }

            if let Err(err) = sender.send(event.clone()) {
                warn!(
                    "Dropping sender for {} due to error: {}",
                    circuit_management_type, err
                );
                return false;
            }

            true
        });
    }
}
