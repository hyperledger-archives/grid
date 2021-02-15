// Copyright 2019-2021 Cargill Incorporated
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

use super::roles::RoleSlice;
use actix::{Handler, Message, SyncContext};
use actix_web::{web, HttpResponse};
use grid_sdk::{
    pike::store::{Agent, Role},
    rest_api::resources::paging::v1::Paging,
};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentSlice {
    pub org_id: String,
    pub public_key: String,
    pub roles: Vec<RoleSlice>,
    pub active: bool,
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

impl TryFrom<(AgentSlice, Vec<RoleSlice>)> for AgentSlice {
    type Error = RestApiResponseError;

    fn try_from((agent, roles): (AgentSlice, Vec<RoleSlice>)) -> Result<Self, Self::Error> {
        Ok(Self {
            org_id: agent.org_id.clone(),
            public_key: agent.public_key.clone(),
            roles,
            active: agent.active,
            metadata: agent.metadata,
            service_id: agent.service_id,
        })
    }
}

impl TryFrom<(Agent, Vec<RoleSlice>)> for AgentSlice {
    type Error = RestApiResponseError;

    fn try_from((agent, roles): (Agent, Vec<RoleSlice>)) -> Result<Self, Self::Error> {
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
            org_id: agent.org_id.clone(),
            public_key: agent.public_key.clone(),
            roles,
            active: agent.active,
            metadata,
            service_id: agent.service_id,
        })
    }
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
            org_id: agent.org_id.clone(),
            public_key: agent.public_key.clone(),
            roles: Vec::new(),
            active: agent.active,
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

        let mut agent_slices = Vec::new();

        for agent in data {
            let mut roles = Vec::new();

            for r in &agent.roles {
                let role: Option<Role> = self
                    .pike_store
                    .fetch_role(&r.name, &r.org_id, msg.service_id.as_deref())
                    .map_err(|err| RestApiResponseError::DatabaseError(format!("{}", err)))?;

                if let Some(role) = role {
                    roles.push(RoleSlice::try_from(role)?)
                }
            }

            agent_slices.push(AgentSlice::try_from((agent, roles))?);
        }

        Ok(AgentListSlice {
            data: agent_slices,
            paging,
        })
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
            Some(agent) => {
                let mut roles = Vec::new();

                let byte_roles = agent.roles.clone();

                for r in byte_roles {
                    if r.contains('.') {
                        let split: Vec<&str> = r.split('.').collect();
                        let org_id = split[0];
                        let role_name = split[1];
                        let role: Option<Role> = self
                            .pike_store
                            .fetch_role(role_name, org_id, msg.service_id.as_deref())
                            .map_err(|err| {
                                RestApiResponseError::DatabaseError(format!("{}", err))
                            })?;

                        if let Some(role) = role {
                            roles.push(RoleSlice::try_from(role)?)
                        }
                    } else {
                        let role: Option<Role> = self
                            .pike_store
                            .fetch_role(&r, &agent.org_id, msg.service_id.as_deref())
                            .map_err(|err| {
                                RestApiResponseError::DatabaseError(format!("{}", err))
                            })?;

                        if let Some(role) = role {
                            roles.push(RoleSlice::try_from(role)?)
                        }
                    }
                }

                Ok(AgentSlice::try_from((agent, roles))?)
            }
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
