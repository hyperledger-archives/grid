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

use crate::grid_db::track_and_trace::store::diesel::schema::*;

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "associated_agent"]
pub struct NewAssociatedAgentModel {
    pub record_id: String,
    pub role: String,
    pub agent_id: String,
    pub timestamp: i64,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "associated_agent"]
pub struct AssociatedAgentModel {
    pub id: i64,
    pub record_id: String,
    pub role: String,
    pub agent_id: String,
    pub timestamp: i64,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "grid_property_definition"]
pub struct NewGridPropertyDefinitionModel {
    pub name: String,
    pub schema_name: String,
    pub data_type: String,
    pub required: bool,
    pub description: String,
    pub number_exponent: i64,
    // comma separated list of enums
    pub enum_options: String,
    pub parent_name: Option<String>,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "grid_property_definition"]
pub struct GridPropertyDefinitionModel {
    pub id: i64,
    pub name: String,
    pub schema_name: String,
    pub data_type: String,
    pub required: bool,
    pub description: String,
    pub number_exponent: i64,
    // comma separated list of enums
    pub enum_options: String,
    pub parent_name: Option<String>,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "property"]
pub struct NewPropertyModel {
    pub name: String,
    pub record_id: String,
    pub property_definition: String,
    pub current_page: i32,
    pub wrapped: bool,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "property"]
pub struct PropertyModel {
    pub id: i64,
    pub name: String,
    pub record_id: String,
    pub property_definition: String,
    pub current_page: i32,
    pub wrapped: bool,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "proposal"]
pub struct NewProposalModel {
    pub record_id: String,
    pub timestamp: i64,
    pub issuing_agent: String,
    pub receiving_agent: String,
    pub role: String,
    // comma separated list of properties
    pub properties: String,
    pub status: String,
    // comma separated list of terms
    pub terms: String,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "proposal"]
pub struct ProposalModel {
    pub id: i64,
    pub record_id: String,
    pub timestamp: i64,
    pub issuing_agent: String,
    pub receiving_agent: String,
    pub role: String,
    // comma separated list of properties
    pub properties: String,
    pub status: String,
    pub terms: String,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "record"]
pub struct NewRecordModel {
    pub record_id: String,
    pub schema: String,
    pub final_: bool,
    // comma separated list of owners
    pub owners: String,
    // comma separated list of custodians
    pub custodians: String,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "record"]
pub struct RecordModel {
    pub id: i64,
    pub record_id: String,
    pub schema: String,
    pub final_: bool,
    // comma separated list of owners
    pub owners: String,
    // comma separated list of custodians
    pub custodians: String,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "reported_value"]
pub struct NewReportedValueModel {
    pub property_name: String,
    pub record_id: String,
    pub reporter_index: i32,
    pub timestamp: i64,
    pub data_type: String,
    pub bytes_value: Option<Vec<u8>>,
    pub boolean_value: Option<bool>,
    pub number_value: Option<i64>,
    pub string_value: Option<String>,
    pub enum_value: Option<i32>,
    pub parent_name: Option<String>,
    pub latitude_value: Option<i64>,
    pub longitude_value: Option<i64>,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "reported_value"]
pub struct ReportedValueModel {
    pub id: i64,
    pub property_name: String,
    pub record_id: String,
    pub reporter_index: i32,
    pub timestamp: i64,
    pub data_type: String,
    pub bytes_value: Option<Vec<u8>>,
    pub boolean_value: Option<bool>,
    pub number_value: Option<i64>,
    pub string_value: Option<String>,
    pub enum_value: Option<i32>,
    pub parent_name: Option<String>,
    pub latitude_value: Option<i64>,
    pub longitude_value: Option<i64>,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "reported_value_reporter_to_agent_metadata"]
pub struct NewReportedValueReporterToAgentMetadataModel {
    pub property_name: String,
    pub record_id: String,
    pub reporter_index: i32,
    pub timestamp: i64,
    pub data_type: String,
    pub bytes_value: Option<Vec<u8>>,
    pub boolean_value: Option<bool>,
    pub number_value: Option<i64>,
    pub string_value: Option<String>,
    pub enum_value: Option<i32>,
    pub parent_name: Option<String>,
    pub latitude_value: Option<i64>,
    pub longitude_value: Option<i64>,
    pub public_key: Option<String>,
    pub authorized: Option<bool>,
    pub metadata: Option<Vec<u8>>,
    pub reported_value_end_commit_num: i64,
    pub reporter_end_commit_num: Option<i64>,
    pub service_id: Option<String>,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "reported_value_reporter_to_agent_metadata"]
pub struct ReportedValueReporterToAgentMetadataModel {
    pub id: i64,
    pub property_name: String,
    pub record_id: String,
    pub reporter_index: i32,
    pub timestamp: i64,
    pub data_type: String,
    pub bytes_value: Option<Vec<u8>>,
    pub boolean_value: Option<bool>,
    pub number_value: Option<i64>,
    pub string_value: Option<String>,
    pub enum_value: Option<i32>,
    pub parent_name: Option<String>,
    pub latitude_value: Option<i64>,
    pub longitude_value: Option<i64>,
    pub public_key: Option<String>,
    pub authorized: Option<bool>,
    pub metadata: Option<Vec<u8>>,
    pub reported_value_end_commit_num: i64,
    pub reporter_end_commit_num: Option<i64>,
    pub service_id: Option<String>,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "reporter"]
pub struct NewReporterModel {
    pub property_name: String,
    pub record_id: String,
    pub public_key: String,
    pub authorized: bool,
    pub reporter_index: i32,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "reporter"]
pub struct ReporterModel {
    pub id: i64,
    pub property_name: String,
    pub record_id: String,
    pub public_key: String,
    pub authorized: bool,
    pub reporter_index: i32,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}
