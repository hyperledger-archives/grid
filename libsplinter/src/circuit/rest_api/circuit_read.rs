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
use std::sync::{Arc, RwLock};

use crate::actix_web::{error::BlockingError, web, Error, HttpRequest, HttpResponse};
use crate::circuit::{
    AuthorizationType, DurabilityType, PersistenceType, Roster, RouteType, SplinterState,
};
use crate::futures::{future::IntoFuture, Future};
use crate::rest_api::{
    paging::{get_response_paging_info, Paging, DEFAULT_LIMIT, DEFAULT_OFFSET},
    Method, Resource,
};

use super::CircuitRouteError;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CircuitResponse {
    id: String,
    auth: AuthorizationType,
    members: Vec<String>,
    roster: Roster,
    persistence: PersistenceType,
    durability: DurabilityType,
    routes: RouteType,
    circuit_management_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ListCircuitsResponse {
    data: Vec<CircuitResponse>,
    paging: Paging,
}

pub fn make_fetch_circuit_resource(state: Arc<RwLock<SplinterState>>) -> Resource {
    Resource::build("/circuits/{circuit_id}").add_method(Method::Get, move |r, _| {
        fetch_circuit(r, web::Data::new(state.clone()))
    })
}

pub fn make_list_circuits_resource(state: Arc<RwLock<SplinterState>>) -> Resource {
    Resource::build("/circuits").add_method(Method::Get, move |r, _| {
        list_circuits(r, web::Data::new(state.clone()))
    })
}

fn fetch_circuit(
    request: HttpRequest,
    state: web::Data<Arc<RwLock<SplinterState>>>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let circuit_id = request
        .match_info()
        .get("circuit_id")
        .unwrap_or("")
        .to_string();
    Box::new(
        web::block(move || {
            let circuit = {
                let state = state.read().map_err(|_| CircuitRouteError::PoisonedLock)?;
                state.circuit(&circuit_id).cloned()
            };
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
                Err(CircuitRouteError::NotFound(format!(
                    "Unable to find circuit: {}",
                    circuit_id
                )))
            }
        })
        .then(|res| match res {
            Ok(circuit) => Ok(HttpResponse::Ok().json(circuit)),
            Err(err) => match err {
                BlockingError::Error(err) => match err {
                    CircuitRouteError::PoisonedLock => {
                        error!("{}", err);
                        Ok(HttpResponse::InternalServerError().into())
                    }
                    CircuitRouteError::NotFound(err) => Ok(HttpResponse::NotFound().json(err)),
                },
                _ => Ok(HttpResponse::InternalServerError().into()),
            },
        }),
    )
}

fn list_circuits(
    req: HttpRequest,
    state: web::Data<Arc<RwLock<SplinterState>>>,
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

    Box::new(query_list_circuits(
        state,
        link,
        filters,
        Some(offset),
        Some(limit),
    ))
}

