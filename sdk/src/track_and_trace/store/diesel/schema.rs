// Copyright 2018-2020 Cargill Incorporated
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
    associated_agent (id) {
        id -> Int8,
        record_id -> Text,
        role -> Text,
        agent_id -> Text,
        timestamp -> Int8,
        start_commit_num -> Int8,
        end_commit_num -> Int8,
        service_id -> Nullable<Text>,
    }
}

table! {
    grid_property_definition (id) {
        id -> Int8,
        name -> Text,
        schema_name -> Text,
        data_type -> Text,
        required -> Bool,
        description -> Text,
        number_exponent -> Int8,
        enum_options -> Text,
        parent_name -> Nullable<Text>,
        start_commit_num -> Int8,
        end_commit_num -> Int8,
        service_id -> Nullable<Text>,
    }
}

table! {
    property (id) {
        id -> Int8,
        name -> Text,
        record_id -> Text,
        property_definition -> Text,
        current_page -> Int4,
        wrapped -> Bool,
        start_commit_num -> Int8,
        end_commit_num -> Int8,
        service_id -> Nullable<Text>,
    }
}

table! {
    proposal (id) {
        id -> Int8,
        record_id -> Text,
        timestamp -> Int8,
        issuing_agent -> Text,
        receiving_agent -> Text,
        role -> Text,
        properties -> Text,
        status -> Text,
        terms -> Text,
        start_commit_num -> Int8,
        end_commit_num -> Int8,
        service_id -> Nullable<Text>,
    }
}

table! {
    record (id) {
        id -> Int8,
        record_id -> Text,
        schema -> Text,
        #[sql_name = "final"]
        final_ -> Bool,
        owners -> Text,
        custodians -> Text,
        start_commit_num -> Int8,
        end_commit_num -> Int8,
        service_id -> Nullable<Text>,
    }
}

table! {
    reported_value (id) {
        id -> Int8,
        property_name -> Text,
        record_id -> Text,
        reporter_index -> Int4,
        timestamp -> Int8,
        data_type -> Text,
        bytes_value -> Nullable<Bytea>,
        boolean_value -> Nullable<Bool>,
        number_value -> Nullable<Int8>,
        string_value -> Nullable<Text>,
        enum_value -> Nullable<Int4>,
        parent_name -> Nullable<Text>,
        latitude_value -> Nullable<Int8>,
        longitude_value -> Nullable<Int8>,
        start_commit_num -> Int8,
        end_commit_num -> Int8,
        service_id -> Nullable<Text>,
    }
}

table! {
    use diesel::sql_types::*;
    reported_value_reporter_to_agent_metadata (id) {
        id -> Int8,
        property_name -> Text,
        record_id -> Text,
        reporter_index -> Int4,
        timestamp -> Int8,
        data_type -> Text,
        bytes_value ->  Nullable<Bytea>,
        boolean_value ->  Nullable<Bool>,
        number_value ->  Nullable<Int8>,
        string_value ->  Nullable<Text>,
        enum_value ->  Nullable<Int4>,
        parent_name ->  Nullable<Text>,
        latitude_value -> Nullable<Int8>,
        longitude_value -> Nullable<Int8>,
        public_key ->  Nullable<Text>,
        authorized ->  Nullable<Bool>,
        metadata ->  Nullable<Binary>,
        reported_value_end_commit_num -> Int8,
        reporter_end_commit_num ->  Nullable<Int8>,
        service_id -> Nullable<Text>,
    }
}

table! {
    reporter (id) {
        id -> Int8,
        property_name -> Text,
        record_id -> Text,
        public_key -> Text,
        authorized -> Bool,
        reporter_index -> Int4,
        start_commit_num -> Int8,
        end_commit_num -> Int8,
        service_id -> Nullable<Text>,
    }
}

allow_tables_to_appear_in_same_query!(
    associated_agent,
    grid_property_definition,
    property,
    proposal,
    record,
    reported_value,
    reporter,
);
