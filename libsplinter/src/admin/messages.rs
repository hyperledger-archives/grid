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

use crate::actix_web::{web, Error as ActixError};
use crate::futures::{stream::Stream, Future, IntoFuture};
use crate::protos::admin::{self, Circuit, CircuitCreateRequest};
use protobuf::{self, RepeatedField};
use serde_json;

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateCircuit {
    pub circuit_id: String,
    pub roster: Vec<SplinterService>,
    pub members: Vec<SplinterNode>,
    pub authorization_type: AuthorizationType,
    pub persistence: PersistenceType,
    pub routes: RouteType,
    pub circuit_management_type: String,
    pub application_metadata: Vec<u8>,
}

impl CreateCircuit {
    pub fn from_payload(payload: web::Payload) -> impl Future<Item = Self, Error = ActixError> {
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

    pub fn into_proto(self) -> Result<CircuitCreateRequest, ProposalMarshallingError> {
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
pub enum AuthorizationType {
    Trust,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PersistenceType {
    Any,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RouteType {
    Any,
}

#[derive(Serialize, Deserialize, Debug)]
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
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SplinterService {
    pub service_id: String,
    pub service_type: String,
    pub allowed_nodes: Vec<String>,
}

impl SplinterService {
    pub fn into_proto(self) -> admin::SplinterService {
        let mut proto = admin::SplinterService::new();
        proto.set_service_id(self.service_id);
        proto.set_service_type(self.service_type);
        proto.set_allowed_nodes(RepeatedField::from_vec(self.allowed_nodes));

        proto
    }
#[derive(Debug)]
pub enum ProposalMarshallingError {
    UnsetField(String)
}

impl std::error::Error for ProposalMarshallingError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ProposalMarshallingError::UnsetField(_) => None
        }
    }
}

impl std::fmt::Display for ProposalMarshallingError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ProposalMarshallingError::UnsetField(_) => write!(f, "Invalid enumerated type")
        }
    }
}
