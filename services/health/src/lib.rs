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

#[macro_use]
extern crate log;

use splinter::{
    actix_web::HttpResponse,
    futures::IntoFuture,
    rest_api::{Method, Request, Resource, RestResourceProvider},
    service::{
        error::{ServiceDestroyError, ServiceError, ServiceStartError, ServiceStopError},
        Service, ServiceMessageContext, ServiceNetworkRegistry,
    },
};
use std::any::Any;

pub struct HealthService {
    service_id: String,
}

impl HealthService {
    pub fn new(node_id: &str) -> Self {
        Self {
            service_id: format!("health::{}", node_id),
        }
    }
}

impl Service for HealthService {
    fn service_id(&self) -> &str {
        &self.service_id
    }

    fn service_type(&self) -> &str {
        "health"
    }

    fn start(
        &mut self,
        service_registry: &dyn ServiceNetworkRegistry,
    ) -> Result<(), ServiceStartError> {
        info!("Starting health service");
        Ok(())
    }

    fn stop(
        &mut self,
        service_registry: &dyn ServiceNetworkRegistry,
    ) -> Result<(), ServiceStopError> {
        info!("Stopping health service");
        Ok(())
    }

    fn destroy(self: Box<Self>) -> Result<(), ServiceDestroyError> {
        info!("Destroying health service");
        Ok(())
    }

    fn handle_message(
        &self,
        message_bytes: &[u8],
        _: &ServiceMessageContext,
    ) -> Result<(), ServiceError> {
        info!("Handling a messge");
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl RestResourceProvider for HealthService {
    fn resources(&self) -> Vec<Resource> {
        vec![make_status_resource()]
    }
}

fn make_status_resource() -> Resource {
    Resource::new(Method::Get, "/health/status", move |_, _| {
        Box::new(HttpResponse::Ok().finish().into_future())
    })
}
