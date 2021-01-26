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

use crate::organizations::store::diesel::schema::*;

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "organization"]
pub struct NewOrganizationModel {
    pub org_id: String,
    pub name: String,
    pub locations: Vec<u8>,
    pub metadata: Vec<u8>,
    // The indicators of the start and stop for the slowly-changing dimensions.
    pub start_commit_num: i64,
    pub end_commit_num: i64,

    pub service_id: Option<String>,
}

#[derive(Queryable, PartialEq, Identifiable, Debug)]
#[table_name = "organization"]
pub struct OrganizationModel {
    ///  This is the record id for the slowly-changing-dimensions table.
    pub id: i64,
    pub org_id: String,
    pub name: String,
    pub locations: Vec<u8>,
    pub metadata: Vec<u8>,
    // The indicators of the start and stop for the slowly-changing dimensions.
    pub start_commit_num: i64,
    pub end_commit_num: i64,

    pub service_id: Option<String>,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "alternate_identifier"]
pub struct NewAltIDModel {
    pub alternate_id: String,
    pub id_type: String,
    pub org_id: String,
    // The indicators of the start and stop for the slowly-changing dimensions.
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "alternate_identifier"]
pub struct AltIDModel {
    ///  This is the record id for the slowly-changing-dimensions table.
    pub id: i64,
    pub alternate_id: String,
    pub id_type: String,
    pub org_id: String,
    // The indicators of the start and stop for the slowly-changing dimensions.
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}
