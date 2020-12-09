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

use crate::commits::store::diesel::schema::*;

#[derive(Insertable, PartialEq, Queryable)]
#[table_name = "commits"]
pub struct NewCommitModel {
    pub commit_id: String,
    pub commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Queryable, PartialEq, Identifiable, Debug)]
#[table_name = "commits"]
pub struct CommitModel {
    pub id: i64,
    pub commit_id: String,
    pub commit_num: i64,
    pub service_id: Option<String>,
}
