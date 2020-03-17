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

//! This module provides the `GET /admin/circuits/{circuit_id} endpoint for fetching the
//! definition of a circuit in Splinter's state by its circuit ID.

use actix_web::{error::BlockingError, web, Error, HttpRequest, HttpResponse};
use futures::Future;

use crate::circuit::store::CircuitStore;
use crate::protocol;
use crate::rest_api::{ErrorResponse, Method, ProtocolVersionRangeGuard, Resource};

use super::super::error::CircuitFetchError;
use super::super::resources::circuits_circuit_id::CircuitResponse;

pub fn make_fetch_circuit_resource<T: CircuitStore + 'static>(store: T) -> Resource {
    Resource::build("/admin/circuits/{circuit_id}")
        .add_request_guard(ProtocolVersionRangeGuard::new(
            protocol::ADMIN_FETCH_CIRCUIT_MIN,
            protocol::ADMIN_PROTOCOL_VERSION,
        ))
        .add_method(Method::Get, move |r, _| {
            fetch_circuit(r, web::Data::new(store.clone()))
        })
}

fn fetch_circuit<T: CircuitStore + 'static>(
    request: HttpRequest,
    store: web::Data<T>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let circuit_id = request
        .match_info()
        .get("circuit_id")
        .unwrap_or("")
        .to_string();
    Box::new(
        web::block(move || {
            store.circuit(&circuit_id)?.ok_or_else(|| {
                CircuitFetchError::NotFound(format!("Unable to find circuit: {}", circuit_id))
            })
        })
        .then(|res| match res {
            Ok(circuit) => Ok(HttpResponse::Ok().json(CircuitResponse::from(&circuit))),
            Err(err) => match err {
                BlockingError::Error(err) => match err {
                    CircuitFetchError::CircuitStoreError(err) => {
                        error!("{}", err);
                        Ok(HttpResponse::InternalServerError()
                            .json(ErrorResponse::internal_error()))
                    }
                    CircuitFetchError::NotFound(err) => {
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

#[cfg(test)]
mod tests {
    use super::*;

    use reqwest::{blocking::Client, StatusCode, Url};

    use crate::circuit::{
        directory::CircuitDirectory, AuthorizationType, Circuit, DurabilityType, PersistenceType,
        RouteType, ServiceDefinition, SplinterState,
    };
    use crate::rest_api::{RestApiBuilder, RestApiServerError, RestApiShutdownHandle};
    use crate::storage::get_storage;

    #[test]
    /// Tests a GET /admin/circuit/{circuit_id} request returns the expected circuit.
    fn test_fetch_circuit_ok() {
        let (_shutdown_handle, _join_handle, bind_url) =
            run_rest_api_on_open_port(vec![make_fetch_circuit_resource(filled_splinter_state())]);

        let url = Url::parse(&format!(
            "http://{}/admin/circuits/{}",
            bind_url,
            get_circuit_1().id()
        ))
        .expect("Failed to parse URL");
        let req = Client::new()
            .get(url)
            .header("SplinterProtocolVersion", protocol::ADMIN_PROTOCOL_VERSION);
        let resp = req.send().expect("Failed to perform request");

        assert_eq!(resp.status(), StatusCode::OK);
        let circuit: CircuitResponse = resp.json().expect("Failed to deserialize body");
        assert_eq!(circuit, get_circuit_1().into());
    }

    #[test]
    /// Tests a GET /admin/circuits/{circuit_id} request returns NotFound when an invalid
    /// circuit_id is passed.
    fn test_fetch_circuit_not_found() {
        let (_shutdown_handle, _join_handle, bind_url) =
            run_rest_api_on_open_port(vec![make_fetch_circuit_resource(filled_splinter_state())]);

        let url = Url::parse(&format!(
            "http://{}/admin/circuits/Circuit-not-valid",
            bind_url,
        ))
        .expect("Failed to parse URL");
        let req = Client::new()
            .get(url)
            .header("SplinterProtocolVersion", protocol::ADMIN_PROTOCOL_VERSION);
        let resp = req.send().expect("Failed to perform request");

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
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
