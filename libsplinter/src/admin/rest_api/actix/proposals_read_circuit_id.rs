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

//! Provides the `GET /admin/proposals/{circuit_id} endpoint for fetching circuit proposals by
//! circuit ID.

use actix_web::{error::BlockingError, web, Error, HttpRequest, HttpResponse};
use futures::Future;

use crate::admin::rest_api::error::ProposalFetchError;
use crate::admin::service::proposal_store::ProposalStore;
use crate::protocol;
use crate::rest_api::{ErrorResponse, Method, ProtocolVersionRangeGuard, Resource};

pub fn make_fetch_proposal_resource<PS: ProposalStore + 'static>(proposal_store: PS) -> Resource {
    Resource::build("admin/proposals/{circuit_id}")
        .add_request_guard(ProtocolVersionRangeGuard::new(
            protocol::ADMIN_FETCH_PROPOSALS_PROTOCOL_MIN,
            protocol::ADMIN_PROTOCOL_VERSION,
        ))
        .add_method(Method::Get, move |r, _| {
            fetch_proposal(r, web::Data::new(proposal_store.clone()))
        })
}

fn fetch_proposal<PS: ProposalStore + 'static>(
    request: HttpRequest,
    proposal_store: web::Data<PS>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let circuit_id = request
        .match_info()
        .get("circuit_id")
        .unwrap_or("")
        .to_string();
    Box::new(
        web::block(move || {
            proposal_store
                .proposal(&circuit_id)
                .map_err(|err| ProposalFetchError::InternalError(err.to_string()))?
                .ok_or_else(|| {
                    ProposalFetchError::NotFound(format!("Unable to find proposal: {}", circuit_id))
                })
        })
        .then(|res| match res {
            Ok(proposal) => Ok(HttpResponse::Ok().json(proposal)),
            Err(err) => match err {
                BlockingError::Error(err) => match err {
                    ProposalFetchError::InternalError(_) => {
                        error!("{}", err);
                        Ok(HttpResponse::InternalServerError()
                            .json(ErrorResponse::internal_error()))
                    }
                    ProposalFetchError::NotFound(err) => {
                        Ok(HttpResponse::NotFound().json(ErrorResponse::not_found(&err)))
                    }
                },
                _ => {
                    error!("{}", err);
                    Ok(HttpResponse::InternalServerError().json(ErrorResponse::internal_error()))
                }
            },
        }),
    )
}
