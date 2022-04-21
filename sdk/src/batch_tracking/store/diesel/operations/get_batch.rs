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
use crate::error::InternalError;

use crate::batch_tracking::store::diesel::{
    models::{
        BatchModel, BatchStatusModel, SubmissionModel, TransactionModel, TransactionReceiptModel,
    },
    schema::{batch_statuses, batches, submissions, transaction_receipts, transactions},
    BatchStatus, InvalidTransaction, SubmissionError, TrackingBatch, TrackingTransaction,
    TransactionReceipt, ValidTransaction,
};

use crate::batch_tracking::store::BatchTrackingStoreError;
use diesel::prelude::*;
use std::convert::TryFrom;

pub(in crate::batch_tracking::store::diesel) trait BatchTrackingStoreGetBatchOperation {
    fn get_batch(
        &self,
        id: &str,
        service_id: &str,
    ) -> Result<Option<TrackingBatch>, BatchTrackingStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> BatchTrackingStoreGetBatchOperation
    for BatchTrackingStoreOperations<'a, diesel::pg::PgConnection>
{
    fn get_batch(
        &self,
        id: &str,
        service_id: &str,
    ) -> Result<Option<TrackingBatch>, BatchTrackingStoreError> {
        self.conn.transaction::<_, BatchTrackingStoreError, _>(|| {
            // This performs a query to select all columns from the batches,
            // batch_statuses, and submissions tables joined on the batch_id
            // column. These rows are then filtered on the batch_id.
            let query = batches::table
                .into_boxed()
                .left_join(
                    batch_statuses::table.on(batches::batch_id
                        .eq(batch_statuses::batch_id)
                        .and(batches::service_id.eq(batch_statuses::service_id))),
                )
                .left_join(
                    submissions::table.on(batches::batch_id
                        .eq(submissions::batch_id)
                        .and(batches::service_id.eq(submissions::service_id))),
                )
                .filter(
                    batches::batch_id
                        .eq(&id)
                        .and(batches::service_id.eq(&service_id)),
                )
                .select((
                    batches::all_columns,
                    batch_statuses::all_columns.nullable(),
                    submissions::all_columns.nullable(),
                ));

            // Diesel will deserialize the joined results into the respective
            // models for the tables in the join.
            let batch_result: Option<(
                BatchModel,
                Option<BatchStatusModel>,
                Option<SubmissionModel>,
            )> = query
                .first::<(
                    BatchModel,
                    Option<BatchStatusModel>,
                    Option<SubmissionModel>,
                )>(self.conn)
                .optional()
                .map_err(|err| {
                    BatchTrackingStoreError::InternalError(InternalError::from_source(Box::new(
                        err,
                    )))
                })?;

            // This query is used to fetch the transactions for a given batch
            // ID. These will be used to construct the `TrackingBatch` struct
            // that is returned to the user and to fetch transaction receipts.
            let query = transactions::table
                .into_boxed()
                .select(transactions::all_columns)
                .filter(
                    transactions::batch_id
                        .eq(&id)
                        .and(transactions::service_id.eq(&service_id)),
                );

            let txn_models: Vec<TransactionModel> =
                query.load::<TransactionModel>(self.conn).map_err(|err| {
                    BatchTrackingStoreError::InternalError(InternalError::from_source(Box::new(
                        err,
                    )))
                })?;

            let mut txns = Vec::new();
            let mut txn_ids = Vec::new();
            let mut valid_txns = Vec::new();
            let mut invalid_txns = Vec::new();

            for t in txn_models {
                txns.push(TrackingTransaction::from(&t));
                txn_ids.push(t.transaction_id.to_string());
            }

            // This query fetches the transaction receipts for the transactions
            // in the batch. These are used to build the valid and invalid
            // transaction structs that are used to build the batch status.
            let query = transaction_receipts::table
                .into_boxed()
                .filter(transaction_receipts::transaction_id.eq_any(txn_ids));

            let receipt_results: Vec<TransactionReceiptModel> = query
                .load::<TransactionReceiptModel>(self.conn)
                .map_err(|err| {
                    BatchTrackingStoreError::InternalError(InternalError::from_source(Box::new(
                        err,
                    )))
                })?;

            for rcpt in receipt_results {
                if rcpt.result_valid {
                    valid_txns.push(ValidTransaction::try_from(TransactionReceipt::from(rcpt))?);
                } else {
                    invalid_txns.push(InvalidTransaction::try_from(TransactionReceipt::from(
                        rcpt,
                    ))?);
                }
            }

            if let Some(res) = batch_result {
                let (b, stat, sub) = res;
                {
                    let sub_err: Option<SubmissionError> = if let Some(sub) = sub {
                        if sub.error_type.is_some() && sub.error_message.is_some() {
                            Some(SubmissionError::try_from(sub)?)
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    let status = if let Some(s) = stat {
                        let grid_status = BatchStatus::try_from((s, invalid_txns, valid_txns))?;
                        Some(grid_status)
                    } else {
                        None
                    };

                    return Ok(Some(TrackingBatch::from((b, txns, status, sub_err))));
                }
            }

            Ok(None)
        })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> BatchTrackingStoreGetBatchOperation
    for BatchTrackingStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn get_batch(
        &self,
        id: &str,
        service_id: &str,
    ) -> Result<Option<TrackingBatch>, BatchTrackingStoreError> {
        self.conn.transaction::<_, BatchTrackingStoreError, _>(|| {
            // This performs a query to select all columns from the batches,
            // batch_statuses, and submissions tables joined on the batch_id
            // column. These rows are then filtered on the batch_id.
            let query = batches::table
                .into_boxed()
                .left_join(
                    batch_statuses::table.on(batches::batch_id
                        .eq(batch_statuses::batch_id)
                        .and(batches::service_id.eq(batch_statuses::service_id))),
                )
                .left_join(
                    submissions::table.on(batches::batch_id
                        .eq(submissions::batch_id)
                        .and(batches::service_id.eq(submissions::service_id))),
                )
                .filter(
                    batches::batch_id
                        .eq(&id)
                        .and(batches::service_id.eq(&service_id)),
                )
                .select((
                    batches::all_columns,
                    batch_statuses::all_columns.nullable(),
                    submissions::all_columns.nullable(),
                ));

            // Diesel will deserialize the joined results into the respective
            // models for the tables in the join.
            let batch_result: Option<(
                BatchModel,
                Option<BatchStatusModel>,
                Option<SubmissionModel>,
            )> = query
                .first::<(
                    BatchModel,
                    Option<BatchStatusModel>,
                    Option<SubmissionModel>,
                )>(self.conn)
                .optional()
                .map_err(|err| {
                    BatchTrackingStoreError::InternalError(InternalError::from_source(Box::new(
                        err,
                    )))
                })?;

            // This query is used to fetch the transactions for a given batch
            // ID. These will be used to construct the `TrackingBatch` struct
            // that is returned to the user and to fetch transaction receipts.
            let query = transactions::table
                .into_boxed()
                .select(transactions::all_columns)
                .filter(
                    transactions::batch_id
                        .eq(&id)
                        .and(transactions::service_id.eq(&service_id)),
                );

            let txn_models: Vec<TransactionModel> =
                query.load::<TransactionModel>(self.conn).map_err(|err| {
                    BatchTrackingStoreError::InternalError(InternalError::from_source(Box::new(
                        err,
                    )))
                })?;

            let mut txns = Vec::new();
            let mut txn_ids = Vec::new();
            let mut valid_txns = Vec::new();
            let mut invalid_txns = Vec::new();

            for t in txn_models {
                txns.push(TrackingTransaction::from(&t));
                txn_ids.push(t.transaction_id.to_string());
            }

            // This query fetches the transaction receipts for the transactions
            // in the batch. These are used to build the valid and invalid
            // transaction structs that are used to build the batch status.
            let query = transaction_receipts::table
                .into_boxed()
                .filter(transaction_receipts::transaction_id.eq_any(txn_ids));

            let receipt_results: Vec<TransactionReceiptModel> = query
                .load::<TransactionReceiptModel>(self.conn)
                .map_err(|err| {
                    BatchTrackingStoreError::InternalError(InternalError::from_source(Box::new(
                        err,
                    )))
                })?;

            for rcpt in receipt_results {
                if rcpt.result_valid {
                    valid_txns.push(ValidTransaction::try_from(TransactionReceipt::from(rcpt))?);
                } else {
                    invalid_txns.push(InvalidTransaction::try_from(TransactionReceipt::from(
                        rcpt,
                    ))?);
                }
            }

            if let Some(res) = batch_result {
                let (b, stat, sub) = res;
                {
                    let sub_err: Option<SubmissionError> = if let Some(sub) = sub {
                        if sub.error_type.is_some() && sub.error_message.is_some() {
                            Some(SubmissionError::try_from(sub)?)
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    let status = if let Some(s) = stat {
                        let grid_status = BatchStatus::try_from((s, invalid_txns, valid_txns))?;
                        Some(grid_status)
                    } else {
                        None
                    };

                    return Ok(Some(TrackingBatch::from((b, txns, status, sub_err))));
                }
            }

            Ok(None)
        })
    }
}
