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

use crate::database::{helpers as db, models::AssociatedAgent, models::Proposal, models::Record};
use crate::rest_api::{error::RestApiResponseError, routes::DbExecutor, AppState};

use actix::{Handler, Message, SyncContext};
use actix_web::{HttpRequest, HttpResponse, Path};
use futures::Future;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AssociatedAgentSlice {
    pub agent_id: String,
    pub timestamp: u64,
}

impl AssociatedAgentSlice {
    pub fn from_model(associated_agent: &AssociatedAgent) -> Self {
        Self {
            agent_id: associated_agent.agent_id.clone(),
            timestamp: associated_agent.timestamp as u64,
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
}

impl ProposalSlice {
    pub fn from_model(proposal: &Proposal) -> Self {
        Self {
            receiving_agent: proposal.receiving_agent.clone(),
            issuing_agent: proposal.issuing_agent.clone(),
            role: proposal.role.clone(),
            properties: proposal.properties.clone(),
            status: proposal.status.clone(),
            terms: proposal.terms.clone(),
            timestamp: proposal.timestamp as u64,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecordSlice {
    pub record_id: String,
    pub owner: String,
    pub custodian: String,
    pub r#final: bool,
    pub proposals: Vec<ProposalSlice>,
    pub owner_updates: Vec<AssociatedAgentSlice>,
    pub custodian_updates: Vec<AssociatedAgentSlice>,
}

impl RecordSlice {
    pub fn from_models(
        record: &Record,
        proposals: &[Proposal],
        associated_agents: &[AssociatedAgent],
    ) -> Self {
        let mut owner_updates: Vec<AssociatedAgentSlice> = associated_agents
            .iter()
            .filter(|agent| agent.role.eq("OWNER"))
            .map(AssociatedAgentSlice::from_model)
            .collect();
        let mut custodian_updates: Vec<AssociatedAgentSlice> = associated_agents
            .iter()
            .filter(|agent| agent.role.eq("CUSTODIAN"))
            .map(AssociatedAgentSlice::from_model)
            .collect();

        owner_updates.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        custodian_updates.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        Self {
            record_id: record.record_id.clone(),
            owner: match owner_updates.last() {
                Some(owner) => owner.agent_id.clone(),
                None => "".to_string(),
            },
            custodian: match custodian_updates.last() {
                Some(custodian) => custodian.agent_id.clone(),
                None => "".to_string(),
            },
            r#final: record.final_,
            proposals: proposals.iter().map(ProposalSlice::from_model).collect(),
            owner_updates,
            custodian_updates,
        }
    }
}

struct ListRecords;

impl Message for ListRecords {
    type Result = Result<Vec<RecordSlice>, RestApiResponseError>;
}

impl Handler<ListRecords> for DbExecutor {
    type Result = Result<Vec<RecordSlice>, RestApiResponseError>;

    fn handle(&mut self, _msg: ListRecords, _: &mut SyncContext<Self>) -> Self::Result {
        let records = db::list_records(&*self.connection_pool.get()?)?;

        let record_ids: Vec<String> = records
            .iter()
            .map(|record| record.record_id.to_string())
            .collect();

        let proposals = db::list_proposals(&*self.connection_pool.get()?, &record_ids)?;
        let associated_agents =
            db::list_associated_agents(&*self.connection_pool.get()?, &record_ids)?;

        Ok(records
            .iter()
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

                RecordSlice::from_models(record, &props, &agents)
            })
            .collect())
    }
}

pub fn list_records(
    req: HttpRequest<AppState>,
) -> impl Future<Item = HttpResponse, Error = RestApiResponseError> {
    req.state()
        .database_connection
        .send(ListRecords)
        .from_err()
        .and_then(move |res| match res {
            Ok(records) => Ok(HttpResponse::Ok().json(records)),
            Err(err) => Err(err),
        })
}

struct FetchRecord {
    record_id: String,
}

impl Message for FetchRecord {
    type Result = Result<RecordSlice, RestApiResponseError>;
}

impl Handler<FetchRecord> for DbExecutor {
    type Result = Result<RecordSlice, RestApiResponseError>;

    fn handle(&mut self, msg: FetchRecord, _: &mut SyncContext<Self>) -> Self::Result {
        let record = match db::fetch_record(&*self.connection_pool.get()?, &msg.record_id)? {
            Some(record) => record,
            None => {
                return Err(RestApiResponseError::NotFoundError(format!(
                    "Could not find record with id: {}",
                    msg.record_id
                )));
            }
        };

        let proposals =
            db::list_proposals(&*self.connection_pool.get()?, &[msg.record_id.clone()])?;

        let associated_agents =
            db::list_associated_agents(&*self.connection_pool.get()?, &[msg.record_id.clone()])?;

        Ok(RecordSlice::from_models(
            &record,
            &proposals,
            &associated_agents,
        ))
    }
}

pub fn fetch_record(
    req: HttpRequest<AppState>,
    record_id: Path<String>,
) -> impl Future<Item = HttpResponse, Error = RestApiResponseError> {
    req.state()
        .database_connection
        .send(FetchRecord {
            record_id: record_id.into_inner(),
        })
        .from_err()
        .and_then(move |res| match res {
            Ok(record) => Ok(HttpResponse::Ok().json(record)),
            Err(err) => Err(err),
        })
}
