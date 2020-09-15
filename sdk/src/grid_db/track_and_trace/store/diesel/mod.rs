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

pub mod models;
mod operations;
pub(in crate::grid_db) mod schema;

use diesel::r2d2::{ConnectionManager, Pool};

use super::diesel::models::{
    AssociatedAgentModel, NewAssociatedAgentModel, NewPropertyModel, NewProposalModel,
    NewRecordModel, NewReportedValueModel, NewReporterModel, PropertyModel, ProposalModel,
    RecordModel, ReportedValueReporterToAgentMetadataModel, ReporterModel,
};
use super::{
    AssociatedAgent, LatLongValue, Property, Proposal, Record, ReportedValue,
    ReportedValueReporterToAgentMetadata, Reporter, TrackAndTraceStore, TrackAndTraceStoreError,
};
use crate::database::DatabaseError;
use crate::grid_db::commits::MAX_COMMIT_NUM;
use operations::add_associated_agents::TrackAndTraceStoreAddAssociatedAgentsOperation as _;
use operations::add_properties::TrackAndTraceStoreAddPropertiesOperation as _;
use operations::add_proposals::TrackAndTraceStoreAddProposalsOperation as _;
use operations::add_records::TrackAndTraceStoreAddRecordsOperation as _;
use operations::add_reported_values::TrackAndTraceStoreAddReportedValuesOperation as _;
use operations::add_reporters::TrackAndTraceStoreAddReportersOperation as _;
use operations::fetch_property_with_data_type::TrackAndTraceStoreFetchPropertyWithDataTypeOperation as _;
use operations::fetch_record::TrackAndTraceStoreFetchRecordOperation as _;
use operations::fetch_reported_value_reporter_to_agent_metadata::TrackAndTraceStoreFetchReportedValueReporterToAgentMetadataOperation as _;
use operations::list_associated_agents::TrackAndTraceStoreListAssociatedAgentsOperation as _;
use operations::list_properties_with_data_type::TrackAndTraceStoreListPropertiesWithDataTypeOperation as _;
use operations::list_proposals::TrackAndTraceStoreListProposalsOperation as _;
use operations::list_records::TrackAndTraceStoreListRecordsOperation as _;
use operations::list_reported_value_reporter_to_agent_metadata::TrackAndTraceStoreListReportedValueReporterToAgentMetadataOperation as _;
use operations::list_reporters::TrackAndTraceStoreListReportersOperation as _;
use operations::TrackAndTraceStoreOperations;

/// Manages creating track and trace elements in the database
#[derive(Clone)]
pub struct DieselTrackAndTraceStore<C: diesel::Connection + 'static> {
    connection_pool: Pool<ConnectionManager<C>>,
}

#[cfg(feature = "diesel")]
impl<C: diesel::Connection> DieselTrackAndTraceStore<C> {
    /// Creates a new DieselTrackAndTraceStore
    ///
    /// # Arguments
    ///
    ///  * `connection_pool`: connection pool to the database
    // Allow dead code if diesel feature is not enabled
    #[allow(dead_code)]
    pub fn new(connection_pool: Pool<ConnectionManager<C>>) -> Self {
        DieselTrackAndTraceStore { connection_pool }
    }
}

