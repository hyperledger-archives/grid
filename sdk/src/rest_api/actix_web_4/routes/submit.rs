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

use actix_web_4::{
    dev, error, http::StatusCode, web, Error, FromRequest, HttpRequest, HttpResponse,
};
use futures_util::future::{FutureExt, LocalBoxFuture};

use crate::error::InternalError;
use crate::rest_api::actix_web_4::{KeyState, StoreState};
use crate::rest_api::resources::submit::v1::{submit_batches, SubmitBatchRequest};

use super::DEFAULT_GRID_PROTOCOL_VERSION;

pub async fn submit(
    store_state: web::Data<StoreState>,
    key_state: web::Data<KeyState>,
    version: ProtocolVersion,
) -> HttpResponse {
    let store = store_state.store_factory.get_batch_store();
    match version {
        ProtocolVersion::V1(payload) => {
            match submit_batches(&key_state.key_file_name, store, payload) {
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

pub enum ProtocolVersion {
    V1(SubmitBatchRequest),
}

impl FromRequest for ProtocolVersion {
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

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
            "1" => dev::JsonBody::new(req, payload, None, false)
                .map(|result| match result {
                    Ok(data) => Ok(ProtocolVersion::V1(data)),
                    Err(err) => Err(Error::from(error::InternalError::new(
                        InternalError::from_source(Box::new(err)),
                        StatusCode::BAD_REQUEST,
                    ))),
                })
                .boxed_local(),
            _ => dev::JsonBody::new(req, payload, None, false)
                .map(|res| match res {
                    Ok(data) => Ok(ProtocolVersion::V1(data)),
                    Err(err) => Err(Error::from(error::InternalError::new(
                        InternalError::from_source(Box::new(err)),
                        StatusCode::BAD_REQUEST,
                    ))),
                })
                .boxed_local(),
        }
    }
}
