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

table! {
    grid_schema (id) {
        id -> Int8,
        start_commit_num -> Int8,
        end_commit_num -> Int8,
        name -> Text,
        description -> Text,
        owner -> Text,
        service_id -> Nullable<Text>,
    }
}

table! {
    grid_property_definition (id) {
        id -> Int8,
        start_commit_num -> Int8,
        end_commit_num -> Int8,
        name -> Text,
        schema_name -> Text,
        data_type -> Text,
        required -> Bool,
        description -> Text,
        number_exponent -> Int8,
        enum_options -> Text,
        parent_name -> Nullable<Text>,
        service_id -> Nullable<Text>,
    }
}
