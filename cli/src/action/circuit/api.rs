// Copyright 2020 Cargill Incorporated
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

use reqwest::{blocking::Client, header, StatusCode};
use serde::{Deserialize, Serialize};
use splinter::circuit::{AuthorizationType, DurabilityType, PersistenceType, Roster, RouteType};

use crate::error::CliError;

/// A wrapper around the Splinter REST API.
pub struct SplinterRestClient<'a> {
    url: &'a str,
}

impl<'a> SplinterRestClient<'a> {
    /// Constructs a new client for a Splinter node at the given URL.
    pub fn new(url: &'a str) -> Self {
        Self { url }
    }

    /// Fetches the node ID of this client's Splinter node.
    pub fn fetch_node_id(&self) -> Result<String, CliError> {
        Client::new()
            .get(&format!("{}/status", self.url))
            .send()
            .and_then(|res| res.json())
            .map(|server_status: ServerStatus| server_status.node_id)
            .map_err(|err| CliError::ActionError(format!("Unable to fetch node id: {}", err)))
    }

    /// Submits an admin payload to this client's Splinter node.
    pub fn submit_admin_payload(&self, payload: Vec<u8>) -> Result<(), CliError> {
        Client::new()
            .post(&format!("{}/admin/submit", self.url))
            .header(header::CONTENT_TYPE, "octet-stream")
            .body(payload)
            .send()
            .map_err(|err| {
                CliError::ActionError(format!("Unable to submit admin payload: {}", err))
            })
            .and_then(|res| match res.status() {
                StatusCode::ACCEPTED => Ok(()),
                StatusCode::BAD_REQUEST | StatusCode::INTERNAL_SERVER_ERROR => {
                    let message = res
                        .json::<ServerError>()
                        .map_err(|err| {
                            CliError::ActionError(format!(
                                "Unable to parse error response: {}",
                                err
                            ))
                        })?
                        .message;

                    Err(CliError::ActionError(format!(
                        "Unable to submit admin payload: {}",
                        message
                    )))
                }
                _ => Err(CliError::ActionError(format!(
                    "Received unknown response status: {}",
                    res.status()
                ))),
            })
    }

    pub fn list_circuits(&self, filter: Option<&str>) -> Result<CircuitListSlice, CliError> {
        let mut request = format!("{}/circuits", self.url);
        if let Some(filter) = filter {
            request = format!("{}?filter={}", &request, &filter);
        }

        Client::new()
            .get(&request)
            .send()
            .map_err(|err| CliError::ActionError(err.to_string()))
            .and_then(|res| match res.status() {
                StatusCode::OK => Ok(res
                    .json::<CircuitListSlice>()
                    .map_err(|err| CliError::ActionError(err.to_string()))?),
                StatusCode::BAD_REQUEST | StatusCode::INTERNAL_SERVER_ERROR => {
                    let message = res
                        .json::<ServerError>()
                        .map_err(|err| {
                            CliError::ActionError(format!(
                                "Unable to parse error response: {}",
                                err
                            ))
                        })?
                        .message;

                    Err(CliError::ActionError(format!(
                        "Unable to fetch circuits: {}",
                        message
                    )))
                }
                _ => Err(CliError::ActionError(format!(
                    "Received unknown response status: {}",
                    res.status()
                ))),
            })
    }

    pub fn fetch_circuit(&self, circuit_id: &str) -> Result<Option<CircuitSlice>, CliError> {
        Client::new()
            .get(&format!("{}/circuits/{}", self.url, circuit_id))
            .send()
            .map_err(|err| CliError::ActionError(err.to_string()))
            .and_then(|res| match res.status() {
                StatusCode::OK => Ok(Some(
                    res.json::<CircuitSlice>()
                        .map_err(|err| CliError::ActionError(err.to_string()))?,
                )),
                StatusCode::NOT_FOUND => Ok(None),
                StatusCode::BAD_REQUEST | StatusCode::INTERNAL_SERVER_ERROR => {
                    let message = res
                        .json::<ServerError>()
                        .map_err(|err| {
                            CliError::ActionError(format!(
                                "Unable to parse error response: {}",
                                err
                            ))
                        })?
                        .message;

                    Err(CliError::ActionError(format!(
                        "Unable to fetch circuit: {}",
                        message
                    )))
                }
                _ => Err(CliError::ActionError(format!(
                    "Received unknown response status: {}",
                    res.status()
                ))),
            })
    }

