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

use core::convert::TryFrom;

use crate::batch_tracking::store::diesel::schema::*;
use crate::error::InternalError;

use super::{
    BatchStatus, InvalidTransaction, SubmissionError, TrackingBatch, TrackingTransaction,
    TransactionReceipt, ValidTransaction,
};
use crate::batch_tracking::store::error::BatchTrackingStoreError;

#[derive(Insertable, Queryable, PartialEq, Debug)]
#[table_name = "batches"]
pub struct BatchModel {
    pub service_id: String,
    pub batch_id: String,
    pub data_change_id: Option<String>,
    pub signer_public_key: String,
    pub trace: bool,
    pub serialized_batch: Vec<u8>,
    pub submitted: bool,
    pub created_at: i64,
}

#[derive(Insertable, Queryable, PartialEq, Debug)]
#[table_name = "transactions"]
pub struct TransactionModel {
    pub service_id: String,
    pub transaction_id: String,
    pub batch_id: String,
    pub batch_service_id: String,
    pub payload: Vec<u8>,
    pub family_name: String,
    pub family_version: String,
    pub signer_public_key: String,
}

#[derive(Insertable, Queryable, PartialEq, Debug, Clone)]
#[table_name = "transaction_receipts"]
pub struct TransactionReceiptModel {
    pub service_id: String,
    pub transaction_id: String,
    pub result_valid: bool,
    pub error_message: Option<String>,
    pub error_data: Option<Vec<u8>>,
    pub serialized_receipt: Vec<u8>,
    pub external_status: Option<String>,
    pub external_error_message: Option<String>,
}

#[derive(Insertable, Debug)]
#[table_name = "batch_statuses"]
pub struct NewBatchStatusModel {
    pub service_id: String,
    pub batch_id: String,
    pub batch_service_id: String,
    pub dlt_status: String,
}

#[derive(Insertable, Queryable, PartialEq, Debug, Clone)]
#[table_name = "batch_statuses"]
pub struct BatchStatusModel {
    pub service_id: String,
    pub batch_id: String,
    pub batch_service_id: String,
    pub dlt_status: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "submissions"]
pub struct NewSubmissionModel {
    pub service_id: String,
    pub batch_service_id: String,
    pub last_checked: Option<i64>,
    pub times_checked: Option<String>,
    pub error_type: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Insertable, Queryable, PartialEq, Debug)]
#[table_name = "submissions"]
pub struct SubmissionModel {
    pub service_id: String,
    pub batch_id: String,
    pub batch_service_id: String,
    pub last_checked: Option<i64>,
    pub times_checked: Option<String>,
    pub error_type: Option<String>,
    pub error_message: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl
    From<(
        BatchModel,
        Vec<TrackingTransaction>,
        Option<BatchStatus>,
        Option<SubmissionError>,
    )> for TrackingBatch
{
    fn from(
        (batch, transactions, batch_status, submission_error): (
            BatchModel,
            Vec<TrackingTransaction>,
            Option<BatchStatus>,
            Option<SubmissionError>,
        ),
    ) -> Self {
        Self {
            service_id: batch.service_id.to_string(),
            batch_header: batch.batch_id.to_string(),
            data_change_id: batch.data_change_id.clone(),
            signer_public_key: batch.signer_public_key.to_string(),
            trace: batch.trace,
            serialized_batch: batch.serialized_batch.to_vec(),
            submitted: batch.submitted,
            created_at: batch.created_at,
            transactions,
            batch_status,
            submission_error,
        }
    }
}

impl From<TransactionModel> for TrackingTransaction {
    fn from(transaction: TransactionModel) -> Self {
        Self {
            family_name: transaction.family_name.to_string(),
            family_version: transaction.family_version.to_string(),
            transaction_id: transaction.transaction_id.to_string(),
            payload: transaction.payload.to_vec(),
            signer_public_key: transaction.signer_public_key.to_string(),
            service_id: transaction.service_id.clone(),
        }
    }
}

impl From<&TransactionModel> for TrackingTransaction {
    fn from(transaction: &TransactionModel) -> Self {
        Self {
            family_name: transaction.family_name.to_string(),
            family_version: transaction.family_version.to_string(),
            transaction_id: transaction.transaction_id.to_string(),
            payload: transaction.payload.to_vec(),
            signer_public_key: transaction.signer_public_key.to_string(),
            service_id: transaction.service_id.clone(),
        }
    }
}

impl From<TransactionReceiptModel> for TransactionReceipt {
    fn from(receipt: TransactionReceiptModel) -> Self {
        Self {
            transaction_id: receipt.transaction_id.to_string(),
            result_valid: receipt.result_valid,
            error_message: receipt.error_message,
            error_data: receipt.error_data,
            serialized_receipt: format!("{:?}", receipt.serialized_receipt),
            external_status: receipt.external_status,
            external_error_message: receipt.external_error_message,
        }
    }
}

impl
    TryFrom<(
        BatchStatusModel,
        Vec<InvalidTransaction>,
        Vec<ValidTransaction>,
    )> for BatchStatus
{
    type Error = BatchTrackingStoreError;

    fn try_from(
        (batch_status, invalid_transactions, valid_transactions): (
            BatchStatusModel,
            Vec<InvalidTransaction>,
            Vec<ValidTransaction>,
        ),
    ) -> Result<Self, Self::Error> {
        match batch_status.dlt_status.as_str() {
            "UNKNOWN" => Ok(BatchStatus::Unknown),
            "PENDING" => Ok(BatchStatus::Pending),
            "DELAYED" => Ok(BatchStatus::Delayed),
            "INVALID" => {
                if invalid_transactions.is_empty() {
                    return Err(BatchTrackingStoreError::InternalError(
                        InternalError::with_message(
                            "Invalid batches must have invalid transactions".to_string(),
                        ),
                    ));
                }

                Ok(BatchStatus::Invalid(invalid_transactions))
            }
            "VALID" => {
                if valid_transactions.is_empty() {
                    return Err(BatchTrackingStoreError::InternalError(
                        InternalError::with_message(
                            "Valid batches must have valid transactions".to_string(),
                        ),
                    ));
                }

                Ok(BatchStatus::Valid(valid_transactions))
            }
            "COMMITTED" => {
                if valid_transactions.is_empty() {
                    return Err(BatchTrackingStoreError::InternalError(
                        InternalError::with_message(
                            "Committed batches must have valid transactions".to_string(),
                        ),
                    ));
                }

                Ok(BatchStatus::Committed(valid_transactions))
            }
            _ => Err(BatchTrackingStoreError::InternalError(
                InternalError::with_message(format!(
                    "{} is not a supported DLT status",
                    batch_status.dlt_status
                )),
            )),
        }
    }
}

impl TryFrom<TransactionReceipt> for InvalidTransaction {
    type Error = BatchTrackingStoreError;

