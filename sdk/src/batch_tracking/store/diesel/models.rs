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
use chrono::NaiveDateTime;

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
    pub created_at: NaiveDateTime,
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

#[derive(Insertable, PartialEq, Debug)]
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

#[derive(Insertable, PartialEq, Debug)]
#[table_name = "batch_statuses"]
pub struct BatchStatusModel {
    pub service_id: String,
    pub batch_id: String,
    pub batch_service_id: String,
    pub dlt_status: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, PartialEq, Debug)]
#[table_name = "submissions"]
pub struct SubmissionModel {
    pub service_id: String,
    pub batch_id: String,
    pub batch_service_id: String,
    pub last_checked: Option<NaiveDateTime>,
    pub times_checked: Option<String>,
    pub error_type: Option<String>,
    pub error_message: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
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
            created_at: batch.created_at.timestamp(),
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
        Option<Vec<InvalidTransaction>>,
        Option<Vec<ValidTransaction>>,
    )> for BatchStatus
{
    type Error = BatchTrackingStoreError;

    fn try_from(
        (batch_status, invalid_transactions, valid_transactions): (
            BatchStatusModel,
            Option<Vec<InvalidTransaction>>,
            Option<Vec<ValidTransaction>>,
        ),
    ) -> Result<Self, Self::Error> {
        match batch_status.dlt_status.as_str() {
            "UNKNOWN" => Ok(BatchStatus::Unknown),
            "PENDING" => Ok(BatchStatus::Pending),
            "INVALID" => {
                if invalid_transactions.is_none() {
                    return Err(BatchTrackingStoreError::InternalError(
                        InternalError::with_message(
                            "Invalid batches must have invalid transactions".to_string(),
                        ),
                    ));
                }

                Ok(BatchStatus::Invalid(invalid_transactions.unwrap()))
            }
            "VALID" => {
                if valid_transactions.is_none() {
                    return Err(BatchTrackingStoreError::InternalError(
                        InternalError::with_message(
                            "Valid batches must have valid transactions".to_string(),
                        ),
                    ));
                }

                Ok(BatchStatus::Valid(valid_transactions.unwrap()))
            }
            "COMMITTED" => {
                if valid_transactions.is_none() {
                    return Err(BatchTrackingStoreError::InternalError(
                        InternalError::with_message(
                            "Committed batches must have valid transactions".to_string(),
                        ),
                    ));
                }

                Ok(BatchStatus::Committed(valid_transactions.unwrap()))
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
        if receipt.error_message.is_none() {
            return Err(BatchTrackingStoreError::InternalError(
                InternalError::with_message(
                    "Invalid transaction receipts must have an error message".to_string(),
                ),
            ));
        }
        let error_message = receipt.error_message.unwrap();

        if receipt.error_data.is_none() {
            return Err(BatchTrackingStoreError::InternalError(
                InternalError::with_message(
                    "Invalid transaction receipts must have error data".to_string(),
                ),
            ));
        }
        let error_data = receipt.error_data.unwrap();

        Ok(Self {
            transaction_id: receipt.transaction_id,
            error_message,
            error_data,
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
