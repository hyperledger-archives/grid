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
    error::RestApiResponseError, routes::DbExecutor, AcceptServiceIdParam, AppState, QueryServiceId,
};

use super::RoleSlice;
use actix::{Handler, Message, SyncContext};
use actix_web::{web, HttpResponse};
use grid_sdk::agents::store::Agent;
use grid_sdk::roles::store::Role;
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

struct ListAgents {
    service_id: Option<String>,
}

impl Message for ListAgents {
    type Result = Result<Vec<AgentSlice>, RestApiResponseError>;
}

impl Handler<ListAgents> for DbExecutor {
    type Result = Result<Vec<AgentSlice>, RestApiResponseError>;

    fn handle(&mut self, msg: ListAgents, _: &mut SyncContext<Self>) -> Self::Result {
        let mut agent_slices = Vec::new();

        let agents = self
            .agent_store
            .list_agents(msg.service_id.as_deref())?
            .into_iter();

        for agent in agents {
            let mut roles = Vec::new();

            let role_string = String::from_utf8(agent.roles.clone()).map_err(|err| {
                RestApiResponseError::RequestHandlerError(format!(
                    "Could not fetch roles for agent {}",
                    err
                ))
            })?;

            let byte_roles: Vec<&str> = role_string.split(",").collect();

            for r in byte_roles {
                let role: Option<Role> =
                    self.role_store
                        .fetch_role(r, msg.service_id.as_deref())
                        .map_err(|err| RestApiResponseError::DatabaseError(format!("{}", err)))?;

                match role {
                    Some(role) => roles.push(RoleSlice::try_from(role)?),
                    None => {}
                }
            }

            agent_slices.push(AgentSlice::try_from((agent, roles))?);
        }

        Ok(agent_slices)
    }
}

pub async fn list_agents(
    state: web::Data<AppState>,
    query: web::Query<QueryServiceId>,
    _: AcceptServiceIdParam,
) -> Result<HttpResponse, RestApiResponseError> {
    state
        .database_connection
        .send(ListAgents {
            service_id: query.into_inner().service_id,
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
            .agent_store
            .fetch_agent(&msg.public_key, msg.service_id.as_deref())?
        {
            Some(agent) => {
                let mut roles = Vec::new();

                let role_string = String::from_utf8(agent.roles.clone()).map_err(|err| {
                    RestApiResponseError::RequestHandlerError(format!(
                        "Could not fetch roles for agent {}",
                        err
                    ))
                })?;

                let byte_roles: Vec<&str> = role_string.split(",").collect();

                for r in byte_roles {
                    let role: Option<Role> = self
                        .role_store
                        .fetch_role(r, msg.service_id.as_deref())
                        .map_err(|err| RestApiResponseError::DatabaseError(format!("{}", err)))?;

                    match role {
                        Some(role) => roles.push(RoleSlice::try_from(role)?),
                        None => {}
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
