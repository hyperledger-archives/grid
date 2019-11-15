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

use std::sync::{Arc, Mutex};

use crate::actix_web::HttpResponse;
use crate::futures::{Future, IntoFuture};
use crate::protos::admin::CircuitManagementPayload;
use crate::rest_api::{into_protobuf, Method, Request, Resource, RestResourceProvider};
use crate::service::ServiceError;

use super::service::{shared::AdminServiceShared, AdminService};

impl RestResourceProvider for AdminService {
    fn resources(&self) -> Vec<Resource> {
        vec![
            make_application_handler_registration_route(self.admin_service_shared.clone()),
            make_submit_route(self.admin_service_shared.clone()),
        ]
    }
}

fn make_submit_route(shared: Arc<Mutex<AdminServiceShared>>) -> Resource {
    Resource::build("/admin/submit").add_method(Method::Post, move |_, payload| {
        let shared = shared.clone();
        Box::new(
            into_protobuf::<CircuitManagementPayload>(payload).and_then(move |payload| {
                let mut shared = match shared.lock() {
                    Ok(shared) => shared,
                    Err(err) => {
                        debug!("Lock poisoned: {}", err);
                        return HttpResponse::InternalServerError().finish().into_future();
                    }
                };

                match shared.submit(payload) {
                    Ok(()) => HttpResponse::Accepted().finish().into_future(),
                    Err(ServiceError::UnableToHandleMessage(err)) => HttpResponse::BadRequest()
                        .json(json!({
                            "message": format!("Unable to handle message: {}", err)
                        }))
                        .into_future(),
                    Err(ServiceError::InvalidMessageFormat(err)) => HttpResponse::BadRequest()
                        .json(json!({
                            "message": format!("Failed to parse payload: {}", err)
                        }))
                        .into_future(),
                    Err(_) => HttpResponse::InternalServerError().finish().into_future(),
                }
            }),
        )
    })
}

fn make_application_handler_registration_route(shared: Arc<Mutex<AdminServiceShared>>) -> Resource {
    Resource::build("/ws/admin/register/{type}").add_method(Method::Get, move |request, payload| {
        let circuit_management_type = if let Some(t) = request.match_info().get("type") {
            t.to_string()
        } else {
            return Box::new(HttpResponse::BadRequest().finish().into_future());
        };

        let unlocked_shared = shared.lock();

        match unlocked_shared {
            Ok(mut shared) => {
                let request = Request::from((request, payload));
                debug!("circuit management type {}", circuit_management_type);
                match shared.add_subscriber(circuit_management_type, request) {
                    Ok(res) => {
                        debug!("Websocket response: {:?}", res);
                        Box::new(res.into_future())
                    }
                    Err(err) => {
                        debug!("Failed to create websocket: {:?}", err);
                        Box::new(HttpResponse::InternalServerError().finish().into_future())
                    }
                }
            }
            Err(err) => {
                debug!("Failed to add socket sender: {:?}", err);
                Box::new(HttpResponse::InternalServerError().finish().into_future())
            }
        }
    })
}