fn query_list_circuits(
    state: web::Data<Arc<RwLock<SplinterState>>>,
    link: String,
    filters: Option<String>,
    offset: Option<usize>,
    limit: Option<usize>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    web::block(move || {
        let circuits = state
            .read()
            .map_err(|_| CircuitRouteError::PoisonedLock)?
            .circuits()
            .clone();
        let offset_value = offset.unwrap_or(0);
        let limit_value = limit.unwrap_or_else(|| circuits.len());
        if !circuits.is_empty() {
            if let Some(filter) = filters {
                let circuits_data: Vec<CircuitResponse> = circuits
                    .into_iter()
                    .filter(|(_, circuit)| circuit.members().contains(&filter))
                    .skip(offset_value)
                    .take(limit_value)
                    .map(|(circuit_id, circuit)| CircuitResponse {
                        id: circuit_id,
                        auth: circuit.auth().clone(),
                        members: circuit.members().to_vec(),
                        roster: circuit.roster().clone(),
                        persistence: circuit.persistence().clone(),
                        durability: circuit.durability().clone(),
                        routes: circuit.routes().clone(),
                        circuit_management_type: circuit.circuit_management_type().to_string(),
                    })
                    .collect();
                let total_count = circuits_data.len();
                Ok((circuits_data, link, limit, offset, total_count))
            } else {
                let circuits_data: Vec<CircuitResponse> = circuits
                    .into_iter()
                    .skip(offset_value)
                    .take(limit_value)
                    .map(|(circuit_id, circuit)| CircuitResponse {
                        id: circuit_id,
                        auth: circuit.auth().clone(),
                        members: circuit.members().to_vec(),
                        roster: circuit.roster().clone(),
                        persistence: circuit.persistence().clone(),
                        durability: circuit.durability().clone(),
                        routes: circuit.routes().clone(),
                        circuit_management_type: circuit.circuit_management_type().to_string(),
                    })
                    .collect();
                let total_count = circuits_data.len();
                Ok((circuits_data, link, limit, offset, total_count))
            }
        } else {
            Ok((vec![], link, limit, offset, circuits.len()))
        }
    })
    .then(|res| match res {
        Ok((circuits, link, limit, offset, total_count)) => {
            Ok(HttpResponse::Ok().json(ListCircuitsResponse {
                data: circuits,
                paging: get_response_paging_info(limit, offset, &link, total_count),
            }))
        }
        Err(err) => match err {
            BlockingError::Error(err) => match err {
                CircuitRouteError::PoisonedLock => {
                    error!("{}", err);
                    Ok(HttpResponse::InternalServerError().into())
                }
                CircuitRouteError::NotFound(err) => Ok(HttpResponse::NotFound().json(err)),
            },
            _ => Ok(HttpResponse::InternalServerError().into()),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::actix_web::{
        http::{header, StatusCode},
        test, web, App,
    };
    use crate::circuit::{
        directory::CircuitDirectory, AuthorizationType, Circuit, DurabilityType, PersistenceType,
        Roster, RouteType, ServiceDefinition,
    };
    use crate::storage::get_storage;

    #[test]
    /// Tests a GET /circuit/{identity} request returns the expected circuit.
    fn test_fetch_circuit_ok() {
        let splinter_state = filled_splinter_state();

        let mut app = test::init_service(App::new().data(splinter_state.clone()).service(
            web::resource("/circuits/{circuit_id}").route(web::get().to_async(fetch_circuit)),
        ));

        let req = test::TestRequest::get()
            .uri(&format!("/circuits/{}", get_circuit_1().id))
            .to_request();

        let resp = test::call_service(&mut app, req);

        assert_eq!(resp.status(), StatusCode::OK);
        let circuit: CircuitResponse = serde_yaml::from_slice(&test::read_body(resp)).unwrap();
        assert_eq!(circuit, get_circuit_1())
    }

    #[test]
    /// Tests a GET /circuits/{identity} request returns NotFound when an invalid identity is
    /// passed
    fn test_fetch_circuit_not_found() {
        let splinter_state = filled_splinter_state();
        let mut app = test::init_service(App::new().data(splinter_state.clone()).service(
            web::resource("/circuits/{circuit_id}").route(web::get().to_async(fetch_circuit)),
        ));

        let req = test::TestRequest::get()
            .uri("/circuit/Circuit-not-valid")
            .to_request();

        let resp = test::call_service(&mut app, req);

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    /// Tests a GET /circuits request with no filters returns the expected circuits.
    fn test_list_circuits_ok() {
        let splinter_state = filled_splinter_state();

        let mut app = test::init_service(
            App::new()
                .data(splinter_state.clone())
                .service(web::resource("/circuits").route(web::get().to_async(list_circuits))),
        );

        let req = test::TestRequest::get().uri("/circuits").to_request();

        let resp = test::call_service(&mut app, req);

        assert_eq!(resp.status(), StatusCode::OK);
        let circuits: ListCircuitsResponse =
            serde_yaml::from_slice(&test::read_body(resp)).unwrap();
        assert_eq!(circuits.data, vec![get_circuit_1(), get_circuit_2()]);
        assert_eq!(
            circuits.paging,
            create_test_paging_response(0, 100, 0, 0, 0, 2, "/circuits?")
        )
    }

    #[test]
    /// Tests a GET /circuits request with filter returns the expected circuit.
    fn test_list_circuit_with_filters_ok() {
        let splinter_state = filled_splinter_state();

        let mut app = test::init_service(
            App::new()
                .data(splinter_state.clone())
                .service(web::resource("/circuits").route(web::get().to_async(list_circuits))),
        );

        let req = test::TestRequest::get()
            .uri(&format!("/circuits?filter={}", "node_1"))
            .header(header::CONTENT_TYPE, "application/json")
            .to_request();

        let resp = test::call_service(&mut app, req);

        assert_eq!(resp.status(), StatusCode::OK);
        let circuits: ListCircuitsResponse =
            serde_yaml::from_slice(&test::read_body(resp)).unwrap();
        assert_eq!(circuits.data, vec![get_circuit_1()]);
        let link = format!("/circuits?filter={}&", "node_1");
        assert_eq!(
            circuits.paging,
            create_test_paging_response(0, 100, 0, 0, 0, 1, &link)
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

    fn get_circuit_2() -> CircuitResponse {
        let service_definition =
            ServiceDefinition::builder("service_2".to_string(), "other_type".to_string())
                .with_allowed_nodes(vec!["node_3".to_string()])
                .build();
        CircuitResponse {
            id: "circuit_2".to_string(),
            auth: AuthorizationType::Trust,
            members: vec!["node_3".to_string(), "node_4".to_string()],
            roster: Roster::Standard(vec![service_definition]),
            persistence: PersistenceType::Any,
            durability: DurabilityType::NoDurability,
            routes: RouteType::Any,
            circuit_management_type: "circuit_2_type".to_string(),
        }
    }

    fn setup_splinter_state() -> Arc<RwLock<SplinterState>> {
        let mut storage = get_storage("memory", CircuitDirectory::new).unwrap();
        let circuit_directory = storage.write().clone();
        let state = Arc::new(RwLock::new(SplinterState::new(
            "memory".to_string(),
            circuit_directory,
        )));
        state
    }

    fn filled_splinter_state() -> Arc<RwLock<SplinterState>> {
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

        let splinter_state = setup_splinter_state();
        splinter_state
            .write()
            .expect("SplinterState lock was poisoned")
            .add_circuit("circuit_1".into(), circuit_1)
            .expect("Unable to add circuit_1");
        splinter_state
            .write()
            .expect("SplinterState lock was poisoned")
            .add_circuit("circuit_2".into(), circuit_2)
            .expect("Unable to add circuit_2");

        splinter_state
    }
}
