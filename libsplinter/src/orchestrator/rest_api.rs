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

use crate::actix_web::HttpResponse;
use crate::futures::IntoFuture;
use crate::rest_api::{Resource, RestResourceProvider};

use super::ServiceOrchestrator;

impl RestResourceProvider for ServiceOrchestrator {
    fn resources(&self) -> Vec<Resource> {
        // Get endpoints for all factories
        self.service_factories
            .iter()
            .fold(vec![], |mut acc, factory| {
                // Get all endpoints for the factory
                let mut resources = factory
                    .get_rest_endpoints()
                    .into_iter()
                    .map(|endpoint| {
                        let route = format!(
                            "/{}/{{circuit}}/{{service_id}}{}",
                            endpoint.service_type, endpoint.route
                        );
                        let services = self.services.clone();

                        let mut resource_builder = Resource::build(&route);

                        for request_guard in endpoint.request_guards.into_iter() {
                            resource_builder = resource_builder.add_request_guard(request_guard);
                        }

                        let service_type = endpoint.service_type;
                        let handler = endpoint.handler;
                        resource_builder.add_method(endpoint.method, move |request, payload| {
                            let circuit = request
                                .match_info()
                                .get("circuit")
                                .unwrap_or("")
                                .to_string();
                            let service_id = request
                                .match_info()
                                .get("service_id")
                                .unwrap_or("")
                                .to_string();

                            let services = match services.lock() {
                                Ok(s) => s,
                                Err(err) => {
                                    error!("Orchestrator's service lock is poisoned: {}", err);
                                    return Box::new(
                                        HttpResponse::InternalServerError()
                                            .json(json!({
                                                "message": "An internal error occurred"
                                            }))
                                            .into_future(),
                                    )
                                    .into_future();
                                }
                            };

                            let service =
                                match services.iter().find_map(|(service_def, managed_service)| {
                                    if service_def.service_type == service_type
                                        && service_def.circuit == circuit
                                        && service_def.service_id == service_id
                                    {
                                        Some(&*managed_service.service)
                                    } else {
                                        None
                                    }
                                }) {
                                    Some(s) => s,
                                    None => {
                                        return Box::new(
                                            HttpResponse::NotFound()
                                                .json(json!({
                                                    "message":
                                                        format!(
                                                            "{} service {} on circuit {} not found",
                                                            service_type, service_id, circuit
                                                        )
                                                }))
                                                .into_future(),
                                        )
                                        .into_future();
                                    }
                                };

                            handler(request, payload, service)
                        })
                    })
                    .collect::<Vec<_>>();

                acc.append(&mut resources);
                acc
            })
    }
}
