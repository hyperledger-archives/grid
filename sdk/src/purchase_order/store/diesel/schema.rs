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
    purchase_order (id) {
        id -> Int8,
        purchase_order_uid -> Text,
        workflow_status -> Text,
        buyer_org_id -> Varchar,
        seller_org_id -> Varchar,
        is_closed -> Bool,
        accepted_version_id -> Nullable<Text>,
        created_at -> Int8,
        workflow_type -> Text,
        start_commit_num -> Int8,
        end_commit_num -> Int8,
        service_id -> Nullable<Text>,
    }
}

table! {
    purchase_order_version (id) {
        id -> Int8,
        purchase_order_uid -> Text,
        version_id -> Text,
        is_draft -> Bool,
        current_revision_id -> Int8,
        start_commit_num -> Int8,
        end_commit_num -> Int8,
        service_id -> Nullable<Text>,
    }
}

table! {
    purchase_order_version_revision (id) {
        id -> Int8,
        purchase_order_uid -> Text,
        version_id -> Text,
        revision_id -> Int8,
        order_xml_v3_4 -> Text,
        submitter -> Text,
        created_at -> Int8,
        start_commit_num -> Int8,
        end_commit_num -> Int8,
        service_id -> Nullable<Text>,
    }
}

table! {
    purchase_order_alternate_id (id) {
        id -> Int8,
        purchase_order_uid -> Text,
        org_id -> Varchar,
        alternate_id_type -> Text,
        alternate_id -> Text,
        start_commit_num -> Int8,
        end_commit_num -> Int8,
        service_id -> Nullable<Text>,
    }
}
