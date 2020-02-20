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

use actix_web::{error::BlockingError, web, Error, HttpRequest, HttpResponse};
use futures::{future::IntoFuture, Future};
use std::collections::HashMap;

use crate::admin::messages::CircuitProposal;
use crate::admin::service::AdminCommands;
use crate::protocol;
use crate::rest_api::paging::{get_response_paging_info, DEFAULT_LIMIT, DEFAULT_OFFSET};
use crate::rest_api::{Method, ProtocolVersionRangeGuard, Resource};

use super::super::error::ProposalListError;
use super::super::resources::proposals_read::ListProposalsResponse;

pub fn make_list_proposals_resource<A: AdminCommands + Clone + 'static>(
    admin_commands: A,
) -> Resource {
    Resource::build("admin/proposals")
        .add_request_guard(ProtocolVersionRangeGuard::new(
            protocol::ADMIN_LIST_PROPOSALS_PROTOCOL_MIN,
            protocol::ADMIN_PROTOCOL_VERSION,
        ))
        .add_method(Method::Get, move |r, _| {
            list_proposals(r, web::Data::new(admin_commands.clone()))
        })
}

fn list_proposals<A: AdminCommands + Clone + 'static>(
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
            .map_err(|err| ProposalListError::InternalError(err.to_string()))?;
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
                ProposalListError::InternalError(_) => {
                    error!("{}", err);
                    Ok(HttpResponse::InternalServerError().into())
                }
            },
            _ => Ok(HttpResponse::InternalServerError().into()),
        },
    })
}
