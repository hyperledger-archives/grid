// Copyright 2021 Cargill Incorporated
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

use crate::hex;

use super::error::BatchBuilderError;
use super::{Batch, NaiveDateTime, Transaction, TransactionReceipt};

/// Builder used to create a Batch
#[derive(Clone, Debug, Default)]
pub struct BatchBuilder {
    header_signature: Option<String>,
    data_change_id: Option<String>,
    signer_public_key: Option<String>,
    trace: Option<bool>,
    serialized_batch: Option<String>,
    submitted: Option<bool>,
    submission_error: Option<String>,
    submission_error_message: Option<String>,
    dlt_status: Option<String>,
    claim_expires: Option<NaiveDateTime>,
    created: Option<NaiveDateTime>,
    service_id: Option<String>,
    transactions: Option<Vec<Transaction>>,
}

impl BatchBuilder {
    ///  Creates a new Batch builder
    pub fn new() -> Self {
        BatchBuilder::default()
    }

    /// Set the header signature of the Batch
    ///
    /// # Arguments
    ///
    /// * `header_signature` - The header signature of the Batch being built
    pub fn with_header_signature(mut self, header_signature: String) -> Self {
        self.header_signature = Some(header_signature);
        self
    }

    /// Set the data change ID of the Batch
    ///
    /// # Arguments
    ///
    /// * `id` - The data change ID of the Batch being built
    pub fn data_change_id(mut self, id: String) -> Self {
        self.data_change_id = Some(id);
        self
    }

    /// Set the signer public key of the Batch
    ///
    /// # Arguments
    ///
    /// * `public_key` - The signer public key of the Batch being built
    pub fn with_signer_public_key(mut self, public_key: String) -> Self {
        self.signer_public_key = Some(public_key);
        self
    }

    /// Set the trace value of the Batch
    ///
    /// # Arguments
    ///
    /// * `trace` - The trace value for the Batch being built
    pub fn with_trace(mut self, trace: bool) -> Self {
        self.trace = Some(trace);
        self
    }

    /// Set the serialized batch value of the Batch
    ///
    /// # Arguments
    ///
    /// * `serialized_batch` - The serialized batch bytes of the Batch being
    ///    built
    pub fn with_serialized_batch(mut self, serialized_batch: &[u8]) -> Self {
        self.serialized_batch = Some(hex::to_hex(serialized_batch));
        self
    }

    /// Set the submitted value of the Batch
    ///
    /// # Arguments
    ///
    /// * `submitted` - The submitted value of the Batch being built
    pub fn with_submitted(mut self, submitted: bool) -> Self {
        self.submitted = Some(submitted);
        self
    }

    /// Set the submission error type of the Batch if it fails
    ///
    /// # Arguments
    ///
    /// * `error type` - The error type of the submission error if it exists
    pub fn with_submission_error(mut self, error_type: String) -> Self {
        self.submission_error = Some(error_type);
        self
    }

    /// Set the submission error message of the Batch if it fails
    ///
    /// # Arguments
    ///
    /// * `error_msg` - The error message of the submission error if it exists
    pub fn with_submission_error_message(mut self, error_msg: String) -> Self {
        self.submission_error_message = Some(error_msg);
        self
    }

    /// Set the dlt status of the Batch
    ///
    /// # Arguments
    ///
    /// * `status` - The dlt status of the Batch being built
    pub fn with_dlt_status(mut self, status: String) -> Self {
        self.dlt_status = Some(status);
        self
    }

    /// Set the claim expiration of the Batch
    ///
    /// # Arguments
    ///
    /// * `expires` - The duration of the claim expiration of the Batch being
    ///    built
    pub fn with_claim_expires(mut self, expires: NaiveDateTime) -> Self {
        self.claim_expires = Some(expires);
        self
    }

    /// Set the created time of the Batch
    ///
    /// # Arguments
    ///
    /// * `created` - The created time of the Batch being built
    pub fn with_created(mut self, created: NaiveDateTime) -> Self {
        self.created = Some(created);
        self
    }

