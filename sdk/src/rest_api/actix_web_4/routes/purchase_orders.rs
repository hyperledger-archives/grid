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

//! Rest endpoint implementations for Grid Purchase Order, powered by Actix Web 3.

use actix_web_4::{dev, http::StatusCode, web, Error, FromRequest, HttpRequest, HttpResponse};
use futures::future;
use futures_util::future::{FutureExt, LocalBoxFuture};

use crate::rest_api::{
    actix_web_4::{request, AcceptServiceIdParam, QueryPaging, QueryServiceId, StoreState},
    resources::purchase_order::v1,
};

use crate::purchase_order::store::{ListPOFilters, ListVersionFilters};

use super::DEFAULT_GRID_PROTOCOL_VERSION;

/// Represents a `version_id` passed to the endpoint in the query string
#[derive(Debug, Serialize, Deserialize)]
pub struct QueryVersionId {
    pub version_id: Option<String>,
}

/// Represents a `revision_number` passed to the endpoint in the query string
#[derive(Debug, Serialize, Deserialize)]
pub struct QueryRevisionNumber {
    pub revision_number: Option<i64>,
}

/// Provides the ability to list purchase orders, with filters and paging
///
/// # Arguments
///
/// `req` - Request submitted to the endpoint
/// `store_state` - Provides a `store_factory` to access Grid's stores
/// `query_filters` - Optional filters that may be applied to the purchase orders listed.
///  Purchase orders may be filtered using `buyer_org_id`, `seller_org_id`, `has_accepted_version`
///  `is_open`, and `alternate_ids`.
/// `query_service_id` - Optional service ID provided in the query string
/// `query_paging` - Optional paging options, including `offset` and `limit`
/// `version` - Determines the type of response, corresponding to the versions of the rest API
pub async fn list_purchase_orders(
    req: HttpRequest,
    store_state: web::Data<StoreState>,
    query_filters: web::Query<ListPOFilters>,
    query_service_id: web::Query<QueryServiceId>,
    query_paging: web::Query<QueryPaging>,
    version: ProtocolVersion,
    _: AcceptServiceIdParam,
) -> HttpResponse {
    let store = store_state.store_factory.get_grid_purchase_order_store();
    match version {
        ProtocolVersion::V1 => {
            let filters = query_filters.into_inner();
            let paging = query_paging.into_inner();
            let service_id = query_service_id.into_inner().service_id;
            match request::get_base_url(&req).and_then(|url| {
                v1::list_purchase_orders(
                    url,
                    store,
                    filters,
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

/// Provides the ability to fetch a purchase order
///
/// # Arguments
///
/// `store_state` - Provides a `store_factory` to access Grid's stores
/// `uid` - The unique identifier of the purchase order to list versions from
/// `version_id` - Optional version ID, specifies the version to return
/// `revision_number` - Optional revision number, specifies the revision to return
/// `query_service_id` - Optional service ID provided in the query string
/// `version` - Determines the type of response, corresponding to the versions of the rest API
pub async fn get_purchase_order(
    store_state: web::Data<StoreState>,
    uid: web::Path<String>,
    version_id: web::Query<QueryVersionId>,
    revision_number: web::Query<QueryRevisionNumber>,
    query_service_id: web::Query<QueryServiceId>,
    version: ProtocolVersion,
    _: AcceptServiceIdParam,
) -> HttpResponse {
    let store = store_state.store_factory.get_grid_purchase_order_store();
    match version {
        ProtocolVersion::V1 => {
            match v1::get_purchase_order(
                store,
                uid.into_inner(),
                version_id.into_inner().version_id.as_deref(),
                revision_number.into_inner().revision_number,
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

/// Provides the ability to list purchase order versions, with filters and paging
///
/// # Arguments
///
/// `req` - Request submitted to the endpoint
/// `store_state` - Provides a `store_factory` to access Grid's stores
/// `uid` - The unique identifier of the purchase order to list versions from
/// `query_filters` - Optional filters that may be applied to the versions listed from the store.
///  Versions may be filtered using `is_accepted` and `is_draft`.
/// `query_service_id` - Optional service ID provided in the query string
/// `query_paging` - Optional paging options, including `offset` and `limit`
/// `version` - Determines the type of response, corresponding to the versions of the rest API
pub async fn list_purchase_order_versions(
    req: HttpRequest,
    store_state: web::Data<StoreState>,
    uid: web::Path<String>,
    query_filters: web::Query<ListVersionFilters>,
    query_service_id: web::Query<QueryServiceId>,
    query_paging: web::Query<QueryPaging>,
    version: ProtocolVersion,
) -> HttpResponse {
    let store = store_state.store_factory.get_grid_purchase_order_store();
    let filters = query_filters.into_inner();
    match version {
        ProtocolVersion::V1 => {
            let paging = query_paging.into_inner();
            match request::get_base_url(&req).and_then(|url| {
                v1::list_purchase_order_versions(
                    url,
                    store,
                    uid.into_inner(),
                    filters,
                    query_service_id.into_inner().service_id.as_deref(),
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

/// Provides the ability to fetch a purchase order version
///
/// # Arguments
///
/// `store_state` - Provides a `store_factory` to access Grid's stores
/// `path` - Used to retrieve the purchase order UID and version ID from the request's path
/// `query_service_id` - Optional service ID provided in the query string
/// `version` - Determines the type of response, corresponding to the versions of the rest API
pub async fn get_purchase_order_version(
    store_state: web::Data<StoreState>,
    path: web::Path<(String, String)>,
    query_service_id: web::Query<QueryServiceId>,
    version: ProtocolVersion,
    _: AcceptServiceIdParam,
) -> HttpResponse {
    let store = store_state.store_factory.get_grid_purchase_order_store();
    let (uid, version_id) = path.into_inner();
    match version {
        ProtocolVersion::V1 => {
            match v1::get_purchase_order_version(
                store,
                uid,
                &version_id,
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

/// Provides the ability to list a purchase order version's revisions
///
/// # Arguments
///
/// `req` - Request submitted to the endpoint
/// `store_state` - Provides a `store_factory` to access Grid's stores
/// `path` - Used to retrieve the purchase order UID and version ID from the request's path
/// `query_service_id` - Optional service ID provided in the query string
/// `query_paging` - Optional paging options, including `offset` and `limit`
/// `version` - Determines the type of response, corresponding to the versions of the rest API
pub async fn list_purchase_order_version_revisions(
    req: HttpRequest,
    store_state: web::Data<StoreState>,
    path: web::Path<(String, String)>,
    query_service_id: web::Query<QueryServiceId>,
    query_paging: web::Query<QueryPaging>,
    version: ProtocolVersion,
    _: AcceptServiceIdParam,
) -> HttpResponse {
    let store = store_state.store_factory.get_grid_purchase_order_store();
    let (uid, version_id) = path.into_inner();
    match version {
        ProtocolVersion::V1 => {
            let paging = query_paging.into_inner();
            match request::get_base_url(&req).and_then(|url| {
                v1::list_purchase_order_revisions(
                    url,
                    store,
                    uid,
                    version_id,
                    query_service_id.into_inner().service_id.as_deref(),
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

/// Provides the ability to fetch the most recent purchase order revision
///
/// # Arguments
///
/// `store_state` - Provides a `store_factory` to access Grid's stores
/// `path` - Used to retrieve the purchase order UID and version ID from the
///  request's path
/// `query_service_id` - Optional service ID provided in the query string
/// `version` - Determines the type of response, corresponding to the versions of the rest API
pub async fn get_latest_revision_id(
    store_state: web::Data<StoreState>,
    path: web::Path<(String, String)>,
    query_service_id: web::Query<QueryServiceId>,
    version: ProtocolVersion,
    _: AcceptServiceIdParam,
) -> HttpResponse {
    let store = store_state.store_factory.get_grid_purchase_order_store();
    let (uid, version_id) = path.into_inner();
    match version {
        ProtocolVersion::V1 => {
            match v1::get_latest_revision_id(
                store,
                uid,
                version_id,
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

/// Provides the ability to fetch a purchase order revision
///
/// # Arguments
///
/// `store_state` - Provides a `store_factory` to access Grid's stores
/// `path` - Used to retrieve the purchase order UID, version ID, revision number from the
///  request's path
/// `query_service_id` - Optional service ID provided in the query string
/// `version` - Determines the type of response, corresponding to the versions of the rest API
pub async fn get_purchase_order_version_revision(
    store_state: web::Data<StoreState>,
    path: web::Path<(String, String, i64)>,
    query_service_id: web::Query<QueryServiceId>,
    version: ProtocolVersion,
    _: AcceptServiceIdParam,
) -> HttpResponse {
    let store = store_state.store_factory.get_grid_purchase_order_store();
    let (uid, version_id, revision_number) = path.into_inner();
    match version {
        ProtocolVersion::V1 => {
            match v1::get_purchase_order_revision(
                store,
                uid,
                version_id,
                revision_number,
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
