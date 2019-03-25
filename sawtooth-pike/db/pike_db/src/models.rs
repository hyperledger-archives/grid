// Copyright 2018 Cargill Incorporated
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

use schema::agents;
use schema::organizations;
use serde_json;

#[derive(Queryable)]
#[derive(Deserialize, Serialize)]
#[derive(Debug)]
pub struct Agent {
    pub public_key: String,
    pub org_id: String,
    pub active: bool,
    pub roles: Vec<String>,
    pub metadata: Vec<serde_json::Value>
}

#[derive(Insertable)]
#[table_name = "agents"]
#[derive(Debug)]
pub struct NewAgent<'a> {
    pub public_key: &'a str,
    pub org_id: &'a str,
    pub active: bool,
    pub roles: Vec<String>,
    pub metadata: Vec<serde_json::Value>

}

#[derive(Queryable)]
#[derive(Deserialize, Serialize)]
#[derive(Debug)]
pub struct Organization {
    pub id: String,
    pub name: String,
    pub address: String
}

#[derive(Insertable)]
#[table_name = "organizations"]
#[derive(Debug)]
pub struct NewOrganization<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub address: &'a str
}

#[derive(Queryable)]
#[derive(Deserialize, Serialize)]
#[derive(Debug)]
pub struct SmartPermission {
    pub org_id: String,
    pub name: String,
    pub address: String
}
