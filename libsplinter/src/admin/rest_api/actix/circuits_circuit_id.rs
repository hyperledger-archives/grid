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
            let circuit = store.circuit(&circuit_id)?;
            if let Some(circuit) = circuit {
                let circuit_response = CircuitResponse {
                    id: circuit_id,
                    auth: circuit.auth().clone(),
                    members: circuit.members().to_vec(),
                    roster: circuit.roster().clone(),
                    persistence: circuit.persistence().clone(),
                    durability: circuit.durability().clone(),
                    routes: circuit.routes().clone(),
                    circuit_management_type: circuit.circuit_management_type().to_string(),
                };
                Ok(circuit_response)
            } else {
                Err(CircuitFetchError::NotFound(format!(
                    "Unable to find circuit: {}",
                    circuit_id
                )))
            }
        })
        .then(|res| match res {
            Ok(circuit) => Ok(HttpResponse::Ok().json(circuit)),
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

    use crate::actix_web::{http::StatusCode, test, web, App};
    use crate::circuit::{
        directory::CircuitDirectory, AuthorizationType, Circuit, DurabilityType, PersistenceType,
        Roster, RouteType, ServiceDefinition, SplinterState,
    };
    use crate::storage::get_storage;

    #[test]
    /// Tests a GET /admin/circuit/{identity} request returns the expected circuit.
    fn test_fetch_circuit_ok() {
        let splinter_state = filled_splinter_state();

        let mut app = test::init_service(
            App::new().data(splinter_state.clone()).service(
                web::resource("/admin/circuits/{circuit_id}")
                    .route(web::get().to_async(fetch_circuit::<SplinterState>)),
            ),
        );

        let req = test::TestRequest::get()
            .uri(&format!("/admin/circuits/{}", get_circuit_1().id))
            .to_request();

        let resp = test::call_service(&mut app, req);

        assert_eq!(resp.status(), StatusCode::OK);
        let circuit: CircuitResponse = serde_yaml::from_slice(&test::read_body(resp)).unwrap();
        assert_eq!(circuit, get_circuit_1())
    }

    #[test]
    /// Tests a GET /admin/circuits/{identity} request returns NotFound when an invalid identity is
    /// passed
    fn test_fetch_circuit_not_found() {
        let splinter_state = filled_splinter_state();
        let mut app = test::init_service(
            App::new().data(splinter_state.clone()).service(
                web::resource("/admin/circuits/{circuit_id}")
                    .route(web::get().to_async(fetch_circuit::<SplinterState>)),
            ),
        );

        let req = test::TestRequest::get()
            .uri("/admin/circuit/Circuit-not-valid")
            .to_request();

        let resp = test::call_service(&mut app, req);

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    fn get_circuit_1() -> CircuitResponse {
        let service_definition =
            ServiceDefinition::builder("service_1".to_string(), "type_a".to_string())
                .with_allowed_nodes(vec!["node_1".to_string()])
                .build();
        CircuitResponse {
            id: "circuit_1".to_string(),
            auth: AuthorizationType::Trust,
            members: vec!["node_1".to_string(), "node_2".to_string()],
            roster: Roster::Standard(vec![service_definition]),
            persistence: PersistenceType::Any,
            durability: DurabilityType::NoDurability,
            routes: RouteType::Any,
            circuit_management_type: "circuit_1_type".to_string(),
        }
    }

    fn setup_splinter_state() -> SplinterState {
        let mut storage = get_storage("memory", CircuitDirectory::new).unwrap();
        let circuit_directory = storage.write().clone();
        SplinterState::new("memory".to_string(), circuit_directory)
    }

    fn filled_splinter_state() -> SplinterState {
        let service_definition_1 =
            ServiceDefinition::builder("service_1".to_string(), "type_a".to_string())
                .with_allowed_nodes(vec!["node_1".to_string()])
                .build();

        let service_definition_2 =
            ServiceDefinition::builder("service_2".to_string(), "other_type".to_string())
                .with_allowed_nodes(vec!["node_3".to_string()])
                .build();

        let circuit_1 = Circuit::builder()
            .with_id("circuit_1".into())
            .with_auth(AuthorizationType::Trust)
            .with_members(vec!["node_1".to_string(), "node_2".to_string()])
            .with_roster(vec![service_definition_1])
            .with_persistence(PersistenceType::Any)
            .with_durability(DurabilityType::NoDurability)
            .with_routes(RouteType::Any)
            .with_circuit_management_type("circuit_1_type".into())
            .build()
            .expect("Should have built a correct circuit");

        let circuit_2 = Circuit::builder()
            .with_id("circuit_2".into())
            .with_auth(AuthorizationType::Trust)
            .with_members(vec!["node_3".to_string(), "node_4".to_string()])
            .with_roster(vec![service_definition_2])
            .with_persistence(PersistenceType::Any)
            .with_durability(DurabilityType::NoDurability)
            .with_routes(RouteType::Any)
            .with_circuit_management_type("circuit_2_type".into())
            .build()
            .expect("Should have built a correct circuit");

        let mut splinter_state = setup_splinter_state();
        splinter_state
            .add_circuit("circuit_1".into(), circuit_1)
            .expect("Unable to add circuit_1");
        splinter_state
            .add_circuit("circuit_2".into(), circuit_2)
            .expect("Unable to add circuit_2");

        splinter_state
    }
}