#[cfg(feature = "postgres")]
impl TrackAndTraceStore for DieselTrackAndTraceStore<diesel::pg::PgConnection> {
    fn add_associated_agents(
        &self,
        agents: Vec<AssociatedAgent>,
    ) -> Result<(), TrackAndTraceStoreError> {
        TrackAndTraceStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            DatabaseError::ConnectionError {
                context: "Could not get connection pool".to_string(),
                source: Box::new(err),
            }
        })?)
        .add_associated_agents(agents.iter().map(|a| a.clone().into()).collect())
    }

    fn add_properties(&self, properties: Vec<Property>) -> Result<(), TrackAndTraceStoreError> {
        TrackAndTraceStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            DatabaseError::ConnectionError {
                context: "Could not get connection pool".to_string(),
                source: Box::new(err),
            }
        })?)
        .add_properties(properties.into_iter().map(|p| p.into()).collect())
    }

    fn add_proposals(&self, proposals: Vec<Proposal>) -> Result<(), TrackAndTraceStoreError> {
        TrackAndTraceStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            DatabaseError::ConnectionError {
                context: "Could not get connection pool".to_string(),
                source: Box::new(err),
            }
        })?)
        .add_proposals(proposals.into_iter().map(|p| p.into()).collect())
    }

    fn add_records(&self, records: Vec<Record>) -> Result<(), TrackAndTraceStoreError> {
        TrackAndTraceStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            DatabaseError::ConnectionError {
                context: "Could not get connection pool".to_string(),
                source: Box::new(err),
            }
        })?)
        .add_records(records.into_iter().map(|r| r.into()).collect())
    }

    fn add_reported_values(
        &self,
        values: Vec<ReportedValue>,
    ) -> Result<(), TrackAndTraceStoreError> {
        TrackAndTraceStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            DatabaseError::ConnectionError {
                context: "Could not get connection pool".to_string(),
                source: Box::new(err),
            }
        })?)
        .add_reported_values(make_reported_value_models(&values, None))
    }

    fn add_reporters(&self, reporters: Vec<Reporter>) -> Result<(), TrackAndTraceStoreError> {
        TrackAndTraceStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            DatabaseError::ConnectionError {
                context: "Could not get connection pool".to_string(),
                source: Box::new(err),
            }
        })?)
        .add_reporters(reporters.into_iter().map(|r| r.into()).collect())
    }

    fn fetch_property_with_data_type(
        &self,
        record_id: &str,
        property_name: &str,
        service_id: Option<String>,
    ) -> Result<Option<(Property, Option<String>)>, TrackAndTraceStoreError> {
        TrackAndTraceStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            DatabaseError::ConnectionError {
                context: "Could not get connection pool".to_string(),
                source: Box::new(err),
            }
        })?)
        .fetch_property_with_data_type(record_id, property_name, service_id)
    }

    fn fetch_record(
        &self,
        record_id: &str,
        service_id: Option<String>,
    ) -> Result<Option<Record>, TrackAndTraceStoreError> {
        TrackAndTraceStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            DatabaseError::ConnectionError {
                context: "Could not get connection pool".to_string(),
                source: Box::new(err),
            }
        })?)
        .fetch_record(record_id, service_id)
    }

    fn fetch_reported_value_reporter_to_agent_metadata(
        &self,
        record_id: &str,
        property_name: &str,
        commit_height: Option<i64>,
        service_id: Option<String>,
    ) -> Result<Option<ReportedValueReporterToAgentMetadata>, TrackAndTraceStoreError> {
        TrackAndTraceStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            DatabaseError::ConnectionError {
                context: "Could not get connection pool".to_string(),
                source: Box::new(err),
            }
        })?)
        .fetch_reported_value_reporter_to_agent_metadata(
            record_id,
            property_name,
            commit_height,
            service_id,
        )
    }

    fn list_associated_agents(
        &self,
        record_ids: &[String],
        service_id: Option<String>,
    ) -> Result<Vec<AssociatedAgent>, TrackAndTraceStoreError> {
        TrackAndTraceStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            DatabaseError::ConnectionError {
                context: "Could not get connection pool".to_string(),
                source: Box::new(err),
            }
        })?)
        .list_associated_agents(record_ids, service_id)
    }

    fn list_properties_with_data_type(
        &self,
        record_ids: &[String],
        service_id: Option<String>,
    ) -> Result<Vec<(Property, Option<String>)>, TrackAndTraceStoreError> {
        TrackAndTraceStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            DatabaseError::ConnectionError {
                context: "Could not get connection pool".to_string(),
                source: Box::new(err),
            }
        })?)
        .list_properties_with_data_type(record_ids, service_id)
    }

    fn list_proposals(
        &self,
        record_ids: &[String],
        service_id: Option<String>,
    ) -> Result<Vec<Proposal>, TrackAndTraceStoreError> {
        TrackAndTraceStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            DatabaseError::ConnectionError {
                context: "Could not get connection pool".to_string(),
                source: Box::new(err),
            }
        })?)
        .list_proposals(record_ids, service_id)
    }

    fn list_records(
        &self,
        service_id: Option<String>,
    ) -> Result<Vec<Record>, TrackAndTraceStoreError> {
        TrackAndTraceStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            DatabaseError::ConnectionError {
                context: "Could not get connection pool".to_string(),
                source: Box::new(err),
            }
        })?)
        .list_records(service_id)
    }

    fn list_reported_value_reporter_to_agent_metadata(
        &self,
        record_id: &str,
        property_name: &str,
        service_id: Option<String>,
    ) -> Result<Vec<ReportedValueReporterToAgentMetadata>, TrackAndTraceStoreError> {
        TrackAndTraceStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            DatabaseError::ConnectionError {
                context: "Could not get connection pool".to_string(),
                source: Box::new(err),
            }
        })?)
        .list_reported_value_reporter_to_agent_metadata(
            record_id,
            property_name,
            service_id,
        )
    }

    fn list_reporters(
        &self,
        record_id: &str,
        property_name: &str,
        service_id: Option<String>,
    ) -> Result<Vec<Reporter>, TrackAndTraceStoreError> {
        TrackAndTraceStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            DatabaseError::ConnectionError {
                context: "Could not get connection pool".to_string(),
                source: Box::new(err),
            }
        })?)
        .list_reporters(record_id, property_name, service_id)
    }
}

