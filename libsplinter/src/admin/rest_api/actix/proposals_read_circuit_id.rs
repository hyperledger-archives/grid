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

use actix_web::{error::BlockingError, web, Error, HttpRequest, HttpResponse};
use futures::Future;

use crate::admin::messages::CircuitProposal;
use crate::admin::rest_api::error::ProposalRouteError;
use crate::admin::service::AdminCommands;
use crate::protocol;
use crate::rest_api::{Method, ProtocolVersionRangeGuard, Resource};

pub fn make_fetch_proposal_resource<A: AdminCommands + Clone + 'static>(
    admin_commands: A,
) -> Resource {
    Resource::build("admin/proposals/{circuit_id}")
        .add_request_guard(ProtocolVersionRangeGuard::new(
            protocol::ADMIN_FETCH_PROPOSALS_PROTOCOL_MIN,
            protocol::ADMIN_PROTOCOL_VERSION,
        ))
        .add_method(Method::Get, move |r, _| {
            fetch_proposal(r, web::Data::new(admin_commands.clone()))
        })
}

fn fetch_proposal<A: AdminCommands + Clone + 'static>(
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
