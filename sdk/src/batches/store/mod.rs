// Copyright 2018-2021 Cargill Incorporated
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

#[cfg(feature = "diesel")]
pub(in crate) mod diesel;
mod error;

use chrono::NaiveDateTime;

use crate::hex;
use crate::paging::Paging;

#[cfg(feature = "diesel")]
pub use self::diesel::{DieselBatchStore, DieselConnectionBatchStore};
pub use error::{BatchBuilderError, BatchStoreError};

#[derive(Clone, Debug, PartialEq)]
pub struct Batch {
    header_signature: String,
    data_change_id: Option<String>,
    signer_public_key: String,
    trace: bool,
    serialized_batch: String,
    submitted: bool,
    submission_error: Option<String>,
    submission_error_message: Option<String>,
    dlt_status: Option<String>,
    claim_expires: Option<NaiveDateTime>,
    created: Option<NaiveDateTime>,
    service_id: Option<String>,
    transactions: Vec<Transaction>,
}

impl Batch {
    pub fn header_signature(&self) -> &str {
        &self.header_signature
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

    pub fn serialized_batch(&self) -> &str {
        &self.serialized_batch
    }

    pub fn submitted(&self) -> bool {
        self.submitted
    }

    pub fn submission_error(&self) -> Option<&str> {
        self.submission_error.as_deref()
    }

    pub fn submission_error_message(&self) -> Option<&str> {
        self.submission_error_message.as_deref()
    }

    pub fn dlt_status(&self) -> Option<&str> {
        self.dlt_status.as_deref()
    }

    pub fn claim_expires(&self) -> Option<NaiveDateTime> {
        self.claim_expires
    }

    pub fn created(&self) -> Option<NaiveDateTime> {
        self.created
    }

    pub fn service_id(&self) -> Option<&str> {
        self.service_id.as_deref()
    }

    pub fn transactions(&self) -> Vec<Transaction> {
        self.transactions.to_vec()
    }
}

#[derive(Default, Clone)]
pub struct BatchBuilder {
    header_signature: String,
    data_change_id: Option<String>,
    signer_public_key: String,
    trace: bool,
    serialized_batch: String,
    submitted: bool,
    submission_error: Option<String>,
    submission_error_message: Option<String>,
    dlt_status: Option<String>,
    claim_expires: Option<NaiveDateTime>,
    created: Option<NaiveDateTime>,
    service_id: Option<String>,
    transactions: Vec<Transaction>,
}

impl BatchBuilder {
    pub fn with_header_signature(mut self, signature: String) -> Self {
        self.header_signature = signature;
        self
    }

    pub fn with_data_change_id(mut self, id: String) -> Self {
        self.data_change_id = Some(id);
        self
    }

    pub fn with_signer_public_key(mut self, key: String) -> Self {
        self.signer_public_key = key;
        self
    }

    pub fn with_trace(mut self, trace: bool) -> Self {
        self.trace = trace;
        self
    }

    pub fn with_serialized_batch(mut self, batch: &[u8]) -> Self {
        self.serialized_batch = hex::to_hex(batch);
        self
    }

    pub fn with_submitted(mut self, submitted: bool) -> Self {
        self.submitted = submitted;
        self
    }

    pub fn with_submission_error(mut self, error: String) -> Self {
        self.submission_error = Some(error);
        self
    }

    pub fn with_submission_error_message(mut self, message: String) -> Self {
        self.submission_error_message = Some(message);
        self
    }

    pub fn with_dlt_status(mut self, status: String) -> Self {
        self.dlt_status = Some(status);
        self
    }

    pub fn with_claim_expires(mut self, expires: NaiveDateTime) -> Self {
        self.claim_expires = Some(expires);
        self
    }

    pub fn with_created(mut self, created: NaiveDateTime) -> Self {
        self.created = Some(created);
        self
    }

    pub fn with_service_id(mut self, service_id: Option<String>) -> Self {
        self.service_id = service_id;
        self
    }

    pub fn add_transactions(mut self, transactions: &[Transaction]) -> Self {
        self.transactions = transactions.to_vec();
        self
    }

