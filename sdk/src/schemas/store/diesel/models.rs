/*
 * Copyright 2020 Cargill Incorporated
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

use super::schema::{grid_property_definition, grid_schema};

#[derive(Clone, Insertable, Debug)]
#[table_name = "grid_schema"]
pub struct NewGridSchema {
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub name: String,
    pub description: String,
    pub owner: String,
    pub service_id: Option<String>,
}

#[derive(Queryable, Debug)]
pub struct GridSchema {
    pub id: i64,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub name: String,
    pub description: String,
    pub owner: String,
    pub service_id: Option<String>,
}

#[derive(Clone, Insertable, Debug)]
#[table_name = "grid_property_definition"]
pub struct NewGridPropertyDefinition {
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub name: String,
    pub schema_name: String,
    pub data_type: String,
    pub required: bool,
    pub description: String,
    pub number_exponent: i64,
    // comma separated list of enums
    pub enum_options: String,
    pub parent_name: Option<String>,
    pub service_id: Option<String>,
}

#[derive(Queryable, Debug)]
pub struct GridPropertyDefinition {
    pub id: i64,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub name: String,
    pub schema_name: String,
    pub data_type: String,
    pub required: bool,
    pub description: String,
    pub number_exponent: i64,
    // comma separated list of enums
    pub enum_options: String,
    pub parent_name: Option<String>,
    pub service_id: Option<String>,
}
