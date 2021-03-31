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

pub mod models;
mod operations;
pub(in crate) mod schema;

use diesel::r2d2::{ConnectionManager, Pool};

use super::diesel::models::BatchModel;
use super::{Batch, BatchList, BatchStatus, BatchStore, BatchStoreError};
use crate::error::ResourceTemporarilyUnavailableError;

use operations::add_batch::AddBatchOperation as _;
use operations::fetch_batch::FetchBatchOperation as _;
use operations::list_batches::ListBatchesOperation as _;
use operations::BatchStoreOperations;

#[derive(Clone)]
pub struct DieselBatchStore<C: diesel::Connection + 'static> {
    connection_pool: Pool<ConnectionManager<C>>,
}

impl<C: diesel::Connection> DieselBatchStore<C> {
    #[allow(dead_code)]
    pub fn new(connection_pool: Pool<ConnectionManager<C>>) -> Self {
        DieselBatchStore { connection_pool }
    }
}

#[cfg(feature = "postgres")]
impl BatchStore for DieselBatchStore<diesel::pg::PgConnection> {
    fn add_batch(&self, batch: Batch) -> Result<(), BatchStoreError> {
        BatchStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            BatchStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .add_batch(batch.into())
    }

    fn fetch_batch(&self, id: &str) -> Result<Option<Batch>, BatchStoreError> {
        BatchStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            BatchStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .fetch_batch(id)
        .map(|op| op.map(|model| model.into()))
    }

    fn list_batches(&self, offset: i64, limit: i64) -> Result<BatchList, BatchStoreError> {
        BatchStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            BatchStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .list_batches(offset, limit)
    }
}

#[cfg(feature = "sqlite")]
impl BatchStore for DieselBatchStore<diesel::sqlite::SqliteConnection> {
    fn add_batch(&self, batch: Batch) -> Result<(), BatchStoreError> {
        BatchStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            BatchStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .add_batch(batch.into())
    }

    fn fetch_batch(&self, id: &str) -> Result<Option<Batch>, BatchStoreError> {
        BatchStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            BatchStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .fetch_batch(id)
        .map(|op| op.map(|model| model.into()))
    }

    fn list_batches(&self, offset: i64, limit: i64) -> Result<BatchList, BatchStoreError> {
        BatchStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            BatchStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .list_batches(offset, limit)
    }

    }
}

    }
}

impl From<Batch> for BatchModel {
    fn from(batch: Batch) -> Self {
        Self {
            id: batch.id,
            data: batch.data,
            status: batch.status.to_string(),
        }
    }
}

impl From<BatchModel> for Batch {
    fn from(model: BatchModel) -> Self {
        let status = match model.status.as_ref() {
            "Committed" => BatchStatus::Committed,
            "Submitted" => BatchStatus::Submitted,
            "NotSubmitted" => BatchStatus::NotSubmitted,
            "Rejected" => BatchStatus::Rejected,
            _ => BatchStatus::Unknown,
        };

        Batch {
            id: model.id,
            data: model.data,
            status,
        }
    }
}
