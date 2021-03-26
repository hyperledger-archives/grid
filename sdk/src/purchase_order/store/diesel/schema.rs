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
        uuid -> Text,
        org_id -> Varchar,
        workflow_status -> Text,
        is_closed -> Bool,
        accepted_version_id -> Text,
        created_at -> Int8,
        start_commit_num -> Int8,
        end_commit_num -> Int8,
        service_id -> Nullable<Text>,
    }
}

table! {
    purchase_order_version (id) {
        id -> Int8,
        purchase_order_uuid -> Text,
        org_id -> Varchar,
        version_id -> Text,
        is_draft -> Bool,
        current_revision_id -> Text,
        start_commit_num -> Int8,
        end_commit_num -> Int8,
        service_id -> Nullable<Text>,
    }
}

table! {
    purchase_order_version_revision (id) {
        id -> Int8,
        version_id -> Text,
        org_id -> Varchar,
        revision_id -> Text,
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
        purchase_order_uuid -> Text,
        org_id -> Varchar,
        alternate_id_type -> Text,
        alternate_id -> Text,
        start_commit_num -> Int8,
        end_commit_num -> Int8,
        service_id -> Nullable<Text>,
    }
}