#[cfg(feature = "sqlite")]
impl TrackAndTraceStore for DieselTrackAndTraceStore<diesel::sqlite::SqliteConnection> {
    fn add_associated_agents(
        &self,
        agents: Vec<AssociatedAgent>,
    ) -> Result<(), TrackAndTraceStoreError> {
        TrackAndTraceStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            DatabaseError::ConnectionError {
                context: "Could not get connection pool".to_string(),
                source: Box::new(err),
            }
        })?)
        .add_associated_agents(agents.iter().map(|a| a.clone().into()).collect())
    }

    fn add_properties(&self, properties: Vec<Property>) -> Result<(), TrackAndTraceStoreError> {
        TrackAndTraceStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            DatabaseError::ConnectionError {
                context: "Could not get connection pool".to_string(),
                source: Box::new(err),
            }
        })?)
        .add_properties(properties.into_iter().map(|p| p.into()).collect())
    }

    fn add_proposals(&self, proposals: Vec<Proposal>) -> Result<(), TrackAndTraceStoreError> {
        TrackAndTraceStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            DatabaseError::ConnectionError {
                context: "Could not get connection pool".to_string(),
                source: Box::new(err),
            }
        })?)
        .add_proposals(proposals.into_iter().map(|p| p.into()).collect())
    }

    fn add_records(&self, records: Vec<Record>) -> Result<(), TrackAndTraceStoreError> {
        TrackAndTraceStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            DatabaseError::ConnectionError {
                context: "Could not get connection pool".to_string(),
                source: Box::new(err),
            }
        })?)
        .add_records(records.into_iter().map(|r| r.into()).collect())
    }

    fn add_reported_values(
        &self,
        values: Vec<ReportedValue>,
    ) -> Result<(), TrackAndTraceStoreError> {
        TrackAndTraceStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            DatabaseError::ConnectionError {
                context: "Could not get connection pool".to_string(),
                source: Box::new(err),
            }
        })?)
        .add_reported_values(make_reported_value_models(&values, None))
    }

    fn add_reporters(&self, reporters: Vec<Reporter>) -> Result<(), TrackAndTraceStoreError> {
        TrackAndTraceStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            DatabaseError::ConnectionError {
                context: "Could not get connection pool".to_string(),
                source: Box::new(err),
            }
        })?)
        .add_reporters(reporters.into_iter().map(|r| r.into()).collect())
    }

    fn fetch_property_with_data_type(
        &self,
        record_id: &str,
        property_name: &str,
        service_id: Option<String>,
    ) -> Result<Option<(Property, Option<String>)>, TrackAndTraceStoreError> {
        TrackAndTraceStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            DatabaseError::ConnectionError {
                context: "Could not get connection pool".to_string(),
                source: Box::new(err),
            }
        })?)
        .fetch_property_with_data_type(record_id, property_name, service_id)
    }

    fn fetch_record(
        &self,
        record_id: &str,
        service_id: Option<String>,
    ) -> Result<Option<Record>, TrackAndTraceStoreError> {
        TrackAndTraceStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            DatabaseError::ConnectionError {
                context: "Could not get connection pool".to_string(),
                source: Box::new(err),
            }
        })?)
        .fetch_record(record_id, service_id)
    }

    fn fetch_reported_value_reporter_to_agent_metadata(
        &self,
        record_id: &str,
        property_name: &str,
        commit_height: Option<i64>,
        service_id: Option<String>,
    ) -> Result<Option<ReportedValueReporterToAgentMetadata>, TrackAndTraceStoreError> {
        TrackAndTraceStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            DatabaseError::ConnectionError {
                context: "Could not get connection pool".to_string(),
                source: Box::new(err),
            }
        })?)
        .fetch_reported_value_reporter_to_agent_metadata(
            record_id,
            property_name,
            commit_height,
            service_id,
        )
    }

    fn list_associated_agents(
        &self,
        record_ids: &[String],
        service_id: Option<String>,
    ) -> Result<Vec<AssociatedAgent>, TrackAndTraceStoreError> {
        TrackAndTraceStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            DatabaseError::ConnectionError {
                context: "Could not get connection pool".to_string(),
                source: Box::new(err),
            }
        })?)
        .list_associated_agents(record_ids, service_id)
    }

    fn list_properties_with_data_type(
        &self,
        record_ids: &[String],
        service_id: Option<String>,
    ) -> Result<Vec<(Property, Option<String>)>, TrackAndTraceStoreError> {
        TrackAndTraceStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            DatabaseError::ConnectionError {
                context: "Could not get connection pool".to_string(),
                source: Box::new(err),
            }
        })?)
        .list_properties_with_data_type(record_ids, service_id)
    }

    fn list_proposals(
        &self,
        record_ids: &[String],
        service_id: Option<String>,
    ) -> Result<Vec<Proposal>, TrackAndTraceStoreError> {
        TrackAndTraceStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            DatabaseError::ConnectionError {
                context: "Could not get connection pool".to_string(),
                source: Box::new(err),
            }
        })?)
        .list_proposals(record_ids, service_id)
    }

    fn list_records(
        &self,
        service_id: Option<String>,
    ) -> Result<Vec<Record>, TrackAndTraceStoreError> {
        TrackAndTraceStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            DatabaseError::ConnectionError {
                context: "Could not get connection pool".to_string(),
                source: Box::new(err),
            }
        })?)
        .list_records(service_id)
    }

    fn list_reported_value_reporter_to_agent_metadata(
        &self,
        record_id: &str,
        property_name: &str,
        service_id: Option<String>,
    ) -> Result<Vec<ReportedValueReporterToAgentMetadata>, TrackAndTraceStoreError> {
        TrackAndTraceStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            DatabaseError::ConnectionError {
                context: "Could not get connection pool".to_string(),
                source: Box::new(err),
            }
        })?)
        .list_reported_value_reporter_to_agent_metadata(
            record_id,
            property_name,
            service_id,
        )
    }

    fn list_reporters(
        &self,
        record_id: &str,
        property_name: &str,
        service_id: Option<String>,
    ) -> Result<Vec<Reporter>, TrackAndTraceStoreError> {
        TrackAndTraceStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            DatabaseError::ConnectionError {
                context: "Could not get connection pool".to_string(),
                source: Box::new(err),
            }
        })?)
        .list_reporters(record_id, property_name, service_id)
    }
}