    fn try_from(receipt: TransactionReceipt) -> Result<Self, Self::Error> {
        let TransactionReceipt {
            transaction_id,
            result_valid,
            error_message,
            error_data,
            serialized_receipt: _,
            external_status,
            external_error_message,
        } = receipt;

        if result_valid {
            return Err(BatchTrackingStoreError::InternalError(
                InternalError::with_message(
                    "Cannot create an invalid transaction with a valid receipt".to_string(),
                ),
            ));
        }

        if error_message.is_none() && external_error_message.is_none() {
            return Err(BatchTrackingStoreError::InternalError(
                InternalError::with_message(
                    "Invalid transaction receipts must have an error message".to_string(),
                ),
            ));
        }

        if error_message.is_some() && error_data.is_none() {
            return Err(BatchTrackingStoreError::InternalError(
                InternalError::with_message(
                    "Invalid transaction receipts must have error data".to_string(),
                ),
            ));
        }

        Ok(Self {
            transaction_id,
            error_message,
            error_data,
            external_error_status: external_status,
            external_error_message,
        })
    }
}

impl TryFrom<TransactionReceipt> for ValidTransaction {
    type Error = BatchTrackingStoreError;

    fn try_from(receipt: TransactionReceipt) -> Result<Self, Self::Error> {
        if receipt.error_message.is_some() {
            return Err(BatchTrackingStoreError::InternalError(
                InternalError::with_message(
                    "Valid transaction receipts must not have an error message".to_string(),
                ),
            ));
        }
        if receipt.error_data.is_some() {
            return Err(BatchTrackingStoreError::InternalError(
                InternalError::with_message(
                    "Valid transaction receipts must not have error data".to_string(),
                ),
            ));
        }
        if receipt.external_status.is_some() {
            return Err(BatchTrackingStoreError::InternalError(
                InternalError::with_message(
                    "Valid transaction receipts must not have an external error status".to_string(),
                ),
            ));
        }
        if receipt.external_error_message.is_some() {
            return Err(BatchTrackingStoreError::InternalError(
                InternalError::with_message(
                    "Valid transaction receipts must not have an external error message"
                        .to_string(),
                ),
            ));
        }

        Ok(Self {
            transaction_id: receipt.transaction_id,
        })
    }
}

impl TryFrom<SubmissionModel> for SubmissionError {
    type Error = BatchTrackingStoreError;

    fn try_from(submission: SubmissionModel) -> Result<Self, Self::Error> {
        if submission.error_message.is_none() {
            return Err(BatchTrackingStoreError::InternalError(
                InternalError::with_message(
                    "Submission errors must have an error message".to_string(),
                ),
            ));
        }
        let error_message = submission.error_message.as_ref().unwrap().to_string();

        if submission.error_type.is_none() {
            return Err(BatchTrackingStoreError::InternalError(
                InternalError::with_message(
                    "Submission errors must have an error type".to_string(),
                ),
            ));
        }
        let error_type = submission.error_type.as_ref().unwrap().to_string();

        Ok(Self {
            error_message,
            error_type,
        })
    }
}

pub fn make_batch_models(batches: &[TrackingBatch]) -> Vec<BatchModel> {
    let mut models = Vec::new();
    for batch in batches {
        let model = BatchModel {
            service_id: batch.service_id().to_string(),
            batch_id: batch.batch_header().to_string(),
            data_change_id: batch.data_change_id().map(String::from),
            signer_public_key: batch.signer_public_key().to_string(),
            trace: batch.trace(),
            serialized_batch: batch.serialized_batch().to_vec(),
            submitted: batch.submitted(),
            created_at: batch.created_at(),
        };

        models.push(model)
    }

    models
}

pub fn make_transaction_models(batches: &[TrackingBatch]) -> Vec<TransactionModel> {
    let mut models = Vec::new();
    for batch in batches {
        for transaction in batch.transactions() {
            let model = TransactionModel {
                service_id: transaction.service_id().to_string(),
                transaction_id: transaction.transaction_id().to_string(),
                batch_id: batch.batch_header().to_string(),
                batch_service_id: batch.service_id().to_string(),
                payload: transaction.payload().to_vec(),
                family_name: transaction.family_name().to_string(),
                family_version: transaction.family_version().to_string(),
                signer_public_key: transaction.signer_public_key().to_string(),
            };

            models.push(model)
        }
    }

    models
}
