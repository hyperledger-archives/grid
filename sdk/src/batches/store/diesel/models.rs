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

use crate::batches::store::{diesel::schema::*, Batch, BatchSubmitInfo, Transaction};
use chrono::NaiveDateTime;

#[derive(Insertable, Queryable, PartialEq, Eq, Debug)]
#[table_name = "batches"]
pub struct BatchModel {
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
}

#[derive(Insertable, Queryable, PartialEq, Eq, Debug)]
#[table_name = "transactions"]
pub struct TransactionModel {
    pub header_signature: String,
    pub batch_id: String,
    pub family_name: String,
    pub family_version: String,
    pub signer_public_key: String,
}

#[derive(Insertable, PartialEq, Eq, Debug)]
#[table_name = "transaction_receipts"]
pub struct TransactionReceiptModel {
    pub transaction_id: String,
    pub result_valid: bool,
    pub error_message: Option<String>,
    pub error_data: Option<String>,
    pub serialized_receipt: String,
    pub external_status: Option<String>,
    pub external_error_message: Option<String>,
}

impl From<(BatchModel, Vec<TransactionModel>)> for Batch {
    fn from((batch_model, transaction_models): (BatchModel, Vec<TransactionModel>)) -> Self {
        Self {
            header_signature: batch_model.header_signature,
            data_change_id: batch_model.data_change_id,
            signer_public_key: batch_model.signer_public_key,
            trace: batch_model.trace,
            serialized_batch: batch_model.serialized_batch,
            submitted: batch_model.submitted,
            submission_error: batch_model.submission_error,
            submission_error_message: batch_model.submission_error_message,
            dlt_status: batch_model.dlt_status,
            claim_expires: batch_model.claim_expires,
            created: batch_model.created,
            service_id: batch_model.service_id,
            transactions: transaction_models
                .into_iter()
                .map(Transaction::from)
                .collect(),
        }
    }
}

impl From<Batch> for (BatchModel, Vec<TransactionModel>) {
    fn from(batch: Batch) -> Self {
        let batch_model = BatchModel {
            header_signature: batch.header_signature,
            data_change_id: batch.data_change_id,
            signer_public_key: batch.signer_public_key,
            trace: batch.trace,
            serialized_batch: batch.serialized_batch,
            submitted: batch.submitted,
            submission_error: batch.submission_error,
            submission_error_message: batch.submission_error_message,
            dlt_status: batch.dlt_status,
            claim_expires: batch.claim_expires,
            created: batch.created,
            service_id: batch.service_id,
        };

        let transaction_models = batch
            .transactions
            .into_iter()
            .map(TransactionModel::from)
            .collect();

        (batch_model, transaction_models)
    }
}

impl From<TransactionModel> for Transaction {
    fn from(model: TransactionModel) -> Self {
        Self {
            header_signature: model.header_signature,
            batch_id: model.batch_id,
            family_name: model.family_name,
            family_version: model.family_version,
            signer_public_key: model.signer_public_key,
        }
    }
}

impl From<Transaction> for TransactionModel {
    fn from(transaction: Transaction) -> Self {
        Self {
            header_signature: transaction.header_signature,
            batch_id: transaction.batch_id,
            family_name: transaction.family_name,
            family_version: transaction.family_version,
            signer_public_key: transaction.signer_public_key,
        }
    }
}

impl From<BatchModel> for BatchSubmitInfo {
    fn from(model: BatchModel) -> Self {
        Self {
            header_signature: model.header_signature,
            serialized_batch: model.serialized_batch,
            service_id: model.service_id,
        }
    }
}
