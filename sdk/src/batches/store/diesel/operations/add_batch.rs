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

use super::BatchStoreOperations;
use crate::batches::store::{
    diesel::{
        schema::{batches, transactions},
        BatchModel, TransactionModel,
    },
    Batch, BatchStoreError,
};
use crate::error::InternalError;

use diesel::{dsl::insert_into, prelude::*};

pub(in crate::batches::store::diesel) trait AddBatchOperation {
    fn add_batch(&self, batch: Batch) -> Result<(), BatchStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> AddBatchOperation for BatchStoreOperations<'a, diesel::pg::PgConnection> {
    fn add_batch(&self, batch: Batch) -> Result<(), BatchStoreError> {
        let (batch_model, transaction_models): (BatchModel, Vec<TransactionModel>) = batch.into();
        self.conn.transaction::<_, BatchStoreError, _>(|| {
            insert_into(batches::table)
                .values(batch_model)
                .execute(&*self.conn)
                .map(|_| ())
                .map_err(|err| {
                    BatchStoreError::InternalError(InternalError::from_source(Box::new(err)))
                })?;

            insert_into(transactions::table)
                .values(transaction_models)
                .execute(&*self.conn)
                .map(|_| ())
                .map_err(|err| {
                    BatchStoreError::InternalError(InternalError::from_source(Box::new(err)))
                })?;

            Ok(())
        })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> AddBatchOperation for BatchStoreOperations<'a, diesel::sqlite::SqliteConnection> {
    fn add_batch(&self, batch: Batch) -> Result<(), BatchStoreError> {
        let (batch_model, transaction_models): (BatchModel, Vec<TransactionModel>) = batch.into();
        self.conn.transaction::<_, BatchStoreError, _>(|| {
            insert_into(batches::table)
                .values(batch_model)
                .execute(&*self.conn)
                .map(|_| ())
                .map_err(|err| {
                    BatchStoreError::InternalError(InternalError::from_source(Box::new(err)))
                })?;

            insert_into(transactions::table)
                .values(transaction_models)
                .execute(&*self.conn)
                .map(|_| ())
                .map_err(|err| {
                    BatchStoreError::InternalError(InternalError::from_source(Box::new(err)))
                })?;

            Ok(())
        })
    }
}
