// Copyright 2018-2021 Cargill Incorporated
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

use std::convert::TryFrom;
use std::sync::Arc;

use crate::{
    rest_api::resources::{error::ErrorResponse, paging::v1::Paging},
    track_and_trace::store::{
        AssociatedAgent, Property, Proposal, ReportedValueReporterToAgentMetadata,
        TrackAndTraceStore, TrackAndTraceStoreError,
    },
};

use super::payloads::{
    PropertySlice, PropertyValueSlice, RecordListSlice, RecordSlice, StructPropertyValue,
};

pub async fn list_records(
    store: Arc<dyn TrackAndTraceStore>,
    service_id: Option<&str>,
    offset: u64,
    limit: u16,
) -> Result<RecordListSlice, ErrorResponse> {
    let offset = i64::try_from(offset).unwrap_or(i64::MAX);

    let limit = i64::try_from(limit).unwrap_or(10);

    let record_list = store
        .list_records(service_id, offset, limit)
        .map_err(|err| match err {
            TrackAndTraceStoreError::InternalError(err) => {
                ErrorResponse::internal_error(Box::new(err))
            }
            TrackAndTraceStoreError::ConstraintViolationError(err) => {
                ErrorResponse::new(400, &format!("{}", err))
            }
            TrackAndTraceStoreError::ResourceTemporarilyUnavailableError(_) => {
                ErrorResponse::new(503, "Service Unavailable")
            }
            TrackAndTraceStoreError::NotFoundError(_) => {
                ErrorResponse::new(404, "Resource not found")
            }
        })?;

    let record_ids: Vec<String> = record_list
        .data
        .iter()
        .map(|record| record.record_id.to_string())
        .collect();

    let proposals = store
        .list_proposals(&record_ids, service_id.as_deref())
        .map_err(|err| match err {
            TrackAndTraceStoreError::InternalError(err) => {
                ErrorResponse::internal_error(Box::new(err))
            }
            TrackAndTraceStoreError::ConstraintViolationError(err) => {
                ErrorResponse::new(400, &format!("{}", err))
            }
            TrackAndTraceStoreError::ResourceTemporarilyUnavailableError(_) => {
                ErrorResponse::new(503, "Service Unavailable")
            }
            TrackAndTraceStoreError::NotFoundError(_) => {
                ErrorResponse::new(404, "Resource not found")
            }
        })?;

    let associated_agents = store
        .list_associated_agents(&record_ids, service_id)
        .map_err(|err| match err {
            TrackAndTraceStoreError::InternalError(err) => {
                ErrorResponse::internal_error(Box::new(err))
            }
            TrackAndTraceStoreError::ConstraintViolationError(err) => {
                ErrorResponse::new(400, &format!("{}", err))
            }
            TrackAndTraceStoreError::ResourceTemporarilyUnavailableError(_) => {
                ErrorResponse::new(503, "Service Unavailable")
            }
            TrackAndTraceStoreError::NotFoundError(_) => {
                ErrorResponse::new(404, "Resource not found")
            }
        })?;

    let properties = store
        .list_properties_with_data_type(&record_ids, service_id)
        .map_err(|err| match err {
            TrackAndTraceStoreError::InternalError(err) => {
                ErrorResponse::internal_error(Box::new(err))
            }
            TrackAndTraceStoreError::ConstraintViolationError(err) => {
                ErrorResponse::new(400, &format!("{}", err))
            }
            TrackAndTraceStoreError::ResourceTemporarilyUnavailableError(_) => {
                ErrorResponse::new(503, "Service Unavailable")
            }
            TrackAndTraceStoreError::NotFoundError(_) => {
                ErrorResponse::new(404, "Resource not found")
            }
        })?
        .iter()
        .map(|(property, data_type)| parse_property_slice(&store, property, data_type, service_id))
        .collect::<Result<Vec<PropertySlice>, _>>()?;

    let data = record_list
        .data
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
        .collect();

    let paging = Paging::new("/record", record_list.paging, service_id);

    Ok(RecordListSlice { data, paging })
}