impl From<(i64, i64)> for LatLongValue {
    fn from((lat, long): (i64, i64)) -> Self {
        Self(lat, long)
    }
}

impl Into<NewAssociatedAgentModel> for AssociatedAgent {
    fn into(self) -> NewAssociatedAgentModel {
        NewAssociatedAgentModel {
            record_id: self.record_id,
            role: self.role,
            agent_id: self.agent_id,
            timestamp: self.timestamp,
            start_commit_num: self.start_commit_num,
            end_commit_num: self.end_commit_num,
            service_id: self.service_id,
        }
    }
}

impl Into<NewPropertyModel> for Property {
    fn into(self) -> NewPropertyModel {
        NewPropertyModel {
            name: self.name,
            record_id: self.record_id,
            property_definition: self.property_definition,
            current_page: self.current_page,
            wrapped: self.wrapped,
            start_commit_num: self.start_commit_num,
            end_commit_num: self.end_commit_num,
            service_id: self.service_id,
        }
    }
}

impl Into<NewProposalModel> for Proposal {
    fn into(self) -> NewProposalModel {
        NewProposalModel {
            record_id: self.record_id,
            timestamp: self.timestamp,
            issuing_agent: self.issuing_agent,
            receiving_agent: self.receiving_agent,
            role: self.role,
            properties: self.properties.join(","),
            status: self.status,
            terms: self.terms,
            start_commit_num: self.start_commit_num,
            end_commit_num: self.end_commit_num,
            service_id: self.service_id,
        }
    }
}

