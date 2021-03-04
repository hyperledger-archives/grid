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

use actix_web::{dev, get, http::StatusCode, web, FromRequest, HttpRequest, HttpResponse};
use futures::future;

use crate::rest_api::{
    actix_web_3::{AcceptServiceIdParam, QueryPaging, QueryServiceId, StoreState},
    resources::agents::v1,
};

const DEFAULT_GRID_PROTOCOL_VERSION: &str = "1";

#[get("/agent/{public_key}")]
pub async fn fetch_agent(
    state: web::Data<StoreState>,
    public_key: web::Path<String>,
    query: web::Query<QueryServiceId>,
    version: ProtocolVersion,
    _: AcceptServiceIdParam,
) -> HttpResponse {
    match version {
        ProtocolVersion::V1 => {
            match v1::fetch_agent(
                state.pike_store.clone(),
                public_key.into_inner(),
                query.into_inner().service_id.as_deref(),
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

#[get("/agent")]
pub async fn list_agents(
    state: web::Data<StoreState>,
    query_service_id: web::Query<QueryServiceId>,
    query_paging: web::Query<QueryPaging>,
    version: ProtocolVersion,
    _: AcceptServiceIdParam,
) -> HttpResponse {
    match version {
        ProtocolVersion::V1 => {
            let paging = query_paging.into_inner();
            let service_id = query_service_id.into_inner().service_id;
            match v1::list_agents(
                state.pike_store.clone(),
                service_id.as_deref(),
                paging.offset(),
                paging.limit(),
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
