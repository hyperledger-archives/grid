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
pub mod diesel;
mod error;

use chrono::NaiveDateTime;

use crate::hex;
use crate::paging::Paging;
pub use error::BatchStoreError;

#[derive(Clone, Debug, PartialEq)]
pub struct Batch {
    pub header_signature: String,
    pub data_change_id: Option<String>,
    pub signer_public_key: String,
    pub trace: bool,
    pub serialized_batch: String,
    pub submitted: bool,
    pub submission_error: Option<String>,
    pub submission_error_message: Option<String>,
    pub dlt_status: Option<String>,
    pub claim_expires: Option<NaiveDateTime>,
    pub created: Option<NaiveDateTime>,
    pub service_id: Option<String>,
    pub transactions: Vec<Transaction>,
}

impl Batch {
    pub fn new(
        header_signature: String,
        signer_public_key: String,
        trace: bool,
        serialized_batch: &[u8],
        service_id: Option<String>,
    ) -> Self {
        Self {
            header_signature,
            data_change_id: None,
            signer_public_key,
            trace,
            serialized_batch: hex::to_hex(serialized_batch),
            submitted: false,
            submission_error: None,
            submission_error_message: None,
            dlt_status: None,
            claim_expires: None,
            created: None,
            service_id,
            transactions: vec![],
        }
    }

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

#[derive(Clone, Debug)]
pub struct BatchList {
    data: Vec<Batch>,
    paging: Paging,
}

impl BatchList {
    fn new(data: Vec<Batch>, paging: Paging) -> Self {
        Self { data, paging }
    }
}

pub trait BatchStore: Send + Sync {
    fn add_batch(&self, batch: Batch) -> Result<(), BatchStoreError>;

    fn get_batch(&self, id: &str) -> Result<Option<Batch>, BatchStoreError>;

    fn list_batches(&self, offset: i64, limit: i64) -> Result<BatchList, BatchStoreError>;

    fn get_unclaimed_batches(
        &self,
        limit: i64,
        secs_claim_is_valid: i64,
    ) -> Result<Vec<BatchSubmitInfo>, BatchStoreError>;

    fn change_batch_to_submitted(&self, id: &str) -> Result<(), BatchStoreError>;

    fn update_submission_error_info(
        &self,
        id: &str,
        error: &str,
        error_message: &str,
    ) -> Result<(), BatchStoreError>;

    fn relinquish_claim(&self, id: &str) -> Result<(), BatchStoreError>;
}
