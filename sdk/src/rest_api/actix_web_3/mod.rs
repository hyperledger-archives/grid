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

use actix_web::{App, HttpServer};
use diesel::r2d2::{ConnectionManager, Pool};

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
        }
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
