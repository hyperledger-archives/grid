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

use crate::rest_api::RestApiResponseError;

#[derive(Debug, Serialize)]
struct ApiCircuitProposal {
    proposal_id: String,
    members: Vec<ApiCircuitMember>,
    requester: String,
    created_time: u64,
    updated_time: u64,
}

impl ApiCircuitProposal {
    fn from(db_proposal: CircuitProposal, db_members: Vec<CircuitMember>) -> Self {
        ApiCircuitProposal {
            proposal_id: db_proposal.id.to_string(),
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
