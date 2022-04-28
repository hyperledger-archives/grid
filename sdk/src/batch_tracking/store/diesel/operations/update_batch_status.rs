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
        models::{
            is_data_change_id, NewBatchStatusModel, NewSubmissionModel, TransactionReceiptModel,
        },
        schema::{batch_statuses, batches, submissions, transaction_receipts},
    },
    BatchStatusName, BatchTrackingStoreError, SubmissionError,
};

use diesel::{
    dsl::{exists, insert_into, update},
    prelude::*,
    select,
};

pub(in crate::batch_tracking::store::diesel) trait BatchTrackingStoreUpdateBatchStatusOperation {
    fn update_batch_status(
        &self,
        id: &str,
        service_id: &str,
        status: Option<&str>,
        txn_receipts: Vec<TransactionReceiptModel>,
        submission_error: Option<SubmissionError>,
    ) -> Result<(), BatchTrackingStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> BatchTrackingStoreUpdateBatchStatusOperation
    for BatchTrackingStoreOperations<'a, diesel::pg::PgConnection>
{
    fn update_batch_status(
        &self,
        id: &str,
        service_id: &str,
        status: Option<&str>,
        txn_receipts: Vec<TransactionReceiptModel>,
        submission_error: Option<SubmissionError>,
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

            if let Some(batch_status) = status {
                let status_string = BatchStatusName::try_from_string(batch_status)?;
                match status_string {
                    BatchStatusName::Pending
                    | BatchStatusName::Invalid
                    | BatchStatusName::Valid
                    | BatchStatusName::Committed => {
                        update(batches::table)
                            .filter(
                                batches::batch_id
                                    .eq(&batch_id)
                                    .and(batches::service_id.eq(&service_id)),
                            )
                            .set(batches::submitted.eq(true))
                            .execute(self.conn)?;
                    }
                    BatchStatusName::Delayed | BatchStatusName::Unknown => {
                        update(batches::table)
                            .filter(
                                batches::batch_id
                                    .eq(&id)
                                    .and(batches::service_id.eq(&service_id)),
                            )
                            .set(batches::submitted.eq(false))
                            .execute(self.conn)?;
                    }
                }

                let status_exists: bool = select(exists(
                    batch_statuses::table.filter(
                        batch_statuses::batch_id
                            .eq(&batch_id)
                            .and(batch_statuses::service_id.eq(&service_id)),
                    ),
                ))
                .get_result(self.conn)?;

                if status_exists {
                    update(batch_statuses::table)
                        .filter(
                            batch_statuses::batch_id
                                .eq(&batch_id)
                                .and(batch_statuses::service_id.eq(&service_id)),
                        )
                        .set(batch_statuses::dlt_status.eq(&batch_status))
                        .execute(self.conn)?;
                } else {
                    let model = NewBatchStatusModel {
                        batch_id: batch_id.to_string(),
                        service_id: service_id.to_string(),
                        dlt_status: batch_status.to_string(),
                    };

                    insert_into(batch_statuses::table)
                        .values(model)
                        .execute(self.conn)?;
                };
            } else {
                update(batches::table)
                    .filter(
                        batches::batch_id
                            .eq(&batch_id)
                            .and(batches::service_id.eq(&service_id)),
                    )
                    .set(batches::submitted.eq(true))
                    .execute(self.conn)?;
            }

            let rcpt_ids = txn_receipts
                .iter()
                .map(|t| t.transaction_id.to_string())
                .collect::<Vec<String>>();

            let existing_rcpts: Vec<String> = transaction_receipts::table
                .into_boxed()
                .select(transaction_receipts::transaction_id)
                .filter(
                    transaction_receipts::transaction_id
                        .eq_any(rcpt_ids)
                        .and(transaction_receipts::service_id.eq(&service_id)),
                )
                .load(self.conn)?;

            for r in txn_receipts {
                if existing_rcpts.contains(&r.transaction_id) {
                    update(transaction_receipts::table)
                        .filter(
                            transaction_receipts::transaction_id
                                .eq(&r.transaction_id)
                                .and(transaction_receipts::service_id.eq(&service_id)),
                        )
                        .set(r.clone())
                        .execute(self.conn)?;
                } else {
                    insert_into(transaction_receipts::table)
                        .values(r.clone())
                        .execute(self.conn)?;
                }
            }

            if let Some(s) = submission_error {
                let bid: &str = &batch_id;
                let model = NewSubmissionModel::from((s, bid, service_id));
                let submission_exists = select(exists(
                    submissions::table.filter(
                        submissions::batch_id
                            .eq(&model.batch_id)
                            .and(submissions::service_id.eq(&model.service_id)),
                    ),
                ))
                .get_result(self.conn)?;

                if submission_exists {
                    update(submissions::table)
                        .filter(
                            submissions::batch_id
                                .eq(&model.batch_id)
                                .and(submissions::service_id.eq(&model.service_id)),
                        )
                        .set(&model)
                        .execute(self.conn)?;
                } else {
                    insert_into(submissions::table)
                        .values(&model)
                        .execute(self.conn)?;
                }
            }

            Ok(())
        })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> BatchTrackingStoreUpdateBatchStatusOperation
    for BatchTrackingStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn update_batch_status(
        &self,
        id: &str,
        service_id: &str,
        status: Option<&str>,
        txn_receipts: Vec<TransactionReceiptModel>,
        submission_error: Option<SubmissionError>,
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

            if let Some(batch_status) = status {
                let status_string = BatchStatusName::try_from_string(batch_status)?;
                match status_string {
                    BatchStatusName::Pending
                    | BatchStatusName::Invalid
                    | BatchStatusName::Valid
                    | BatchStatusName::Committed => {
                        update(batches::table)
                            .filter(
                                batches::batch_id
                                    .eq(&batch_id)
                                    .and(batches::service_id.eq(&service_id)),
                            )
                            .set(batches::submitted.eq(true))
                            .execute(self.conn)?;
                    }
                    BatchStatusName::Delayed | BatchStatusName::Unknown => {
                        update(batches::table)
                            .filter(
                                batches::batch_id
                                    .eq(&id)
                                    .and(batches::service_id.eq(&service_id)),
                            )
                            .set(batches::submitted.eq(false))
                            .execute(self.conn)?;
                    }
                }

                let status_exists: bool = select(exists(
                    batch_statuses::table.filter(
                        batch_statuses::batch_id
                            .eq(&batch_id)
                            .and(batch_statuses::service_id.eq(&service_id)),
                    ),
                ))
                .get_result(self.conn)?;

                if status_exists {
                    update(batch_statuses::table)
                        .filter(
                            batch_statuses::batch_id
                                .eq(&batch_id)
                                .and(batch_statuses::service_id.eq(&service_id)),
                        )
                        .set(batch_statuses::dlt_status.eq(&batch_status))
                        .execute(self.conn)?;
                } else {
                    let model = NewBatchStatusModel {
                        batch_id: batch_id.to_string(),
                        service_id: service_id.to_string(),
                        dlt_status: batch_status.to_string(),
                    };

                    insert_into(batch_statuses::table)
                        .values(model)
                        .execute(self.conn)?;
                };
            } else {
                update(batches::table)
                    .filter(
                        batches::batch_id
                            .eq(&batch_id)
                            .and(batches::service_id.eq(&service_id)),
                    )
                    .set(batches::submitted.eq(true))
                    .execute(self.conn)?;
            }

            let rcpt_ids = txn_receipts
                .iter()
                .map(|t| t.transaction_id.to_string())
                .collect::<Vec<String>>();

            let existing_rcpts: Vec<String> = transaction_receipts::table
                .into_boxed()
                .select(transaction_receipts::transaction_id)
                .filter(
                    transaction_receipts::transaction_id
                        .eq_any(rcpt_ids)
                        .and(transaction_receipts::service_id.eq(&service_id)),
                )
                .load(self.conn)?;

            for r in txn_receipts {
                if existing_rcpts.contains(&r.transaction_id) {
                    update(transaction_receipts::table)
                        .filter(
                            transaction_receipts::transaction_id
                                .eq(&r.transaction_id)
                                .and(transaction_receipts::service_id.eq(&service_id)),
                        )
                        .set(r.clone())
                        .execute(self.conn)?;
                } else {
                    insert_into(transaction_receipts::table)
                        .values(r.clone())
                        .execute(self.conn)?;
                }
            }

            if let Some(s) = submission_error {
                let bid: &str = &batch_id;
                let model = NewSubmissionModel::from((s, bid, service_id));
                let submission_exists = select(exists(
                    submissions::table.filter(
                        submissions::batch_id
                            .eq(&model.batch_id)
                            .and(submissions::service_id.eq(&model.service_id)),
                    ),
                ))
                .get_result(self.conn)?;

                if submission_exists {
                    update(submissions::table)
                        .filter(
                            submissions::batch_id
                                .eq(&model.batch_id)
                                .and(submissions::service_id.eq(&model.service_id)),
                        )
                        .set(&model)
                        .execute(self.conn)?;
                } else {
                    insert_into(submissions::table)
                        .values(&model)
                        .execute(self.conn)?;
                }
            }

            Ok(())
        })
    }
}
