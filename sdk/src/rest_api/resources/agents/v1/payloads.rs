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

use std::{convert::TryFrom, str::FromStr};

use crate::{
    pike::store::Agent,
    rest_api::resources::{error::ErrorResponse, paging::v1::Paging},
};

use serde_json::Value as JsonValue;

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentSlice {
    pub public_key: String,
    pub org_id: String,
    pub active: bool,
    pub roles: Vec<String>,
    pub metadata: JsonValue,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentListSlice {
    pub data: Vec<AgentSlice>,
    pub paging: Paging,
}

impl TryFrom<Agent> for AgentSlice {
    type Error = ErrorResponse;

    fn try_from(agent: Agent) -> Result<Self, Self::Error> {
        let metadata = if !agent.metadata.is_empty() {
            JsonValue::from_str(
                &String::from_utf8(agent.metadata.clone())
                    .map_err(|err| ErrorResponse::internal_error(Box::new(err)))?,
            )
            .map_err(|err| ErrorResponse::internal_error(Box::new(err)))?
        } else {
            json!([])
        };

        Ok(Self {
            public_key: agent.public_key.clone(),
            org_id: agent.org_id.clone(),
            active: agent.active,
            roles: agent.roles.clone(),
            metadata,
            service_id: agent.service_id,
        })
    }
}
