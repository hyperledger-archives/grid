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

use crate::database::{helpers as db, models::Agent};
use crate::rest_api::{error::RestApiResponseError, routes::DbExecutor, AppState};

use actix::{Handler, Message, SyncContext};
use actix_web::{AsyncResponder, HttpRequest, HttpResponse, Path};
use futures::Future;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentSlice {
    pub public_key: String,
    pub org_id: String,
    pub active: bool,
    pub roles: Vec<String>,
    pub metadata: Vec<JsonValue>,
}

impl AgentSlice {
    pub fn from_agent(agent: &Agent) -> Self {
        Self {
            public_key: agent.public_key.clone(),
            org_id: agent.org_id.clone(),
            active: agent.active,
            roles: agent.roles.clone(),
            metadata: agent.metadata.clone(),
        }
    }
}

struct ListAgents;

impl Message for ListAgents {
    type Result = Result<Vec<AgentSlice>, RestApiResponseError>;
}

impl Handler<ListAgents> for DbExecutor {
    type Result = Result<Vec<AgentSlice>, RestApiResponseError>;

    fn handle(&mut self, _msg: ListAgents, _: &mut SyncContext<Self>) -> Self::Result {
        let fetched_agents = db::get_agents(&*self.connection_pool.get()?)?
            .iter()
            .map(|agent| AgentSlice::from_agent(agent))
            .collect();

        Ok(fetched_agents)
    }
}

pub fn list_agents(
    req: HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = RestApiResponseError>> {
    req.state()
        .database_connection
        .send(ListAgents)
        .from_err()
        .and_then(move |res| match res {
            Ok(agents) => Ok(HttpResponse::Ok().json(agents)),
            Err(err) => Err(err),
        })
        .responder()
}

struct FetchAgent {
    public_key: String,
}

impl Message for FetchAgent {
    type Result = Result<AgentSlice, RestApiResponseError>;
}

impl Handler<FetchAgent> for DbExecutor {
    type Result = Result<AgentSlice, RestApiResponseError>;

    fn handle(&mut self, msg: FetchAgent, _: &mut SyncContext<Self>) -> Self::Result {
        let fetched_agent = match db::get_agent(&*self.connection_pool.get()?, &msg.public_key)? {
            Some(agent) => AgentSlice::from_agent(&agent),
            None => {
                return Err(RestApiResponseError::NotFoundError(format!(
                    "Could not find agent with public key: {}",
                    msg.public_key
                )));
            }
        };

        Ok(fetched_agent)
    }
}

pub fn fetch_agent(
    req: HttpRequest<AppState>,
    public_key: Path<String>,
) -> impl Future<Item = HttpResponse, Error = RestApiResponseError> {
    req.state()
        .database_connection
        .send(FetchAgent {
            public_key: public_key.into_inner(),
        })
        .from_err()
        .and_then(move |res| match res {
            Ok(agent) => Ok(HttpResponse::Ok().json(agent)),
            Err(err) => Err(err),
        })
}
