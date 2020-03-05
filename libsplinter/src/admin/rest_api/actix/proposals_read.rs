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

//! Provides the `GET /admin/proposals` endpoint for listing circuit proposals.

use actix_web::{error::BlockingError, web, Error, HttpRequest, HttpResponse};
use futures::{future::IntoFuture, Future};
use std::collections::HashMap;

use crate::admin::messages::CircuitProposal;
use crate::admin::service::proposal_store::ProposalStore;
use crate::protocol;
use crate::rest_api::paging::{get_response_paging_info, DEFAULT_LIMIT, DEFAULT_OFFSET};
use crate::rest_api::{ErrorResponse, Method, ProtocolVersionRangeGuard, Resource};

use super::super::error::ProposalListError;
use super::super::resources::proposals_read::ListProposalsResponse;

pub fn make_list_proposals_resource<PS: ProposalStore + 'static>(proposal_store: PS) -> Resource {
    Resource::build("admin/proposals")
        .add_request_guard(ProtocolVersionRangeGuard::new(
            protocol::ADMIN_LIST_PROPOSALS_PROTOCOL_MIN,
            protocol::ADMIN_PROTOCOL_VERSION,
        ))
        .add_method(Method::Get, move |r, _| {
            list_proposals(r, web::Data::new(proposal_store.clone()))
        })
}

fn list_proposals<PS: ProposalStore + 'static>(
    req: HttpRequest,
    proposal_store: web::Data<PS>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let query: web::Query<HashMap<String, String>> =
        if let Ok(q) = web::Query::from_query(req.query_string()) {
            q
        } else {
            return Box::new(
                HttpResponse::BadRequest()
                    .json(ErrorResponse::bad_request("Invalid query"))
                    .into_future(),
            );
        };

    let offset = match query.get("offset") {
        Some(value) => match value.parse::<usize>() {
            Ok(val) => val,
            Err(err) => {
                return Box::new(
                    HttpResponse::BadRequest()
                        .json(ErrorResponse::bad_request(&format!(
                            "Invalid offset value passed: {}. Error: {}",
                            value, err
                        )))
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
                        .json(ErrorResponse::bad_request(&format!(
                            "Invalid limit value passed: {}. Error: {}",
                            value, err
                        )))
                        .into_future(),
                )
            }
        },
        None => DEFAULT_LIMIT,
    };

    let mut link = req.uri().path().to_string();

    let filters = match query.get("filter") {
        Some(value) => {
            link.push_str(&format!("?filter={}&", value));
            Some(value.to_string())
        }
        None => None,
    };

    Box::new(query_list_proposals(
        proposal_store,
        link,
        filters,
        Some(offset),
        Some(limit),
    ))
}

fn query_list_proposals<PS: ProposalStore + 'static>(
    proposal_store: web::Data<PS>,
    link: String,
    filters: Option<String>,
    offset: Option<usize>,
    limit: Option<usize>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    web::block(move || {
        let proposals = proposal_store
            .proposals()
            .map_err(|err| ProposalListError::InternalError(err.to_string()))?;
        let offset_value = offset.unwrap_or(0);
        let limit_value = limit.unwrap_or_else(|| proposals.total());
        if proposals.total() != 0 {
            if let Some(filter) = filters {
                let filtered_proposals: Vec<CircuitProposal> = proposals
                    .filter(|proposal| proposal.circuit.circuit_management_type == filter)
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
                let proposals_data: Vec<CircuitProposal> =
                    proposals.skip(offset_value).take(limit_value).collect();

                Ok((proposals_data, link, limit, offset, total_count))
            }
        } else {
            Ok((vec![], link, limit, offset, proposals.total()))
        }
    })
    .then(|res| match res {
        Ok((proposals, link, limit, offset, total_count)) => {
            Ok(HttpResponse::Ok().json(ListProposalsResponse {
                data: proposals,
                paging: get_response_paging_info(limit, offset, &link, total_count),
            }))
        }
        Err(err) => match err {
            BlockingError::Error(err) => match err {
                ProposalListError::InternalError(_) => {
                    error!("{}", err);
                    Ok(HttpResponse::InternalServerError().into())
                }
            },
            _ => Ok(HttpResponse::InternalServerError().into()),
        },
    })
}
