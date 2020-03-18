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

use crate::admin::service::proposal_store::{ProposalFilter, ProposalStore};
use crate::protocol;
use crate::rest_api::paging::{get_response_paging_info, DEFAULT_LIMIT, DEFAULT_OFFSET};
use crate::rest_api::{ErrorResponse, Method, ProtocolVersionRangeGuard, Resource};

use super::super::error::ProposalListError;
use super::super::resources::proposals::{ListProposalsResponse, ProposalResponse};

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

    let management_type_filter = query.get("management_type").map(|management_type| {
        link.push_str(&format!("?management_type={}&", management_type));
        management_type.to_string()
    });

    Box::new(query_list_proposals(
        proposal_store,
        link,
        management_type_filter,
        Some(offset),
        Some(limit),
    ))
}

fn query_list_proposals<PS: ProposalStore + 'static>(
    proposal_store: web::Data<PS>,
    link: String,
    management_type_filter: Option<String>,
    offset: Option<usize>,
    limit: Option<usize>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    web::block(move || {
        let mut filters = vec![];
        if let Some(management_type) = management_type_filter {
            filters.push(ProposalFilter::WithManagementType(management_type));
        }

        let proposals = proposal_store
            .proposals(filters)
            .map_err(|err| ProposalListError::InternalError(err.to_string()))?;
        let offset_value = offset.unwrap_or(0);
        let total = proposals.total() as usize;
        let limit_value = limit.unwrap_or(total);

        let proposals = proposals
            .skip(offset_value)
            .take(limit_value)
            .collect::<Vec<_>>();

        Ok((proposals, link, limit, offset, total))
    })
    .then(|res| match res {
        Ok((proposals, link, limit, offset, total_count)) => {
            Ok(HttpResponse::Ok().json(ListProposalsResponse {
                data: proposals.iter().map(ProposalResponse::from).collect(),
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

#[cfg(test)]
mod tests {
    use super::*;

    use reqwest::{blocking::Client, StatusCode, Url};

    use crate::admin::{
        messages::{
            AuthorizationType, CircuitProposal, CreateCircuit, DurabilityType, PersistenceType,
            ProposalType, RouteType,
        },
        service::proposal_store::{ProposalFilter, ProposalIter, ProposalStoreError},
    };
    use crate::rest_api::{
        paging::Paging, RestApiBuilder, RestApiServerError, RestApiShutdownHandle,
    };

    #[test]
    /// Tests a GET /admin/proposals request with no filters returns the expected proposals.
    fn test_list_proposals_ok() {
        let (_shutdown_handle, _join_handle, bind_url) =
            run_rest_api_on_open_port(vec![make_list_proposals_resource(MockProposalStore)]);

        let url = Url::parse(&format!("http://{}/admin/proposals", bind_url))
            .expect("Failed to parse URL");
        let req = Client::new()
            .get(url)
            .header("SplinterProtocolVersion", protocol::ADMIN_PROTOCOL_VERSION);
        let resp = req.send().expect("Failed to perform request");

        assert_eq!(resp.status(), StatusCode::OK);
        let proposals: ListProposalsResponse = resp.json().expect("Failed to deserialize body");
        assert_eq!(
            proposals.data,
            vec![get_proposal_1().into(), get_proposal_2().into()]
        );
        assert_eq!(
            proposals.paging,
            create_test_paging_response(0, 100, 0, 0, 0, 2, "/admin/proposals?")
        )
    }

    #[test]
    /// Tests a GET /admin/proposals request with the `management_type` filter returns the expected
    /// proposal.
    fn test_list_proposals_with_management_type_ok() {
        let (_shutdown_handle, _join_handle, bind_url) =
            run_rest_api_on_open_port(vec![make_list_proposals_resource(MockProposalStore)]);

        let url = Url::parse(&format!(
            "http://{}/admin/proposals?management_type=mgmt_type_1",
            bind_url
        ))
        .expect("Failed to parse URL");
        let req = Client::new()
            .get(url)
            .header("SplinterProtocolVersion", protocol::ADMIN_PROTOCOL_VERSION);
        let resp = req.send().expect("Failed to perform request");

        assert_eq!(resp.status(), StatusCode::OK);
        let proposals: ListProposalsResponse = resp.json().expect("Failed to deserialize body");
        assert_eq!(proposals.data, vec![get_proposal_1().into()]);
        let link = format!("/admin/proposals?management_type=mgmt_type_1&");
        assert_eq!(
            proposals.paging,
            create_test_paging_response(0, 100, 0, 0, 0, 1, &link)
        )
    }

    #[test]
    /// Tests a GET /admin/proposals?limit=1 request returns the expected proposal.
    fn test_list_proposal_with_limit() {
        let (_shutdown_handle, _join_handle, bind_url) =
            run_rest_api_on_open_port(vec![make_list_proposals_resource(MockProposalStore)]);

        let url = Url::parse(&format!("http://{}/admin/proposals?limit=1", bind_url))
            .expect("Failed to parse URL");
        let req = Client::new()
            .get(url)
            .header("SplinterProtocolVersion", protocol::ADMIN_PROTOCOL_VERSION);
        let resp = req.send().expect("Failed to perform request");

        assert_eq!(resp.status(), StatusCode::OK);
        let proposals: ListProposalsResponse = resp.json().expect("Failed to deserialize body");
        assert_eq!(proposals.data, vec![get_proposal_1().into()]);
        assert_eq!(
            proposals.paging,
            create_test_paging_response(0, 1, 1, 0, 1, 2, "/admin/proposals?")
        )
    }

    #[test]
    /// Tests a GET /admin/proposals?offset=1 request returns the expected proposal.
    fn test_list_proposal_with_offset() {
        let (_shutdown_handle, _join_handle, bind_url) =
            run_rest_api_on_open_port(vec![make_list_proposals_resource(MockProposalStore)]);

        let url = Url::parse(&format!("http://{}/admin/proposals?offset=1", bind_url))
            .expect("Failed to parse URL");
        let req = Client::new()
            .get(url)
            .header("SplinterProtocolVersion", protocol::ADMIN_PROTOCOL_VERSION);
        let resp = req.send().expect("Failed to perform request");

        assert_eq!(resp.status(), StatusCode::OK);
        let proposals: ListProposalsResponse = resp.json().expect("Failed to deserialize body");
        assert_eq!(proposals.data, vec![get_proposal_2().into()]);
        assert_eq!(
            proposals.paging,
            create_test_paging_response(1, 100, 0, 0, 0, 2, "/admin/proposals?")
        )
    }

    fn create_test_paging_response(
        offset: usize,
        limit: usize,
        next_offset: usize,
        previous_offset: usize,
        last_offset: usize,
        total: usize,
        link: &str,
    ) -> Paging {
        let base_link = format!("{}limit={}&", link, limit);
        let current_link = format!("{}offset={}", base_link, offset);
        let first_link = format!("{}offset=0", base_link);
        let next_link = format!("{}offset={}", base_link, next_offset);
        let previous_link = format!("{}offset={}", base_link, previous_offset);
        let last_link = format!("{}offset={}", base_link, last_offset);

        Paging {
            current: current_link,
            offset,
            limit,
            total,
            first: first_link,
            prev: previous_link,
            next: next_link,
            last: last_link,
        }
    }

    #[derive(Clone)]
    struct MockProposalStore;

    impl ProposalStore for MockProposalStore {
        fn proposals(
            &self,
            filters: Vec<ProposalFilter>,
        ) -> Result<ProposalIter, ProposalStoreError> {
            let proposals = vec![get_proposal_1(), get_proposal_2()];

            let total = proposals
                .iter()
                .filter(|proposal| filters.iter().all(|filter| filter.matches(&proposal)))
                .count();

            let iter =
                Box::new(proposals.into_iter().filter(move |proposal| {
                    filters.iter().all(|filter| filter.matches(&proposal))
                }));

            Ok(ProposalIter::new(iter, total))
        }

        fn proposal(
            &self,
            _circuit_id: &str,
        ) -> Result<Option<CircuitProposal>, ProposalStoreError> {
            unimplemented!()
        }
    }

    fn get_proposal_1() -> CircuitProposal {
        CircuitProposal {
            proposal_type: ProposalType::Create,
            circuit_id: "circuit1".into(),
            circuit_hash: "012345".into(),
            circuit: CreateCircuit {
                circuit_id: "circuit1".into(),
                roster: vec![],
                members: vec![],
                authorization_type: AuthorizationType::Trust,
                persistence: PersistenceType::Any,
                durability: DurabilityType::NoDurability,
                routes: RouteType::Any,
                circuit_management_type: "mgmt_type_1".into(),
                application_metadata: vec![],
                comments: "mock circuit 1".into(),
            },
            votes: vec![],
            requester: vec![],
            requester_node_id: "node_id".into(),
        }
    }

    fn get_proposal_2() -> CircuitProposal {
        CircuitProposal {
            proposal_type: ProposalType::Create,
            circuit_id: "circuit2".into(),
            circuit_hash: "abcdef".into(),
            circuit: CreateCircuit {
                circuit_id: "circuit2".into(),
                roster: vec![],
                members: vec![],
                authorization_type: AuthorizationType::Trust,
                persistence: PersistenceType::Any,
                durability: DurabilityType::NoDurability,
                routes: RouteType::Any,
                circuit_management_type: "mgmt_type_2".into(),
                application_metadata: vec![],
                comments: "mock circuit 2".into(),
            },
            votes: vec![],
            requester: vec![],
            requester_node_id: "node_id".into(),
        }
    }

    fn run_rest_api_on_open_port(
        resources: Vec<Resource>,
    ) -> (RestApiShutdownHandle, std::thread::JoinHandle<()>, String) {
        (10000..20000)
            .find_map(|port| {
                let bind_url = format!("127.0.0.1:{}", port);
                let result = RestApiBuilder::new()
                    .with_bind(&bind_url)
                    .add_resources(resources.clone())
                    .build()
                    .expect("Failed to build REST API")
                    .run();
                match result {
                    Ok((shutdown_handle, join_handle)) => {
                        Some((shutdown_handle, join_handle, bind_url))
                    }
                    Err(RestApiServerError::BindError(_)) => None,
                    Err(err) => panic!("Failed to run REST API: {}", err),
                }
            })
            .expect("No port available")
    }
}