impl Into<NewRecordModel> for Record {
    fn into(self) -> NewRecordModel {
        NewRecordModel {
            record_id: self.record_id,
            schema: self.schema,
            final_: self.final_,
            owners: self.owners.join(","),
            custodians: self.custodians.join(","),
            start_commit_num: self.start_commit_num,
            end_commit_num: self.end_commit_num,
            service_id: self.service_id,
        }
    }
}

impl Into<NewReporterModel> for Reporter {
    fn into(self) -> NewReporterModel {
        NewReporterModel {
            property_name: self.property_name,
            record_id: self.record_id,
            public_key: self.public_key,
            authorized: self.authorized,
            reporter_index: self.reporter_index,
            start_commit_num: self.start_commit_num,
            end_commit_num: self.end_commit_num,
            service_id: self.service_id,
        }
    }
}

fn make_reported_value_models(
    values: &[ReportedValue],
    parent_name: Option<String>,
) -> Vec<NewReportedValueModel> {
    let mut vals = Vec::new();

    for val in values {
        vals.push(NewReportedValueModel {
            property_name: val.property_name.to_string(),
            record_id: val.record_id.to_string(),
            reporter_index: val.reporter_index,
            timestamp: val.timestamp,
            data_type: val.data_type.to_string(),
            bytes_value: val.bytes_value.clone(),
            boolean_value: val.boolean_value,
            number_value: val.number_value,
            string_value: val.string_value.clone(),
            enum_value: val.enum_value,
            parent_name: parent_name.clone(),
            latitude_value: val.lat_long_value.clone().map(|lat_long| lat_long.0),
            longitude_value: val.lat_long_value.clone().map(|lat_long| lat_long.1),
            start_commit_num: val.start_commit_num,
            end_commit_num: MAX_COMMIT_NUM,
            service_id: val.service_id.clone(),
        });

        if val.struct_values.is_some() {
            let vs = val.struct_values.as_ref().unwrap();
            if !vals.is_empty() {
                vals.append(&mut make_reported_value_models(
                    vs,
                    Some(val.property_name.clone()),
                ));
            }
        }
    }

    vals
}

impl From<AssociatedAgentModel> for AssociatedAgent {
    fn from(model: AssociatedAgentModel) -> Self {
        Self {
            id: model.id,
            record_id: model.record_id,
            role: model.role,
            agent_id: model.agent_id,
            timestamp: model.timestamp,
            start_commit_num: model.start_commit_num,
            end_commit_num: model.end_commit_num,
            service_id: model.service_id,
        }
    }
}

impl From<AssociatedAgent> for AssociatedAgentModel {
    fn from(agent: AssociatedAgent) -> Self {
        Self {
            id: agent.id,
            record_id: agent.record_id,
            role: agent.role,
            agent_id: agent.agent_id,
            timestamp: agent.timestamp,
            start_commit_num: agent.start_commit_num,
            end_commit_num: agent.end_commit_num,
            service_id: agent.service_id,
        }
    }
}

impl From<PropertyModel> for Property {
    fn from(model: PropertyModel) -> Self {
        Self {
            id: model.id,
            name: model.name,
            record_id: model.record_id,
            property_definition: model.property_definition,
            current_page: model.current_page,
            wrapped: model.wrapped,
            start_commit_num: model.start_commit_num,
            end_commit_num: model.end_commit_num,
            service_id: model.service_id,
        }
    }
}

impl From<ProposalModel> for Proposal {
    fn from(model: ProposalModel) -> Self {
        Self {
            id: model.id,
            record_id: model.record_id,
            timestamp: model.timestamp,
            issuing_agent: model.issuing_agent,
            receiving_agent: model.receiving_agent,
            role: model.role,
            properties: model.properties.split(',').map(String::from).collect(),
            status: model.status,
            terms: model.terms,
            start_commit_num: model.start_commit_num,
            end_commit_num: model.end_commit_num,
            service_id: model.service_id,
        }
    }
}

