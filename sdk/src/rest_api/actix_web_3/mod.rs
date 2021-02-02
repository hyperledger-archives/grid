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

use routes::submit;

use crate::batches::{store::diesel::DieselBatchStore, BatchStore};
use crate::error::InternalError;

use actix_web::{App, HttpServer};
use diesel::r2d2::{ConnectionManager, Pool};

#[derive(Clone)]
pub struct State {
    pub key_file_name: String,
    pub batch_store: Arc<dyn BatchStore>,
}

impl State {
    pub fn with_pg_pool(
        key_file_name: &str,
        connection_pool: Pool<ConnectionManager<diesel::pg::PgConnection>>,
    ) -> Self {
        let batch_store = Arc::new(DieselBatchStore::new(connection_pool));

        Self {
            key_file_name: key_file_name.to_string(),
            batch_store,
        }
    }

    pub fn with_sqlite_pool(
        key_file_name: &str,
        connection_pool: Pool<ConnectionManager<diesel::sqlite::SqliteConnection>>,
    ) -> Self {
        let batch_store = Arc::new(DieselBatchStore::new(connection_pool));

        Self {
            key_file_name: key_file_name.to_string(),
            batch_store,
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
