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

use actix_web::{dev, get, web, FromRequest, HttpRequest, HttpResponse};
use futures::future;

use crate::rest_api::actix_web_3::{AcceptServiceIdParam, QueryPaging, QueryServiceId, StoreState};

const DEFAULT_GRID_PROTOCOL_VERSION: &str = "1";

#[get("/purchase-order")]
pub async fn list_purchase_orders(
    _state: web::Data<StoreState>,
    _query_service_id: web::Query<QueryServiceId>,
    _query_paging: web::Query<QueryPaging>,
    version: ProtocolVersion,
    _: AcceptServiceIdParam,
) -> HttpResponse {
    match version {
        ProtocolVersion::V1 => {
            unimplemented!()
        }
    }
}

#[get("/purchase-order/{uuid}")]
pub async fn fetch_purchase_order(
    _state: web::Data<StoreState>,
    _uuid: web::Path<String>,
    _query: web::Query<QueryServiceId>,
    version: ProtocolVersion,
    _: AcceptServiceIdParam,
) -> HttpResponse {
    match version {
        ProtocolVersion::V1 => {
            unimplemented!()
        }
    }
}

#[get("/purchase-order/{uuid}/versions")]
pub async fn list_purchase_order_versions(
    _state: web::Data<StoreState>,
    _uuid: web::Path<String>,
    _query: web::Query<QueryServiceId>,
    version: ProtocolVersion,
    _: AcceptServiceIdParam,
) -> HttpResponse {
    match version {
        ProtocolVersion::V1 => {
            unimplemented!()
        }
    }
}

#[get("/purchase-order/{uuid}/versions/{version_id}")]
pub async fn fetch_purchase_order_version(
    _state: web::Data<StoreState>,
    _uuid: web::Path<String>,
    _version_id: web::Path<String>,
    _query: web::Query<QueryServiceId>,
    version: ProtocolVersion,
    _: AcceptServiceIdParam,
) -> HttpResponse {
    match version {
        ProtocolVersion::V1 => {
            unimplemented!()
        }
    }
}

#[get("/purchase-order/{uuid}/versions/{version_id}/revisions")]
pub async fn list_purchase_order_version_revisions(
    _state: web::Data<StoreState>,
    _uuid: web::Path<String>,
    _version_id: web::Path<String>,
    _query: web::Query<QueryServiceId>,
    version: ProtocolVersion,
    _: AcceptServiceIdParam,
) -> HttpResponse {
    match version {
        ProtocolVersion::V1 => {
            unimplemented!()
        }
    }
}

#[get("/purchase-order/{uuid}/versions/{version_id}/revisions/{revision_number}")]
pub async fn fetch_purchase_order_version_revision(
    _state: web::Data<StoreState>,
    _uuid: web::Path<String>,
    _version_id: web::Path<String>,
    _revision_number: web::Path<u64>,
    _query: web::Query<QueryServiceId>,
    version: ProtocolVersion,
    _: AcceptServiceIdParam,
) -> HttpResponse {
    match version {
        ProtocolVersion::V1 => {
            unimplemented!()
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
