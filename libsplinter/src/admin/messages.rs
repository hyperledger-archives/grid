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

use protobuf::{self, RepeatedField};
use serde::de::DeserializeOwned;
use serde_json;

use crate::actix_web::{error::ErrorBadRequest, web, Error as ActixError};
use crate::futures::{stream::Stream, Future, IntoFuture};
use crate::protos::admin::{self, CircuitCreateRequest};

use super::error::MarshallingError;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CreateCircuit {
    pub circuit_id: String,
    pub roster: Vec<SplinterService>,
    pub members: Vec<SplinterNode>,
    pub authorization_type: AuthorizationType,
    pub persistence: PersistenceType,
    pub durability: DurabilityType,
    pub routes: RouteType,
    pub circuit_management_type: String,
    pub application_metadata: Vec<u8>,
}

pub fn from_payload<T: DeserializeOwned>(
    payload: web::Payload,
) -> impl Future<Item = T, Error = ActixError> {
    payload
        .from_err::<ActixError>()
        .fold(web::BytesMut::new(), move |mut body, chunk| {
            body.extend_from_slice(&chunk);
            Ok::<_, ActixError>(body)
        })
        .and_then(|body| Ok(serde_json::from_slice::<T>(&body)?))
        .or_else(|err| Err(ErrorBadRequest(json!({ "message": format!("{}", err) }))))
        .into_future()
}

impl CreateCircuit {
    pub fn from_proto(mut proto: admin::Circuit) -> Result<Self, MarshallingError> {
        let authorization_type = match proto.get_authorization_type() {
            admin::Circuit_AuthorizationType::TRUST_AUTHORIZATION => AuthorizationType::Trust,
            admin::Circuit_AuthorizationType::UNSET_AUTHORIZATION_TYPE => {
                return Err(MarshallingError::UnsetField(
                    "Unset authorization type".to_string(),
                ));
            }
        };

        let persistence = match proto.get_persistence() {
            admin::Circuit_PersistenceType::ANY_PERSISTENCE => PersistenceType::Any,
            admin::Circuit_PersistenceType::UNSET_PERSISTENCE_TYPE => {
                return Err(MarshallingError::UnsetField(
                    "Unset persistence type".to_string(),
                ));
            }
        };

        let durability = match proto.get_durability() {
            admin::Circuit_DurabilityType::NO_DURABILITY => DurabilityType::NoDurabilty,
            admin::Circuit_DurabilityType::UNSET_DURABILITY_TYPE => {
                return Err(MarshallingError::UnsetField(
                    "Unset durability type".to_string(),
                ));
            }
        };

        let routes = match proto.get_routes() {
            admin::Circuit_RouteType::ANY_ROUTE => RouteType::Any,
            admin::Circuit_RouteType::UNSET_ROUTE_TYPE => {
                return Err(MarshallingError::UnsetField("Unset route type".to_string()));
            }
        };

        Ok(Self {
            circuit_id: proto.take_circuit_id(),
            roster: proto
                .take_roster()
                .into_iter()
                .map(SplinterService::from_proto)
                .collect::<Result<Vec<SplinterService>, MarshallingError>>()?,
            members: proto
                .take_members()
                .into_iter()
                .map(SplinterNode::from_proto)
                .collect::<Result<Vec<SplinterNode>, MarshallingError>>()?,
            authorization_type,
            persistence,
            durability,
            routes,
            circuit_management_type: proto.take_circuit_management_type(),
            application_metadata: proto.take_application_metadata(),
        })
    }

