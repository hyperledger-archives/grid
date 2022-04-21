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
    models::{BatchStatusModel, TransactionReceiptModel},
    schema::{batch_statuses, transaction_receipts, transactions},
    BatchStatus, InvalidTransaction, TransactionReceipt, ValidTransaction,
};

use crate::batch_tracking::store::BatchTrackingStoreError;
use diesel::prelude::*;
use std::convert::TryFrom;

pub(in crate::batch_tracking::store::diesel) trait BatchTrackingStoreGetBatchStatusOperation {
    fn get_batch_status(
        &self,
        id: &str,
        service_id: &str,
    ) -> Result<Option<BatchStatus>, BatchTrackingStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> BatchTrackingStoreGetBatchStatusOperation
    for BatchTrackingStoreOperations<'a, diesel::pg::PgConnection>
{
    fn get_batch_status(
        &self,
        id: &str,
        service_id: &str,
    ) -> Result<Option<BatchStatus>, BatchTrackingStoreError> {
        self.conn.transaction::<_, BatchTrackingStoreError, _>(|| {
            // This query fetches the batch status for the batch with the given
            // batch ID
            let batch_status_query = batch_statuses::table
                .into_boxed()
                .select(batch_statuses::all_columns)
                .filter(
                    batch_statuses::batch_id
                        .eq(&id)
                        .and(batch_statuses::service_id.eq(&service_id)),
                );

            let batch_status_model: Option<BatchStatusModel> = batch_status_query
                .first::<BatchStatusModel>(self.conn)
                .optional()
                .map_err(|err| {
                    BatchTrackingStoreError::InternalError(InternalError::from_source(Box::new(
                        err,
                    )))
                })?;

            if batch_status_model.is_none() {
                return Ok(None);
            }

            // This query fetches the transactions and any associated receipts
            // for the given batch ID
            let txn_query = transactions::table
                .into_boxed()
                .left_join(
                    transaction_receipts::table.on(transaction_receipts::transaction_id
                        .eq(transactions::transaction_id)
                        .and(transaction_receipts::service_id.eq(transactions::service_id))),
                )
                .filter(
                    transactions::batch_id
                        .eq(&id)
                        .and(transactions::service_id.eq(&service_id)),
                )
                .select((
                    transactions::transaction_id,
                    transaction_receipts::all_columns.nullable(),
                ));

            let txn_query_result: Vec<(String, Option<TransactionReceiptModel>)> = txn_query
                .load::<(String, Option<TransactionReceiptModel>)>(self.conn)
                .map_err(|err| {
                    BatchTrackingStoreError::InternalError(InternalError::from_source(Box::new(
                        err,
                    )))
                })?;

            let mut invalid_txns = Vec::new();
            let mut valid_txns = Vec::new();

            for (_, rcpt) in &txn_query_result {
                if let Some(r) = rcpt {
                    if r.result_valid {
                        valid_txns.push(
                            ValidTransaction::try_from(TransactionReceipt::from(r)).map_err(
                                |err| {
                                    BatchTrackingStoreError::InternalError(
                                        InternalError::from_source(Box::new(err)),
                                    )
                                },
                            )?,
                        );
                    } else {
                        invalid_txns.push(
                            InvalidTransaction::try_from(TransactionReceipt::from(r)).map_err(
                                |err| {
                                    BatchTrackingStoreError::InternalError(
                                        InternalError::from_source(Box::new(err)),
                                    )
                                },
                            )?,
                        );
                    }
                }
            }

            let batch_status = batch_status_model.unwrap();

            let status = BatchStatus::try_from((batch_status, invalid_txns, valid_txns))?;

            Ok(Some(status))
        })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> BatchTrackingStoreGetBatchStatusOperation
    for BatchTrackingStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn get_batch_status(
        &self,
        id: &str,
        service_id: &str,
    ) -> Result<Option<BatchStatus>, BatchTrackingStoreError> {
        self.conn.transaction::<_, BatchTrackingStoreError, _>(|| {
            // This query fetches the batch status for the batch with the given
            // batch ID
            let batch_status_query = batch_statuses::table
                .into_boxed()
                .select(batch_statuses::all_columns)
                .filter(
                    batch_statuses::batch_id
                        .eq(&id)
                        .and(batch_statuses::service_id.eq(&service_id)),
                );

            let batch_status_model: Option<BatchStatusModel> = batch_status_query
                .first::<BatchStatusModel>(self.conn)
                .optional()
                .map_err(|err| {
                    BatchTrackingStoreError::InternalError(InternalError::from_source(Box::new(
                        err,
                    )))
                })?;

            if batch_status_model.is_none() {
                return Ok(None);
            }

            // This query fetches the transactions and any associated receipts
            // for the given batch ID
            let txn_query = transactions::table
                .into_boxed()
                .left_join(
                    transaction_receipts::table.on(transaction_receipts::transaction_id
                        .eq(transactions::transaction_id)
                        .and(transaction_receipts::service_id.eq(transactions::service_id))),
                )
                .filter(
                    transactions::batch_id
                        .eq(&id)
                        .and(transactions::service_id.eq(&service_id)),
                )
                .select((
                    transactions::transaction_id,
                    transaction_receipts::all_columns.nullable(),
                ));

            let txn_query_result: Vec<(String, Option<TransactionReceiptModel>)> = txn_query
                .load::<(String, Option<TransactionReceiptModel>)>(self.conn)
                .map_err(|err| {
                    BatchTrackingStoreError::InternalError(InternalError::from_source(Box::new(
                        err,
                    )))
                })?;

            let mut invalid_txns = Vec::new();
            let mut valid_txns = Vec::new();

            for (_, rcpt) in &txn_query_result {
                if let Some(r) = rcpt {
                    if r.result_valid {
                        valid_txns.push(
                            ValidTransaction::try_from(TransactionReceipt::from(r)).map_err(
                                |err| {
                                    BatchTrackingStoreError::InternalError(
                                        InternalError::from_source(Box::new(err)),
                                    )
                                },
                            )?,
                        );
                    } else {
                        invalid_txns.push(
                            InvalidTransaction::try_from(TransactionReceipt::from(r)).map_err(
                                |err| {
                                    BatchTrackingStoreError::InternalError(
                                        InternalError::from_source(Box::new(err)),
                                    )
                                },
                            )?,
                        );
                    }
                }
            }

            let batch_status = batch_status_model.unwrap();

            let status = BatchStatus::try_from((batch_status, invalid_txns, valid_txns))?;

            Ok(Some(status))
        })
    }
}
