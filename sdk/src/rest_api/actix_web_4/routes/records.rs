// Copyright 2018-2022 Cargill Incorporated
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

use actix_web_4::{dev, http::StatusCode, web, Error, FromRequest, HttpRequest, HttpResponse};
use futures::future;
use futures_util::future::{FutureExt, LocalBoxFuture};

use crate::rest_api::{
    actix_web_4::{request, AcceptServiceIdParam, QueryPaging, QueryServiceId, StoreState},
    resources::track_and_trace::v1,
};

use super::DEFAULT_GRID_PROTOCOL_VERSION;

pub async fn get_record(
    store_state: web::Data<StoreState>,
    record_id: web::Path<String>,
    query: web::Query<QueryServiceId>,
    version: ProtocolVersion,
    _: AcceptServiceIdParam,
) -> HttpResponse {
    let store = store_state.store_factory.get_grid_track_and_trace_store();
    match version {
        ProtocolVersion::V1 => {
            match v1::get_record(
                store,
                record_id.into_inner(),
                query.into_inner().service_id.as_deref(),
            ) {
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

pub async fn list_records(
    req: HttpRequest,
    store_state: web::Data<StoreState>,
    query_service_id: web::Query<QueryServiceId>,
    query_paging: web::Query<QueryPaging>,
    version: ProtocolVersion,
    _: AcceptServiceIdParam,
) -> HttpResponse {
    let store = store_state.store_factory.get_grid_track_and_trace_store();
    match version {
        ProtocolVersion::V1 => {
            let paging = query_paging.into_inner();
            let service_id = query_service_id.into_inner().service_id;
            match request::get_base_url(&req).and_then(|url| {
                v1::list_records(
                    url,
                    store,
                    service_id.as_deref(),
                    paging.offset(),
                    paging.limit(),
                )
            }) {
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

pub async fn get_record_property_name(
    store_state: web::Data<StoreState>,
    path_variables: web::Path<(String, String)>,
    query: web::Query<QueryServiceId>,
    version: ProtocolVersion,
    _: AcceptServiceIdParam,
) -> HttpResponse {
    let store = store_state.store_factory.get_grid_track_and_trace_store();
    match version {
        ProtocolVersion::V1 => {
            let (record_id, property_name) = path_variables.into_inner();
            match v1::get_record_property(
                store,
                record_id,
                property_name,
                query.into_inner().service_id.as_deref(),
            ) {
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
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

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
            "1" => future::ok(ProtocolVersion::V1).boxed_local(),
            _ => future::ok(ProtocolVersion::V1).boxed_local(),
        }
    }
}