    /// Set the service ID of the Batch
    ///
    /// # Arguments
    ///
    /// * `service_id` - The service ID of the Batch being built
    pub fn with_service_id(mut self, service_id: String) -> Self {
        self.service_id = Some(service_id);
        self
    }

    /// Set the transactions of the Batch
    ///
    /// # Arguments
    ///
    /// * `txns` - The list of transactions of the Batch being built
    pub fn with_transactions(mut self, txns: Vec<Transaction>) -> Self {
        self.transactions = Some(txns);
        self
    }

    pub fn build(self) -> Result<Batch, BatchBuilderError> {
        let header_signature = self.header_signature.ok_or_else(|| {
            BatchBuilderError::MissingRequiredField("header_signature".to_string())
        })?;
        let signer_public_key = self.signer_public_key.ok_or_else(|| {
            BatchBuilderError::MissingRequiredField("signer_public_key".to_string())
        })?;
        let trace = self
            .trace
            .ok_or_else(|| BatchBuilderError::MissingRequiredField("trace".to_string()))?;
        let serialized_batch = self.serialized_batch.ok_or_else(|| {
            BatchBuilderError::MissingRequiredField("serialized_batch".to_string())
        })?;
        let submitted = self
            .submitted
            .ok_or_else(|| BatchBuilderError::MissingRequiredField("submitted".to_string()))?;
        let transactions = self.transactions.unwrap_or_default();

        let data_change_id = self.data_change_id;
        let submission_error = self.submission_error;
        let submission_error_message = self.submission_error_message;
        let dlt_status = self.dlt_status;
        let claim_expires = self.claim_expires;
        let created = self.created;
        let service_id = self.service_id;

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

#[derive(Clone, Debug, Default)]
pub struct TransactionBuilder {
    header_signature: Option<String>,
    batch_id: Option<String>,
    family_name: Option<String>,
    family_version: Option<String>,
    signer_public_key: Option<String>,
}

impl TransactionBuilder {
    ///  Creates a new Transaction builder
    pub fn new() -> Self {
        TransactionBuilder::default()
    }

    /// Set the header signature of the Transaction
    ///
    /// # Arguments
    ///
    /// * `header_signature` - The header signature of the Transaction being built
    pub fn with_header_signature(mut self, header_signature: String) -> Self {
        self.header_signature = Some(header_signature);
        self
    }

    /// Set the batch ID of the Transaction
    ///
    /// # Arguments
    ///
    /// * `batch_id` - The batch_id of the Transaction being built
    pub fn with_batch_id(mut self, batch_id: String) -> Self {
        self.batch_id = Some(batch_id);
        self
    }

    /// Set the family name of the Transaction
    ///
    /// # Arguments
    ///
    /// * `family_name` - The family name of the Transaction being built
    pub fn with_family_name(mut self, family_name: String) -> Self {
        self.family_name = Some(family_name);
        self
    }

    /// Set the family version of the Transaction
    ///
    /// # Arguments
    ///
    /// * `family_version` - The family version of the Transaction being built
    pub fn with_family_version(mut self, family_version: String) -> Self {
        self.family_version = Some(family_version);
        self
    }

    /// Set the signer public key of the Transaction
    ///
    /// # Arguments
    ///
    /// * `public_key` - The signer public key of the Transaction being built
    pub fn with_signer_public_key(mut self, public_key: String) -> Self {
        self.signer_public_key = Some(public_key);
        self
    }

    pub fn build(self) -> Result<Transaction, BatchBuilderError> {
        let header_signature = self.header_signature.ok_or_else(|| {
            BatchBuilderError::MissingRequiredField("header_signature".to_string())
        })?;
        let batch_id = self
            .batch_id
            .ok_or_else(|| BatchBuilderError::MissingRequiredField("batch_id".to_string()))?;
        let family_name = self
            .family_name
            .ok_or_else(|| BatchBuilderError::MissingRequiredField("family_name".to_string()))?;
        let family_version = self
            .family_version
            .ok_or_else(|| BatchBuilderError::MissingRequiredField("family_version".to_string()))?;
        let signer_public_key = self.signer_public_key.ok_or_else(|| {
            BatchBuilderError::MissingRequiredField("signer_public_key".to_string())
        })?;

        Ok(Transaction {
            header_signature,
            batch_id,
            family_name,
            family_version,
            signer_public_key,
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct TransactionReceiptBuilder {
    transaction_id: Option<String>,
    result_valid: Option<bool>,
    error_message: Option<String>,
    error_data: Option<String>,
    serialized_receipt: Option<String>,
    external_status: Option<String>,
    external_error_message: Option<String>,
}

impl TransactionReceiptBuilder {
    ///  Creates a new TransactionReceipt builder
    pub fn new() -> Self {
        TransactionReceiptBuilder::default()
    }

    /// Set the transaction ID of the TransactionReceipt
    ///
    /// # Arguments
    ///
    /// * `id` - The transaction ID of the TransactionReceipt
    pub fn with_transaction_id(mut self, id: String) -> Self {
        self.transaction_id = Some(id);
        self
    }

    /// Set the result valid value of the TransactionReceipt
    ///
    /// # Arguments
    ///
    /// * `valid` - The header signature of the TransactionReceipt being built
    pub fn with_result_valid(mut self, valid: bool) -> Self {
        self.result_valid = Some(valid);
        self
    }

    /// Set the error message of the TransactionReceipt
    ///
    /// # Arguments
    ///
    /// * `msg` - The error message of the TransactionReceipt being built
    pub fn with_error_message(mut self, msg: String) -> Self {
        self.error_message = Some(msg);
        self
    }

    /// Set the error data of the TransactionReceipt
    ///
    /// # Arguments
    ///
    /// * `data` - The error data of the TransactionReceipt being built
    pub fn with_error_data(mut self, data: String) -> Self {
        self.error_data = Some(data);
        self
    }

    /// Set the serialized receipt for the TransactionReceipt
    ///
    /// # Arguments
    ///
    /// * `receipt` - The serialized receipt of the
    ///   TransactionReceipt being built
    pub fn with_serialized_receipt(mut self, receipt: String) -> Self {
        self.serialized_receipt = Some(receipt);
        self
    }

    /// Set the external status of the TransactionReceipt
    ///
    /// # Arguments
    ///
    /// * `status` - The external status of the TransactionReceipt being built
    pub fn with_external_status(mut self, status: String) -> Self {
        self.external_status = Some(status);
        self
    }

    /// Set the external error message of the TransactionReceipt
    ///
    /// # Arguments
    ///
    /// * `msg` - The external error message of the TransactionReceipt being
    ///   built
    pub fn with_external_error_message(mut self, msg: String) -> Self {
        self.external_error_message = Some(msg);
        self
    }

    pub fn build(self) -> Result<TransactionReceipt, BatchBuilderError> {
        let transaction_id = self
            .transaction_id
            .ok_or_else(|| BatchBuilderError::MissingRequiredField("transaction_id".to_string()))?;
        let result_valid = self
            .result_valid
            .ok_or_else(|| BatchBuilderError::MissingRequiredField("result_valid".to_string()))?;
        let serialized_receipt = self.serialized_receipt.ok_or_else(|| {
            BatchBuilderError::MissingRequiredField("serialized_receipt".to_string())
        })?;
        let error_message = self.error_message;
        let error_data = self.error_data;
        let external_status = self.external_status;
        let external_error_message = self.external_error_message;

        Ok(TransactionReceipt {
            transaction_id,
            result_valid,
            error_message,
            error_data,
            serialized_receipt,
            external_status,
            external_error_message,
        })
    }
}