    pub fn into_proto(self) -> Result<CircuitCreateRequest, MarshallingError> {
        let mut circuit = admin::Circuit::new();

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
        match self.durability {
            DurabilityType::NoDurabilty => {
                circuit.set_durability(admin::Circuit_DurabilityType::NO_DURABILITY);
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

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum AuthorizationType {
    Trust,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum PersistenceType {
    Any,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum DurabilityType {
    NoDurabilty,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum RouteType {
    Any,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SplinterNode {
    pub node_id: String,
    pub endpoint: String,
}

impl SplinterNode {
    pub fn into_proto(self) -> admin::SplinterNode {
        let mut proto = admin::SplinterNode::new();

        proto.set_node_id(self.node_id);
        proto.set_endpoint(self.endpoint);

        proto
    }

    pub fn from_proto(mut proto: admin::SplinterNode) -> Result<Self, MarshallingError> {
        Ok(Self {
            node_id: proto.take_node_id(),
            endpoint: proto.take_endpoint(),
        })
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SplinterService {
    pub service_id: String,
    pub service_type: String,
    pub allowed_nodes: Vec<String>,
    pub arguments: HashMap<String, String>,
}

impl SplinterService {
    pub fn into_proto(self) -> admin::SplinterService {
        let mut proto = admin::SplinterService::new();
        proto.set_service_id(self.service_id);
        proto.set_service_type(self.service_type);
        proto.set_allowed_nodes(RepeatedField::from_vec(self.allowed_nodes));
        proto.set_arguments(RepeatedField::from_vec(
            self.arguments
                .into_iter()
                .map(|(k, v)| {
                    let mut argument = admin::SplinterService_Argument::new();
                    argument.set_key(k);
                    argument.set_value(v);
                    argument
                })
                .collect(),
        ));

        proto
    }

    pub fn from_proto(mut proto: admin::SplinterService) -> Result<Self, MarshallingError> {
        Ok(Self {
            service_id: proto.take_service_id(),
            service_type: proto.take_service_type(),
            allowed_nodes: proto
                .take_allowed_nodes()
                .into_iter()
                .map(String::from)
                .collect(),
            arguments: proto
                .take_arguments()
                .into_iter()
                .map(|mut argument| (argument.take_key(), argument.take_value()))
                .collect(),
        })
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CircuitProposal {
    pub proposal_type: ProposalType,
    pub circuit_id: String,
    pub circuit_hash: String,
    pub circuit: CreateCircuit,
    pub votes: Vec<VoteRecord>,
    pub requester: String,
}

impl CircuitProposal {
    pub fn from_proto(mut proto: admin::CircuitProposal) -> Result<Self, MarshallingError> {
        let proposal_type = match proto.get_proposal_type() {
            admin::CircuitProposal_ProposalType::CREATE => ProposalType::Create,
            admin::CircuitProposal_ProposalType::UPDATE_ROSTER => ProposalType::UpdateRoster,
            admin::CircuitProposal_ProposalType::ADD_NODE => ProposalType::AddNode,
            admin::CircuitProposal_ProposalType::REMOVE_NODE => ProposalType::RemoveNode,
            admin::CircuitProposal_ProposalType::DESTROY => ProposalType::Destroy,
            admin::CircuitProposal_ProposalType::UNSET_PROPOSAL_TYPE => {
                return Err(MarshallingError::UnsetField(
                    "Unset proposal type".to_string(),
                ));
            }
        };

        let votes = proto
            .take_votes()
            .into_iter()
            .map(VoteRecord::from_proto)
            .collect::<Result<Vec<VoteRecord>, MarshallingError>>()?;

        Ok(Self {
            proposal_type,
            circuit_id: proto.take_circuit_id(),
            circuit_hash: proto.take_circuit_hash(),
            circuit: CreateCircuit::from_proto(proto.take_circuit_proposal())?,
            votes,
            requester: proto.take_requester(),
        })
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum ProposalType {
    Create,
    UpdateRoster,
    AddNode,
    RemoveNode,
    Destroy,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct VoteRecord {
    pub public_key: Vec<u8>,
    pub vote: Vote,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CircuitProposalVote {
    pub circuit_id: String,
    pub circuit_hash: String,
    pub vote: Vote,
}

impl CircuitProposalVote {
    pub fn from_proto(mut proto: admin::CircuitProposalVote) -> Result<Self, MarshallingError> {
        let vote = match proto.get_vote() {
            admin::CircuitProposalVote_Vote::ACCEPT => Vote::Accept,
            admin::CircuitProposalVote_Vote::REJECT => Vote::Reject,
        };

        Ok(CircuitProposalVote {
            circuit_id: proto.take_circuit_id(),
            circuit_hash: proto.take_circuit_hash(),
            vote,
        })
    }

    pub fn into_proto(self) -> admin::CircuitProposalVote {
        let mut vote = admin::CircuitProposalVote::new();
        vote.set_circuit_id(self.circuit_id);
        vote.set_circuit_hash(self.circuit_hash);
        match self.vote {
            Vote::Accept => vote.set_vote(admin::CircuitProposalVote_Vote::ACCEPT),
            Vote::Reject => vote.set_vote(admin::CircuitProposalVote_Vote::REJECT),
        }
        vote
    }
}

impl VoteRecord {
    fn from_proto(mut proto: admin::CircuitProposal_VoteRecord) -> Result<Self, MarshallingError> {
        let vote = match proto.get_vote() {
            admin::CircuitProposalVote_Vote::ACCEPT => Vote::Accept,
            admin::CircuitProposalVote_Vote::REJECT => Vote::Reject,
        };

        Ok(Self {
            public_key: proto.take_public_key(),
            vote,
        })
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Vote {
    Accept,
    Reject,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "eventType", content = "message")]
pub enum AdminServiceEvent {
    ProposalSubmitted(CircuitProposal),
    ProposalVote((CircuitProposalVote, Vec<u8>)),
    ProposalAccepted(CircuitProposal),
    ProposalRejected(CircuitProposal),
}
