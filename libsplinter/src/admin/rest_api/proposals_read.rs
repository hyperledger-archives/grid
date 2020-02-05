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
use std::collections::HashMap;

use crate::actix_web::{error::BlockingError, web, Error, HttpRequest, HttpResponse};
use crate::admin::messages::CircuitProposal;
use crate::admin::rest_api::error::ProposalRouteError;
use crate::admin::service::AdminCommands;
use crate::futures::{future::IntoFuture, Future};
use crate::rest_api::paging::{get_response_paging_info, Paging, DEFAULT_LIMIT, DEFAULT_OFFSET};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ListProposalsResponse {
    data: Vec<CircuitProposal>,
    paging: Paging,
}

pub fn fetch_proposal<A: AdminCommands + Clone + 'static>(
    request: HttpRequest,
    admin_commands: web::Data<A>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let circuit_id = request
        .match_info()
        .get("circuit_id")
        .unwrap_or("")
        .to_string();
    Box::new(
        web::block(move || {
            let proposal = admin_commands
                .fetch_proposal(circuit_id.clone())
                .map_err(|err| ProposalRouteError::InternalError(err.to_string()))?;
            if let Some(proposal) = proposal {
                let proposal = CircuitProposal::from_proto(proposal)
                    .map_err(|err| ProposalRouteError::InternalError(err.to_string()))?;

                Ok(proposal)
            } else {
                Err(ProposalRouteError::NotFound(format!(
                    "Unable to find proposal: {}",
                    circuit_id
                )))
            }
        })
        .then(|res| match res {
            Ok(proposal) => Ok(HttpResponse::Ok().json(proposal)),
            Err(err) => match err {
                BlockingError::Error(err) => match err {
                    ProposalRouteError::InternalError(_) => {
                        error!("{}", err);
                        Ok(HttpResponse::InternalServerError().into())
                    }
                    ProposalRouteError::NotFound(err) => Ok(HttpResponse::NotFound().json(err)),
                },
                _ => Ok(HttpResponse::InternalServerError().into()),
            },
        }),
    )
}

pub fn list_proposals<A: AdminCommands + Clone + 'static>(
    req: HttpRequest,
    admin_commands: web::Data<A>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let query: web::Query<HashMap<String, String>> =
        if let Ok(q) = web::Query::from_query(req.query_string()) {
            q
        } else {
            return Box::new(
                HttpResponse::BadRequest()
                    .json(json!({
                        "message": "Invalid query"
                    }))
                    .into_future(),
            );
        };

    let offset = match query.get("offset") {
        Some(value) => match value.parse::<usize>() {
            Ok(val) => val,
            Err(err) => {
                return Box::new(
                    HttpResponse::BadRequest()
                        .json(format!(
                            "Invalid offset value passed: {}. Error: {}",
                            value, err
                        ))
                        .into_future(),
                )
            }
        },
        None => DEFAULT_OFFSET,
    };

    let limit = match query.get("limit") {
        Some(value) => match value.parse::<usize>() {
            Ok(val) => val,
            Err(err) => {
                return Box::new(
                    HttpResponse::BadRequest()
                        .json(format!(
                            "Invalid limit value passed: {}. Error: {}",
                            value, err
                        ))
                        .into_future(),
                )
            }
        },
        None => DEFAULT_LIMIT,
    };

    let mut link = format!("{}?", req.uri().path());

    let filters = match query.get("filter") {
        Some(value) => {
            link.push_str(&format!("filter={}&", value));
            Some(value.to_string())
        }
        None => None,
    };

    Box::new(query_list_proposals(
        admin_commands,
        link,
        filters,
        Some(offset),
        Some(limit),
    ))
}

fn query_list_proposals<A: AdminCommands + Clone + 'static>(
    admin_commands: web::Data<A>,
    link: String,
    filters: Option<String>,
    offset: Option<usize>,
    limit: Option<usize>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    web::block(move || {
        let proposals = admin_commands
            .list_proposals()
            .map_err(|err| ProposalRouteError::InternalError(err.to_string()))?;
        let offset_value = offset.unwrap_or(0);
        let limit_value = limit.unwrap_or_else(|| proposals.total());
        if proposals.total() != 0 {
            if let Some(filter) = filters {
                let filtered_proposals: Vec<CircuitProposal> = proposals
                    .filter(|(_, proposal)| proposal.circuit.circuit_management_type == filter)
                    .map(|(_, proposal)| proposal)
                    .collect();

                let total_count = filtered_proposals.len();

                let proposals_data: Vec<CircuitProposal> = filtered_proposals
                    .into_iter()
                    .skip(offset_value)
                    .take(limit_value)
                    .collect();

                Ok((proposals_data, link, limit, offset, total_count))
            } else {
                let total_count = proposals.total();
                let proposals_data: Vec<CircuitProposal> = proposals
                    .skip(offset_value)
                    .take(limit_value)
                    .map(|(_, proposal)| proposal)
                    .collect();

                Ok((proposals_data, link, limit, offset, total_count))
            }
        } else {
            Ok((vec![], link, limit, offset, proposals.total()))
        }
    })
    .then(|res| match res {
        Ok((circuits, link, limit, offset, total_count)) => {
            Ok(HttpResponse::Ok().json(ListProposalsResponse {
                data: circuits,
                paging: get_response_paging_info(limit, offset, &link, total_count),
            }))
        }
        Err(err) => match err {
            BlockingError::Error(err) => match err {
                ProposalRouteError::InternalError(_) => {
                    error!("{}", err);
                    Ok(HttpResponse::InternalServerError().into())
                }
                ProposalRouteError::NotFound(err) => Ok(HttpResponse::NotFound().json(err)),
            },
            _ => Ok(HttpResponse::InternalServerError().into()),
        },
    })
}