impl From<RecordModel> for Record {
    fn from(model: RecordModel) -> Self {
        Self {
            id: model.id,
            record_id: model.record_id,
            schema: model.schema,
            final_: model.final_,
            owners: model.owners.split(',').map(String::from).collect(),
            custodians: model.custodians.split(',').map(String::from).collect(),
            start_commit_num: model.start_commit_num,
            end_commit_num: model.end_commit_num,
            service_id: model.service_id,
        }
    }
}

impl From<ReporterModel> for Reporter {
    fn from(model: ReporterModel) -> Self {
        Self {
            id: model.id,
            property_name: model.property_name,
            record_id: model.record_id,
            public_key: model.public_key,
            authorized: model.authorized,
            reporter_index: model.reporter_index,
            start_commit_num: model.start_commit_num,
            end_commit_num: model.end_commit_num,
            service_id: model.service_id,
        }
    }
}

impl From<ReportedValueReporterToAgentMetadataModel> for ReportedValueReporterToAgentMetadata {
    fn from(model: ReportedValueReporterToAgentMetadataModel) -> Self {
        Self {
            id: model.id,
            property_name: model.property_name,
            record_id: model.record_id,
            reporter_index: model.reporter_index,
            timestamp: model.timestamp,
            data_type: model.data_type,
            bytes_value: model.bytes_value,
            boolean_value: model.boolean_value,
            number_value: model.number_value,
            string_value: model.string_value,
            enum_value: model.enum_value,
            struct_values: Vec::new(),
            lat_long_value: create_lat_long_value(model.latitude_value, model.longitude_value),
            public_key: model.public_key,
            authorized: model.authorized,
            metadata: model.metadata,
            reported_value_end_commit_num: model.reported_value_end_commit_num,
            reporter_end_commit_num: model.reporter_end_commit_num,
            service_id: model.service_id,
        }
    }
}

impl
    From<(
        ReportedValueReporterToAgentMetadataModel,
        Vec<ReportedValueReporterToAgentMetadata>,
    )> for ReportedValueReporterToAgentMetadata
{
    fn from(
        (model, values): (
            ReportedValueReporterToAgentMetadataModel,
            Vec<ReportedValueReporterToAgentMetadata>,
        ),
    ) -> Self {
        Self {
            id: model.id,
            property_name: model.property_name,
            record_id: model.record_id,
            reporter_index: model.reporter_index,
            timestamp: model.timestamp,
            data_type: model.data_type,
            bytes_value: model.bytes_value,
            boolean_value: model.boolean_value,
            number_value: model.number_value,
            string_value: model.string_value,
            enum_value: model.enum_value,
            struct_values: values,
            lat_long_value: create_lat_long_value(model.latitude_value, model.longitude_value),
            public_key: model.public_key,
            authorized: model.authorized,
            metadata: model.metadata,
            reported_value_end_commit_num: model.reported_value_end_commit_num,
            reporter_end_commit_num: model.reporter_end_commit_num,
            service_id: model.service_id,
        }
    }
}

pub fn make_property_with_data_type(
    (model, data_type): (PropertyModel, Option<String>),
) -> (Property, Option<String>) {
    (Property::from(model), data_type)
}

pub fn create_lat_long_value(lat: Option<i64>, long: Option<i64>) -> Option<LatLongValue> {
    if let Some(latitude) = lat {
        if let Some(longitude) = long {
            Some(LatLongValue::from((latitude, longitude)))
        } else {
            None
        }
    } else {
        None
    }
}

impl From<DatabaseError> for TrackAndTraceStoreError {
    fn from(err: DatabaseError) -> TrackAndTraceStoreError {
        TrackAndTraceStoreError::ConnectionError(Box::new(err))
    }
}

impl From<diesel::result::Error> for TrackAndTraceStoreError {
    fn from(err: diesel::result::Error) -> TrackAndTraceStoreError {
        TrackAndTraceStoreError::QueryError {
            context: "Diesel query failed".to_string(),
            source: Box::new(err),
        }
    }
}

impl From<diesel::r2d2::PoolError> for TrackAndTraceStoreError {
    fn from(err: diesel::r2d2::PoolError) -> TrackAndTraceStoreError {
        TrackAndTraceStoreError::ConnectionError(Box::new(err))
    }
}
