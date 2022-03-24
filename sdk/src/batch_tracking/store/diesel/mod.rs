// Copyright 2022 Cargill Incorporated
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

use diesel::connection::AnsiTransactionManager;
use diesel::r2d2::{ConnectionManager, Pool};

use super::{
    BatchStatus, BatchTrackingStore, BatchTrackingStoreError, InvalidTransaction, SubmissionError,
    TrackingBatch, TrackingBatchList, TrackingTransaction, TransactionReceipt, ValidTransaction,
};

use crate::error::ResourceTemporarilyUnavailableError;

use operations::add_batches::BatchTrackingStoreAddBatchesOperation as _;
use operations::BatchTrackingStoreOperations;

/// Manages batches in the database
#[derive(Clone)]
pub struct DieselBatchTrackingStore<C: diesel::Connection + 'static> {
    connection_pool: Pool<ConnectionManager<C>>,
}

impl<C: diesel::Connection> DieselBatchTrackingStore<C> {
    /// Creates a new DieselBatchTrackingStore
    ///
    /// # Arguments
    ///
    ///  * `connection_pool`: connection pool to the database
    #[allow(dead_code)]
    pub fn new(connection_pool: Pool<ConnectionManager<C>>) -> Self {
        DieselBatchTrackingStore { connection_pool }
    }
}

#[cfg(feature = "postgres")]
impl BatchTrackingStore for DieselBatchTrackingStore<diesel::pg::PgConnection> {
    fn get_batch_status(
        &self,
        _id: &str,
        _service_id: Option<&str>,
    ) -> Result<BatchStatus, BatchTrackingStoreError> {
        unimplemented!();
    }

    fn update_batch_status(
        &self,
        _id: String,
        _service_id: Option<&str>,
        _status: BatchStatus,
        _errors: Vec<SubmissionError>,
    ) -> Result<(), BatchTrackingStoreError> {
        unimplemented!();
    }

    fn add_batches(&self, batches: Vec<TrackingBatch>) -> Result<(), BatchTrackingStoreError> {
        BatchTrackingStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            BatchTrackingStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .add_batches(batches)
    }

    fn change_batch_to_submitted(
        &self,
        _batch_id: &str,
        _service_id: Option<&str>,
    ) -> Result<(), BatchTrackingStoreError> {
        unimplemented!();
    }

    fn get_batch(
        &self,
        _id: &str,
        _service_id: Option<&str>,
    ) -> Result<Option<TrackingBatch>, BatchTrackingStoreError> {
        unimplemented!();
    }

    fn list_batches_by_status(
        &self,
        _status: BatchStatus,
    ) -> Result<TrackingBatchList, BatchTrackingStoreError> {
        unimplemented!();
    }

    fn clean_stale_records(
        &self,
        _submitted_by: &str,
    ) -> Result<TrackingBatchList, BatchTrackingStoreError> {
        unimplemented!();
    }

    fn get_unsubmitted_batches(&self) -> Result<TrackingBatchList, BatchTrackingStoreError> {
        unimplemented!();
    }

    fn get_failed_batches(&self) -> Result<TrackingBatchList, BatchTrackingStoreError> {
        unimplemented!();
    }
}

#[cfg(feature = "sqlite")]
impl BatchTrackingStore for DieselBatchTrackingStore<diesel::sqlite::SqliteConnection> {
    fn get_batch_status(
        &self,
        _id: &str,
        _service_id: Option<&str>,
    ) -> Result<BatchStatus, BatchTrackingStoreError> {
        unimplemented!();
    }

    fn update_batch_status(
        &self,
        _id: String,
        _service_id: Option<&str>,
        _status: BatchStatus,
        _errors: Vec<SubmissionError>,
    ) -> Result<(), BatchTrackingStoreError> {
        unimplemented!();
    }

    fn add_batches(&self, batches: Vec<TrackingBatch>) -> Result<(), BatchTrackingStoreError> {
        BatchTrackingStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            BatchTrackingStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .add_batches(batches)
    }

    fn change_batch_to_submitted(
        &self,
        _batch_id: &str,
        _service_id: Option<&str>,
    ) -> Result<(), BatchTrackingStoreError> {
        unimplemented!();
    }

    fn get_batch(
        &self,
        _id: &str,
        _service_id: Option<&str>,
    ) -> Result<Option<TrackingBatch>, BatchTrackingStoreError> {
        unimplemented!();
    }

    fn list_batches_by_status(
        &self,
        _status: BatchStatus,
    ) -> Result<TrackingBatchList, BatchTrackingStoreError> {
        unimplemented!();
    }

    fn clean_stale_records(
        &self,
        _submitted_by: &str,
    ) -> Result<TrackingBatchList, BatchTrackingStoreError> {
        unimplemented!();
    }

