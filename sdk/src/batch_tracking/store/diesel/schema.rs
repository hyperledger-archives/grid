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

table! {
    batch_statuses (batch_id) {
        batch_id -> Text,
        service_id -> Text,
        dlt_status -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    batches (batch_id) {
        batch_id -> Text,
        service_id -> Text,
        batch_header -> Text,
        data_change_id -> Nullable<Text>,
        signer_public_key -> Text,
        trace -> Bool,
        serialized_batch -> Binary,
        submitted -> Bool,
        created_at -> Timestamp,
    }
}

table! {
    submissions (batch_id) {
        batch_id -> Text,
        service_id -> Text,
        last_checked -> Nullable<Timestamp>,
        times_checked -> Nullable<Text>,
        error_type -> Nullable<Text>,
        error_message -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    transaction_receipts (transaction_id) {
        transaction_id -> Text,
        service_id -> Text,
        result_valid -> Bool,
        error_message -> Nullable<Text>,
        error_data -> Nullable<Binary>,
        serialized_receipt -> Binary,
        external_status -> Nullable<Text>,
        external_error_message -> Nullable<Text>,
    }
}

table! {
    transactions (transaction_id) {
        transaction_id -> Text,
        service_id -> Text,
        transaction_header -> Text,
        batch_id -> Text,
        payload -> Binary,
        family_name -> Text,
        family_version -> Text,
        signer_public_key -> Text,
    }
}

joinable!(batch_statuses -> batches (batch_id));
joinable!(submissions -> batches (batch_id));
joinable!(transaction_receipts -> transactions (transaction_id));
joinable!(transactions -> batches (batch_id));

allow_tables_to_appear_in_same_query!(
    batch_statuses,
    batches,
    submissions,
    transaction_receipts,
    transactions,
);
