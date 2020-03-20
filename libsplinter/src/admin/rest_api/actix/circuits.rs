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

//! This module provides the `GET /admin/circuits` endpoint for listing the definitions of circuits
//! in Splinter's state.

use actix_web::{error::BlockingError, web, Error, HttpRequest, HttpResponse};
use futures::{future::IntoFuture, Future};
use std::collections::HashMap;

use crate::circuit::store::{CircuitFilter, CircuitStore};
use crate::protocol;
use crate::rest_api::{
    paging::{get_response_paging_info, DEFAULT_LIMIT, DEFAULT_OFFSET},
    ErrorResponse, Method, ProtocolVersionRangeGuard, Resource,
};

use super::super::error::CircuitListError;
use super::super::resources::circuits::{CircuitResponse, ListCircuitsResponse};

pub fn make_list_circuits_resource<T: CircuitStore + 'static>(store: T) -> Resource {
    Resource::build("/admin/circuits")
        .add_request_guard(ProtocolVersionRangeGuard::new(
            protocol::ADMIN_LIST_CIRCUITS_MIN,
            protocol::ADMIN_PROTOCOL_VERSION,
        ))
        .add_method(Method::Get, move |r, _| {
            list_circuits(r, web::Data::new(store.clone()))
        })
}

fn list_circuits<T: CircuitStore + 'static>(
    req: HttpRequest,
    store: web::Data<T>,
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

    Box::new(query_list_circuits(
        store,
        link,
        filters,
        Some(offset),
        Some(limit),
    ))
}

