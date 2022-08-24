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

use super::BatchTrackingStoreOperations;
use crate::batch_tracking::store::{
    diesel::{
        models::{make_new_batch_models, make_transaction_models},
        schema::{batches, transactions},
    },
    BatchTrackingStoreError, TrackingBatch,
};

use diesel::{dsl::insert_into, prelude::*};

pub(in crate::batch_tracking::store::diesel) trait BatchTrackingStoreAddBatchesOperation {
    fn add_batches(&self, batches: Vec<TrackingBatch>) -> Result<(), BatchTrackingStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> BatchTrackingStoreAddBatchesOperation
    for BatchTrackingStoreOperations<'a, diesel::pg::PgConnection>
{
    fn add_batches(&self, batches: Vec<TrackingBatch>) -> Result<(), BatchTrackingStoreError> {
        let batch_models = make_new_batch_models(&batches);
        let transaction_models = make_transaction_models(&batches);
        self.conn.transaction::<_, BatchTrackingStoreError, _>(|| {
            insert_into(batches::table)
                .values(batch_models)
                .execute(self.conn)
                .map(|_| ())
                .map_err(BatchTrackingStoreError::from)?;

            insert_into(transactions::table)
                .values(transaction_models)
                .execute(self.conn)
                .map(|_| ())
                .map_err(BatchTrackingStoreError::from)?;

            Ok(())
        })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> BatchTrackingStoreAddBatchesOperation
    for BatchTrackingStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn add_batches(&self, batches: Vec<TrackingBatch>) -> Result<(), BatchTrackingStoreError> {
        let batch_models = make_new_batch_models(&batches);
        let transaction_models = make_transaction_models(&batches);
        self.conn.transaction::<_, BatchTrackingStoreError, _>(|| {
            insert_into(batches::table)
                .values(batch_models)
                .execute(self.conn)
                .map(|_| ())
                .map_err(BatchTrackingStoreError::from)?;

            insert_into(transactions::table)
                .values(transaction_models)
                .execute(self.conn)
                .map(|_| ())
                .map_err(BatchTrackingStoreError::from)?;

            Ok(())
        })
    }
}
