// Copyright 2018-2021 Cargill Incorporated
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

use actix_web::{dev, http::StatusCode, web, FromRequest, HttpRequest, HttpResponse, Result};
use futures::future;

use crate::rest_api::resources::error::ErrorResponse;

use super::Endpoint;

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryServiceId {
    pub service_id: Option<String>,
    pub wait: Option<u64>,
}

pub struct AcceptServiceIdParam;

impl FromRequest for AcceptServiceIdParam {
    type Error = HttpResponse;
    type Future = future::Ready<Result<Self, Self::Error>>;
    type Config = ();

    fn from_request(req: &HttpRequest, _: &mut dev::Payload) -> Self::Future {
        let endpoint: Endpoint = if let Some(endpoint) = req.app_data::<Endpoint>() {
            endpoint.clone()
        } else {
            return future::err(
                HttpResponse::build(
                    StatusCode::from_u16(500).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                )
                .json(ErrorResponse::new(500, "App state not found")),
            );
        };

        let service_id =
            if let Ok(query) = web::Query::<QueryServiceId>::from_query(req.query_string()) {
                query.service_id.clone()
            } else {
                return future::err(
                    HttpResponse::build(
                        StatusCode::from_u16(400).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                    )
                    .json(ErrorResponse::new(400, "Malformed query param")),
                );
            };

        if service_id.is_some() && endpoint.is_sawtooth() {
            return future::err(
                HttpResponse::build(
                    StatusCode::from_u16(400).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                )
                .json(ErrorResponse::new(
                    400,
                    "Service ID present, but grid is running in sawtooth mode",
                )),
            );
        } else if service_id.is_none() && !endpoint.is_sawtooth() {
            return future::err(
                HttpResponse::build(
                    StatusCode::from_u16(400).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                )
                .json(ErrorResponse::new(
                    400,
                    "Service ID is not present, but grid is running in splinter mode",
                )),
            );
        }

        future::ok(AcceptServiceIdParam)
    }
}
