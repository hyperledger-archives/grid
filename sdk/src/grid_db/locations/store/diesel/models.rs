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

use crate::grid_db::locations::store::diesel::schema::{location, location_attribute};

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "location"]
pub struct NewLocationModel {
    pub location_id: String,
    pub location_namespace: String,
    pub owner: String,

    // The indicators of the start and stop for the slowly-changing dimensions.
    pub start_commit_num: i64,
    pub end_commit_num: i64,

    pub service_id: Option<String>,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "location"]
pub struct LocationModel {
    pub id: i64,
    pub location_id: String,
    pub location_namespace: String,
    pub owner: String,

    // The indicators of the start and stop for the slowly-changing dimensions.
    pub start_commit_num: i64,
    pub end_commit_num: i64,

    pub service_id: Option<String>,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "location_attribute"]
pub struct NewLocationAttributeModel {
    pub location_id: String,
    pub location_address: String,
    pub property_name: String,
    pub parent_property_name: Option<String>,
    pub data_type: String,
    pub bytes_value: Option<Vec<u8>>,
    pub boolean_value: Option<bool>,
    pub number_value: Option<i64>,
    pub string_value: Option<String>,
    pub enum_value: Option<i32>,
    pub latitude_value: Option<i64>,
    pub longitude_value: Option<i64>,

    // The indicators of the start and stop for the slowly-changing dimensions.
    pub start_commit_num: i64,
    pub end_commit_num: i64,

    pub service_id: Option<String>,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "location_attribute"]
pub struct LocationAttributeModel {
    pub id: i64,
    pub location_id: String,
    pub location_address: String,
    pub property_name: String,
    pub parent_property_name: Option<String>,
    pub data_type: String,
    pub bytes_value: Option<Vec<u8>>,
    pub boolean_value: Option<bool>,
    pub number_value: Option<i64>,
    pub string_value: Option<String>,
    pub enum_value: Option<i32>,
    pub latitude_value: Option<i64>,
    pub longitude_value: Option<i64>,

    // The indicators of the start and stop for the slowly-changing dimensions.
    pub start_commit_num: i64,
    pub end_commit_num: i64,

    pub service_id: Option<String>,
}
