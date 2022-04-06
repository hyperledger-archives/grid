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

use crate::error::InternalError;
use transact::protocol::{
    batch::Batch,
    transaction::{Transaction, TransactionHeader},
};
use transact::protos::FromBytes;

#[cfg(feature = "diesel")]
pub(in crate) mod diesel;
mod error;

pub use error::{BatchBuilderError, BatchTrackingStoreError};

const NON_SPLINTER_SERVICE_ID_DEFAULT: &str = "----";

#[derive(Clone, Debug, PartialEq)]
pub enum BatchStatus {
    Unknown,
    Pending,
    Invalid(Vec<InvalidTransaction>),
    Valid(Vec<ValidTransaction>),
    Committed(Vec<ValidTransaction>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct InvalidTransaction {
    transaction_id: String,
    // These are for errors from the DLT itself
    error_message: Option<String>,
    error_data: Option<Vec<u8>>,
    // These are for other errors, such as a 404 when attempting to submit
    // to the DLT
    external_error_status: Option<String>,
    external_error_message: Option<String>,
}

impl InvalidTransaction {
    pub fn transaction_id(&self) -> &str {
        &self.transaction_id
    }

    pub fn error_message(&self) -> Option<&str> {
        self.error_message.as_deref()
    }

    pub fn error_data(&self) -> Option<&[u8]> {
        self.error_data.as_deref()
    }

    pub fn external_error_status(&self) -> Option<&str> {
        self.external_error_status.as_deref()
    }

    pub fn external_error_message(&self) -> Option<&str> {
        self.external_error_message.as_deref()
    }
}

pub struct InvalidTransactionBuilder {
    transaction_id: String,
    error_message: Option<String>,
    error_data: Option<Vec<u8>>,
    external_error_status: Option<String>,
    external_error_message: Option<String>,
}

impl InvalidTransactionBuilder {
    pub fn with_transaction_id(mut self, transaction_id: String) -> Self {
        self.transaction_id = transaction_id;
        self
    }

    pub fn with_error_message(mut self, error_message: String) -> Self {
        self.error_message = Some(error_message);
        self
    }

    pub fn error_data(mut self, error_data: Vec<u8>) -> Self {
        self.error_data = Some(error_data);
        self
    }

    pub fn with_external_error_status(mut self, status: String) -> Self {
        self.external_error_status = Some(status);
        self
    }

    pub fn with_external_error_message(mut self, error_message: String) -> Self {
        self.external_error_message = Some(error_message);
        self
    }

    pub fn build(self) -> Result<InvalidTransaction, BatchBuilderError> {
        let InvalidTransactionBuilder {
            transaction_id,
            error_message,
            error_data,
            external_error_status,
            external_error_message,
        } = self;

        if transaction_id.is_empty() {
            return Err(BatchBuilderError::MissingRequiredField(
                "transaction_id".to_string(),
            ));
        };

        if error_message.is_none() && external_error_message.is_none() {
            return Err(BatchBuilderError::MissingRequiredField(
                "error_message".to_string(),
            ));
        };

        if error_message.is_some() && error_data.is_none() {
            return Err(BatchBuilderError::MissingRequiredField(
                "error_data".to_string(),
            ));
        };

        if external_error_status.is_some() && external_error_message.is_none() {
            return Err(BatchBuilderError::MissingRequiredField(
                "external_error_message".to_string(),
            ));
        }

        if external_error_status.is_none() && external_error_message.is_some() {
            return Err(BatchBuilderError::MissingRequiredField(
                "external_error_status".to_string(),
            ));
        }

        Ok(InvalidTransaction {
            transaction_id,
            error_message,
            error_data,
            external_error_status,
            external_error_message,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ValidTransaction {
    transaction_id: String,
}

impl ValidTransaction {
    pub fn transaction_id(&self) -> &str {
        &self.transaction_id
    }
}

pub struct ValidTransactionBuilder {
    transaction_id: String,
}

impl ValidTransactionBuilder {
    pub fn with_transaction_id(mut self, transaction_id: String) -> Self {
        self.transaction_id = transaction_id;
        self
    }

    pub fn build(self) -> Result<ValidTransaction, BatchBuilderError> {
        let ValidTransactionBuilder { transaction_id } = self;

        if transaction_id.is_empty() {
            return Err(BatchBuilderError::MissingRequiredField(
                "transaction_id".to_string(),
            ));
        };

        Ok(ValidTransaction { transaction_id })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SubmissionError {
    error_type: String,
    error_message: String,
}

impl SubmissionError {
    pub fn error_type(&self) -> &str {
        &self.error_type
    }

    pub fn error_message(&self) -> &str {
        &self.error_message
    }
}

pub struct SubmissionErrorBuilder {
    error_type: String,
    error_message: String,
}

impl SubmissionErrorBuilder {
    pub fn with_error_type(mut self, error_type: String) -> Self {
        self.error_type = error_type;
        self
    }

    pub fn with_error_message(mut self, error_message: String) -> Self {
        self.error_message = error_message;
        self
    }

    pub fn build(self) -> Result<SubmissionError, BatchBuilderError> {
        let SubmissionErrorBuilder {
            error_type,
            error_message,
        } = self;

        if error_type.is_empty() {
            return Err(BatchBuilderError::MissingRequiredField(
                "error_type".to_string(),
            ));
        };

        if error_message.is_empty() {
            return Err(BatchBuilderError::MissingRequiredField(
                "error_message".to_string(),
            ));
        };

        Ok(SubmissionError {
            error_type,
            error_message,
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct TrackingBatch {
    service_id: String,
    batch_header: String,
    data_change_id: Option<String>,
    signer_public_key: String,
    trace: bool,
    serialized_batch: Vec<u8>,
    submitted: bool,
    created_at: i64,
    transactions: Vec<TrackingTransaction>,
    batch_status: Option<BatchStatus>,
    submission_error: Option<SubmissionError>,
}

impl TrackingBatch {
    pub fn service_id(&self) -> &str {
        &self.service_id
    }

    pub fn batch_header(&self) -> &str {
        &self.batch_header
    }

    pub fn data_change_id(&self) -> Option<&str> {
        self.data_change_id.as_deref()
    }

    pub fn signer_public_key(&self) -> &str {
        &self.signer_public_key
    }

    pub fn trace(&self) -> bool {
        self.trace
    }

    pub fn serialized_batch(&self) -> &[u8] {
        &self.serialized_batch
    }

    pub fn submitted(&self) -> bool {
        self.submitted
    }

    pub fn created_at(&self) -> i64 {
        self.created_at
    }

    pub fn transactions(&self) -> &[TrackingTransaction] {
        &self.transactions
    }

    pub fn batch_status(&self) -> Option<&BatchStatus> {
        self.batch_status.as_ref()
    }

    pub fn submission_error(&self) -> Option<&SubmissionError> {
        self.submission_error.as_ref()
    }
}

#[derive(Default, Clone)]
pub struct TrackingBatchBuilder {
    service_id: String,
    batch: Option<Batch>,
    data_change_id: Option<String>,
    signer_public_key: String,
    submitted: bool,
    created_at: i64,
    batch_status: Option<BatchStatus>,
    submission_error: Option<SubmissionError>,
}

impl TrackingBatchBuilder {
    pub fn with_batch(mut self, batch: Batch) -> Self {
        self.batch = Some(batch);
        self
    }

    pub fn with_service_id(mut self, service_id: String) -> Self {
        self.service_id = service_id;
        self
    }

    pub fn with_data_change_id(mut self, data_change_id: String) -> Self {
        self.data_change_id = Some(data_change_id);
        self
    }

    pub fn with_signer_public_key(mut self, signer_public_key: String) -> Self {
        self.signer_public_key = signer_public_key;
        self
    }

    pub fn with_submitted(mut self, submitted: bool) -> Self {
        self.submitted = submitted;
        self
    }

    pub fn with_created_at(mut self, created_at: i64) -> Self {
        self.created_at = created_at;
        self
    }

    pub fn with_batch_status(mut self, status: BatchStatus) -> Self {
        self.batch_status = Some(status);
        self
    }

    pub fn with_submission_error(mut self, submission_error: SubmissionError) -> Self {
        self.submission_error = Some(submission_error);
        self
    }

    pub fn build(self) -> Result<TrackingBatch, BatchBuilderError> {
        let TrackingBatchBuilder {
            service_id,
            batch,
            data_change_id,
            signer_public_key,
            submitted,
            created_at,
            batch_status,
            submission_error,
        } = self;

        if batch.is_none() {
            return Err(BatchBuilderError::MissingRequiredField("batch".to_string()));
        };

        let transact_batch = batch.unwrap();

        if transact_batch.header_signature().is_empty()
            || transact_batch.header().is_empty()
            || transact_batch.transactions().is_empty()
        {
            return Err(BatchBuilderError::MissingRequiredField("batch".to_string()));
        };

        let mut serv_id = service_id.to_string();

        if service_id.is_empty() {
            serv_id = NON_SPLINTER_SERVICE_ID_DEFAULT.to_string();
        };

        let batch_header = transact_batch.header_signature().to_string();
        let serialized_batch = transact_batch.header().to_vec();
        let trace = transact_batch.trace();

        let transactions: Vec<TrackingTransaction> = transact_batch
            .transactions()
            .iter()
            .map(|t| {
                TrackingTransactionBuilder::default()
                    .with_transaction(t.clone())
                    .with_service_id(serv_id.clone())
                    .build()
            })
            .collect::<Result<Vec<TrackingTransaction>, _>>()?;

        if batch_header.is_empty() {
            return Err(BatchBuilderError::MissingRequiredField(
                "batch_header".to_string(),
            ));
        };

        if signer_public_key.is_empty() {
            return Err(BatchBuilderError::MissingRequiredField(
                "signer_public_key".to_string(),
            ));
        };

        if serialized_batch.is_empty() {
            return Err(BatchBuilderError::MissingRequiredField(
                "serialized_batch".to_string(),
            ));
        };

        if created_at <= 0 {
            return Err(BatchBuilderError::MissingRequiredField(
                "created_at".to_string(),
            ));
        };

        if transactions.is_empty() {
            return Err(BatchBuilderError::MissingRequiredField(
                "transactions".to_string(),
            ));
        };

        Ok(TrackingBatch {
            service_id: serv_id,
            batch_header,
            data_change_id,
            signer_public_key,
            trace,
            serialized_batch,
            submitted,
            created_at,
            transactions,
            batch_status,
            submission_error,
        })
    }
}

pub struct TrackingBatchList {
    pub batches: Vec<TrackingBatch>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TrackingTransaction {
    family_name: String,
    family_version: String,
    transaction_id: String,
    payload: Vec<u8>,
    signer_public_key: String,
    service_id: String,
}

impl TrackingTransaction {
    pub fn family_name(&self) -> &str {
        &self.family_name
    }

    pub fn family_version(&self) -> &str {
        &self.family_version
    }

    pub fn transaction_id(&self) -> &str {
        &self.transaction_id
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    pub fn signer_public_key(&self) -> &str {
        &self.signer_public_key
    }

    pub fn service_id(&self) -> &str {
        &self.service_id
    }
}

#[derive(Default, Clone)]
pub struct TrackingTransactionBuilder {
    transaction: Option<Transaction>,
    service_id: String,
}

impl TrackingTransactionBuilder {
    pub fn with_transaction(mut self, transaction: Transaction) -> Self {
        self.transaction = Some(transaction);
        self
    }

    pub fn with_service_id(mut self, service_id: String) -> Self {
        self.service_id = service_id;
        self
    }

    pub fn build(self) -> Result<TrackingTransaction, BatchBuilderError> {
        let TrackingTransactionBuilder {
            transaction,
            service_id,
        } = self;

        if transaction.is_none() {
            return Err(BatchBuilderError::MissingRequiredField(
                "transaction".to_string(),
            ));
        }

        let transact_transaction = transaction.unwrap();

        let mut serv_id = service_id.to_string();

        if service_id.is_empty() {
            serv_id = NON_SPLINTER_SERVICE_ID_DEFAULT.to_string();
        };

        let txn_header =
            TransactionHeader::from_bytes(transact_transaction.header()).map_err(|err| {
                BatchBuilderError::BuildError(Box::new(InternalError::with_message(format!(
                    "Could not convert transaction header from bytes: {}",
                    err
                ))))
            })?;

        let family_name = txn_header.family_name().to_string();
        let family_version = txn_header.family_version().to_string();
        let signer_public_key = format!("{:?}", txn_header.signer_public_key());
        let transaction_id = transact_transaction.header_signature().to_string();
        let payload = transact_transaction.payload().to_vec();

        if family_name.is_empty() {
            return Err(BatchBuilderError::MissingRequiredField(
                "family_name".to_string(),
            ));
        }

        if family_version.is_empty() {
            return Err(BatchBuilderError::MissingRequiredField(
                "family_version".to_string(),
            ));
        }

        if transaction_id.is_empty() {
            return Err(BatchBuilderError::MissingRequiredField(
                "transaction_id".to_string(),
            ));
        }

        if payload.is_empty() {
            return Err(BatchBuilderError::MissingRequiredField(
                "payload".to_string(),
            ));
        }

        if signer_public_key.is_empty() {
            return Err(BatchBuilderError::MissingRequiredField(
                "signer_public_key".to_string(),
            ));
        }

        Ok(TrackingTransaction {
            family_name,
            family_version,
            transaction_id,
            payload,
            signer_public_key,
            service_id: serv_id,
        })
    }
}

pub struct TransactionReceipt {
    transaction_id: String,
    result_valid: bool,
    error_message: Option<String>,
    error_data: Option<Vec<u8>>,
    serialized_receipt: String,
    external_status: Option<String>,
    external_error_message: Option<String>,
}

impl TransactionReceipt {
    pub fn transaction_id(&self) -> &str {
        &self.transaction_id
    }

    pub fn result_valid(&self) -> bool {
        self.result_valid
    }

    pub fn error_message(&self) -> Option<&str> {
        self.error_message.as_deref()
    }

    pub fn error_data(&self) -> Option<&[u8]> {
        self.error_data.as_deref()
    }

    pub fn serialized_receipt(&self) -> &str {
        &self.serialized_receipt
    }

    pub fn external_status(&self) -> Option<&str> {
        self.external_status.as_deref()
    }

    pub fn external_error_message(&self) -> Option<&str> {
        self.external_error_message.as_deref()
    }
}

pub struct TransactionReceiptBuilder {
    transaction_id: String,
    result_valid: bool,
    error_message: Option<String>,
    error_data: Option<Vec<u8>>,
    serialized_receipt: String,
    external_status: Option<String>,
    external_error_message: Option<String>,
}

impl TransactionReceiptBuilder {
    pub fn with_transaction_id(mut self, id: String) -> Self {
        self.transaction_id = id;
        self
    }

    pub fn with_result_valid(mut self, result_valid: bool) -> Self {
        self.result_valid = result_valid;
        self
    }

    pub fn with_error_message(mut self, error_message: String) -> Self {
        self.error_message = Some(error_message);
        self
    }

    pub fn with_error_data(mut self, data: Vec<u8>) -> Self {
        self.error_data = Some(data);
        self
    }

    pub fn with_serialized_receipt(mut self, receipt: String) -> Self {
        self.serialized_receipt = receipt;
        self
    }

    pub fn with_external_status(mut self, status: String) -> Self {
        self.external_status = Some(status);
        self
    }

    pub fn with_external_error_message(mut self, message: String) -> Self {
        self.external_error_message = Some(message);
        self
    }
}

pub trait BatchTrackingStore {
    /// Gets the status of a batch from the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `id` - The ID or data change ID of the batch with the status to
    ///    fetch
    ///  * `service_id` - The service ID
    fn get_batch_status(
        &self,
        id: &str,
        service_id: &str,
    ) -> Result<Option<BatchStatus>, BatchTrackingStoreError>;

    /// Updates the status of a batch in the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `id` - The ID or data change ID of the batch with the status to
    ///    update
    ///  * `service_id` - The service ID
    ///  * `status` - The new status for the batch
    ///  * `errors` - Any errors encountered while attempting to submit the
    ///    batch
    fn update_batch_status(
        &self,
        id: String,
        service_id: Option<&str>,
        status: BatchStatus,
        errors: Vec<SubmissionError>,
    ) -> Result<(), BatchTrackingStoreError>;

    /// Adds batches to the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `batches` - The batches to be added
    fn add_batches(&self, batches: Vec<TrackingBatch>) -> Result<(), BatchTrackingStoreError>;

    /// Updates a batch's status to a submitted state
    ///
    /// # Arguments
    ///
    ///  * `batch_id` - The ID or data change ID of the batch to update
    ///  * `service_id` - The service ID
    fn change_batch_to_submitted(
        &self,
        batch_id: &str,
        service_id: Option<&str>,
    ) -> Result<(), BatchTrackingStoreError>;

    /// Gets a batch from the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `id` - The ID or data change ID of the batch to fetch
    ///  * `service_id` - The service ID
    fn get_batch(
        &self,
        id: &str,
        service_id: &str,
    ) -> Result<Option<TrackingBatch>, BatchTrackingStoreError>;

    /// Lists batches with a given status from the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `status` - The status to fetch batches for
    fn list_batches_by_status(
        &self,
        status: BatchStatus,
    ) -> Result<TrackingBatchList, BatchTrackingStoreError>;

    /// Removes records for batches and batch submissions before a given time
    ///
    /// # Arguments
    ///
    ///  * `submitted_by` - The timestamp for which to delete records submitted before
    fn clean_stale_records(
        &self,
        submitted_by: &str,
    ) -> Result<TrackingBatchList, BatchTrackingStoreError>;

    /// Gets batches that have not yet been submitted from the underlying storage
    fn get_unsubmitted_batches(&self) -> Result<TrackingBatchList, BatchTrackingStoreError>;

    /// Gets batches that failed either due to validation or submission errors
    /// from the underlying storage
    fn get_failed_batches(&self) -> Result<TrackingBatchList, BatchTrackingStoreError>;
}

impl<BS> BatchTrackingStore for Box<BS>
where
    BS: BatchTrackingStore + ?Sized,
{
    fn get_batch_status(
        &self,
        _id: &str,
        _service_id: &str,
    ) -> Result<Option<BatchStatus>, BatchTrackingStoreError> {
        unimplemented!();
    }

    fn update_batch_status(
        &self,
        _id: String,
        _service_id: Option<&str>,
        _status: BatchStatus,
        _errors: Vec<SubmissionError>,
    ) -> Result<(), BatchTrackingStoreError> {
        unimplemented!();
    }

    fn add_batches(&self, _batches: Vec<TrackingBatch>) -> Result<(), BatchTrackingStoreError> {
        unimplemented!();
    }

    fn change_batch_to_submitted(
        &self,
        _batch_id: &str,
        _service_id: Option<&str>,
    ) -> Result<(), BatchTrackingStoreError> {
        unimplemented!();
    }

    fn get_batch(
        &self,
        _id: &str,
        _service_id: &str,
    ) -> Result<Option<TrackingBatch>, BatchTrackingStoreError> {
        unimplemented!();
    }

    fn list_batches_by_status(
        &self,
        _status: BatchStatus,
    ) -> Result<TrackingBatchList, BatchTrackingStoreError> {
        unimplemented!();
    }

    fn clean_stale_records(
        &self,
        _submitted_by: &str,
    ) -> Result<TrackingBatchList, BatchTrackingStoreError> {
        unimplemented!();
    }

    fn get_unsubmitted_batches(&self) -> Result<TrackingBatchList, BatchTrackingStoreError> {
        unimplemented!();
    }

    fn get_failed_batches(&self) -> Result<TrackingBatchList, BatchTrackingStoreError> {
        unimplemented!();
    }
}
