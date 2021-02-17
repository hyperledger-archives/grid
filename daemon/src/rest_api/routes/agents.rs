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

use std::{convert::TryFrom, str::FromStr};

use crate::rest_api::{
    error::RestApiResponseError, routes::DbExecutor, AcceptServiceIdParam, AppState, QueryPaging,
    QueryServiceId,
};

use actix::{Handler, Message, SyncContext};
use actix_web::{web, HttpResponse};
use grid_sdk::{pike::store::Agent, rest_api::resources::paging::v1::Paging};
use serde::{Deserialize, Serialize};
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
    type Error = RestApiResponseError;

    fn try_from(agent: Agent) -> Result<Self, Self::Error> {
        let metadata = if !agent.metadata.is_empty() {
            JsonValue::from_str(
                &String::from_utf8(agent.metadata.clone())
                    .map_err(|err| RestApiResponseError::DatabaseError(format!("{}", err)))?,
            )
            .map_err(|err| RestApiResponseError::DatabaseError(format!("{}", err)))?
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

struct ListAgents {
    service_id: Option<String>,
    offset: u64,
    limit: u16,
}

impl Message for ListAgents {
    type Result = Result<AgentListSlice, RestApiResponseError>;
}

impl Handler<ListAgents> for DbExecutor {
    type Result = Result<AgentListSlice, RestApiResponseError>;

    fn handle(&mut self, msg: ListAgents, _: &mut SyncContext<Self>) -> Self::Result {
        let offset = i64::try_from(msg.offset).unwrap_or(i64::MAX);

        let limit = i64::try_from(msg.limit).unwrap_or(10);

        let agent_list = self
            .pike_store
            .list_agents(msg.service_id.as_deref(), offset, limit)?;

        let data = agent_list
            .data
            .into_iter()
            .map(AgentSlice::try_from)
            .collect::<Result<Vec<AgentSlice>, RestApiResponseError>>()?;

        let paging = Paging::new("/agent", agent_list.paging, msg.service_id.as_deref());

        Ok(AgentListSlice { data, paging })
    }
}

pub async fn list_agents(
    state: web::Data<AppState>,
    query_service_id: web::Query<QueryServiceId>,
    query_paging: web::Query<QueryPaging>,
    _: AcceptServiceIdParam,
) -> Result<HttpResponse, RestApiResponseError> {
    let paging = query_paging.into_inner();
    state
        .database_connection
        .send(ListAgents {
            service_id: query_service_id.into_inner().service_id,
            offset: paging.offset(),
            limit: paging.limit(),
        })
        .await?
        .map(|agents| HttpResponse::Ok().json(agents))
}

struct FetchAgent {
    public_key: String,
    service_id: Option<String>,
}

impl Message for FetchAgent {
    type Result = Result<AgentSlice, RestApiResponseError>;
}

impl Handler<FetchAgent> for DbExecutor {
    type Result = Result<AgentSlice, RestApiResponseError>;

    fn handle(&mut self, msg: FetchAgent, _: &mut SyncContext<Self>) -> Self::Result {
        match self
            .pike_store
            .fetch_agent(&msg.public_key, msg.service_id.as_deref())?
        {
            Some(agent) => AgentSlice::try_from(agent),
            None => Err(RestApiResponseError::NotFoundError(format!(
                "Could not find agent with public key: {}",
                msg.public_key
            ))),
        }
    }
}

pub async fn fetch_agent(
    state: web::Data<AppState>,
    public_key: web::Path<String>,
    query: web::Query<QueryServiceId>,
    _: AcceptServiceIdParam,
) -> Result<HttpResponse, RestApiResponseError> {
    state
        .database_connection
        .send(FetchAgent {
            public_key: public_key.into_inner(),
            service_id: query.into_inner().service_id,
        })
        .await?
        .map(|agent| HttpResponse::Ok().json(agent))
}
