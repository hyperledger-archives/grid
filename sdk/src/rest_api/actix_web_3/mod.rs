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

pub mod routes;

use std::sync::Arc;

#[cfg(feature = "batch-store")]
use crate::batches::{store::diesel::DieselBatchStore, BatchStore};
use crate::error::InternalError;
#[cfg(feature = "location")]
use crate::locations::{store::diesel::DieselLocationStore, LocationStore};
#[cfg(feature = "pike")]
use crate::pike::{store::diesel::DieselPikeStore, PikeStore};
#[cfg(feature = "product")]
use crate::products::{store::diesel::DieselProductStore, ProductStore};
use crate::rest_api::resources::error::ErrorResponse;
#[cfg(feature = "schema")]
use crate::schemas::{store::diesel::DieselSchemaStore, SchemaStore};
#[cfg(feature = "batch-submitter")]
use crate::submitter::BatchSubmitter;
#[cfg(feature = "track-and-trace")]
use crate::track_and_trace::{store::diesel::DieselTrackAndTraceStore, TrackAndTraceStore};

use actix_web::{
    dev, http::StatusCode, web, App, FromRequest, HttpRequest, HttpResponse, HttpServer, Result,
};
use diesel::r2d2::{ConnectionManager, Pool};
use futures::future;

pub use routes::submit;

#[derive(Clone)]
pub struct State {
    pub key_file_name: String,
    #[cfg(feature = "batch-submitter")]
    pub batch_submitter: Option<Arc<dyn BatchSubmitter + 'static>>,
    #[cfg(feature = "batch-store")]
    pub batch_store: Arc<dyn BatchStore>,
    #[cfg(feature = "location")]
    pub location_store: Arc<dyn LocationStore>,
    #[cfg(feature = "pike")]
    pub pike_store: Arc<dyn PikeStore>,
    #[cfg(feature = "product")]
    pub product_store: Arc<dyn ProductStore>,
    #[cfg(feature = "schema")]
    pub schema_store: Arc<dyn SchemaStore>,
    #[cfg(feature = "track-and-trace")]
    pub tnt_store: Arc<dyn TrackAndTraceStore>,
}

impl State {
    pub fn with_pg_pool(
        key_file_name: &str,
        connection_pool: Pool<ConnectionManager<diesel::pg::PgConnection>>,
    ) -> Self {
        #[cfg(feature = "batch-store")]
        let batch_store = Arc::new(DieselBatchStore::new(connection_pool.clone()));
        #[cfg(feature = "location")]
        let location_store = Arc::new(DieselLocationStore::new(connection_pool.clone()));
        #[cfg(feature = "pike")]
        let pike_store = Arc::new(DieselPikeStore::new(connection_pool.clone()));
        #[cfg(feature = "product")]
        let product_store = Arc::new(DieselProductStore::new(connection_pool.clone()));
        #[cfg(feature = "schema")]
        let schema_store = Arc::new(DieselSchemaStore::new(connection_pool.clone()));
        #[cfg(feature = "track-and-trace")]
        let tnt_store = Arc::new(DieselTrackAndTraceStore::new(connection_pool));

        Self {
            key_file_name: key_file_name.to_string(),
            #[cfg(feature = "batch-submitter")]
            batch_submitter: None,
            #[cfg(feature = "batch-store")]
            batch_store,
            #[cfg(feature = "location")]
            location_store,
            #[cfg(feature = "pike")]
            pike_store,
            #[cfg(feature = "product")]
            product_store,
            #[cfg(feature = "schema")]
            schema_store,
            #[cfg(feature = "track-and-trace")]
            tnt_store,
        }
    }

    pub fn with_sqlite_pool(
        key_file_name: &str,
        connection_pool: Pool<ConnectionManager<diesel::sqlite::SqliteConnection>>,
    ) -> Self {
        #[cfg(feature = "batch-store")]
        let batch_store = Arc::new(DieselBatchStore::new(connection_pool.clone()));
        #[cfg(feature = "location")]
        let location_store = Arc::new(DieselLocationStore::new(connection_pool.clone()));
        #[cfg(feature = "pike")]
        let pike_store = Arc::new(DieselPikeStore::new(connection_pool.clone()));
        #[cfg(feature = "product")]
        let product_store = Arc::new(DieselProductStore::new(connection_pool.clone()));
        #[cfg(feature = "schema")]
        let schema_store = Arc::new(DieselSchemaStore::new(connection_pool.clone()));
        #[cfg(feature = "track-and-trace")]
        let tnt_store = Arc::new(DieselTrackAndTraceStore::new(connection_pool));

        Self {
            key_file_name: key_file_name.to_string(),
            #[cfg(feature = "batch-submitter")]
            batch_submitter: None,
            #[cfg(feature = "batch-store")]
            batch_store,
            #[cfg(feature = "location")]
            location_store,
            #[cfg(feature = "pike")]
            pike_store,
            #[cfg(feature = "product")]
            product_store,
            #[cfg(feature = "schema")]
            schema_store,
            #[cfg(feature = "track-and-trace")]
            tnt_store,
        }
    }

    #[cfg(feature = "batch-submitter")]
    pub fn set_batch_submitter(&mut self, batch_submitter: Arc<dyn BatchSubmitter + 'static>) {
        self.batch_submitter = Some(batch_submitter);
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryServiceId {
    pub service_id: Option<String>,
    pub wait: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryPaging {
    pub offset: Option<u64>,
    pub limit: Option<u16>,
}

impl QueryPaging {
    pub fn offset(&self) -> u64 {
        self.offset.unwrap_or(0)
    }

    pub fn limit(&self) -> u16 {
        self.limit
            .map(|l| if l > 1024 { 1024 } else { l })
            .unwrap_or(10)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Endpoint {
    backend: Backend,
    url: String,
}

impl Endpoint {
    pub fn url(&self) -> String {
        self.url.clone()
    }

    pub fn is_sawtooth(&self) -> bool {
        self.backend == Backend::Sawtooth
    }

    pub fn backend(&self) -> &Backend {
        &self.backend
    }
}

impl From<&str> for Endpoint {
    fn from(s: &str) -> Self {
        let s = s.to_lowercase();

        if s.starts_with("splinter:") {
            let url = s.replace("splinter:", "");
            Endpoint {
                backend: Backend::Splinter,
                url,
            }
        } else if s.starts_with("sawtooth:") {
            let url = s.replace("sawtooth:", "");
            Endpoint {
                backend: Backend::Sawtooth,
                url,
            }
        } else {
            Endpoint {
                backend: Backend::Sawtooth,
                url: s,
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Backend {
    Splinter,
    Sawtooth,
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
                    "Circuit ID present, but grid is running in sawtooth mode",
                )),
            );
        } else if service_id.is_none() && !endpoint.is_sawtooth() {
            return future::err(
                HttpResponse::build(
                    StatusCode::from_u16(400).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                )
                .json(ErrorResponse::new(
                    400,
                    "Circuit ID is not present, but grid is running in splinter mode",
                )),
            );
        }

        future::ok(AcceptServiceIdParam)
    }
}

pub async fn run(bind: &str, state: State) -> Result<(), InternalError> {
    HttpServer::new(move || App::new().data(state.clone()).service(submit))
        .bind(bind)
        .map_err(|err| InternalError::from_source(Box::new(err)))?
        .run()
        .await
        .map_err(|err| InternalError::from_source(Box::new(err)))
}
