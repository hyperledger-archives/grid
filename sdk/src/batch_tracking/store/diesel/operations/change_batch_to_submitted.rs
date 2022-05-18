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

use crate::batch_tracking::store::{
    diesel::{
        models::{
            is_data_change_id, NewBatchStatusModel, NewSubmissionModel, TransactionModel,
            TransactionReceiptModel,
        },
        schema::{batch_statuses, batches, submissions, transaction_receipts, transactions},
    },
    BatchStatus, BatchStatusName, BatchTrackingStoreError,
};
use diesel::{
    dsl::{exists, insert_into, update},
    prelude::*,
    select,
};

pub(in crate::batch_tracking::store::diesel) trait BatchTrackingStoreChangeBatchToSubmittedOperation
{
    fn change_batch_to_submitted(
        &self,
        id: &str,
        service_id: &str,
        txn_receipts: Vec<TransactionReceiptModel>,
        status: Option<NewBatchStatusModel>,
        submission: NewSubmissionModel,
    ) -> Result<(), BatchTrackingStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> BatchTrackingStoreChangeBatchToSubmittedOperation
    for BatchTrackingStoreOperations<'a, diesel::pg::PgConnection>
{
    fn change_batch_to_submitted(
        &self,
        id: &str,
        service_id: &str,
        txn_receipts: Vec<TransactionReceiptModel>,
        status: Option<NewBatchStatusModel>,
        submission: NewSubmissionModel,
    ) -> Result<(), BatchTrackingStoreError> {
        self.conn.transaction::<_, BatchTrackingStoreError, _>(|| {
            let mut batch_id = id.to_string();
            let is_dcid = is_data_change_id(id)?;
            if is_dcid {
                batch_id = batches::table
                    .select(batches::batch_id)
                    .filter(
                        batches::data_change_id
                            .eq(&id)
                            .and(batches::service_id.eq(&service_id)),
                    )
                    .first::<String>(self.conn)?;
            }

            let batch_exists: bool = select(exists(
                batches::table.filter(
                    batches::batch_id
                        .eq(&batch_id)
                        .and(batches::service_id.eq(&service_id)),
                ),
            ))
            .get_result(self.conn)?;

            if !batch_exists {
                return Err(BatchTrackingStoreError::NotFoundError(format!(
                    "Could not find batch with ID {}",
                    batch_id
                )));
            }

            let txns = transactions::table
                .into_boxed()
                .filter(
                    transactions::batch_id
                        .eq(&batch_id)
                        .and(transactions::service_id.eq(&service_id)),
                )
                .select(transactions::all_columns)
                .load::<TransactionModel>(self.conn)?;

            if let Some(batch_status) = status {
                let status_string = BatchStatusName::try_from_string(&batch_status.dlt_status)?;
                match status_string {
                    BatchStatusName::Pending
                    | BatchStatusName::Invalid
                    | BatchStatusName::Valid
                    | BatchStatusName::Committed => {
                        let status_exists = select(exists(
                            batch_statuses::table.filter(
                                batch_statuses::batch_id
                                    .eq(&batch_status.batch_id)
                                    .and(batch_statuses::service_id.eq(&batch_status.service_id)),
                            ),
                        ))
                        .get_result(self.conn)?;

                        if status_exists {
                            update(batch_statuses::table)
                                .filter(
                                    batch_statuses::batch_id.eq(&batch_status.batch_id).and(
                                        batch_statuses::service_id.eq(&batch_status.service_id),
                                    ),
                                )
                                .set(&batch_status)
                                .execute(self.conn)?;
                        } else {
                            insert_into(batch_statuses::table)
                                .values(&batch_status)
                                .execute(self.conn)?;
                        }

                        if txns.len() != txn_receipts.len()
                            && batch_status.dlt_status != BatchStatus::Pending.to_string()
                        {
                            return Err(BatchTrackingStoreError::InternalError(
                                InternalError::with_message(
                                    "Receipts for all transactions must be provided".to_string(),
                                ),
                            ));
                        }
                    }
                    _ => {
                        return Err(BatchTrackingStoreError::NotFoundError(format!(
                            "Status {} is not a submitted status",
                            batch_status.dlt_status
                        )));
                    }
                }
            }

            for rcpt in txn_receipts {
                let exists = select(exists(
                    transaction_receipts::table.filter(
                        transaction_receipts::transaction_id
                            .eq(&rcpt.transaction_id)
                            .and(transaction_receipts::service_id.eq(&rcpt.service_id)),
                    ),
                ))
                .get_result(self.conn)?;

                if exists {
                    update(transaction_receipts::table)
                        .filter(
                            transaction_receipts::transaction_id
                                .eq(&rcpt.transaction_id)
                                .and(transaction_receipts::service_id.eq(&rcpt.service_id)),
                        )
                        .set(&rcpt)
                        .execute(self.conn)?;
                } else {
                    insert_into(transaction_receipts::table)
                        .values(&rcpt)
                        .execute(self.conn)?;
                }
            }

            let submission_exists = select(exists(
                submissions::table.filter(
                    submissions::batch_id
                        .eq(&submission.batch_id)
                        .and(submissions::service_id.eq(&submission.service_id)),
                ),
            ))
            .get_result(self.conn)?;

            if submission_exists {
                update(submissions::table)
                    .filter(
                        submissions::batch_id
                            .eq(&submission.batch_id)
                            .and(submissions::service_id.eq(&submission.service_id)),
                    )
                    .set(&submission)
                    .execute(self.conn)?;
            } else {
                insert_into(submissions::table)
                    .values(&submission)
                    .execute(self.conn)?;
            }

            update(batches::table)
                .filter(
                    batches::batch_id
                        .eq(&batch_id)
                        .and(batches::service_id.eq(&service_id)),
                )
                .set(batches::submitted.eq(true))
                .execute(self.conn)?;

            Ok(())
        })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> BatchTrackingStoreChangeBatchToSubmittedOperation
    for BatchTrackingStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn change_batch_to_submitted(
        &self,
        id: &str,
        service_id: &str,
        txn_receipts: Vec<TransactionReceiptModel>,
        status: Option<NewBatchStatusModel>,
        submission: NewSubmissionModel,
    ) -> Result<(), BatchTrackingStoreError> {
        self.conn.transaction::<_, BatchTrackingStoreError, _>(|| {
            let mut batch_id = id.to_string();
            let is_dcid = is_data_change_id(id)?;
            if is_dcid {
                batch_id = batches::table
                    .select(batches::batch_id)
                    .filter(
                        batches::data_change_id
                            .eq(&id)
                            .and(batches::service_id.eq(&service_id)),
                    )
                    .first::<String>(self.conn)?;
            }
            let batch_exists: bool = select(exists(
                batches::table.filter(
                    batches::batch_id
                        .eq(&batch_id)
                        .and(batches::service_id.eq(&service_id)),
                ),
            ))
            .get_result(self.conn)?;

            if !batch_exists {
                return Err(BatchTrackingStoreError::NotFoundError(format!(
                    "Could not find batch with ID {}",
                    batch_id
                )));
            }

            let txns = transactions::table
                .into_boxed()
                .filter(
                    transactions::batch_id
                        .eq(&batch_id)
                        .and(transactions::service_id.eq(&service_id)),
                )
                .select(transactions::all_columns)
                .load::<TransactionModel>(self.conn)?;

            if let Some(batch_status) = status {
                let status_string = BatchStatusName::try_from_string(&batch_status.dlt_status)?;
                match status_string {
                    BatchStatusName::Pending
                    | BatchStatusName::Invalid
                    | BatchStatusName::Valid
                    | BatchStatusName::Committed => {
                        let status_exists = select(exists(
                            batch_statuses::table.filter(
                                batch_statuses::batch_id
                                    .eq(&batch_status.batch_id)
                                    .and(batch_statuses::service_id.eq(&batch_status.service_id)),
                            ),
                        ))
                        .get_result(self.conn)?;

                        if status_exists {
                            update(batch_statuses::table)
                                .filter(
                                    batch_statuses::batch_id.eq(&batch_status.batch_id).and(
                                        batch_statuses::service_id.eq(&batch_status.service_id),
                                    ),
                                )
                                .set(&batch_status)
                                .execute(self.conn)?;
                        } else {
                            insert_into(batch_statuses::table)
                                .values(&batch_status)
                                .execute(self.conn)?;
                        }

                        if txns.len() != txn_receipts.len()
                            && batch_status.dlt_status != BatchStatus::Pending.to_string()
                        {
                            return Err(BatchTrackingStoreError::InternalError(
                                InternalError::with_message(
                                    "Receipts for all transactions must be provided".to_string(),
                                ),
                            ));
                        }
                    }
                    _ => {
                        return Err(BatchTrackingStoreError::NotFoundError(format!(
                            "Status {} is not a submitted status",
                            batch_status.dlt_status
                        )));
                    }
                }
            }

            for rcpt in txn_receipts {
                let exists = select(exists(
                    transaction_receipts::table.filter(
                        transaction_receipts::transaction_id
                            .eq(&rcpt.transaction_id)
                            .and(transaction_receipts::service_id.eq(&rcpt.service_id)),
                    ),
                ))
                .get_result(self.conn)?;

                if exists {
                    update(transaction_receipts::table)
                        .filter(
                            transaction_receipts::transaction_id
                                .eq(&rcpt.transaction_id)
                                .and(transaction_receipts::service_id.eq(&rcpt.service_id)),
                        )
                        .set(&rcpt)
                        .execute(self.conn)?;
                } else {
                    insert_into(transaction_receipts::table)
                        .values(&rcpt)
                        .execute(self.conn)?;
                }
            }

            let submission_exists = select(exists(
                submissions::table.filter(
                    submissions::batch_id
                        .eq(&submission.batch_id)
                        .and(submissions::service_id.eq(&submission.service_id)),
                ),
            ))
            .get_result(self.conn)?;

            if submission_exists {
                update(submissions::table)
                    .filter(
                        submissions::batch_id
                            .eq(&submission.batch_id)
                            .and(submissions::service_id.eq(&submission.service_id)),
                    )
                    .set(&submission)
                    .execute(self.conn)?;
            } else {
                insert_into(submissions::table)
                    .values(&submission)
                    .execute(self.conn)?;
            }

            update(batches::table)
                .filter(
                    batches::batch_id
                        .eq(&batch_id)
                        .and(batches::service_id.eq(&service_id)),
                )
                .set(batches::submitted.eq(true))
                .execute(self.conn)?;

            Ok(())
        })
    }
}
