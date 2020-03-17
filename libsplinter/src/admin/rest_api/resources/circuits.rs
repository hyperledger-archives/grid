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

use std::collections::BTreeMap;

use crate::circuit::{Circuit, Roster, ServiceDefinition};
use crate::rest_api::paging::Paging;

#[derive(Debug, Serialize, Clone, PartialEq)]
pub(crate) struct ListCircuitsResponse<'a> {
    pub data: Vec<CircuitResponse<'a>>,
    pub paging: Paging,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub(crate) struct CircuitResponse<'a> {
    pub id: &'a str,
    pub members: Vec<String>,
    pub roster: Vec<ServiceResponse<'a>>,
    pub management_type: &'a str,
}

impl<'a> From<&'a Circuit> for CircuitResponse<'a> {
    fn from(circuit: &'a Circuit) -> Self {
        Self {
            id: circuit.id(),
            members: circuit.members().to_vec(),
            roster: circuit.roster().into(),
            management_type: circuit.circuit_management_type(),
        }
    }
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub(crate) struct ServiceResponse<'a> {
    pub service_id: &'a str,
    pub service_type: &'a str,
    pub allowed_nodes: &'a [String],
    pub arguments: &'a BTreeMap<String, String>,
}

impl<'a> From<&'a ServiceDefinition> for ServiceResponse<'a> {
    fn from(service_def: &'a ServiceDefinition) -> Self {
        Self {
            service_id: service_def.service_id(),
            service_type: service_def.service_type(),
            allowed_nodes: service_def.allowed_nodes(),
            arguments: service_def.arguments(),
        }
    }
}

impl<'a> From<&'a Roster> for Vec<ServiceResponse<'a>> {
    fn from(roster: &'a Roster) -> Self {
        match roster {
            Roster::Standard(service_defs) => {
                service_defs.iter().map(ServiceResponse::from).collect()
            }
            Roster::Admin => vec![],
        }
    }
}
