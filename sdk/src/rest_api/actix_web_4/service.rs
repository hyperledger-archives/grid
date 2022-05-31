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

use actix_web_4::{dev, error, http::StatusCode, web, Error, FromRequest, HttpRequest, Result};
use futures::future;
use futures_util::future::{FutureExt, LocalBoxFuture};

use crate::error::InternalError;

use super::Endpoint;

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryServiceId {
    pub service_id: Option<String>,
    pub wait: Option<u64>,
}

pub struct AcceptServiceIdParam;

impl FromRequest for AcceptServiceIdParam {
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut dev::Payload) -> Self::Future {
        let endpoint: Endpoint = if let Some(endpoint) = req.app_data::<Endpoint>() {
            endpoint.clone()
        } else {
            return future::err(Error::from(error::InternalError::new(
                InternalError::with_message("App state not found".to_string()),
                StatusCode::INTERNAL_SERVER_ERROR,
            )))
            .boxed_local();
        };

        let service_id =
            if let Ok(query) = web::Query::<QueryServiceId>::from_query(req.query_string()) {
                query.service_id.clone()
            } else {
                return future::err(Error::from(error::InternalError::new(
                    InternalError::with_message("Malformed query param".to_string()),
                    StatusCode::BAD_REQUEST,
                )))
                .boxed_local();
            };

        if service_id.is_some() && endpoint.is_sawtooth() {
            return future::err(Error::from(error::InternalError::new(
                InternalError::with_message(
                    "Service ID present, but grid is running in sawtooth mode".to_string(),
                ),
                StatusCode::BAD_REQUEST,
            )))
            .boxed_local();
        } else if service_id.is_none() && !endpoint.is_sawtooth() {
            return future::err(Error::from(error::InternalError::new(
                InternalError::with_message(
                    "Service ID is not present, but grid is running in splinter mode".to_string(),
                ),
                StatusCode::BAD_REQUEST,
            )))
            .boxed_local();
        }

        future::ok(AcceptServiceIdParam).boxed_local()
    }
}
