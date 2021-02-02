// Copyright 2019 Cargill Incorporated
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

use std::sync::Arc;

use crate::rest_api::{
    error::RestApiResponseError, routes::DbExecutor, AcceptServiceIdParam, AppState, QueryServiceId,
};

use actix::{Handler, Message, SyncContext};
use actix_web::{web, HttpResponse};
use grid_sdk::track_and_trace::store::{
    AssociatedAgent, LatLongValue, Property, Proposal, Record,
    ReportedValueReporterToAgentMetadata, TrackAndTraceStore,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AssociatedAgentSlice {
    pub agent_id: String,
    pub timestamp: u64,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_id: Option<String>,
}

impl From<AssociatedAgent> for AssociatedAgentSlice {
    fn from(associated_agent: AssociatedAgent) -> Self {
        Self {
            agent_id: associated_agent.agent_id.clone(),
            timestamp: associated_agent.timestamp as u64,
            service_id: associated_agent.service_id,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProposalSlice {
    pub receiving_agent: String,
    pub issuing_agent: String,
    pub role: String,
    pub properties: Vec<String>,
    pub status: String,
    pub terms: String,
    pub timestamp: u64,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_id: Option<String>,
}

impl From<Proposal> for ProposalSlice {
    fn from(proposal: Proposal) -> Self {
        Self {
            receiving_agent: proposal.receiving_agent.clone(),
            issuing_agent: proposal.issuing_agent.clone(),
            role: proposal.role.clone(),
            properties: proposal.properties.clone(),
            status: proposal.status.clone(),
            terms: proposal.terms.clone(),
            timestamp: proposal.timestamp as u64,
            service_id: proposal.service_id,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecordSlice {
    pub record_id: String,
    pub schema: String,
    pub owner: String,
    pub custodian: String,
    pub properties: Vec<PropertySlice>,
    pub r#final: bool,
    pub proposals: Vec<ProposalSlice>,
    pub owner_updates: Vec<AssociatedAgentSlice>,
    pub custodian_updates: Vec<AssociatedAgentSlice>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_id: Option<String>,
}

impl RecordSlice {
    pub fn from_models(
        record: Record,
        proposals: Vec<Proposal>,
        associated_agents: Vec<AssociatedAgent>,
        properties: Vec<PropertySlice>,
    ) -> Self {
        let mut owner_updates: Vec<AssociatedAgentSlice> = associated_agents
            .clone()
            .into_iter()
            .filter(|agent| agent.role.eq("OWNER"))
            .map(AssociatedAgentSlice::from)
            .collect();
        let mut custodian_updates: Vec<AssociatedAgentSlice> = associated_agents
            .into_iter()
            .filter(|agent| agent.role.eq("CUSTODIAN"))
            .map(AssociatedAgentSlice::from)
            .collect();

        owner_updates.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        custodian_updates.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        Self {
            record_id: record.record_id.clone(),
            schema: record.schema.clone(),
            owner: match owner_updates.last() {
                Some(owner) => owner.agent_id.clone(),
                None => "".to_string(),
            },
            custodian: match custodian_updates.last() {
                Some(custodian) => custodian.agent_id.clone(),
                None => "".to_string(),
            },
            properties: properties.to_vec(),
            r#final: record.final_,
            proposals: proposals.into_iter().map(ProposalSlice::from).collect(),
            owner_updates,
            custodian_updates,
            service_id: record.service_id,
        }
    }
}

struct ListRecords {
    service_id: Option<String>,
}

impl Message for ListRecords {
    type Result = Result<Vec<RecordSlice>, RestApiResponseError>;
}

impl Handler<ListRecords> for DbExecutor {
    type Result = Result<Vec<RecordSlice>, RestApiResponseError>;

    fn handle(&mut self, msg: ListRecords, _: &mut SyncContext<Self>) -> Self::Result {
        let records = self.tnt_store.list_records(msg.service_id.as_deref())?;

        let record_ids: Vec<String> = records
            .iter()
            .map(|record| record.record_id.to_string())
            .collect();

        let proposals = self
            .tnt_store
            .list_proposals(&record_ids, msg.service_id.as_deref())?;
        let associated_agents = self
            .tnt_store
            .list_associated_agents(&record_ids, msg.service_id.as_deref())?;

        let properties = self
            .tnt_store
            .list_properties_with_data_type(&record_ids, msg.service_id.as_deref())?
            .iter()
            .map(|(property, data_type)| {
                parse_property_slice(
                    &self.tnt_store,
                    property,
                    data_type,
                    msg.service_id.as_deref(),
                )
            })
            .collect::<Result<Vec<PropertySlice>, _>>()?;

        Ok(records
            .into_iter()
            .map(|record| {
                let props: Vec<Proposal> = proposals
                    .iter()
                    .filter(|proposal| proposal.record_id.eq(&record.record_id))
                    .cloned()
                    .collect();
                let agents: Vec<AssociatedAgent> = associated_agents
                    .iter()
                    .filter(|agent| agent.record_id.eq(&record.record_id))
                    .cloned()
                    .collect();

                let record_properties: Vec<PropertySlice> = properties
                    .iter()
                    .filter(|property| property.record_id.eq(&record.record_id))
                    .cloned()
                    .collect();

                RecordSlice::from_models(record, props, agents, record_properties)
            })
            .collect())
    }
}

pub async fn list_records(
    state: web::Data<AppState>,
    query: web::Query<QueryServiceId>,
    _: AcceptServiceIdParam,
) -> Result<HttpResponse, RestApiResponseError> {
    state
        .database_connection
        .send(ListRecords {
            service_id: query.into_inner().service_id,
        })
        .await?
        .map(|records| HttpResponse::Ok().json(records))
}

struct FetchRecord {
    record_id: String,
    service_id: Option<String>,
}

impl Message for FetchRecord {
    type Result = Result<RecordSlice, RestApiResponseError>;
}

impl Handler<FetchRecord> for DbExecutor {
    type Result = Result<RecordSlice, RestApiResponseError>;

    fn handle(&mut self, msg: FetchRecord, _: &mut SyncContext<Self>) -> Self::Result {
        let record = match self
            .tnt_store
            .fetch_record(&msg.record_id, msg.service_id.as_deref())?
        {
            Some(record) => record,
            None => {
                return Err(RestApiResponseError::NotFoundError(format!(
                    "Could not find record with id: {}",
                    msg.record_id
                )));
            }
        };

        let proposals = self
            .tnt_store
            .list_proposals(&[msg.record_id.clone()], msg.service_id.as_deref())?;

        let properties = self
            .tnt_store
            .list_properties_with_data_type(&[msg.record_id.clone()], msg.service_id.as_deref())?
            .iter()
            .map(|(property, data_type)| {
                parse_property_slice(
                    &self.tnt_store,
                    property,
                    data_type,
                    msg.service_id.as_deref(),
                )
            })
            .collect::<Result<Vec<PropertySlice>, _>>()?;

        let associated_agents = self
            .tnt_store
            .list_associated_agents(&[msg.record_id], msg.service_id.as_deref())?;

        Ok(RecordSlice::from_models(
            record,
            proposals,
            associated_agents,
            properties,
        ))
    }
}

pub async fn fetch_record(
    state: web::Data<AppState>,
    record_id: web::Path<String>,
    query: web::Query<QueryServiceId>,
    _: AcceptServiceIdParam,
) -> Result<HttpResponse, RestApiResponseError> {
    state
        .database_connection
        .send(FetchRecord {
            record_id: record_id.into_inner(),
            service_id: query.into_inner().service_id,
        })
        .await?
        .map(|record| HttpResponse::Ok().json(record))
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PropertySlice {
    pub name: String,
    pub record_id: String,
    pub data_type: String,
    pub reporters: Vec<String>,
    pub updates: Vec<PropertyValueSlice>,
    pub value: Option<PropertyValueSlice>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_id: Option<String>,
}
impl PropertySlice {
    pub fn from_model(
        property: &Property,
        reporters: &[String],
        data_type: &str,
        updates: &[PropertyValueSlice],
        value: Option<PropertyValueSlice>,
    ) -> PropertySlice {
        PropertySlice {
            name: property.name.clone(),
            record_id: property.record_id.clone(),
            data_type: data_type.to_string(),
            reporters: reporters.to_vec(),
            updates: updates.to_vec(),
            value,
            service_id: property.service_id.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PropertyValueSlice {
    pub timestamp: u64,
    pub value: Value,
    pub reporter: ReporterSlice,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum Value {
    String(String),
    Bool(bool),
    Struct(Vec<StructPropertyValue>),
    LatLong(LatLong),
    Number(i64),
    Enum(i32),
    Bytes(String),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LatLong {
    pub latitude: i64,
    pub longitude: i64,
}

impl LatLong {
    pub fn from_model(lat_long_value: LatLongValue) -> LatLong {
        LatLong {
            latitude: lat_long_value.0,
            longitude: lat_long_value.1,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReporterSlice {
    pub public_key: String,
    pub metadata: ReportedValueReporterToAgentMetadata,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct StructPropertyValue {
    pub name: String,
    pub data_type: String,
    pub value: Value,
}

impl StructPropertyValue {
    pub fn from_model(
        value_name: &str,
        reported_value_with_reporter: &ReportedValueReporterToAgentMetadata,
        struct_values: Option<Vec<StructPropertyValue>>,
    ) -> Result<StructPropertyValue, RestApiResponseError> {
        Ok(StructPropertyValue {
            name: value_name.to_string(),
            data_type: reported_value_with_reporter.data_type.clone(),
            value: parse_value(reported_value_with_reporter, struct_values)?,
        })
    }
}

impl PropertyValueSlice {
    pub fn from_model(
        reported_value_with_reporter: &ReportedValueReporterToAgentMetadata,
        struct_values: Option<Vec<StructPropertyValue>>,
    ) -> Result<PropertyValueSlice, RestApiResponseError> {
        Ok(PropertyValueSlice {
            timestamp: reported_value_with_reporter.timestamp as u64,
            value: parse_value(reported_value_with_reporter, struct_values)?,
            reporter: ReporterSlice {
                public_key: reported_value_with_reporter
                    .public_key
                    .clone()
                    .unwrap_or_else(|| "".to_string()),
                metadata: reported_value_with_reporter.clone(),
                service_id: reported_value_with_reporter.service_id.clone(),
            },
            service_id: reported_value_with_reporter.service_id.clone(),
        })
    }
}

fn parse_value(
    val: &ReportedValueReporterToAgentMetadata,
    struct_values: Option<Vec<StructPropertyValue>>,
) -> Result<Value, RestApiResponseError> {
    match val.data_type.as_ref() {
        "String" => {
            let string_value = val.string_value.clone().ok_or_else(|| {
                RestApiResponseError::DatabaseError(
                    "ReportedValue is of String data_type, but is missing string value".to_string(),
                )
            })?;

            Ok(Value::String(string_value))
        }
        "Boolean" => {
            let boolean_value = val.boolean_value.ok_or_else(|| {
                RestApiResponseError::DatabaseError(
                    "ReportedValue is of Boolean data_type, but is missing boolean value"
                        .to_string(),
                )
            })?;

            Ok(Value::Bool(boolean_value))
        }
        "Enum" => {
            let enum_value = val.enum_value.ok_or_else(|| {
                RestApiResponseError::DatabaseError(
                    "ReportedValue is of Enum data_type, but is missing enum value".to_string(),
                )
            })?;

            Ok(Value::Enum(enum_value))
        }
        "LatLong" => {
            let lat_long = match val.lat_long_value.clone() {
                Some(lat_long_value) => LatLong::from_model(lat_long_value),
                None => {
                    return Err(RestApiResponseError::DatabaseError(
                        "ReportedValue is of LatLong data_type, but is missing lat_long value"
                            .to_string(),
                    ))
                }
            };
            Ok(Value::LatLong(lat_long))
        }
        "Number" => {
            let number_value = val.number_value.ok_or_else(|| {
                RestApiResponseError::DatabaseError(
                    "ReportedValue is of Number data_type, but is missing number value".to_string(),
                )
            })?;

            Ok(Value::Number(number_value))
        }
        "Bytes" => {
            let bytes_value = val.bytes_value.clone().ok_or_else(|| {
                RestApiResponseError::DatabaseError(
                    "ReportedValue is of Bytes data_type, but is missing bytes value".to_string(),
                )
            })?;
            let encoded_bytes = base64::encode(&bytes_value);
            Ok(Value::Bytes(encoded_bytes))
        }
        "Struct" => {
            let value = struct_values.ok_or_else(|| {
                RestApiResponseError::DatabaseError(
                    "ReportedValue is of Struct data_type, but is missing struct value".to_string(),
                )
            })?;

            Ok(Value::Struct(value))
        }
        _ => Err(RestApiResponseError::DatabaseError(format!(
            "Invalid data type in ReportedValue: {}",
            val.data_type
        ))),
    }
}

struct FetchRecordProperty {
    record_id: String,
    property_name: String,
    service_id: Option<String>,
}

impl Message for FetchRecordProperty {
    type Result = Result<PropertySlice, RestApiResponseError>;
}

pub async fn fetch_record_property(
    state: web::Data<AppState>,
    params: web::Path<(String, String)>,
    query: web::Query<QueryServiceId>,
    _: AcceptServiceIdParam,
) -> Result<HttpResponse, RestApiResponseError> {
    let (record_id, property_name) = params.into_inner();
    state
        .database_connection
        .send(FetchRecordProperty {
            record_id,
            property_name,
            service_id: query.into_inner().service_id,
        })
        .await?
        .map(|record| HttpResponse::Ok().json(record))
}

impl Handler<FetchRecordProperty> for DbExecutor {
    type Result = Result<PropertySlice, RestApiResponseError>;

    fn handle(&mut self, msg: FetchRecordProperty, _: &mut SyncContext<Self>) -> Self::Result {
        let (property, data_type) = self
            .tnt_store
            .fetch_property_with_data_type(
                &msg.record_id,
                &msg.property_name,
                msg.service_id.as_deref(),
            )?
            .ok_or_else(|| {
                RestApiResponseError::NotFoundError(format!(
                    "Could not find property {} for record {}",
                    msg.property_name, msg.record_id
                ))
            })?;

        parse_property_slice(
            &self.tnt_store,
            &property,
            &data_type,
            msg.service_id.as_deref(),
        )
    }
}

fn parse_property_slice(
    store: &Arc<dyn TrackAndTraceStore>,
    property: &Property,
    data_type: &Option<String>,
    service_id: Option<&str>,
) -> Result<PropertySlice, RestApiResponseError> {
    let reporters = store.list_reporters(&property.record_id, &property.name, service_id)?;

    let reported_value = store.fetch_reported_value_reporter_to_agent_metadata(
        &property.record_id,
        &property.name,
        None,
        service_id,
    )?;

    let property_value_slice = match reported_value {
        Some(value) => Some(parse_reported_values(&value, service_id)?),
        None => None,
    };

    let active_reporters = reporters
        .iter()
        .filter_map(|reporter| {
            if reporter.authorized {
                Some(reporter.public_key.clone())
            } else {
                None
            }
        })
        .collect::<Vec<String>>();

    let mut updates = store
        .list_reported_value_reporter_to_agent_metadata(
            &property.record_id,
            &property.name,
            service_id,
        )?
        .iter()
        .map(|reported_value| parse_reported_values(reported_value, service_id))
        .collect::<Result<Vec<PropertyValueSlice>, _>>()?;

    // Sort updates from oldest to newest.
    updates.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    let property_info = PropertySlice::from_model(
        &property,
        &active_reporters,
        &data_type.clone().unwrap_or_else(|| "Unknown".to_string()),
        &updates,
        property_value_slice,
    );

    Ok(property_info)
}

fn parse_reported_values(
    reported_value: &ReportedValueReporterToAgentMetadata,
    service_id: Option<&str>,
) -> Result<PropertyValueSlice, RestApiResponseError> {
    let struct_values = if reported_value.data_type == "Struct" {
        Some(parse_struct_values(
            &reported_value.record_id,
            &reported_value.struct_values,
            service_id,
        )?)
    } else {
        None
    };

    PropertyValueSlice::from_model(&reported_value, struct_values)
}

fn parse_struct_values(
    record_id: &str,
    struct_values: &[ReportedValueReporterToAgentMetadata],
    service_id: Option<&str>,
) -> Result<Vec<StructPropertyValue>, RestApiResponseError> {
    let mut inner_values = vec![];

    for struct_value in struct_values {
        if struct_value.data_type == "Struct" {
            let inner_struct_values =
                parse_struct_values(record_id, &struct_value.struct_values, service_id)?;
            inner_values.push(StructPropertyValue::from_model(
                &struct_value.property_name,
                &struct_value,
                Some(inner_struct_values),
            )?);
        } else {
            inner_values.push(StructPropertyValue::from_model(
                &struct_value.property_name,
                &struct_value,
                None,
            )?);
        }
    }
    Ok(inner_values)
}
