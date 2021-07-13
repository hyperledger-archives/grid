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

mod builder;
#[cfg(feature = "diesel")]
pub(in crate) mod diesel;
mod error;

use chrono::NaiveDateTime;

use crate::paging::Paging;

#[cfg(feature = "diesel")]
pub use self::diesel::DieselBatchStore;
pub use builder::{BatchBuilder, TransactionBuilder, TransactionReceiptBuilder};
pub use error::BatchStoreError;

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
    pub fn add_transaction(
        &mut self,
        header_signature: &str,
        family_name: &str,
        family_version: &str,
    ) {
        self.transactions.push(Transaction {
            header_signature: header_signature.to_string(),
            batch_id: self.header_signature.clone(),
            family_name: family_name.to_string(),
            family_version: family_version.to_string(),
            signer_public_key: self.signer_public_key.clone(),
        });
    }

    /// Returns the header signature of the batch
    pub fn header_signature(&self) -> &str {
        &self.header_signature
    }

    /// Returns the data change ID of the batch
    pub fn data_change_id(&self) -> Option<&str> {
        self.data_change_id.as_deref()
    }

    /// Returns the public key of the signer of the batch
    pub fn signer_public_key(&self) -> &str {
        &self.signer_public_key
    }

    /// Returns the trace value of the batch
    pub fn trace(&self) -> bool {
        self.trace
    }

    /// Returns the serialized batch data
    pub fn serialized_batch(&self) -> &str {
        &self.serialized_batch
    }

    /// Returns the submission status of the batch
    pub fn submitted(&self) -> bool {
        self.submitted
    }

    /// Returns the error status of the batch
    pub fn submission_error(&self) -> Option<&str> {
        self.submission_error.as_deref()
    }

    /// Returns the message of an error in submission of the batch
    pub fn submission_error_message(&self) -> Option<&str> {
        self.submission_error_message.as_deref()
    }

    /// Returns the dlt status of the batch
    pub fn dlt_status(&self) -> Option<&str> {
        self.dlt_status.as_deref()
    }

    /// Returns the expiration time remaining on the batch
    pub fn claim_expires(&self) -> Option<&NaiveDateTime> {
        self.claim_expires.as_ref()
    }

    /// Returns the created date/time of the batch
    pub fn created(&self) -> Option<&NaiveDateTime> {
        self.created.as_ref()
    }

    /// Returns the Splinter service ID of the batch
    pub fn service_id(&self) -> Option<&str> {
        self.service_id.as_deref()
    }

    /// Returns the transactions in the batch
    pub fn transactions(&self) -> &[Transaction] {
        &self.transactions
    }
}

/// Data needed to submit a batch
#[derive(Clone, Debug, PartialEq)]
pub struct BatchSubmitInfo {
    pub header_signature: String,
    pub serialized_batch: String,
    pub service_id: Option<String>,
}

impl BatchSubmitInfo {
    /// Returns the header signature of the batch info
    pub fn header_signature(&self) -> &str {
        &self.header_signature
    }

    /// Returns the serialized batch data
    pub fn serialized_batch(&self) -> &str {
        &self.serialized_batch
    }

    /// Returns the Splinter service ID of the batch
    pub fn service_id(&self) -> Option<&str> {
        self.service_id.as_deref()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Transaction {
    header_signature: String,
    batch_id: String,
    family_name: String,
    family_version: String,
    signer_public_key: String,
}

impl Transaction {
    /// Returns the header signature of the transaction
    pub fn header_signature(&self) -> &str {
        &self.header_signature
    }

    /// Returns the ID of the batch the transaction is in
    pub fn batch_id(&self) -> &str {
        &self.batch_id
    }

    /// Returns the family name of the transaction
    pub fn family_name(&self) -> &str {
        &self.family_name
    }

    // Returns the family version of the transaction
    pub fn family_version(&self) -> &str {
        &self.family_version
    }

    /// Returns the public key of the signer of the  transaction
    pub fn signer_public_key(&self) -> &str {
        &self.signer_public_key
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TransactionReceipt {
    transaction_id: String,
    result_valid: bool,
    error_message: Option<String>,
    error_data: Option<String>,
    serialized_receipt: String,
    external_status: Option<String>,
    external_error_message: Option<String>,
}

impl TransactionReceipt {
    /// Returns the ID of the transaction
    pub fn transaction_id(&self) -> &str {
        &self.transaction_id
    }

    /// Returns the result validity of the transaction submission
    pub fn result_valid(&self) -> bool {
        self.result_valid
    }

    /// Returns the error message for the transaction
    pub fn error_message(&self) -> Option<&str> {
        self.error_message.as_deref()
    }

    /// Returns the error data for the transaction
    pub fn error_data(&self) -> Option<&str> {
        self.error_data.as_deref()
    }

    /// Returns the serialized receipt for the transaction
    pub fn serialized_receipt(&self) -> &str {
        &self.serialized_receipt
    }

    /// Returns the external status of the transaction
    pub fn external_status(&self) -> Option<&str> {
        self.external_status.as_deref()
    }

    /// Returns the external error message of the transaction
    pub fn external_error_message(&self) -> Option<&str> {
        self.external_error_message.as_deref()
    }
}

#[derive(Clone, Debug)]
pub struct BatchList {
    data: Vec<Batch>,
    paging: Paging,
}

impl BatchList {
    pub fn new(data: Vec<Batch>, paging: Paging) -> Self {
        Self { data, paging }
    }
}

pub trait BatchStore: Send + Sync {
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
