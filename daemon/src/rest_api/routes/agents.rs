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
use actix_web::{AsyncResponder, HttpRequest, HttpResponse};
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