pub async fn fetch_record(
    store: Arc<dyn TrackAndTraceStore>,
    record_id: String,
    service_id: Option<&str>,
) -> Result<RecordSlice, ErrorResponse> {
    let record = store
        .fetch_record(&record_id, service_id)
        .map_err(|err| match err {
            TrackAndTraceStoreError::InternalError(err) => {
                println!("WTF: {}", err);
                ErrorResponse::internal_error(Box::new(err))
            }
            TrackAndTraceStoreError::ConstraintViolationError(err) => {
                ErrorResponse::new(400, &format!("{}", err))
            }
            TrackAndTraceStoreError::ResourceTemporarilyUnavailableError(_) => {
                ErrorResponse::new(503, "Service Unavailable")
            }
            TrackAndTraceStoreError::NotFoundError(_) => {
                ErrorResponse::new(404, &format!("Record {} not found", record_id))
            }
        })?
        .ok_or_else(|| ErrorResponse::new(404, &format!("Resource {} not found", record_id)))?;

    let proposals = store
        .list_proposals(&[record_id.clone()], service_id)
        .map_err(|err| match err {
            TrackAndTraceStoreError::InternalError(err) => {
                ErrorResponse::internal_error(Box::new(err))
            }
            TrackAndTraceStoreError::ConstraintViolationError(err) => {
                ErrorResponse::new(400, &format!("{}", err))
            }
            TrackAndTraceStoreError::ResourceTemporarilyUnavailableError(_) => {
                ErrorResponse::new(503, "Service Unavailable")
            }
            TrackAndTraceStoreError::NotFoundError(_) => {
                ErrorResponse::new(404, "Resource not found")
            }
        })?;

    let properties = store
        .list_properties_with_data_type(&[record_id.clone()], service_id)
        .map_err(|err| match err {
            TrackAndTraceStoreError::InternalError(err) => {
                ErrorResponse::internal_error(Box::new(err))
            }
            TrackAndTraceStoreError::ConstraintViolationError(err) => {
                ErrorResponse::new(400, &format!("{}", err))
            }
            TrackAndTraceStoreError::ResourceTemporarilyUnavailableError(_) => {
                ErrorResponse::new(503, "Service Unavailable")
            }
            TrackAndTraceStoreError::NotFoundError(_) => {
                ErrorResponse::new(404, "Resource not found")
            }
        })?
        .iter()
        .map(|(property, data_type)| parse_property_slice(&store, property, data_type, service_id))
        .collect::<Result<Vec<PropertySlice>, _>>()?;

    let associated_agents = store
        .list_associated_agents(&[record_id], service_id)
        .map_err(|err| match err {
            TrackAndTraceStoreError::InternalError(err) => {
                ErrorResponse::internal_error(Box::new(err))
            }
            TrackAndTraceStoreError::ConstraintViolationError(err) => {
                ErrorResponse::new(400, &format!("{}", err))
            }
            TrackAndTraceStoreError::ResourceTemporarilyUnavailableError(_) => {
                ErrorResponse::new(503, "Service Unavailable")
            }
            TrackAndTraceStoreError::NotFoundError(_) => {
                ErrorResponse::new(404, "Resource not found")
            }
        })?;

    Ok(RecordSlice::from_models(
        record,
        proposals,
        associated_agents,
        properties,
    ))
}

pub async fn fetch_record_property(
    store: Arc<dyn TrackAndTraceStore>,
    record_id: String,
    property_name: String,
    service_id: Option<&str>,
) -> Result<PropertySlice, ErrorResponse> {
    let (property, data_type) = store
        .fetch_property_with_data_type(&record_id, &property_name, service_id)
        .map_err(|err| match err {
            TrackAndTraceStoreError::InternalError(err) => {
                ErrorResponse::internal_error(Box::new(err))
            }
            TrackAndTraceStoreError::ConstraintViolationError(err) => {
                ErrorResponse::new(400, &format!("{}", err))
            }
            TrackAndTraceStoreError::ResourceTemporarilyUnavailableError(_) => {
                ErrorResponse::new(503, "Service Unavailable")
            }
            TrackAndTraceStoreError::NotFoundError(_) => {
                ErrorResponse::new(404, &format!("Property {} not found", record_id))
            }
        })?
        .ok_or_else(|| ErrorResponse::new(404, &format!("Property {} not found", property_name)))?;

    parse_property_slice(&store, &property, &data_type, service_id)
}

fn parse_property_slice(
    store: &Arc<dyn TrackAndTraceStore>,
    property: &Property,
    data_type: &Option<String>,
    service_id: Option<&str>,
) -> Result<PropertySlice, ErrorResponse> {
    let reporters = store
        .list_reporters(&property.record_id, &property.name, service_id)
        .map_err(|err| match err {
            TrackAndTraceStoreError::InternalError(err) => {
                ErrorResponse::internal_error(Box::new(err))
            }
            TrackAndTraceStoreError::ConstraintViolationError(err) => {
                ErrorResponse::new(400, &format!("{}", err))
            }
            TrackAndTraceStoreError::ResourceTemporarilyUnavailableError(_) => {
                ErrorResponse::new(503, "Service Unavailable")
            }
            TrackAndTraceStoreError::NotFoundError(_) => {
                ErrorResponse::new(404, "Resource not found")
            }
        })?;

    let reported_value = store
        .fetch_reported_value_reporter_to_agent_metadata(
            &property.record_id,
            &property.name,
            None,
            service_id,
        )
        .map_err(|err| match err {
            TrackAndTraceStoreError::InternalError(err) => {
                ErrorResponse::internal_error(Box::new(err))
            }
            TrackAndTraceStoreError::ConstraintViolationError(err) => {
                ErrorResponse::new(400, &format!("{}", err))
            }
            TrackAndTraceStoreError::ResourceTemporarilyUnavailableError(_) => {
                ErrorResponse::new(503, "Service Unavailable")
            }
            TrackAndTraceStoreError::NotFoundError(_) => {
                ErrorResponse::new(404, "Resource not found")
            }
        })?;

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
        )
        .map_err(|err| match err {
            TrackAndTraceStoreError::InternalError(err) => {
                ErrorResponse::internal_error(Box::new(err))
            }
            TrackAndTraceStoreError::ConstraintViolationError(err) => {
                ErrorResponse::new(400, &format!("{}", err))
            }
            TrackAndTraceStoreError::ResourceTemporarilyUnavailableError(_) => {
                ErrorResponse::new(503, "Service Unavailable")
            }
            TrackAndTraceStoreError::NotFoundError(_) => {
                ErrorResponse::new(404, "Resource not found")
            }
        })?
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
) -> Result<PropertyValueSlice, ErrorResponse> {
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
) -> Result<Vec<StructPropertyValue>, ErrorResponse> {
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
