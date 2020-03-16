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

use std::time::SystemTime;

use crate::models::{
    ActiveGameroom, Gameroom, GameroomMember, GameroomProposal, GameroomService, NewGameroomMember,
    NewGameroomProposal, NewGameroomService, NewProposalVoteRecord,
};
use crate::schema::{
    gameroom, gameroom_member, gameroom_proposal, gameroom_service, proposal_vote_record,
};
use diesel::{
    dsl::insert_into, pg::PgConnection, prelude::*, result::Error::NotFound, QueryResult,
};

pub fn fetch_proposal_by_id(conn: &PgConnection, id: i64) -> QueryResult<Option<GameroomProposal>> {
    gameroom_proposal::table
        .filter(gameroom_proposal::id.eq(id))
        .first::<GameroomProposal>(conn)
        .map(Some)
        .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
}

pub fn fetch_gameroom_members_by_circuit_id_and_status(
    conn: &PgConnection,
    circuit_id: &str,
    status: &str,
) -> QueryResult<Vec<GameroomMember>> {
    gameroom_member::table
        .filter(
            gameroom_member::circuit_id
                .eq(circuit_id)
                .and(gameroom_member::status.eq(status)),
        )
        .load::<GameroomMember>(conn)
}

pub fn list_proposals_with_paging(
    conn: &PgConnection,
    limit: i64,
    offset: i64,
) -> QueryResult<Vec<GameroomProposal>> {
    gameroom_proposal::table
        .select(gameroom_proposal::all_columns)
        .limit(limit)
        .offset(offset)
        .load::<GameroomProposal>(conn)
}

pub fn get_proposal_count(conn: &PgConnection) -> QueryResult<i64> {
    gameroom_proposal::table.count().get_result(conn)
}

pub fn list_gameroom_members_with_status(
    conn: &PgConnection,
    status: &str,
) -> QueryResult<Vec<GameroomMember>> {
    gameroom_member::table
        .filter(gameroom_member::status.eq(status))
        .load::<GameroomMember>(conn)
}

pub fn insert_gameroom_proposal(
    conn: &PgConnection,
    proposal: NewGameroomProposal,
) -> QueryResult<()> {
    insert_into(gameroom_proposal::table)
        .values(&vec![proposal])
        .execute(conn)
        .map(|_| ())
}

pub fn insert_gameroom(conn: &PgConnection, gameroom: Gameroom) -> QueryResult<()> {
    insert_into(gameroom::table)
        .values(&vec![gameroom])
        .execute(conn)
        .map(|_| ())
}

pub fn update_gameroom_proposal_status(
    conn: &PgConnection,
    proposal_id: i64,
    updated_time: &SystemTime,
    status: &str,
) -> QueryResult<()> {
    diesel::update(gameroom_proposal::table.find(proposal_id))
        .set((
            gameroom_proposal::updated_time.eq(updated_time),
            gameroom_proposal::status.eq(status),
        ))
        .execute(conn)
        .map(|_| ())
}

pub fn gameroom_service_is_active(conn: &PgConnection, circuit_id: &str) -> QueryResult<bool> {
    gameroom_service::table
        .filter(
            gameroom_service::circuit_id
                .eq(circuit_id)
                .and(gameroom_service::status.eq("Active")),
        )
        .first::<GameroomService>(conn)
        .map(|_| true)
        .or_else(|err| if err == NotFound { Ok(false) } else { Err(err) })
}

pub fn get_last_updated_proposal_time(conn: &PgConnection) -> QueryResult<Option<SystemTime>> {
    gameroom_proposal::table
        .select(gameroom_proposal::updated_time)
        .order_by(gameroom_proposal::updated_time.desc())
        .first(conn)
        .map(Some)
        .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
}

pub fn fetch_active_gamerooms(
    conn: &PgConnection,
    node_id: &str,
) -> QueryResult<Vec<ActiveGameroom>> {
    gameroom_service::table
        .inner_join(
            gameroom_proposal::table
                .on(gameroom_service::circuit_id.eq(gameroom_proposal::circuit_id)),
        )
        .select((
            gameroom_service::circuit_id,
            gameroom_service::service_id,
            gameroom_service::status,
            gameroom_service::last_event,
            gameroom_proposal::requester,
            gameroom_proposal::requester_node_id,
        ))
        .filter(
            gameroom_service::status
                .eq("Active")
                .and(gameroom_service::allowed_nodes.contains(vec![node_id])),
        )
        .load(conn)
}

pub fn update_gameroom_status(
    conn: &PgConnection,
    circuit_id: &str,
    updated_time: &SystemTime,
    status: &str,
) -> QueryResult<()> {
    diesel::update(gameroom::table.find(circuit_id))
        .set((
            gameroom::updated_time.eq(updated_time),
            gameroom::status.eq(status),
        ))
        .execute(conn)
        .map(|_| ())
}

