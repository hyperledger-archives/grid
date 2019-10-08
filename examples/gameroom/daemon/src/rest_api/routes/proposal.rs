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
use std::time::{Duration, SystemTime};

use actix_web::{error, web, Error, HttpResponse};
use futures::Future;
use gameroom_database::{
    helpers,
    models::{GameroomMember, GameroomProposal},
    ConnectionPool,
};
use splinter::admin::messages::CircuitProposalVote;
use splinter::node_registry::Node;
use splinter::protos::admin::{
    CircuitManagementPayload, CircuitManagementPayload_Action as Action,
    CircuitManagementPayload_Header as Header,
};
use openssl::hash::{hash, MessageDigest};
use protobuf::Message;

use super::{
    get_response_paging_info, ErrorResponse, SuccessResponse, DEFAULT_LIMIT, DEFAULT_OFFSET,
};
use crate::rest_api::RestApiResponseError;

#[derive(Debug, Serialize)]
struct ApiGameroomProposal {
    proposal_id: String,
    circuit_id: String,
    circuit_hash: String,
    members: Vec<ApiGameroomMember>,
    requester: String,
    requester_node_id: String,
    created_time: u64,
    updated_time: u64,
}

impl ApiGameroomProposal {
    fn from(db_proposal: GameroomProposal, db_members: Vec<GameroomMember>) -> Self {
        ApiGameroomProposal {
            proposal_id: db_proposal.id.to_string(),
            circuit_id: db_proposal.circuit_id.to_string(),
            circuit_hash: db_proposal.circuit_hash.to_string(),
            members: db_members
                .into_iter()
                .map(ApiGameroomMember::from)
                .collect(),
            requester: db_proposal.requester.to_string(),
            requester_node_id: db_proposal.requester_node_id.to_string(),
            created_time: db_proposal
                .created_time
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_else(|_| Duration::new(0, 0))
                .as_secs(),
            updated_time: db_proposal
                .updated_time
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_else(|_| Duration::new(0, 0))
                .as_secs(),
        }
    }
}

#[derive(Debug, Serialize)]
struct ApiGameroomMember {
    node_id: String,
    endpoint: String,
}

impl ApiGameroomMember {
    fn from(db_circuit_member: GameroomMember) -> Self {
        ApiGameroomMember {
            node_id: db_circuit_member.node_id.to_string(),
            endpoint: db_circuit_member.endpoint.to_string(),
        }
    }
}

pub fn fetch_proposal(
    pool: web::Data<ConnectionPool>,
    proposal_id: web::Path<i64>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    Box::new(
        web::block(move || get_proposal_from_db(pool, *proposal_id)).then(|res| match res {
            Ok(proposal) => Ok(HttpResponse::Ok().json(SuccessResponse::new(proposal))),
            Err(err) => match err {
                error::BlockingError::Error(err) => {
                    match err {
                        RestApiResponseError::NotFound(err) => Ok(HttpResponse::NotFound()
                            .json(ErrorResponse::not_found(&err.to_string()))),
                        _ => Ok(HttpResponse::BadRequest()
                            .json(ErrorResponse::bad_request(&err.to_string()))),
                    }
                }
                error::BlockingError::Canceled => {
                    debug!("Internal Server Error: {}", err);
                    Ok(HttpResponse::InternalServerError().json(ErrorResponse::internal_error()))
                }
            },
        }),
    )
}

fn get_proposal_from_db(
    pool: web::Data<ConnectionPool>,
    id: i64,
) -> Result<ApiGameroomProposal, RestApiResponseError> {
    if let Some(proposal) = helpers::fetch_proposal_by_id(&*pool.get()?, id)? {
        let members = helpers::fetch_gameroom_members_by_circuit_id_and_status(
            &*pool.get()?,
            &proposal.circuit_id,
            "Pending",
        )?;
        return Ok(ApiGameroomProposal::from(proposal, members));
    }
    Err(RestApiResponseError::NotFound(format!(
        "Proposal with id {} not found",
        id
    )))
}

pub fn list_proposals(
    pool: web::Data<ConnectionPool>,
    query: web::Query<HashMap<String, usize>>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let offset: usize = query
        .get("offset")
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| DEFAULT_OFFSET);

    let limit: usize = query
        .get("limit")
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| DEFAULT_LIMIT);

    Box::new(
        web::block(move || list_proposals_from_db(pool, limit, offset)).then(
            move |res| match res {
                Ok((proposals, query_count)) => {
                    let paging_info = get_response_paging_info(
                        limit,
                        offset,
                        "api/proposals?",
                        query_count as usize,
                    );
                    Ok(HttpResponse::Ok().json(SuccessResponse::list(proposals, paging_info)))
                }
                Err(err) => {
                    debug!("Internal Server Error: {}", err);
                    Ok(HttpResponse::InternalServerError().json(ErrorResponse::internal_error()))
                }
            },
        ),
    )
}

