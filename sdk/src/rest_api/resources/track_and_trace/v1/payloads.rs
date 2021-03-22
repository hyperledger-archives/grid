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

use crate::{
    rest_api::resources::{error::ErrorResponse, paging::v1::Paging},
    track_and_trace::store::{
        AssociatedAgent, LatLongValue, Property, Proposal, Record,
        ReportedValueReporterToAgentMetadata,
    },
};

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

#[derive(Debug, Serialize, Deserialize)]
pub struct RecordListSlice {
    pub data: Vec<RecordSlice>,
    pub paging: Paging,
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
    ) -> Result<StructPropertyValue, ErrorResponse> {
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
    ) -> Result<PropertyValueSlice, ErrorResponse> {
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
) -> Result<Value, ErrorResponse> {
    match val.data_type.as_ref() {
        "String" => {
            let string_value = val.string_value.clone().ok_or_else(|| {
                ErrorResponse::new(
                    500,
                    "ReportedValue is of String data_type, but is missing string value",
                )
            })?;

            Ok(Value::String(string_value))
        }
        "Boolean" => {
            let boolean_value = val.boolean_value.ok_or_else(|| {
                ErrorResponse::new(
                    500,
                    "ReportedValue is of Boolean data_type, but is missing boolean value",
                )
            })?;

            Ok(Value::Bool(boolean_value))
        }
        "Enum" => {
            let enum_value = val.enum_value.ok_or_else(|| {
                ErrorResponse::new(
                    500,
                    "ReportedValue is of Enum data_type, but is missing enum value",
                )
            })?;

            Ok(Value::Enum(enum_value))
        }
        "LatLong" => {
            let lat_long = match val.lat_long_value.clone() {
                Some(lat_long_value) => LatLong::from_model(lat_long_value),
                None => {
                    return Err(ErrorResponse::new(
                        500,
                        "ReportedValue is of LatLong data_type, but is missing lat_long value",
                    ))
                }
            };
            Ok(Value::LatLong(lat_long))
        }
        "Number" => {
            let number_value = val.number_value.ok_or_else(|| {
                ErrorResponse::new(
                    500,
                    "ReportedValue is of Number data_type, but is missing number value",
                )
            })?;

            Ok(Value::Number(number_value))
        }
        "Bytes" => {
            let bytes_value = val.bytes_value.clone().ok_or_else(|| {
                ErrorResponse::new(
                    500,
                    "ReportedValue is of Bytes data_type, but is missing bytes value",
                )
            })?;
            let encoded_bytes = base64::encode(&bytes_value);
            Ok(Value::Bytes(encoded_bytes))
        }
        "Struct" => {
            let value = struct_values.ok_or_else(|| {
                ErrorResponse::new(
                    500,
                    "ReportedValue is of Struct data_type, but is missing struct value",
                )
            })?;

            Ok(Value::Struct(value))
        }
        _ => Err(ErrorResponse::new(
            500,
            &format!("Invalid data type in ReportedValue: {}", val.data_type),
        )),
    }
}
