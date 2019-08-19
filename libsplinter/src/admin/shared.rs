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
use crate::network::{
    auth::{AuthorizationInquisitor, PeerAuthorizationState},
    peer::PeerConnector,
};
use crate::orchestrator::ServiceOrchestrator;
use crate::protos::admin::{
    Circuit, CircuitCreateRequest, CircuitManagementPayload, CircuitManagementPayload_Action,
    CircuitProposal, CircuitProposal_ProposalType,
};
use crate::service::error::ServiceError;
use crate::service::ServiceNetworkSender;

use super::error::AdminStateError;
use super::messages;
use super::sha256;

type UnpeeredPendingPayload = (Vec<String>, CircuitManagementPayload);

pub struct AdminServiceShared {
    // the node id of the connected splinter node
    node_id: String,
    // the list of circuit proposal that are being voted on by members of a circuit
    open_proposals: HashMap<String, CircuitProposal>,
    // orchestrator used to initialize and shutdown services
    orchestrator: ServiceOrchestrator,
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
    socket_senders: Vec<(String, Sender<messages::AdminServiceEvent>)>,
}

impl AdminServiceShared {
    pub fn new(
        node_id: String,
        orchestrator: ServiceOrchestrator,
        peer_connector: PeerConnector,
        auth_inquisitor: Box<dyn AuthorizationInquisitor>,
    ) -> Self {
        AdminServiceShared {
            node_id: node_id.to_string(),
            network_sender: None,
            open_proposals: Default::default(),
            orchestrator,
            peer_connector,
            auth_inquisitor,
            unpeered_payloads: Vec::new(),
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
        debug!(
            "received circuit proposal for {}",
            proposed_circuit.get_circuit_id()
        );

        let mut unauthorized_peers = vec![];
        for node in proposed_circuit.get_members() {
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

        let mut create_request = CircuitCreateRequest::new();
        create_request.set_circuit(proposed_circuit);

        let mut envelope = CircuitManagementPayload::new();
        envelope.set_action(CircuitManagementPayload_Action::CIRCUIT_CREATE_REQUEST);
        envelope.set_circuit_create_request(create_request);

        if unauthorized_peers.is_empty() {
            self.pending_circuit_payloads.push_back(envelope);
        } else {
            debug!(
                "Members {:?} added; awaiting network authorization before proceeding",
                &unauthorized_peers
            );

            self.unpeered_payloads.push((unauthorized_peers, envelope));
        }
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
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::mesh::Mesh;
    use crate::network::{
        auth::{AuthorizationCallback, AuthorizationCallbackError},
        Network,
    };
    use crate::protos::admin;
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
        let mut shared = AdminServiceShared::new(
            "my_peer_id".into(),
            orchestrator,
            peer_connector,
            Box::new(MockAuthInquisitor),
        );

        let mut circuit = Circuit::new();
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

        shared
            .propose_circuit(circuit)
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
        let mut shared = AdminServiceShared::new(
            "my_peer_id".into(),
            orchestrator,
            peer_connector,
            Box::new(MockAuthInquisitor),
        );

        let mut circuit = Circuit::new();
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

        shared
            .propose_circuit(circuit)
            .expect("Proposal not accepted");

        // None of the proposed members are peered
        assert_eq!(1, shared.unpeered_payloads.len());
        assert_eq!(0, shared.pending_circuit_payloads.len());
        shared.on_authorization_change("test-node", PeerAuthorizationState::Unauthorized);

        // The message should be dropped
        assert_eq!(0, shared.pending_circuit_payloads.len());
        assert_eq!(0, shared.unpeered_payloads.len());
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
