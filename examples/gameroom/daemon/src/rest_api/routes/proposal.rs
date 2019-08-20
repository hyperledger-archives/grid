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
    models::{CircuitMember, CircuitProposal},
    ConnectionPool,
};
use libsplinter::admin::messages::CircuitProposalVote;

use crate::authorization_handler;
use crate::rest_api::RestApiResponseError;

use super::{get_response_paging_info, Paging, DEFAULT_LIMIT, DEFAULT_OFFSET};

#[derive(Debug, Serialize)]
struct ProposalListResponse {
    data: Vec<ApiCircuitProposal>,
    paging: Paging,
}

#[derive(Debug, Serialize)]
struct ApiCircuitProposal {
    proposal_id: String,
    circuit_id: String,
    circuit_hash: String,
    members: Vec<ApiCircuitMember>,
    requester: String,
    created_time: u64,
    updated_time: u64,
}

impl ApiCircuitProposal {
    fn from(db_proposal: CircuitProposal, db_members: Vec<CircuitMember>) -> Self {
        ApiCircuitProposal {
            proposal_id: db_proposal.id.to_string(),
            circuit_id: db_proposal.circuit_id.to_string(),
            circuit_hash: db_proposal.circuit_hash.to_string(),
            members: db_members.into_iter().map(ApiCircuitMember::from).collect(),
            requester: db_proposal.requester.to_string(),
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
struct ApiCircuitMember {
    node_id: String,
    endpoint: String,
}

impl ApiCircuitMember {
    fn from(db_circuit_member: CircuitMember) -> Self {
        ApiCircuitMember {
            node_id: db_circuit_member.node_id.to_string(),
            endpoint: db_circuit_member.endpoint.to_string(),
        }
    }
}

pub fn fetch_proposal(
    pool: web::Data<ConnectionPool>,
    proposal_id: web::Path<String>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    Box::new(
        web::block(move || get_proposal_from_db(pool, &proposal_id)).then(|res| match res {
            Ok(proposal) => Ok(HttpResponse::Ok().json(proposal)),
            Err(err) => match err {
                error::BlockingError::Error(err) => match err {
                    RestApiResponseError::NotFound(err) => {
                        Ok(HttpResponse::NotFound().json(err.to_string()))
                    }
                    _ => Ok(HttpResponse::BadRequest().json(err.to_string())),
                },
                error::BlockingError::Canceled => Ok(HttpResponse::InternalServerError().into()),
            },
        }),
    )
}

fn get_proposal_from_db(
    pool: web::Data<ConnectionPool>,
    id: &str,
) -> Result<ApiCircuitProposal, RestApiResponseError> {
    if let Some(proposal) = helpers::fetch_proposal_by_id(&*pool.get()?, id)? {
        let members = helpers::fetch_circuit_members_by_proposal_id(&*pool.get()?, id)?;
        return Ok(ApiCircuitProposal::from(proposal, members));
    }
    Err(RestApiResponseError::NotFound(format!("Proposal {}", id)))
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
                    Ok(HttpResponse::Ok().json(ProposalListResponse {
                        data: proposals,
                        paging: paging_info,
                    }))
                }
                Err(err) => Ok(HttpResponse::InternalServerError().json(err.to_string())),
            },
        ),
    )
}

fn list_proposals_from_db(
    pool: web::Data<ConnectionPool>,
    limit: usize,
    offset: usize,
) -> Result<(Vec<ApiCircuitProposal>, i64), RestApiResponseError> {
    let db_limit = limit as i64;
    let db_offset = offset as i64;

    let mut proposal_members: HashMap<String, Vec<CircuitMember>> =
        helpers::list_proposal_circuit_members(&*pool.get()?)?
            .into_iter()
            .fold(HashMap::new(), |mut acc, member| {
                acc.entry(member.proposal_id.to_string())
                    .or_insert_with(|| vec![])
                    .push(member);
                acc
            });
    let proposals = helpers::list_proposals_with_paging(&*pool.get()?, db_limit, db_offset)?
        .into_iter()
        .map(|proposal| {
            let proposal_id = proposal.id.to_string();
            ApiCircuitProposal::from(
                proposal,
                proposal_members
                    .remove(&proposal_id)
                    .unwrap_or_else(|| vec![]),
            )
        })
        .collect::<Vec<ApiCircuitProposal>>();

    Ok((proposals, helpers::get_proposal_count(&*pool.get()?)?))
}

pub fn proposal_vote(
    vote: web::Json<CircuitProposalVote>,
    proposal_id: web::Path<String>,
    pool: web::Data<ConnectionPool>,
    splinterd_url: web::Data<String>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    Box::new(
        web::block(move || {
            check_proposal_exists(&proposal_id, pool)?;
            authorization_handler::submit_vote(&splinterd_url, &vote.into_inner()).map_err(|err| {
                RestApiResponseError::InternalError(format!(
                    "Ann error occurred while submitting vote: {}",
                    err
                ))
            })
        })
        .then(|res| match res {
            Ok(()) => {
                Ok(HttpResponse::Accepted().json(json!({ "message": "The vote was accepted"})))
            }
            Err(err) => match err {
                error::BlockingError::Error(err) => match err {
                    RestApiResponseError::NotFound(err) => {
                        Ok(HttpResponse::NotFound().json(json!({ "message": err.to_string() })))
                    }
                    RestApiResponseError::BadRequest(err) => {
                        Ok(HttpResponse::BadRequest().json(json!({ "message": err.to_string() })))
                    }
                    _ => Ok(HttpResponse::InternalServerError()
                        .json(json!({ "message": format!("{}", err) }))),
                },
                error::BlockingError::Canceled => Ok(HttpResponse::InternalServerError()
                    .json(json!({ "message": "Failed to submit vote"}))),
            },
        }),
    )
}

fn check_proposal_exists(
    proposal_id: &str,
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
