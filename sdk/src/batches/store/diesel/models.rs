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

use crate::batches::store::diesel::schema::*;
use chrono::NaiveDateTime;

#[derive(Insertable, Queryable, PartialEq, Debug)]
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

#[derive(Insertable, Queryable, PartialEq, Debug)]
#[table_name = "transactions"]
pub struct TransactionModel {
    pub header_signature: String,
    pub batch_id: String,
    pub family_name: String,
    pub family_version: String,
    pub signer_public_key: String,
}

#[derive(Insertable, PartialEq, Debug)]
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
