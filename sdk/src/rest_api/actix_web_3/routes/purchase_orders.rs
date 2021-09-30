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
    resources::purchase_order::v1,
};

use super::DEFAULT_GRID_PROTOCOL_VERSION;

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryOrgId {
    pub org_id: Option<String>,
}

#[get("/purchase-order")]
pub async fn list_purchase_orders(
    store_state: web::Data<StoreState>,
    query_buyer_org_id: web::Query<QueryOrgId>,
    query_seller_org_id: web::Query<QueryOrgId>,
    query_service_id: web::Query<QueryServiceId>,
    query_paging: web::Query<QueryPaging>,
    version: ProtocolVersion,
    _: AcceptServiceIdParam,
) -> HttpResponse {
    let store = store_state.store_factory.get_grid_purchase_order_store();
    match version {
        ProtocolVersion::V1 => {
            let paging = query_paging.into_inner();
            let buyer_org_id = query_buyer_org_id.into_inner().org_id;
            let seller_org_id = query_seller_org_id.into_inner().org_id;
            let service_id = query_service_id.into_inner().service_id;
            match v1::list_purchase_orders(
                store,
                buyer_org_id,
                seller_org_id,
                service_id.as_deref(),
                paging.offset(),
                paging.limit(),
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

#[get("/purchase-order/{uuid}")]
pub async fn get_purchase_order(
    store_state: web::Data<StoreState>,
    uuid: web::Path<String>,
    query_service_id: web::Query<QueryServiceId>,
    version: ProtocolVersion,
    _: AcceptServiceIdParam,
) -> HttpResponse {
    let store = store_state.store_factory.get_grid_purchase_order_store();
    match version {
        ProtocolVersion::V1 => {
            match v1::get_purchase_order(
                store,
                uuid.into_inner(),
                query_service_id.into_inner().service_id.as_deref(),
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

#[get("/purchase-order/{uid}/version")]
pub async fn list_purchase_order_versions(
    store_state: web::Data<StoreState>,
    uid: web::Path<String>,
    query_service_id: web::Query<QueryServiceId>,
    query_paging: web::Query<QueryPaging>,
    version: ProtocolVersion,
    _: AcceptServiceIdParam,
) -> HttpResponse {
    let store = store_state.store_factory.get_grid_purchase_order_store();
    match version {
        ProtocolVersion::V1 => {
            let paging = query_paging.into_inner();
            match v1::list_purchase_order_versions(
                store,
                uid.into_inner(),
                query_service_id.into_inner().service_id.as_deref(),
                paging.offset(),
                paging.limit(),
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

#[get("/purchase-order/{uid}/version/{version_id}")]
pub async fn get_purchase_order_version(
    store_state: web::Data<StoreState>,
    uid: web::Path<String>,
    version_id: web::Path<String>,
    query_service_id: web::Query<QueryServiceId>,
    version: ProtocolVersion,
    _: AcceptServiceIdParam,
) -> HttpResponse {
    let store = store_state.store_factory.get_grid_purchase_order_store();
    match version {
        ProtocolVersion::V1 => {
            match v1::get_purchase_order_version(
                store,
                uid.into_inner(),
                &version_id.into_inner(),
                query_service_id.into_inner().service_id.as_deref(),
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

#[get("/purchase-order/{uid}/version/{version_id}/revision")]
pub async fn list_purchase_order_version_revisions(
    store_state: web::Data<StoreState>,
    uid: web::Path<String>,
    version_id: web::Path<String>,
    query_service_id: web::Query<QueryServiceId>,
    query_paging: web::Query<QueryPaging>,
    version: ProtocolVersion,
    _: AcceptServiceIdParam,
) -> HttpResponse {
    let store = store_state.store_factory.get_grid_purchase_order_store();
    match version {
        ProtocolVersion::V1 => {
            let paging = query_paging.into_inner();
            match v1::list_purchase_order_revisions(
                store,
                uid.into_inner(),
                version_id.into_inner(),
                query_service_id.into_inner().service_id.as_deref(),
                paging.offset(),
                paging.limit(),
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

#[get("/purchase-order/{uuid}/version/{version_id}/revision/{revision_number}")]
pub async fn get_purchase_order_version_revision(
    store_state: web::Data<StoreState>,
    uuid: web::Path<String>,
    version_id: web::Path<String>,
    revision_number: web::Path<i64>,
    query_service_id: web::Query<QueryServiceId>,
    version: ProtocolVersion,
    _: AcceptServiceIdParam,
) -> HttpResponse {
    let store = store_state.store_factory.get_grid_purchase_order_store();
    match version {
        ProtocolVersion::V1 => {
            match v1::get_purchase_order_revision(
                store,
                uuid.into_inner(),
                version_id.into_inner(),
                revision_number.into_inner(),
                query_service_id.into_inner().service_id.as_deref(),
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
