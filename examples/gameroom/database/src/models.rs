/*
 * Copyright 2019 Cargill Incorporated
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * -----------------------------------------------------------------------------
 */

use super::schema::*;
use std::time::SystemTime;

#[derive(Insertable, Queryable)]
#[table_name = "gameroom_user"]
pub struct GameroomUser {
    pub public_key: String,
    pub encrypted_private_key: String,
    pub email: String,
    pub hashed_password: String,
}

#[derive(Insertable, Queryable, Identifiable, PartialEq, Debug)]
#[table_name = "circuit_proposal"]
pub struct CircuitProposal {
    pub id: String,
    pub proposal_type: String,
    pub circuit_id: String,
    pub circuit_hash: String,
    pub requester: String,
    pub authorization_type: String,
    pub persistence: String,
    pub routes: String,
    pub circuit_management_type: String,
    pub application_metadata: Vec<u8>,
    pub status: String,
    pub created_time: SystemTime,
    pub updated_time: SystemTime,
}

#[derive(Queryable, Identifiable, Associations, PartialEq, Debug)]
#[table_name = "proposal_vote_record"]
#[belongs_to(CircuitProposal, foreign_key = "proposal_id")]
pub struct ProposalVoteRecord {
    pub id: i64,
    pub proposal_id: String,
    pub voter_public_key: String,
    pub vote: String,
    pub created_time: SystemTime,
}

#[derive(Insertable, PartialEq, Debug)]
#[table_name = "proposal_vote_record"]
pub struct NewProposalVoteRecord {
    pub proposal_id: String,
    pub voter_public_key: String,
    pub vote: String,
    pub created_time: SystemTime,
}

#[derive(Queryable, Identifiable, Associations, PartialEq, Debug)]
#[table_name = "proposal_circuit_member"]
#[belongs_to(CircuitProposal, foreign_key = "proposal_id")]
pub struct CircuitMember {
    pub id: i64,
    pub proposal_id: String,
    pub node_id: String,
    pub endpoint: String,
}

#[derive(Insertable, PartialEq, Debug)]
#[table_name = "proposal_circuit_member"]
pub struct NewCircuitMember {
    pub proposal_id: String,
    pub node_id: String,
    pub endpoint: String,
}

#[derive(Queryable, Identifiable, Associations, PartialEq, Debug)]
#[table_name = "proposal_circuit_service"]
#[belongs_to(CircuitProposal, foreign_key = "proposal_id")]
pub struct CircuitService {
    pub id: i64,
    pub proposal_id: String,
    pub service_id: String,
    pub service_type: String,
    pub allowed_nodes: Vec<String>,
}

#[derive(Insertable, PartialEq, Debug)]
#[table_name = "proposal_circuit_service"]
pub struct NewCircuitService {
    pub proposal_id: String,
    pub service_id: String,
    pub service_type: String,
    pub allowed_nodes: Vec<String>,
}

#[derive(Queryable, Identifiable, Associations)]
#[table_name = "gameroom_notification"]
pub struct GameroomNotification {
    pub id: i64,
    pub notification_type: String,
    pub requester: String,
    pub target: String,
    pub created_time: SystemTime,
    pub read: bool,
}

#[derive(Debug, Insertable)]
#[table_name = "gameroom_notification"]
pub struct NewGameroomNotification {
    pub notification_type: String,
    pub requester: String,
    pub target: String,
    pub created_time: SystemTime,
    pub read: bool,
}
