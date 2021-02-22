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
    pike_agent (id) {
        id -> Int8,
        public_key -> Varchar,
        org_id -> Varchar,
        active -> Bool,
        metadata -> Binary,
        start_commit_num -> Int8,
        end_commit_num -> Int8,
        service_id -> Nullable<Text>,
    }
}

table! {
    pike_role (id) {
        id -> Int8,
        public_key -> Varchar,
        role_name -> Varchar,
        start_commit_num -> Int8,
        end_commit_num -> Int8,
        service_id -> Nullable<Text>,
    }
}

table! {
    pike_inherit_from (id) {
        id -> Int8,
        role_name -> Varchar,
        org_id -> Varchar,
        inherit_from_org_id -> Varchar,
        start_commit_num -> Int8,
        end_commit_num -> Int8,
        service_id -> Nullable<Text>,
    }
}

table! {
    pike_permissions (id) {
        id -> Int8,
        role_name -> Varchar,
        org_id -> Varchar,
        name -> Varchar,
        start_commit_num -> Int8,
        end_commit_num -> Int8,
        service_id -> Nullable<Text>,
    }
}

table! {
    pike_allowed_orgs (id) {
        id -> Int8,
        role_name -> Varchar,
        org_id -> Varchar,
        allowed_org_id -> Varchar,
        start_commit_num -> Int8,
        end_commit_num -> Int8,
        service_id -> Nullable<Text>,
    }
}

table! {
    pike_agent_role_assoc (id) {
        id -> Int8,
        agent_public_key -> Varchar,
        org_id -> Varchar,
        role_name -> Varchar,
        start_commit_num -> Int8,
        end_commit_num -> Int8,
        service_id -> Nullable<Text>,
    }
}

table! {
    pike_organization (id) {
        id -> Int8,
        org_id -> Varchar,
        name -> Varchar,
        address -> Varchar,
        start_commit_num -> Int8,
        end_commit_num -> Int8,
        service_id -> Nullable<Text>,
    }
}

table! {
    pike_organization_metadata (id) {
        id -> Int8,
        org_id -> Varchar,
        key -> Varchar,
        value -> Binary,
        start_commit_num -> Int8,
        end_commit_num -> Int8,
        service_id -> Nullable<Text>,
    }
}

table! {
    pike_organization_alternate_id (id) {
        id -> Int8,
        org_id -> Varchar,
        alternate_id_type -> Varchar,
        alternate_id -> Varchar,
        start_commit_num -> Int8,
        end_commit_num -> Int8,
        service_id -> Nullable<Text>,
    }
}

table! {
    pike_organization_location_assoc (id) {
        id -> Int8,
        location_id -> Varchar,
        start_commit_num -> Int8,
        end_commit_num -> Int8,
        service_id -> Nullable<Text>,
    }
}

allow_tables_to_appear_in_same_query!(
    pike_agent,
    pike_agent_role_assoc,
    pike_role,
    pike_inherit_from,
    pike_permissions,
    pike_allowed_orgs,
    pike_organization,
    pike_organization_metadata,
    pike_organization_alternate_id,
    pike_organization_location_assoc,
);
