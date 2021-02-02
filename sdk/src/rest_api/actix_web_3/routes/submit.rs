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

use actix_web::{dev, http::StatusCode, post, web, FromRequest, HttpRequest, HttpResponse};
use futures_util::future::{FutureExt, LocalBoxFuture};

use crate::rest_api::actix_web_3::State;
use crate::rest_api::resources::{
    error::ErrorResponse,
    submit::v1::{submit_batches, SubmitBatchRequest},
};

const DEFAULT_GRID_PROTOCOL_VERSION: &str = "1";

#[post("/submit")]
async fn submit(state: web::Data<State>, version: ProtocolVersion) -> HttpResponse {
    match version {
        ProtocolVersion::V1(payload) => {
            match submit_batches(&state.key_file_name, state.batch_store.clone(), payload).await {
                Ok(res) => HttpResponse::Accepted().json(res),
                Err(err) => HttpResponse::build(
                    StatusCode::from_u16(err.status_code())
                        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                )
                .json(err),
            }
        }
    }
}

enum ProtocolVersion {
    V1(SubmitBatchRequest),
}

impl FromRequest for ProtocolVersion {
    type Error = HttpResponse;
    type Future = LocalBoxFuture<'static, Result<Self, HttpResponse>>;
    type Config = ();

    fn from_request(req: &HttpRequest, payload: &mut dev::Payload) -> Self::Future {
        let protocol_version = match req
            .headers()
            .get("GridProtocolVersion")
            .map(|ver| ver.to_str().map(String::from))
        {
            Some(Ok(ver)) => ver,
            Some(Err(err)) => {
                error!(
                    "Failed to parse version using default version {}: {}",
                    DEFAULT_GRID_PROTOCOL_VERSION, err
                );
                DEFAULT_GRID_PROTOCOL_VERSION.to_string()
            }
            None => {
                warn!(
                    "No Protocol version specified, defaulting to version {}",
                    DEFAULT_GRID_PROTOCOL_VERSION
                );
                DEFAULT_GRID_PROTOCOL_VERSION.to_string()
            }
        };

        match protocol_version.as_str() {
            "1" => dev::JsonBody::new(req, payload, None)
                .map(|result| match result {
                    Ok(data) => Ok(ProtocolVersion::V1(data)),
                    Err(err) => Err(HttpResponse::build(
                        StatusCode::from_u16(400).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                    )
                    .json(ErrorResponse::new(400, &format!("{}", err)))),
                })
                .boxed_local(),
            _ => dev::JsonBody::new(req, payload, None)
                .map(|res| match res {
                    Ok(data) => Ok(ProtocolVersion::V1(data)),
                    Err(err) => Err(HttpResponse::build(
                        StatusCode::from_u16(400).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                    )
                    .json(ErrorResponse::new(400, &format!("{}", err)))),
                })
                .boxed_local(),
        }
    }
}
