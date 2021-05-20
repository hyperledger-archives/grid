// Copyright 2021 Cargill Incorporated
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
    diesel::{schema::batches, BatchModel},
    BatchStoreError, BatchSubmitInfo,
};
use crate::error::InternalError;

use chrono::NaiveDateTime;
use diesel::{dsl::update, prelude::*, select};

pub(in crate::batches::store::diesel) trait GetUnclaimedBatchesOperation {
    fn get_unclaimed_batches(
        &self,
        limit: i64,
        secs_claim_is_valid: i64,
    ) -> Result<Vec<BatchSubmitInfo>, BatchStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> GetUnclaimedBatchesOperation for BatchStoreOperations<'a, diesel::pg::PgConnection> {
    fn get_unclaimed_batches(
        &self,
        limit: i64,
        secs_claim_is_valid: i64,
    ) -> Result<Vec<BatchSubmitInfo>, BatchStoreError> {
        self.conn.transaction::<_, BatchStoreError, _>(|| {
            let current_timestamp = select(diesel::dsl::now)
                .get_result::<NaiveDateTime>(&*self.conn)
                .map_err(|err| {
                    BatchStoreError::InternalError(InternalError::from_source(Box::new(err)))
                })?;

            let claim_expires = NaiveDateTime::from_timestamp(
                current_timestamp.timestamp() + secs_claim_is_valid,
                0,
            );

            let batches = batches::table
                .select(batches::all_columns)
                .filter(
                    batches::submitted.eq(false).and(
                        batches::claim_expires
                            .is_null()
                            .or(batches::claim_expires.lt(current_timestamp)),
                    ),
                )
                .limit(limit)
                .load::<BatchModel>(self.conn)
                .map_err(|err| {
                    BatchStoreError::InternalError(InternalError::from_source(Box::new(err)))
                })?;

            let mut batch_submit_info = Vec::new();
            for batch in batches {
                update(batches::table)
                    .filter(batches::header_signature.eq(&batch.header_signature))
                    .set(batches::claim_expires.eq(claim_expires))
                    .execute(self.conn)
                    .map_err(|err| {
                        BatchStoreError::InternalError(InternalError::from_source(Box::new(err)))
                    })?;

                batch_submit_info.push(batch.into());
            }

            Ok(batch_submit_info)
        })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> GetUnclaimedBatchesOperation
    for BatchStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn get_unclaimed_batches(
        &self,
        limit: i64,
        secs_claim_is_valid: i64,
    ) -> Result<Vec<BatchSubmitInfo>, BatchStoreError> {
        self.conn.transaction::<_, BatchStoreError, _>(|| {
            let current_timestamp = select(diesel::dsl::now)
                .get_result::<NaiveDateTime>(&*self.conn)
                .map_err(|err| {
                    BatchStoreError::InternalError(InternalError::from_source(Box::new(err)))
                })?;

            let claim_expires = NaiveDateTime::from_timestamp(
                current_timestamp.timestamp() + secs_claim_is_valid,
                0,
            );

            let batches = batches::table
                .select(batches::all_columns)
                .filter(
                    batches::submitted.eq(false).and(
                        batches::claim_expires
                            .is_null()
                            .or(batches::claim_expires.lt(current_timestamp)),
                    ),
                )
                .limit(limit)
                .load::<BatchModel>(self.conn)
                .map_err(|err| {
                    BatchStoreError::InternalError(InternalError::from_source(Box::new(err)))
                })?;

            let mut batch_submit_info = Vec::new();
            for batch in batches {
                update(batches::table)
                    .filter(batches::header_signature.eq(&batch.header_signature))
                    .set(batches::claim_expires.eq(claim_expires))
                    .execute(self.conn)
                    .map_err(|err| {
                        BatchStoreError::InternalError(InternalError::from_source(Box::new(err)))
                    })?;

                batch_submit_info.push(batch.into());
            }

            Ok(batch_submit_info)
        })
    }
}