pub fn update_gameroom_member_status(
    conn: &PgConnection,
    circuit_id: &str,
    updated_time: &SystemTime,
    old_status: &str,
    new_status: &str,
) -> QueryResult<()> {
    diesel::update(
        gameroom_member::table.filter(
            gameroom_member::circuit_id
                .eq(circuit_id)
                .and(gameroom_member::status.eq(old_status)),
        ),
    )
    .set((
        gameroom_member::updated_time.eq(updated_time),
        gameroom_member::status.eq(new_status),
    ))
    .execute(conn)
    .map(|_| ())
}

pub fn update_gameroom_service_status(
    conn: &PgConnection,
    circuit_id: &str,
    updated_time: &SystemTime,
    old_status: &str,
    new_status: &str,
) -> QueryResult<()> {
    diesel::update(
        gameroom_service::table.filter(
            gameroom_service::circuit_id
                .eq(circuit_id)
                .and(gameroom_service::status.eq(old_status)),
        ),
    )
    .set((
        gameroom_service::updated_time.eq(updated_time),
        gameroom_service::status.eq(new_status),
    ))
    .execute(conn)
    .map(|_| ())
}

pub fn update_gameroom_service_last_event(
    conn: &PgConnection,
    circuit_id: &str,
    updated_time: &SystemTime,
    event_id: &str,
) -> QueryResult<()> {
    diesel::update(gameroom_service::table.filter(gameroom_service::circuit_id.eq(circuit_id)))
        .set((
            gameroom_service::updated_time.eq(updated_time),
            gameroom_service::last_event.eq(event_id),
        ))
        .execute(conn)
        .map(|_| ())
}

pub fn insert_proposal_vote_record(
    conn: &PgConnection,
    vote_records: &[NewProposalVoteRecord],
) -> QueryResult<()> {
    insert_into(proposal_vote_record::table)
        .values(vote_records)
        .execute(conn)
        .map(|_| ())
}

pub fn insert_gameroom_services(
    conn: &PgConnection,
    gameroom_services: &[NewGameroomService],
) -> QueryResult<()> {
    insert_into(gameroom_service::table)
        .values(gameroom_services)
        .execute(conn)
        .map(|_| ())
}

pub fn insert_gameroom_members(
    conn: &PgConnection,
    gameroom_members: &[NewGameroomMember],
) -> QueryResult<()> {
    insert_into(gameroom_member::table)
        .values(gameroom_members)
        .execute(conn)
        .map(|_| ())
}

pub fn fetch_gameroom_proposal_with_status(
    conn: &PgConnection,
    circuit_id: &str,
    status: &str,
) -> QueryResult<Option<GameroomProposal>> {
    gameroom_proposal::table
        .select(gameroom_proposal::all_columns)
        .filter(
            gameroom_proposal::circuit_id
                .eq(circuit_id)
                .and(gameroom_proposal::status.eq(status)),
        )
        .first(conn)
        .map(Some)
        .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
}

pub fn list_gamerooms_with_paging_and_status(
    conn: &PgConnection,
    status: &str,
    limit: i64,
    offset: i64,
) -> QueryResult<Vec<Gameroom>> {
    gameroom::table
        .select(gameroom::all_columns)
        .filter(gameroom::status.eq(status))
        .limit(limit)
        .offset(offset)
        .load::<Gameroom>(conn)
}

pub fn get_gameroom_count(conn: &PgConnection) -> QueryResult<i64> {
    gameroom::table.count().get_result(conn)
}

pub fn list_gamerooms_with_paging(
    conn: &PgConnection,
    limit: i64,
    offset: i64,
) -> QueryResult<Vec<Gameroom>> {
    gameroom::table
        .select(gameroom::all_columns)
        .limit(limit)
        .offset(offset)
        .load::<Gameroom>(conn)
}

pub fn fetch_gameroom(conn: &PgConnection, circuit_id: &str) -> QueryResult<Option<Gameroom>> {
    gameroom::table
        .filter(gameroom::circuit_id.eq(circuit_id))
        .first(conn)
        .map(Some)
        .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
}

pub fn fetch_gameroom_by_alias(conn: &PgConnection, alias: &str) -> QueryResult<Option<Gameroom>> {
    gameroom::table
        .filter(gameroom::alias.eq(alias))
        .first(conn)
        .map(Some)
        .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
}

pub fn fetch_service_id_for_gameroom_service(
    conn: &PgConnection,
    circuit_id: &str,
    node_id: &str,
) -> QueryResult<Option<String>> {
    gameroom_service::table
        .filter(
            gameroom_service::circuit_id
                .eq(circuit_id)
                .and(gameroom_service::allowed_nodes.contains(vec![node_id])),
        )
        .first::<GameroomService>(conn)
        .map(|service| Some(service.service_id))
        .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
}
