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

use actix_web::{dev, get, http::StatusCode, post, web, FromRequest, HttpRequest, HttpResponse};
use futures::future;
use futures_util::StreamExt;

use crate::rest_api::{
    actix_web_3::{AcceptServiceIdParam, BatchSubmitterState, QueryServiceId},
    resources::{batches::v1, error::ErrorResponse},
};

const DEFAULT_GRID_PROTOCOL_VERSION: &str = "1";

#[post("/batches")]
pub async fn submit_batches(
    req: HttpRequest,
    mut body: web::Payload,
    state: web::Data<BatchSubmitterState>,
    query_service_id: web::Query<QueryServiceId>,
    version: ProtocolVersion,
    _: AcceptServiceIdParam,
) -> HttpResponse {
    match version {
        ProtocolVersion::V1 => {
            let service_id = query_service_id.into_inner().service_id;

            let response_url = match req.url_for_static("fetch_batch_statuses") {
                Ok(response_url) => response_url,
                Err(err) => {
                    let json = ErrorResponse::internal_error(Box::new(err));
                    return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).json(json);
                }
            };

            let mut bytes = web::BytesMut::new();
            while let Some(item) = body.next().await {
                let item = match item {
                    Ok(item) => item,
                    Err(err) => {
                        let json = ErrorResponse::internal_error(Box::new(err));
                        return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).json(json);
                    }
                };
                bytes.extend_from_slice(&item);
            }

            match v1::submit_batches(
                response_url,
                state.batch_submitter.clone(),
                &*bytes,
                service_id,
            )
            .await
            {
                Ok(res) => HttpResponse::Ok().json(res),
                Err(err) => HttpResponse::build(
                    StatusCode::from_u16(err.status_code())
                        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                )
                .json(err),
            }
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct QueryParams {
    pub id: Option<String>,
    pub wait: Option<String>,
    pub service_id: Option<String>,
}

#[get("/batch_statuses")]
pub async fn fetch_batch_statuses(
    req: HttpRequest,
    state: web::Data<BatchSubmitterState>,
    query: web::Query<QueryParams>,
    version: ProtocolVersion,
    _: AcceptServiceIdParam,
) -> HttpResponse {
    match version {
        ProtocolVersion::V1 => {
            let query = query.into_inner();
            let service_id = query.service_id;
            let wait = query.wait;
            let id = query.id.unwrap_or_else(|| "".to_string());

            let response_url = match req.url_for_static("fetch_batch_statuses") {
                Ok(url) => format!("{}?{}", url, req.query_string()),
                Err(err) => {
                    error!("{}", err);
                    let json = ErrorResponse::internal_error(Box::new(err));
                    return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).json(json);
                }
            };

            match v1::fetch_batch_statuses(
                response_url,
                state.batch_submitter.clone(),
                id,
                wait,
                service_id,
            )
            .await
            {
                Ok(res) => HttpResponse::Ok().json(res),
                Err(err) => HttpResponse::build(
                    StatusCode::from_u16(err.status_code())
                        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                )
                .json(err),
            }
        }
    }
}

pub enum ProtocolVersion {
    V1,
}

impl FromRequest for ProtocolVersion {
    type Error = HttpResponse;
    type Future = future::Ready<Result<Self, Self::Error>>;
    type Config = ();

    fn from_request(req: &HttpRequest, _: &mut dev::Payload) -> Self::Future {
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
            "1" => future::ok(ProtocolVersion::V1),
            _ => future::ok(ProtocolVersion::V1),
        }
    }
}