    fn get_unsubmitted_batches(&self) -> Result<TrackingBatchList, BatchTrackingStoreError> {
        unimplemented!();
    }

    fn get_failed_batches(&self) -> Result<TrackingBatchList, BatchTrackingStoreError> {
        unimplemented!();
    }
}

pub struct DieselConnectionBatchTrackingStore<'a, C>
where
    C: diesel::Connection<TransactionManager = AnsiTransactionManager> + 'static,
    C::Backend: diesel::backend::UsesAnsiSavepointSyntax,
{
    connection: &'a C,
}

impl<'a, C> DieselConnectionBatchTrackingStore<'a, C>
where
    C: diesel::Connection<TransactionManager = AnsiTransactionManager> + 'static,
    C::Backend: diesel::backend::UsesAnsiSavepointSyntax,
{
    #[allow(dead_code)]
    pub fn new(connection: &'a C) -> Self {
        DieselConnectionBatchTrackingStore { connection }
    }
}

#[cfg(feature = "postgres")]
impl<'a> BatchTrackingStore for DieselConnectionBatchTrackingStore<'a, diesel::pg::PgConnection> {
    fn get_batch_status(
        &self,
        _id: &str,
        _service_id: Option<&str>,
    ) -> Result<BatchStatus, BatchTrackingStoreError> {
        unimplemented!();
    }

    fn update_batch_status(
        &self,
        _id: String,
        _service_id: Option<&str>,
        _status: BatchStatus,
        _errors: Vec<SubmissionError>,
    ) -> Result<(), BatchTrackingStoreError> {
        unimplemented!();
    }

    fn add_batches(&self, batches: Vec<TrackingBatch>) -> Result<(), BatchTrackingStoreError> {
        BatchTrackingStoreOperations::new(self.connection).add_batches(batches)
    }

    fn change_batch_to_submitted(
        &self,
        _batch_id: &str,
        _service_id: Option<&str>,
    ) -> Result<(), BatchTrackingStoreError> {
        unimplemented!();
    }

    fn get_batch(
        &self,
        _id: &str,
        _service_id: Option<&str>,
    ) -> Result<Option<TrackingBatch>, BatchTrackingStoreError> {
        unimplemented!();
    }

    fn list_batches_by_status(
        &self,
        _status: BatchStatus,
    ) -> Result<TrackingBatchList, BatchTrackingStoreError> {
        unimplemented!();
    }

    fn clean_stale_records(
        &self,
        _submitted_by: &str,
    ) -> Result<TrackingBatchList, BatchTrackingStoreError> {
        unimplemented!();
    }

    fn get_unsubmitted_batches(&self) -> Result<TrackingBatchList, BatchTrackingStoreError> {
        unimplemented!();
    }

    fn get_failed_batches(&self) -> Result<TrackingBatchList, BatchTrackingStoreError> {
        unimplemented!();
    }
}

#[cfg(feature = "sqlite")]
impl<'a> BatchTrackingStore
    for DieselConnectionBatchTrackingStore<'a, diesel::sqlite::SqliteConnection>
{
    fn get_batch_status(
        &self,
        _id: &str,
        _service_id: Option<&str>,
    ) -> Result<BatchStatus, BatchTrackingStoreError> {
        unimplemented!();
    }

    fn update_batch_status(
        &self,
        _id: String,
        _service_id: Option<&str>,
        _status: BatchStatus,
        _errors: Vec<SubmissionError>,
    ) -> Result<(), BatchTrackingStoreError> {
        unimplemented!();
    }

    fn add_batches(&self, batches: Vec<TrackingBatch>) -> Result<(), BatchTrackingStoreError> {
        BatchTrackingStoreOperations::new(self.connection).add_batches(batches)
    }

    fn change_batch_to_submitted(
        &self,
        _batch_id: &str,
        _service_id: Option<&str>,
    ) -> Result<(), BatchTrackingStoreError> {
        unimplemented!();
    }

    fn get_batch(
        &self,
        _id: &str,
        _service_id: Option<&str>,
    ) -> Result<Option<TrackingBatch>, BatchTrackingStoreError> {
        unimplemented!();
    }

    fn list_batches_by_status(
        &self,
        _status: BatchStatus,
    ) -> Result<TrackingBatchList, BatchTrackingStoreError> {
        unimplemented!();
    }

    fn clean_stale_records(
        &self,
        _submitted_by: &str,
    ) -> Result<TrackingBatchList, BatchTrackingStoreError> {
        unimplemented!();
    }

    fn get_unsubmitted_batches(&self) -> Result<TrackingBatchList, BatchTrackingStoreError> {
        unimplemented!();
    }

    fn get_failed_batches(&self) -> Result<TrackingBatchList, BatchTrackingStoreError> {
        unimplemented!();
    }
}
