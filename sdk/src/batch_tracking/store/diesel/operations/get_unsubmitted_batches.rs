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

use crate::batch_tracking::store::diesel::{
    models::{
        BatchModel, BatchStatusModel, SubmissionModel, TransactionModel, TransactionReceiptModel,
    },
    schema::{batch_statuses, batches},
    BatchStatus, TrackingBatchList,
};

use crate::batch_tracking::store::BatchTrackingStoreError;
use diesel::{prelude::*, sql_query};
use std::convert::TryFrom;

pub(in crate::batch_tracking::store::diesel) trait BatchTrackingStoreGetUnsubmittedBatchesOperation
{
    fn get_unsubmitted_batches(&self) -> Result<TrackingBatchList, BatchTrackingStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> BatchTrackingStoreGetUnsubmittedBatchesOperation
    for BatchTrackingStoreOperations<'a, diesel::pg::PgConnection>
{
    fn get_unsubmitted_batches(&self) -> Result<TrackingBatchList, BatchTrackingStoreError> {
        self.conn.transaction::<_, BatchTrackingStoreError, _>(|| {
            let unsubmitted_statuses: Vec<String> = vec![
                BatchStatus::Unknown.to_string(),
                BatchStatus::Delayed.to_string(),
            ];

            let batches_and_statuses: Vec<(BatchModel, Option<BatchStatusModel>)> = batches::table
                .into_boxed()
                .left_join(batch_statuses::table.on(
                    batches::batch_id
                        .eq(batch_statuses::batch_id)
                        .and(batches::service_id.eq(batch_statuses::service_id))
                ))
                .filter(batch_statuses::dlt_status.eq_any(unsubmitted_statuses))
                .or_filter(batches::submitted.eq(false))
                .select((batches::all_columns, batch_statuses::all_columns.nullable()))
                .load::<(BatchModel, Option<BatchStatusModel>)>(self.conn)?;

            if batches_and_statuses.is_empty() {
                return Ok(TrackingBatchList {
                    batches: Vec::new(),
                });
            }

            let (batch_models, batch_status_model_options): (Vec<BatchModel>, Vec<Option<BatchStatusModel>>) =
                batches_and_statuses.iter().cloned().unzip();

            let mut batch_status_models: Vec<BatchStatusModel> = Vec::new();

            batch_status_model_options.iter().for_each(|m| {
                if let Some(model) = m {
                    batch_status_models.push(model.clone());
                }
            });

            let submission_models: Vec<SubmissionModel> = sql_query(
                "WITH bbs AS (
                    SELECT b.batch_id, b.service_id FROM batches b
                    LEFT JOIN batch_statuses bs ON bs.batch_id = b.batch_id AND bs.service_id = b.service_id
                    WHERE bs.dlt_status = 'Delayed' OR bs.dlt_status = 'Unknown' OR b.submitted = false
                )
                SELECT * FROM submissions s
                WHERE (s.service_id, s.batch_id) IN (SELECT service_id, batch_id FROM bbs);"
            )
            .load(self.conn)?;

            let txn_models: Vec<TransactionModel> = sql_query(
                "WITH bbs AS (
                    SELECT b.batch_id, b.service_id FROM batches b
                    LEFT JOIN batch_statuses bs ON bs.batch_id = b.batch_id AND bs.service_id = b.service_id
                    WHERE bs.dlt_status = 'Delayed' OR bs.dlt_status = 'Unknown' OR b.submitted = false
                )
                SELECT * FROM transactions t
                WHERE (t.service_id, t.batch_id) IN (SELECT service_id, batch_id FROM bbs);"
            )
            .load(self.conn)?;

            let receipt_models: Vec<TransactionReceiptModel> = sql_query(
                "WITH bbs AS (
                    SELECT b.batch_id, b.service_id FROM batches b
                    LEFT JOIN batch_statuses bs ON bs.batch_id = b.batch_id AND bs.service_id = b.service_id
                    WHERE bs.dlt_status = 'Delayed' OR bs.dlt_status = 'Unknown' OR b.submitted = false
                ), txn_models AS (
                    SELECT t.transaction_id, t.service_id FROM transactions t
                    WHERE (t.service_id, t.batch_id) IN (SELECT service_id, batch_id FROM bbs)
                )
                SELECT * FROM transaction_receipts tr
                WHERE (tr.service_id, tr.transaction_id) IN (SELECT service_id, transaction_id FROM txn_models);"
            )
            .load(self.conn)?;

            let batches = TrackingBatchList::try_from((batch_models, batch_status_models, txn_models, receipt_models, submission_models))?;
            Ok(batches)
        })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> BatchTrackingStoreGetUnsubmittedBatchesOperation
    for BatchTrackingStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn get_unsubmitted_batches(&self) -> Result<TrackingBatchList, BatchTrackingStoreError> {
        self.conn.transaction::<_, BatchTrackingStoreError, _>(|| {
            let unsubmitted_statuses: Vec<String> = vec![
                BatchStatus::Unknown.to_string(),
                BatchStatus::Delayed.to_string(),
            ];

            let batches_and_statuses: Vec<(BatchModel, Option<BatchStatusModel>)> = batches::table
                .into_boxed()
                .left_join(batch_statuses::table.on(
                    batches::batch_id
                        .eq(batch_statuses::batch_id)
                        .and(batches::service_id.eq(batch_statuses::service_id))
                ))
                .filter(batch_statuses::dlt_status.eq_any(unsubmitted_statuses))
                .or_filter(batches::submitted.eq(false))
                .select((batches::all_columns, batch_statuses::all_columns.nullable()))
                .load::<(BatchModel, Option<BatchStatusModel>)>(self.conn)?;

            if batches_and_statuses.is_empty() {
                return Ok(TrackingBatchList {
                    batches: Vec::new(),
                });
            }

            let (batch_models, batch_status_model_options): (Vec<BatchModel>, Vec<Option<BatchStatusModel>>) =
                batches_and_statuses.iter().cloned().unzip();

            let mut batch_status_models: Vec<BatchStatusModel> = Vec::new();

            batch_status_model_options.iter().for_each(|m| {
                if let Some(model) = m {
                    batch_status_models.push(model.clone());
                }
            });

            let submission_models: Vec<SubmissionModel> = sql_query(
                "WITH bbs AS (
                    SELECT b.batch_id, b.service_id FROM batches b
                    LEFT JOIN batch_statuses bs ON bs.batch_id = b.batch_id AND bs.service_id = b.service_id
                    WHERE bs.dlt_status = 'Delayed' OR bs.dlt_status = 'Unknown' OR b.submitted = false
                )
                SELECT * FROM submissions s
                WHERE (s.service_id, s.batch_id) IN (SELECT service_id, batch_id FROM bbs);"
            )
            .load(self.conn)?;

            let txn_models: Vec<TransactionModel> = sql_query(
                "WITH bbs AS (
                    SELECT b.batch_id, b.service_id FROM batches b
                    LEFT JOIN batch_statuses bs ON bs.batch_id = b.batch_id AND bs.service_id = b.service_id
                    WHERE bs.dlt_status = 'Delayed' OR bs.dlt_status = 'Unknown' OR b.submitted = false
                )
                SELECT * FROM transactions t
                WHERE (t.service_id, t.batch_id) IN (SELECT service_id, batch_id FROM bbs);"
            )
            .load(self.conn)?;

            let receipt_models: Vec<TransactionReceiptModel> = sql_query(
                "WITH bbs AS (
                    SELECT b.batch_id, b.service_id FROM batches b
                    LEFT JOIN batch_statuses bs ON bs.batch_id = b.batch_id AND bs.service_id = b.service_id
                    WHERE bs.dlt_status = 'Delayed' OR bs.dlt_status = 'Unknown' OR b.submitted = false
                ), txn_models AS (
                    SELECT t.transaction_id, t.service_id FROM transactions t
                    WHERE (t.service_id, t.batch_id) IN (SELECT service_id, batch_id FROM bbs)
                )
                SELECT * FROM transaction_receipts tr
                WHERE (tr.service_id, tr.transaction_id) IN (SELECT service_id, transaction_id FROM txn_models);"
            )
            .load(self.conn)?;

            let batches = TrackingBatchList::try_from((batch_models, batch_status_models, txn_models, receipt_models, submission_models))?;
            Ok(batches)
        })
    }
}
