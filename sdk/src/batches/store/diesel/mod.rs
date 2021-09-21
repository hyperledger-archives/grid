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

use diesel::connection::AnsiTransactionManager;
use diesel::r2d2::{ConnectionManager, Pool};

use super::diesel::models::{BatchModel, TransactionModel};
use super::{Batch, BatchList, BatchStore, BatchStoreError, BatchSubmitInfo};
use crate::error::ResourceTemporarilyUnavailableError;

use operations::add_batch::AddBatchOperation as _;
use operations::change_batch_to_submitted::ChangeBatchToSubmittedOperation as _;
use operations::get_batch::GetBatchOperation as _;
use operations::get_unclaimed_batches::GetUnclaimedBatchesOperation as _;
use operations::list_batches::ListBatchesOperation as _;
use operations::relinquish_claim::RelinquishClaimOperation as _;
use operations::update_submission_error_info::UpdateSubmissionErrorInfoOperation as _;
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
        .add_batch(batch)
    }

    fn get_batch(&self, id: &str) -> Result<Option<Batch>, BatchStoreError> {
        BatchStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            BatchStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .get_batch(id)
    }

    fn list_batches(&self, offset: i64, limit: i64) -> Result<BatchList, BatchStoreError> {
        BatchStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            BatchStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .list_batches(offset, limit)
    }

    fn get_unclaimed_batches(
        &self,
        limit: i64,
        secs_claim_is_valid: i64,
    ) -> Result<Vec<BatchSubmitInfo>, BatchStoreError> {
        BatchStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            BatchStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .get_unclaimed_batches(limit, secs_claim_is_valid)
    }

    fn change_batch_to_submitted(&self, id: &str) -> Result<(), BatchStoreError> {
        BatchStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            BatchStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .change_batch_to_submitted(id)
    }

    fn update_submission_error_info(
        &self,
        id: &str,
        error: &str,
        error_message: &str,
    ) -> Result<(), BatchStoreError> {
        BatchStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            BatchStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .update_submission_error_info(id, error, error_message)
    }

    fn relinquish_claim(&self, id: &str) -> Result<(), BatchStoreError> {
        BatchStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            BatchStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .relinquish_claim(id)
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
        .add_batch(batch)
    }

    fn get_batch(&self, id: &str) -> Result<Option<Batch>, BatchStoreError> {
        BatchStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            BatchStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .get_batch(id)
    }

    fn list_batches(&self, offset: i64, limit: i64) -> Result<BatchList, BatchStoreError> {
        BatchStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            BatchStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .list_batches(offset, limit)
    }

    fn get_unclaimed_batches(
        &self,
        limit: i64,
        secs_claim_is_valid: i64,
    ) -> Result<Vec<BatchSubmitInfo>, BatchStoreError> {
        BatchStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            BatchStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .get_unclaimed_batches(limit, secs_claim_is_valid)
    }

    fn change_batch_to_submitted(&self, id: &str) -> Result<(), BatchStoreError> {
        BatchStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            BatchStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .change_batch_to_submitted(id)
    }

    fn update_submission_error_info(
        &self,
        id: &str,
        error: &str,
        error_message: &str,
    ) -> Result<(), BatchStoreError> {
        BatchStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            BatchStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .update_submission_error_info(id, error, error_message)
    }

    fn relinquish_claim(&self, id: &str) -> Result<(), BatchStoreError> {
        BatchStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            BatchStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .relinquish_claim(id)
    }
}

pub struct DieselConnectionBatchStore<'a, C>
where
    C: diesel::Connection<TransactionManager = AnsiTransactionManager> + 'static,
    C::Backend: diesel::backend::UsesAnsiSavepointSyntax,
{
    connection: &'a C,
}

impl<'a, C> DieselConnectionBatchStore<'a, C>
where
    C: diesel::Connection<TransactionManager = AnsiTransactionManager> + 'static,
    C::Backend: diesel::backend::UsesAnsiSavepointSyntax,
{
    pub fn new(connection: &'a C) -> Self {
        DieselConnectionBatchStore { connection }
    }
}

#[cfg(feature = "postgres")]
impl<'a> BatchStore for DieselConnectionBatchStore<'a, diesel::pg::PgConnection> {
    fn add_batch(&self, batch: Batch) -> Result<(), BatchStoreError> {
        BatchStoreOperations::new(self.connection).add_batch(batch)
    }

    fn get_batch(&self, id: &str) -> Result<Option<Batch>, BatchStoreError> {
        BatchStoreOperations::new(self.connection).get_batch(id)
    }

    fn list_batches(&self, offset: i64, limit: i64) -> Result<BatchList, BatchStoreError> {
        BatchStoreOperations::new(self.connection).list_batches(offset, limit)
    }

    fn get_unclaimed_batches(
        &self,
        limit: i64,
        secs_claim_is_valid: i64,
    ) -> Result<Vec<BatchSubmitInfo>, BatchStoreError> {
        BatchStoreOperations::new(self.connection).get_unclaimed_batches(limit, secs_claim_is_valid)
    }

    fn change_batch_to_submitted(&self, id: &str) -> Result<(), BatchStoreError> {
        BatchStoreOperations::new(self.connection).change_batch_to_submitted(id)
    }

    fn update_submission_error_info(
        &self,
        id: &str,
        error: &str,
        error_message: &str,
    ) -> Result<(), BatchStoreError> {
        BatchStoreOperations::new(self.connection).update_submission_error_info(
            id,
            error,
            error_message,
        )
    }

    fn relinquish_claim(&self, id: &str) -> Result<(), BatchStoreError> {
        BatchStoreOperations::new(self.connection).relinquish_claim(id)
    }
}

#[cfg(feature = "sqlite")]
impl<'a> BatchStore for DieselConnectionBatchStore<'a, diesel::sqlite::SqliteConnection> {
    fn add_batch(&self, batch: Batch) -> Result<(), BatchStoreError> {
        BatchStoreOperations::new(self.connection).add_batch(batch)
    }

    fn get_batch(&self, id: &str) -> Result<Option<Batch>, BatchStoreError> {
        BatchStoreOperations::new(self.connection).get_batch(id)
    }

    fn list_batches(&self, offset: i64, limit: i64) -> Result<BatchList, BatchStoreError> {
        BatchStoreOperations::new(self.connection).list_batches(offset, limit)
    }

    fn get_unclaimed_batches(
        &self,
        limit: i64,
        secs_claim_is_valid: i64,
    ) -> Result<Vec<BatchSubmitInfo>, BatchStoreError> {
        BatchStoreOperations::new(self.connection).get_unclaimed_batches(limit, secs_claim_is_valid)
    }

    fn change_batch_to_submitted(&self, id: &str) -> Result<(), BatchStoreError> {
        BatchStoreOperations::new(self.connection).change_batch_to_submitted(id)
    }

    fn update_submission_error_info(
        &self,
        id: &str,
        error: &str,
        error_message: &str,
    ) -> Result<(), BatchStoreError> {
        BatchStoreOperations::new(self.connection).update_submission_error_info(
            id,
            error,
            error_message,
        )
    }

    fn relinquish_claim(&self, id: &str) -> Result<(), BatchStoreError> {
        BatchStoreOperations::new(self.connection).relinquish_claim(id)
    }
}