    pub fn build(self) -> Result<Batch, BatchBuilderError> {
        let BatchBuilder {
            header_signature,
            data_change_id,
            signer_public_key,
            trace,
            serialized_batch,
            submitted,
            submission_error,
            submission_error_message,
            dlt_status,
            claim_expires,
            created,
            service_id,
            transactions,
        } = self;

        if header_signature.is_empty() {
            return Err(BatchBuilderError::MissingRequiredField(
                "header_signature".to_string(),
            ));
        }

        if signer_public_key.is_empty() {
            return Err(BatchBuilderError::MissingRequiredField(
                "signer_public_key".to_string(),
            ));
        }

        if serialized_batch.is_empty() {
            return Err(BatchBuilderError::MissingRequiredField(
                "serialized_batch".to_string(),
            ));
        }

        Ok(Batch {
            header_signature,
            data_change_id,
            signer_public_key,
            trace,
            serialized_batch,
            submitted,
            submission_error,
            submission_error_message,
            dlt_status,
            claim_expires,
            created,
            service_id,
            transactions,
        })
    }
}

/// Data needed to submit a batch
#[derive(Clone, Debug, PartialEq)]
pub struct BatchSubmitInfo {
    pub header_signature: String,
    pub serialized_batch: String,
    pub service_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Transaction {
    pub header_signature: String,
    pub batch_id: String,
    pub family_name: String,
    pub family_version: String,
    pub signer_public_key: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TransactionReceipt {
    pub transaction_id: String,
    pub result_valid: bool,
    pub error_message: Option<String>,
    pub error_data: Option<String>,
    pub serialized_receipt: String,
    pub external_status: Option<String>,
    pub external_error_message: Option<String>,
}

// BatchList is serialized as byte string; Rust doesn't recognize this as the
// fields being read, so we need to allow "dead" code
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct BatchList {
    data: Vec<Batch>,
    paging: Paging,
}

impl BatchList {
    pub fn new(data: Vec<Batch>, paging: Paging) -> Self {
        Self { data, paging }
    }
}

pub trait BatchStore {
    /// Adds a batch to the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `batch` - The batch to be added
    fn add_batch(&self, batch: Batch) -> Result<(), BatchStoreError>;

    /// Fetches a batch from the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `id` - The id of the batch to fetch
    fn get_batch(&self, id: &str) -> Result<Option<Batch>, BatchStoreError>;

    ///  Lists batches from the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `offset` - The index of the first in storage to retrieve
    ///  * `limit` - The number of items to retrieve from the offset
    fn list_batches(&self, offset: i64, limit: i64) -> Result<BatchList, BatchStoreError>;

    /// Fetches batches from the underlying storage that have either an expired
    /// or `null` `claim_expires` value. This then extends that value a
    /// specified number of seconds into the future. The updated batches are
    /// then returned.
    ///
    /// # Arguments
    ///
    ///  * `limit` - The number of items to retrieve
    ///  * `secs_claim_is_valid` - The number of seconds to extend the claims'
    ///     validity
    fn get_unclaimed_batches(
        &self,
        limit: i64,
        secs_claim_is_valid: i64,
    ) -> Result<Vec<BatchSubmitInfo>, BatchStoreError>;

    ///  Updates a batch's status to submitted in the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `id` - The id of the batch to update
    fn change_batch_to_submitted(&self, id: &str) -> Result<(), BatchStoreError>;

    ///  Updates a batch's submission error status with the appropriate error
    /// info if submission fails
    ///
    /// # Arguments
    ///
    ///  * `id` - The id of the batch to update
    ///  * `error` - The error type
    ///  * `error_message` - The explanation of the error
    fn update_submission_error_info(
        &self,
        id: &str,
        error: &str,
        error_message: &str,
    ) -> Result<(), BatchStoreError>;

    ///  Updates a batch's claim status to reflect relinquishing a claim
    ///
    /// # Arguments
    ///
    ///  * `id` - The id of the batch to update
    fn relinquish_claim(&self, id: &str) -> Result<(), BatchStoreError>;
}

impl<BS> BatchStore for Box<BS>
where
    BS: BatchStore + ?Sized,
{
    fn add_batch(&self, batch: Batch) -> Result<(), BatchStoreError> {
        (**self).add_batch(batch)
    }

    fn get_batch(&self, id: &str) -> Result<Option<Batch>, BatchStoreError> {
        (**self).get_batch(id)
    }

    fn list_batches(&self, offset: i64, limit: i64) -> Result<BatchList, BatchStoreError> {
        (**self).list_batches(offset, limit)
    }

    fn get_unclaimed_batches(
        &self,
        limit: i64,
        secs_claim_is_valid: i64,
    ) -> Result<Vec<BatchSubmitInfo>, BatchStoreError> {
        (**self).get_unclaimed_batches(limit, secs_claim_is_valid)
    }

    fn change_batch_to_submitted(&self, id: &str) -> Result<(), BatchStoreError> {
        (**self).change_batch_to_submitted(id)
    }

    fn update_submission_error_info(
        &self,
        id: &str,
        error: &str,
        error_message: &str,
    ) -> Result<(), BatchStoreError> {
        (**self).update_submission_error_info(id, error, error_message)
    }

    fn relinquish_claim(&self, id: &str) -> Result<(), BatchStoreError> {
        (**self).relinquish_claim(id)
    }
}