fn list_proposals_from_db(
    pool: web::Data<ConnectionPool>,
    limit: usize,
    offset: usize,
) -> Result<(Vec<ApiGameroomProposal>, i64), RestApiResponseError> {
    let db_limit = limit as i64;
    let db_offset = offset as i64;

    let mut proposal_members: HashMap<String, Vec<GameroomMember>> =
        helpers::list_gameroom_members_with_status(&*pool.get()?, "Pending")?
            .into_iter()
            .fold(HashMap::new(), |mut acc, member| {
                acc.entry(member.circuit_id.to_string())
                    .or_insert_with(|| vec![])
                    .push(member);
                acc
            });
    let proposals = helpers::list_proposals_with_paging(&*pool.get()?, db_limit, db_offset)?
        .into_iter()
        .map(|proposal| {
            let circuit_id = proposal.circuit_id.to_string();
            ApiGameroomProposal::from(
                proposal,
                proposal_members
                    .remove(&circuit_id)
                    .unwrap_or_else(|| vec![]),
            )
        })
        .collect::<Vec<ApiGameroomProposal>>();

    Ok((proposals, helpers::get_proposal_count(&*pool.get()?)?))
}

pub fn proposal_vote(
    vote: web::Json<CircuitProposalVote>,
    proposal_id: web::Path<i64>,
    pool: web::Data<ConnectionPool>,
    node_info: web::Data<Node>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let node_identity = node_info.identity.to_string();
    Box::new(
        web::block(move || check_proposal_exists(*proposal_id, pool)).then(|res| match res {
            Ok(()) => match make_payload(vote.into_inner(), node_identity) {
                Ok(bytes) => Ok(HttpResponse::Ok()
                    .json(SuccessResponse::new(json!({ "payload_bytes": bytes })))),
                Err(err) => {
                    debug!("Failed to prepare circuit management payload {}", err);
                    Ok(HttpResponse::InternalServerError().json(ErrorResponse::internal_error()))
                }
            },
            Err(err) => match err {
                error::BlockingError::Error(err) => {
                    match err {
                        RestApiResponseError::NotFound(err) => Ok(HttpResponse::NotFound()
                            .json(ErrorResponse::not_found(&err.to_string()))),
                        RestApiResponseError::BadRequest(err) => Ok(HttpResponse::BadRequest()
                            .json(ErrorResponse::bad_request(&err.to_string()))),
                        _ => Ok(HttpResponse::InternalServerError()
                            .json(ErrorResponse::internal_error())),
                    }
                }
                error::BlockingError::Canceled => {
                    debug!("Internal Server Error: {}", err);
                    Ok(HttpResponse::InternalServerError().json(ErrorResponse::internal_error()))
                }
            },
        }),
    )
}

fn check_proposal_exists(
    proposal_id: i64,
    pool: web::Data<ConnectionPool>,
) -> Result<(), RestApiResponseError> {
    if let Some(proposal) = helpers::fetch_proposal_by_id(&*pool.get()?, proposal_id)? {
        if proposal.status == "Pending" {
            return Ok(());
        } else {
            return Err(RestApiResponseError::BadRequest(format!(
                "Cannot vote on proposal with id {}. The proposal status is {}",
                proposal_id, proposal.status
            )));
        }
    }

    Err(RestApiResponseError::NotFound(format!(
        "Proposal with id {} not found.",
        proposal_id
    )))
}

fn make_payload(
    vote: CircuitProposalVote,
    local_node: String,
) -> Result<Vec<u8>, RestApiResponseError> {
    let vote_proto = vote.into_proto();
    let vote_bytes = vote_proto.write_to_bytes()?;
    let hashed_bytes = hash(MessageDigest::sha512(), &vote_bytes)?;

    let mut header = Header::new();
    header.set_action(Action::CIRCUIT_PROPOSAL_VOTE);
    header.set_payload_sha512(hashed_bytes.to_vec());
    header.set_requester_node_id(local_node);
    let header_bytes = header.write_to_bytes()?;

    let mut circuit_management_payload = CircuitManagementPayload::new();
    circuit_management_payload.set_header(header_bytes);
    circuit_management_payload.set_circuit_proposal_vote(vote_proto);
    let payload_bytes = circuit_management_payload.write_to_bytes()?;
    Ok(payload_bytes)
}
