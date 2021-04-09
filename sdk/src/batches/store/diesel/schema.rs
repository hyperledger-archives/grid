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

table! {
    batches (header_signature) {
        header_signature -> Text,
        data_change_id -> Nullable<Text>,
        signer_public_key -> Text,
        trace -> Bool,
        serialized_batch -> Text,
        submitted -> Bool,
        submission_error -> Nullable<Text>,
        submission_error_message -> Nullable<Text>,
        dlt_status -> Nullable<Text>,
        claim_expires -> Nullable<Timestamp>,
        created -> Nullable<Timestamp>,
        service_id -> Nullable<Text>,
    }
}

table! {
    transactions (header_signature) {
        header_signature -> Text,
        batch_id -> Text,
        family_name -> Text,
        family_version -> Text,
        signer_public_key -> Text,
    }
}

table! {
    transaction_receipts (id) {
        id -> Integer,
        transaction_id -> Text,
        result_valid -> Bool,
        error_message -> Nullable<Text>,
        error_data -> Nullable<Text>,
        serialized_receipt -> Text,
        external_status -> Nullable<Text>,
        external_error_message -> Nullable<Text>,
    }
}
