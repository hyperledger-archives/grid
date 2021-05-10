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
use crate::{
    batches::store::{
        diesel::{
            schema::{batches, transactions},
            BatchModel,
        },
        Batch, BatchList, BatchStoreError,
    },
    paging::Paging,
};

use crate::error::InternalError;
use diesel::prelude::*;

pub(in crate::batches::store::diesel) trait ListBatchesOperation {
    fn list_batches(&self, offset: i64, limit: i64) -> Result<BatchList, BatchStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> ListBatchesOperation for BatchStoreOperations<'a, diesel::pg::PgConnection> {
    fn list_batches(&self, offset: i64, limit: i64) -> Result<BatchList, BatchStoreError> {
        self.conn.transaction::<_, BatchStoreError, _>(|| {
            let batch_models: Vec<BatchModel> = batches::table
                .select(batches::all_columns)
                .offset(offset)
                .limit(limit)
                .load::<BatchModel>(self.conn)
                .map_err(|err| {
                    BatchStoreError::InternalError(InternalError::from_source(Box::new(err)))
                })?;

            let mut batches = Vec::new();
            for batch_model in batch_models {
                let transaction_models = transactions::table
                    .select(transactions::all_columns)
                    .filter(transactions::batch_id.eq(&batch_model.header_signature))
                    .load(self.conn)
                    .map_err(|err| {
                        BatchStoreError::InternalError(InternalError::from_source(Box::new(err)))
                    })?;
                batches.push(Batch::from((batch_model, transaction_models)))
            }

            let total = batches::table
                .select(batches::all_columns)
                .count()
                .get_result(self.conn)
                .map_err(|err| {
                    BatchStoreError::InternalError(InternalError::from_source(Box::new(err)))
                })?;

            Ok(BatchList::new(batches, Paging::new(offset, limit, total)))
        })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> ListBatchesOperation for BatchStoreOperations<'a, diesel::sqlite::SqliteConnection> {
    fn list_batches(&self, offset: i64, limit: i64) -> Result<BatchList, BatchStoreError> {
        self.conn.transaction::<_, BatchStoreError, _>(|| {
            let batch_models: Vec<BatchModel> = batches::table
                .select(batches::all_columns)
                .offset(offset)
                .limit(limit)
                .load::<BatchModel>(self.conn)
                .map_err(|err| {
                    BatchStoreError::InternalError(InternalError::from_source(Box::new(err)))
                })?;

            let mut batches = Vec::new();
            for batch_model in batch_models {
                let transaction_models = transactions::table
                    .select(transactions::all_columns)
                    .filter(transactions::batch_id.eq(&batch_model.header_signature))
                    .load(self.conn)
                    .map_err(|err| {
                        BatchStoreError::InternalError(InternalError::from_source(Box::new(err)))
                    })?;
                batches.push(Batch::from((batch_model, transaction_models)))
            }

            let total = batches::table
                .select(batches::all_columns)
                .count()
                .get_result(self.conn)
                .map_err(|err| {
                    BatchStoreError::InternalError(InternalError::from_source(Box::new(err)))
                })?;

            Ok(BatchList::new(batches, Paging::new(offset, limit, total)))
        })
    }
}