fn query_list_circuits<T: CircuitStore + 'static>(
    store: web::Data<T>,
    link: String,
    filters: Option<String>,
    offset: Option<usize>,
    limit: Option<usize>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    web::block(move || {
        let circuits = store.circuits(filters.map(CircuitFilter::WithMember))?;
        let offset_value = offset.unwrap_or(0);
        let total = circuits.total();
        let limit_value = limit.unwrap_or_else(|| total as usize);

        let circuits = circuits
            .skip(offset_value)
            .take(limit_value)
            .collect::<Vec<_>>();

        Ok((circuits, link, limit, offset, total as usize))
    })
    .then(|res| match res {
        Ok((circuits, link, limit, offset, total_count)) => {
            Ok(HttpResponse::Ok().json(ListCircuitsResponse {
                data: circuits.iter().map(CircuitResponse::from).collect(),
                paging: get_response_paging_info(limit, offset, &link, total_count),
            }))
        }
        Err(err) => match err {
            BlockingError::Error(err) => match err {
                CircuitListError::CircuitStoreError(err) => {
                    error!("{}", err);
                    Ok(HttpResponse::InternalServerError().json(ErrorResponse::internal_error()))
                }
            },
            _ => {
                error!("{}", err);
                Ok(HttpResponse::InternalServerError().json(ErrorResponse::internal_error()))
            }
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use reqwest::{blocking::Client, StatusCode, Url};
    use serde_json::{to_value, Value as JsonValue};

    use crate::circuit::{
        directory::CircuitDirectory, AuthorizationType, Circuit, DurabilityType, PersistenceType,
        RouteType, ServiceDefinition, SplinterState,
    };
    use crate::rest_api::{
        paging::Paging, RestApiBuilder, RestApiServerError, RestApiShutdownHandle,
    };
    use crate::storage::get_storage;

    #[test]
    /// Tests a GET /admin/circuits request with no filters returns the expected circuits.
    fn test_list_circuits_ok() {
        let (_shutdown_handle, _join_handle, bind_url) =
            run_rest_api_on_open_port(vec![make_list_circuits_resource(filled_splinter_state())]);

        let url = Url::parse(&format!("http://{}/admin/circuits", bind_url))
            .expect("Failed to parse URL");
        let req = Client::new()
            .get(url)
            .header("SplinterProtocolVersion", protocol::ADMIN_PROTOCOL_VERSION);
        let resp = req.send().expect("Failed to perform request");

        assert_eq!(resp.status(), StatusCode::OK);
        let circuits: JsonValue = resp.json().expect("Failed to deserialize body");

        assert_eq!(
            circuits.get("data").expect("no data field in response"),
            &to_value(vec![
                CircuitResponse::from(&get_circuit_1()),
                CircuitResponse::from(&get_circuit_2())
            ])
            .expect("failed to convert expected data"),
        );
        assert_eq!(
            circuits.get("paging").expect("no paging field in response"),
            &to_value(create_test_paging_response(
                0,
                100,
                0,
                0,
                0,
                2,
                "/admin/circuits?",
            ))
            .expect("failed to convert expected paging")
        )
    }

    #[test]
    /// Tests a GET /admin/circuits request with filter returns the expected circuit.
    fn test_list_circuit_with_filters_ok() {
        let (_shutdown_handle, _join_handle, bind_url) =
            run_rest_api_on_open_port(vec![make_list_circuits_resource(filled_splinter_state())]);

        let url = Url::parse(&format!("http://{}/admin/circuits?filter=node_1", bind_url))
            .expect("Failed to parse URL");
        let req = Client::new()
            .get(url)
            .header("SplinterProtocolVersion", protocol::ADMIN_PROTOCOL_VERSION);
        let resp = req.send().expect("Failed to perform request");

        assert_eq!(resp.status(), StatusCode::OK);
        let circuits: JsonValue = resp.json().expect("Failed to deserialize body");

        assert_eq!(
            circuits.get("data").expect("no data field in response"),
            &to_value(vec![CircuitResponse::from(&get_circuit_1())])
                .expect("failed to convert expected data"),
        );

        assert_eq!(
            circuits.get("paging").expect("no paging field in response"),
            &to_value(create_test_paging_response(
                0,
                100,
                0,
                0,
                0,
                1,
                &format!("/admin/circuits?filter=node_1&"),
            ))
            .expect("failed to convert expected paging")
        )
    }

    #[test]
    /// Tests a GET /admin/circuits?limit=1 request returns the expected circuit.
    fn test_list_circuit_with_limit() {
        let (_shutdown_handle, _join_handle, bind_url) =
            run_rest_api_on_open_port(vec![make_list_circuits_resource(filled_splinter_state())]);

        let url = Url::parse(&format!("http://{}/admin/circuits?limit=1", bind_url))
            .expect("Failed to parse URL");
        let req = Client::new()
            .get(url)
            .header("SplinterProtocolVersion", protocol::ADMIN_PROTOCOL_VERSION);
        let resp = req.send().expect("Failed to perform request");

        assert_eq!(resp.status(), StatusCode::OK);
        let circuits: JsonValue = resp.json().expect("Failed to deserialize body");

        assert_eq!(
            circuits.get("data").expect("no data field in response"),
            &to_value(vec![CircuitResponse::from(&get_circuit_1())])
                .expect("failed to convert expected data"),
        );

        assert_eq!(
            circuits.get("paging").expect("no paging field in response"),
            &to_value(create_test_paging_response(
                0,
                1,
                1,
                0,
                1,
                2,
                "/admin/circuits?",
            ))
            .expect("failed to convert expected paging")
        )
    }

    #[test]
    /// Tests a GET /admin/circuits?offset=1 request returns the expected circuit.
    fn test_list_circuit_with_offset() {
        let (_shutdown_handle, _join_handle, bind_url) =
            run_rest_api_on_open_port(vec![make_list_circuits_resource(filled_splinter_state())]);

        let url = Url::parse(&format!("http://{}/admin/circuits?offset=1", bind_url))
            .expect("Failed to parse URL");
        let req = Client::new()
            .get(url)
            .header("SplinterProtocolVersion", protocol::ADMIN_PROTOCOL_VERSION);
        let resp = req.send().expect("Failed to perform request");

        assert_eq!(resp.status(), StatusCode::OK);
        let circuits: JsonValue = resp.json().expect("Failed to deserialize body");

        assert_eq!(
            circuits.get("data").expect("no data field in response"),
            &to_value(vec![CircuitResponse::from(&get_circuit_2())])
                .expect("failed to convert expected data"),
        );

        assert_eq!(
            circuits.get("paging").expect("no paging field in response"),
            &to_value(create_test_paging_response(
                1,
                100,
                0,
                0,
                0,
                2,
                "/admin/circuits?"
            ))
            .expect("failed to convert expected paging")
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

    fn get_circuit_1() -> Circuit {
        let service_definition =
            ServiceDefinition::builder("service_1".to_string(), "type_a".to_string())
                .with_allowed_nodes(vec!["node_1".to_string()])
                .build();
        Circuit::builder()
            .with_id("circuit_1".into())
            .with_auth(AuthorizationType::Trust)
            .with_members(vec!["node_1".to_string(), "node_2".to_string()])
            .with_roster(vec![service_definition])
            .with_persistence(PersistenceType::Any)
            .with_durability(DurabilityType::NoDurability)
            .with_routes(RouteType::Any)
            .with_circuit_management_type("circuit_1_type".into())
            .build()
            .expect("Should have built a correct circuit")
    }

    fn get_circuit_2() -> Circuit {
        let service_definition =
            ServiceDefinition::builder("service_2".to_string(), "other_type".to_string())
                .with_allowed_nodes(vec!["node_3".to_string()])
                .build();
        Circuit::builder()
            .with_id("circuit_2".into())
            .with_auth(AuthorizationType::Trust)
            .with_members(vec!["node_3".to_string(), "node_4".to_string()])
            .with_roster(vec![service_definition])
            .with_persistence(PersistenceType::Any)
            .with_durability(DurabilityType::NoDurability)
            .with_routes(RouteType::Any)
            .with_circuit_management_type("circuit_2_type".into())
            .build()
            .expect("Should have built a correct circuit")
    }

    fn setup_splinter_state() -> SplinterState {
        let mut storage = get_storage("memory", CircuitDirectory::new).unwrap();
        let circuit_directory = storage.write().clone();
        SplinterState::new("memory".to_string(), circuit_directory)
    }

    fn filled_splinter_state() -> SplinterState {
        let mut splinter_state = setup_splinter_state();
        splinter_state
            .add_circuit("circuit_1".into(), get_circuit_1())
            .expect("Unable to add circuit_1");
        splinter_state
            .add_circuit("circuit_2".into(), get_circuit_2())
            .expect("Unable to add circuit_2");

        splinter_state
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
