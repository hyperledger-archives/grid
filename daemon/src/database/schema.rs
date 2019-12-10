/*
 * Copyright 2019 Cargill Incorporated
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * -----------------------------------------------------------------------------
 */

use crate::database::models::LatLong;

table! {
    agent (id) {
        id -> Int8,
        start_block_num -> Int8,
        end_block_num -> Int8,
        public_key -> Varchar,
        org_id -> Varchar,
        active -> Bool,
        roles -> Array<Text>,
        metadata -> Json,
        source -> Nullable<Text>,
    }
}

table! {
    associated_agent (id) {
        id -> Int8,
        record_id -> Text,
        role -> Text,
        start_block_num -> Int8,
        end_block_num -> Int8,
        agent_id -> Text,
        timestamp -> Int8,
        source -> Nullable<Text>,
    }
}

table! {
    block (block_id) {
        block_id -> Varchar,
        block_num -> Int8,
        state_root_hash -> Varchar,
    }
}

table! {
    chain_record (id) {
        id -> Int8,
        start_block_num -> Int8,
        end_block_num -> Int8,
        source -> Nullable<Text>,
    }
}

table! {
    grid_property_definition (id) {
        id -> Int8,
        start_block_num -> Int8,
        end_block_num -> Int8,
        name -> Text,
        schema_name -> Text,
        data_type -> Text,
        required -> Bool,
        description -> Text,
        number_exponent -> Int8,
        enum_options -> Array<Text>,
        struct_properties -> Array<Text>,
        source -> Nullable<Text>,
    }
}

table! {
    grid_schema (id) {
        id -> Int8,
        start_block_num -> Int8,
        end_block_num -> Int8,
        name -> Text,
        description -> Text,
        owner -> Text,
        source -> Nullable<Text>,
    }
}

table! {
    organization (id) {
        id -> Int8,
        org_id -> Varchar,
        name -> Varchar,
        address -> Varchar,
        metadata -> Array<Json>,
        start_block_num -> Int8,
        end_block_num -> Int8,
        source -> Nullable<Text>,
    }
}

table! {
    product (id) {
        id -> Int8,
        product_id -> Varchar,
        product_address -> Varchar,
        product_namespace -> Text,
        owner -> Varchar,
        start_block_num -> Int8,
        end_block_num -> Int8,
        source -> Nullable<Text>,
    }
}

table! {
    use diesel::sql_types::*;
    use super::LatLong;
    product_property_value (id) {
        id -> Int8,
        product_id -> Varchar,
        product_address -> Varchar,
        property_name -> Text,
        data_type -> Text,
        bytes_value -> Nullable<Bytea>,
        boolean_value -> Nullable<Bool>,
        number_value -> Nullable<Int8>,
        string_value -> Nullable<Text>,
        enum_value -> Nullable<Int4>,
        struct_values -> Nullable<Array<Text>>,
        lat_long_value -> Nullable<LatLong>,
        start_block_num -> Int8,
        end_block_num -> Int8,
        source -> Nullable<Text>,
    }
}

table! {
    property (id) {
        id -> Int8,
        start_block_num -> Int8,
        end_block_num -> Int8,
        name -> Text,
        record_id -> Text,
        property_definition -> Text,
        current_page -> Int4,
        wrapped -> Bool,
        source -> Nullable<Text>,
    }
}

table! {
    proposal (id) {
        id -> Int8,
        start_block_num -> Int8,
        end_block_num -> Int8,
        record_id -> Text,
        timestamp -> Int8,
        issuing_agent -> Text,
        receiving_agent -> Text,
        role -> Text,
        properties -> Array<Text>,
        status -> Text,
        terms -> Text,
        source -> Nullable<Text>,
    }
}

table! {
    record (id) {
        id -> Int8,
        start_block_num -> Int8,
        end_block_num -> Int8,
        record_id -> Text,
        schema -> Text,
        #[sql_name = "final"]
        final_ -> Bool,
        owners -> Array<Text>,
        custodians -> Array<Text>,
        source -> Nullable<Text>,
    }
}

table! {
    use diesel::sql_types::*;
    use super::LatLong;
    reported_value (id) {
        id -> Int8,
        start_block_num -> Int8,
        end_block_num -> Int8,
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
        struct_values -> Nullable<Array<Text>>,
        lat_long_value -> Nullable<LatLong>,
        source -> Nullable<Text>,
    }
}

table! {
    reporter (id) {
        id -> Int8,
        start_block_num -> Int8,
        end_block_num -> Int8,
        property_name -> Text,
        record_id -> Text,
        public_key -> Text,
        authorized -> Bool,
        reporter_index -> Int4,
        source -> Nullable<Text>,
    }
}

table! {
    use diesel::sql_types::*;
    use super::LatLong;
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
        struct_values ->  Nullable<Array<Text>>,
        lat_long_value -> Nullable<LatLong>,
        public_key ->  Nullable<Text>,
        authorized ->  Nullable<Bool>,
        metadata ->  Nullable<Json>,
        reported_value_end_block_num -> Int8,
        reporter_end_block_num ->  Nullable<Int8>,
        source -> Nullable<Text>,
    }
}

table! {
    reporter_to_agent_metadata (id) {
        id -> Int8,
        property_name -> Text,
        record_id -> Text,
        public_key -> Text,
        authorized -> Bool,
        reporter_index -> Int4,
        metadata -> Nullable<Json>,
        reporter_end_block_num -> Int8,
        source -> Nullable<Text>,
    }
}

allow_tables_to_appear_in_same_query!(
    agent,
    associated_agent,
    block,
    chain_record,
    grid_property_definition,
    grid_schema,
    organization,
    product,
    product_property_value,
    property,
    proposal,
    record,
    reported_value,
    reporter,
);
