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

#[cfg(feature = "diesel")]
pub mod diesel;
mod error;

pub use error::TrackAndTraceStoreError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssociatedAgent {
    pub id: Option<i64>,
    pub record_id: String,
    pub role: String,
    pub agent_id: String,
    pub timestamp: i64,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Property {
    pub id: Option<i64>,
    pub name: String,
    pub record_id: String,
    pub property_definition: String,
    pub current_page: i32,
    pub wrapped: bool,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: Option<i64>,
    pub record_id: String,
    pub timestamp: i64,
    pub issuing_agent: String,
    pub receiving_agent: String,
    pub role: String,
    pub properties: Vec<String>,
    pub status: String,
    pub terms: String,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    pub id: Option<i64>,
    pub record_id: String,
    pub schema: String,
    pub final_: bool,
    pub owners: Vec<String>,
    pub custodians: Vec<String>,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ReportedValue {
    pub id: Option<i64>,
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
    pub struct_values: Option<Vec<ReportedValue>>,
    pub lat_long_value: Option<LatLongValue>,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reporter {
    pub id: Option<i64>,
    pub property_name: String,
    pub record_id: String,
    pub public_key: String,
    pub authorized: bool,
    pub reporter_index: i32,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportedValueReporterToAgentMetadata {
    pub id: Option<i64>,
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
    pub struct_values: Vec<ReportedValueReporterToAgentMetadata>,
    pub lat_long_value: Option<LatLongValue>,
    pub public_key: Option<String>,
    pub authorized: Option<bool>,
    pub metadata: Option<Vec<u8>>,
    pub reported_value_end_commit_num: i64,
    pub reporter_end_commit_num: Option<i64>,
    pub service_id: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct LatLong;

#[derive(Default, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct LatLongValue(pub i64, pub i64);

pub trait TrackAndTraceStore: Send + Sync {
    /// Adds an associated agent to the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `agents` - The agents to be added
    fn add_associated_agents(
        &self,
        agents: Vec<AssociatedAgent>,
    ) -> Result<(), TrackAndTraceStoreError>;

    /// Adds properties to the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `properties` - The properties to be added
    fn add_properties(&self, properties: Vec<Property>) -> Result<(), TrackAndTraceStoreError>;

    /// Adds proposals to the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `proposals` - The proposals to be added
    fn add_proposals(&self, proposals: Vec<Proposal>) -> Result<(), TrackAndTraceStoreError>;

    /// Adds records to the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `records` - The records to be added
    fn add_records(&self, records: Vec<Record>) -> Result<(), TrackAndTraceStoreError>;

    /// Adds reported values to the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `values` - The reported values to be added
    fn add_reported_values(
        &self,
        values: Vec<ReportedValue>,
    ) -> Result<(), TrackAndTraceStoreError>;

    /// Adds reporters to the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `reporters` - The reporters to be added
    fn add_reporters(&self, reporters: Vec<Reporter>) -> Result<(), TrackAndTraceStoreError>;

    /// Fetches a property and its data type from the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `record_id` - The record ID to fetch for
    ///  * `property_name` - The property name to fetch
    ///  * `service_id` - The service ID to fetch for
    fn fetch_property_with_data_type(
        &self,
        record_id: &str,
        property_name: &str,
        service_id: Option<&str>,
    ) -> Result<Option<(Property, Option<String>)>, TrackAndTraceStoreError>;

    /// Fetches a record from the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `record_id` - The record ID to fetch for
    ///  * `service_id` - The service ID to fetch for
    fn fetch_record(
        &self,
        record_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Record>, TrackAndTraceStoreError>;

    /// Fetches a reported value reported to agent metadata object from the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `record_id` - The record ID to fetch for
    ///  * `property_name` - The property name to fetch
    ///  * `commit_height` - The commit height of the reported value to fetch
    ///  * `service_id` - The service ID to fetch for
    fn fetch_reported_value_reporter_to_agent_metadata(
        &self,
        record_id: &str,
        property_name: &str,
        commit_height: Option<i64>,
        service_id: Option<&str>,
    ) -> Result<Option<ReportedValueReporterToAgentMetadata>, TrackAndTraceStoreError>;

    /// Fetches a list of associated agents from the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `record_ids` - The record IDs to fetch for
    ///  * `service_id` - The service ID to fetch for
    fn list_associated_agents(
        &self,
        record_ids: &[String],
        service_id: Option<&str>,
    ) -> Result<Vec<AssociatedAgent>, TrackAndTraceStoreError>;

    /// Fetches a list of properties and their data types from the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `record_ids` - The list of record IDs to fetch for
    ///  * `service_id` - The service ID to fetch for
    fn list_properties_with_data_type(
        &self,
        record_ids: &[String],
        service_id: Option<&str>,
    ) -> Result<Vec<(Property, Option<String>)>, TrackAndTraceStoreError>;

    /// Fetches a list of proposals from the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `record_ids` - The list of record IDs to fetch for
    ///  * `service_id` - The service ID to fetch for
    fn list_proposals(
        &self,
        record_ids: &[String],
        service_id: Option<&str>,
    ) -> Result<Vec<Proposal>, TrackAndTraceStoreError>;

    /// Fetches a list of records from the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `service_id` - The service ID to fetch for
    fn list_records(
        &self,
        service_id: Option<&str>,
    ) -> Result<Vec<Record>, TrackAndTraceStoreError>;

    /// Fetches a list of reported value reported to agent metadata objects from the underlying
    /// storage
    ///
    /// # Arguments
    ///
    ///  * `record_id` - The record ID to fetch for
    ///  * `property_name` - The property name to fetch
    ///  * `service_id` - The service ID to fetch for
    fn list_reported_value_reporter_to_agent_metadata(
        &self,
        record_id: &str,
        property_name: &str,
        service_id: Option<&str>,
    ) -> Result<Vec<ReportedValueReporterToAgentMetadata>, TrackAndTraceStoreError>;

    /// Fetches a list of reporters from the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `record_id` - The record ID to fetch for
    ///  * `property_name` - The property name to fetch
    ///  * `service_id` - The service ID to fetch for
    fn list_reporters(
        &self,
        record_id: &str,
        property_name: &str,
        service_id: Option<&str>,
    ) -> Result<Vec<Reporter>, TrackAndTraceStoreError>;
}

impl<TS> TrackAndTraceStore for Box<TS>
where
    TS: TrackAndTraceStore + ?Sized,
{
    fn add_associated_agents(
        &self,
        agents: Vec<AssociatedAgent>,
    ) -> Result<(), TrackAndTraceStoreError> {
        (**self).add_associated_agents(agents)
    }

    fn add_properties(&self, properties: Vec<Property>) -> Result<(), TrackAndTraceStoreError> {
        (**self).add_properties(properties)
    }

    fn add_proposals(&self, proposals: Vec<Proposal>) -> Result<(), TrackAndTraceStoreError> {
        (**self).add_proposals(proposals)
    }

    fn add_records(&self, records: Vec<Record>) -> Result<(), TrackAndTraceStoreError> {
        (**self).add_records(records)
    }

    fn add_reported_values(
        &self,
        values: Vec<ReportedValue>,
    ) -> Result<(), TrackAndTraceStoreError> {
        (**self).add_reported_values(values)
    }

    fn add_reporters(&self, reporters: Vec<Reporter>) -> Result<(), TrackAndTraceStoreError> {
        (**self).add_reporters(reporters)
    }

    fn fetch_property_with_data_type(
        &self,
        record_id: &str,
        property_name: &str,
        service_id: Option<&str>,
    ) -> Result<Option<(Property, Option<String>)>, TrackAndTraceStoreError> {
        (**self).fetch_property_with_data_type(record_id, property_name, service_id)
    }

    fn fetch_record(
        &self,
        record_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Record>, TrackAndTraceStoreError> {
        (**self).fetch_record(record_id, service_id)
    }

    fn fetch_reported_value_reporter_to_agent_metadata(
        &self,
        record_id: &str,
        property_name: &str,
        commit_height: Option<i64>,
        service_id: Option<&str>,
    ) -> Result<Option<ReportedValueReporterToAgentMetadata>, TrackAndTraceStoreError> {
        (**self).fetch_reported_value_reporter_to_agent_metadata(
            record_id,
            property_name,
            commit_height,
            service_id,
        )
    }

    fn list_associated_agents(
        &self,
        record_ids: &[String],
        service_id: Option<&str>,
    ) -> Result<Vec<AssociatedAgent>, TrackAndTraceStoreError> {
        (**self).list_associated_agents(record_ids, service_id)
    }

    fn list_properties_with_data_type(
        &self,
        record_ids: &[String],
        service_id: Option<&str>,
    ) -> Result<Vec<(Property, Option<String>)>, TrackAndTraceStoreError> {
        (**self).list_properties_with_data_type(record_ids, service_id)
    }

    fn list_proposals(
        &self,
        record_ids: &[String],
        service_id: Option<&str>,
    ) -> Result<Vec<Proposal>, TrackAndTraceStoreError> {
        (**self).list_proposals(record_ids, service_id)
    }

    fn list_records(
        &self,
        service_id: Option<&str>,
    ) -> Result<Vec<Record>, TrackAndTraceStoreError> {
        (**self).list_records(service_id)
    }

    fn list_reported_value_reporter_to_agent_metadata(
        &self,
        record_id: &str,
        property_name: &str,
        service_id: Option<&str>,
    ) -> Result<Vec<ReportedValueReporterToAgentMetadata>, TrackAndTraceStoreError> {
        (**self).list_reported_value_reporter_to_agent_metadata(
            record_id,
            property_name,
            service_id,
        )
    }

    fn list_reporters(
        &self,
        record_id: &str,
        property_name: &str,
        service_id: Option<&str>,
    ) -> Result<Vec<Reporter>, TrackAndTraceStoreError> {
        (**self).list_reporters(record_id, property_name, service_id)
    }
}
