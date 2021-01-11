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

use crate::roles::store::diesel::schema::*;

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "role"]
pub struct NewRoleModel {
    pub org_id: String,
    pub name: String,
    pub description: String,
    pub permissions: Vec<u8>,
    pub allowed_orgs: Vec<u8>,
    pub inherit_from: Vec<u8>,
    // The indicators of the start and stop for the slowly-changing dimensions.
    pub start_commit_num: i64,
    pub end_commit_num: i64,

    pub service_id: Option<String>,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "role"]
pub struct RoleModel {
    pub id: i64,
    pub org_id: String,
    pub name: String,
    pub description: String,
    pub permissions: Vec<u8>,
    pub allowed_orgs: Vec<u8>,
    pub inherit_from: Vec<u8>,
    // The indicators of the start and stop for the slowly-changing dimensions.
    pub start_commit_num: i64,
    pub end_commit_num: i64,

    pub service_id: Option<String>,
}
