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
        public_key -> Varchar,
        org_id -> Varchar,
        active -> Bool,
        roles -> Array<Varchar>,
        metadata -> Array<Json>,
        start_block_num -> Int8,
        end_block_num -> Int8,
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
    }
}

table! {
    use diesel::sql_types::*;
    use super::LatLong;
    grid_property_value (id) {
        id -> Int8,
        start_block_num -> Int8,
        end_block_num -> Int8,
        name -> Text,
        data_type -> Text,
        bytes_value -> Nullable<Bytea>,
        boolean_value -> Nullable<Bool>,
        number_value -> Nullable<Int8>,
        string_value -> Nullable<Text>,
        enum_value -> Nullable<Int4>,
        struct_values -> Nullable<Array<Text>>,
        lat_long_value -> Nullable<LatLong>,
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
    }
}

allow_tables_to_appear_in_same_query!(
    agent,
    block,
    chain_record,
    grid_property_definition,
    grid_property_value,
    grid_schema,
    organization,
);