    pub fn list_proposals(&self, filter: Option<&str>) -> Result<ProposalListSlice, CliError> {
        let mut request = format!("{}/admin/proposals", self.url);
        if let Some(filter) = filter {
            request = format!("{}?filter={}", &request, &filter);
        }

        Client::new()
            .get(&request)
            .send()
            .map_err(|err| CliError::ActionError(err.to_string()))
            .and_then(|res| match res.status() {
                StatusCode::OK => Ok(res
                    .json::<ProposalListSlice>()
                    .map_err(|err| CliError::ActionError(err.to_string()))?),
                StatusCode::BAD_REQUEST | StatusCode::INTERNAL_SERVER_ERROR => {
                    let message = res
                        .json::<ServerError>()
                        .map_err(|err| {
                            CliError::ActionError(format!(
                                "Unable to parse error response: {}",
                                err
                            ))
                        })?
                        .message;

                    Err(CliError::ActionError(format!(
                        "Unable to fetch proposals: {}",
                        message
                    )))
                }
                _ => Err(CliError::ActionError(format!(
                    "Received unknown response status: {}",
                    res.status()
                ))),
            })
    }

    pub fn fetch_proposal(&self, circuit_id: &str) -> Result<Option<ProposalSlice>, CliError> {
        Client::new()
            .get(&format!("{}/admin/proposals/{}", self.url, circuit_id))
            .send()
            .map_err(|err| CliError::ActionError(err.to_string()))
            .and_then(|res| match res.status() {
                StatusCode::OK => Ok(Some(
                    res.json::<ProposalSlice>()
                        .map_err(|err| CliError::ActionError(err.to_string()))?,
                )),
                StatusCode::NOT_FOUND => Ok(None),
                StatusCode::BAD_REQUEST | StatusCode::INTERNAL_SERVER_ERROR => {
                    let message = res
                        .json::<ServerError>()
                        .map_err(|err| {
                            CliError::ActionError(format!(
                                "Unable to parse error response: {}",
                                err
                            ))
                        })?
                        .message;

                    Err(CliError::ActionError(format!(
                        "Unable to fetch proposal: {}",
                        message
                    )))
                }
                _ => Err(CliError::ActionError(format!(
                    "Received unknown response status: {}",
                    res.status()
                ))),
            })
    }
}

#[derive(Deserialize)]
struct ServerStatus {
    node_id: String,
}

#[derive(Deserialize)]
struct ServerError {
    message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CircuitSlice {
    pub id: String,
    pub auth: AuthorizationType,
    pub persistence: PersistenceType,
    pub durability: DurabilityType,
    pub routes: RouteType,
    pub circuit_management_type: String,
    pub members: Vec<String>,
    pub roster: Roster,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CircuitListSlice {
    pub data: Vec<CircuitSlice>,
    pub paging: Paging,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct ProposalSlice {
    pub proposal_type: String,
    pub circuit_id: String,
    pub circuit_hash: String,
    pub circuit: ProposalCircuitSlice,
    pub votes: Vec<VoteRecord>,
    pub requester: String,
    pub requester_node_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ProposalCircuitSlice {
    pub circuit_id: String,
    pub authorization_type: AuthorizationType,
    pub persistence: PersistenceType,
    pub durability: DurabilityType,
    pub routes: RouteType,
    pub circuit_management_type: String,
    pub members: Vec<CircuitMembers>,
    pub roster: Vec<CircuitService>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CircuitMembers {
    pub node_id: String,
    pub endpoint: String,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct CircuitService {
    pub service_id: String,
    pub service_type: String,
    pub allowed_nodes: Vec<String>,
    pub arguments: Vec<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ProposalListSlice {
    pub data: Vec<ProposalSlice>,
    pub paging: Paging,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct VoteRecord {
    pub public_key: String,
    pub vote: Vote,
    pub voter_node_id: String,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum Vote {
    Accept,
    Reject,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum ProposalType {
    Create,
    UpdateRoster,
    AddNode,
    RemoveNode,
    Destroy,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Paging {
    pub current: String,
    pub offset: usize,
    pub limit: usize,
    pub total: usize,
    pub first: String,
    pub prev: String,
    pub next: String,
    pub last: String,
}
