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
    product_property_value (id) {
        id -> Int8,
        product_id -> Varchar,
        product_address -> Varchar,
        property_name -> Text,
        parent_property -> Nullable<Text>,
        data_type -> Text,
        bytes_value -> Nullable<Bytea>,
        boolean_value -> Nullable<Bool>,
        number_value -> Nullable<Int8>,
        string_value -> Nullable<Text>,
        enum_value -> Nullable<Int4>,
        latitude_value -> Nullable<Int8>,
        longitude_value -> Nullable<Int8>,
        start_commit_num -> Int8,
        end_commit_num -> Int8,
        service_id -> Nullable<Text>,
    }
}

table! {
    product (id) {
        id -> Int8,
        product_id -> Varchar,
        product_address -> Varchar,
        product_namespace -> Text,
        owner -> Varchar,
        start_commit_num -> Int8,
        end_commit_num -> Int8,
        service_id -> Nullable<Text>,
    }
}
