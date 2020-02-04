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

use std::sync::Arc;

use crate::actix_web::{web, Error as ActixError, HttpRequest, HttpResponse};
use crate::futures::Future;
use crate::rest_api::{Continuation, Method, RequestGuard};

use super::Service;

type Handler = Arc<
    dyn Fn(
            HttpRequest,
            web::Payload,
            &dyn Service,
        ) -> Box<dyn Future<Item = HttpResponse, Error = ActixError>>
        + Send
        + Sync
        + 'static,
>;

pub struct ServiceEndpoint {
    pub service_type: String,
    pub route: String,
    pub method: Method,
    pub handler: Handler,
    pub request_guards: Vec<Box<dyn ServiceRequestGuard>>,
}

/// This trait enforces that the Request guard is Clone.
pub trait ServiceRequestGuard: RequestGuard {
    fn clone_box(&self) -> Box<dyn ServiceRequestGuard>;
}

// Much of the following implementations are a bit of gymnastics to ensure that the compiler is
// happy with the types that are required elsewhere in the system.
impl<R> ServiceRequestGuard for R
where
    R: RequestGuard + Clone + 'static,
{
    fn clone_box(&self) -> Box<dyn ServiceRequestGuard> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn ServiceRequestGuard> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl RequestGuard for Box<dyn ServiceRequestGuard> {
    fn evaluate(&self, req: &HttpRequest) -> Continuation {
        (**self).evaluate(req)
    }
}
