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

use std::time::SystemTime;

use crate::helpers::{create_new_notification, insert_gameroom_notification};
use crate::models::{
    CircuitMember, CircuitProposal, NewCircuitMember, NewCircuitService, NewGameroomNotification,
    NewProposalVoteRecord,
};
use crate::schema::{
    circuit_proposal, proposal_circuit_member, proposal_circuit_service, proposal_vote_record,
};
use diesel::{
    dsl::insert_into, pg::PgConnection, prelude::*, result::Error::NotFound, QueryResult,
};

pub fn fetch_proposal_by_id(conn: &PgConnection, id: &str) -> QueryResult<Option<CircuitProposal>> {
    circuit_proposal::table
        .filter(circuit_proposal::id.eq(id))
        .first::<CircuitProposal>(conn)
        .map(Some)
        .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
}

pub fn fetch_circuit_members_by_proposal_id(
    conn: &PgConnection,
    proposal_id: &str,
) -> QueryResult<Vec<CircuitMember>> {
    proposal_circuit_member::table
        .filter(proposal_circuit_member::proposal_id.eq(proposal_id))
        .load::<CircuitMember>(conn)
}

pub fn list_proposals_with_paging(
    conn: &PgConnection,
    limit: i64,
    offset: i64,
) -> QueryResult<Vec<CircuitProposal>> {
    circuit_proposal::table
        .select(circuit_proposal::all_columns)
        .limit(limit)
        .offset(offset)
        .load::<CircuitProposal>(conn)
}

pub fn get_proposal_count(conn: &PgConnection) -> QueryResult<i64> {
    circuit_proposal::table.count().get_result(conn)
}

pub fn list_proposal_circuit_members(conn: &PgConnection) -> QueryResult<Vec<CircuitMember>> {
    proposal_circuit_member::table
        .select(proposal_circuit_member::all_columns)
        .load::<CircuitMember>(conn)
}

pub fn insert_circuit_proposal(conn: &PgConnection, proposal: CircuitProposal) -> QueryResult<()> {
    insert_into(circuit_proposal::table)
        .values(&vec![proposal])
        .execute(conn)
        .map(|_| ())
}

pub fn insert_circuit_proposal_and_notification(
    conn: &PgConnection,
    proposal: CircuitProposal,
) -> QueryResult<()> {
    conn.transaction::<_, _, _>(|| {
        let notification = create_new_notification(
            "circuit_proposal",
            &proposal.requester,
            &proposal.circuit_id,
        );
        insert_gameroom_notification(conn, &[notification])?;
        insert_circuit_proposal(conn, proposal)?;
        Ok(())
    })
}

pub fn update_circuit_proposal_status(
    conn: &PgConnection,
    proposal_id: &str,
    updated_time: &SystemTime,
    status: &str,
) -> QueryResult<()> {
    diesel::update(circuit_proposal::table.find(proposal_id))
        .set((
            circuit_proposal::updated_time.eq(updated_time),
            circuit_proposal::status.eq(status),
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

pub fn insert_proposal_vote_record_and_notification(
    conn: &PgConnection,
    vote_records: &[NewProposalVoteRecord],
) -> QueryResult<()> {
    conn.transaction::<_, _, _>(|| {
        let notifications = vote_records
            .iter()
            .map(|vote| {
                create_new_notification(
                    "proposal_vote_record",
                    &vote.voter_public_key,
                    &vote.proposal_id,
                )
            })
            .collect::<Vec<NewGameroomNotification>>();
        insert_gameroom_notification(conn, &notifications)?;
        insert_proposal_vote_record(conn, vote_records)?;

        Ok(())
    })
}

pub fn insert_circuit_service(
    conn: &PgConnection,
    circuit_services: &[NewCircuitService],
) -> QueryResult<()> {
    insert_into(proposal_circuit_service::table)
        .values(circuit_services)
        .execute(conn)
        .map(|_| ())
}

pub fn insert_circuit_member(
    conn: &PgConnection,
    circuit_members: &[NewCircuitMember],
) -> QueryResult<()> {
    insert_into(proposal_circuit_member::table)
        .values(circuit_members)
        .execute(conn)
        .map(|_| ())
}

pub fn insert_proposal_information(
    conn: &PgConnection,
    proposal: CircuitProposal,
    proposal_votes: &[NewProposalVoteRecord],
    circuit_services: &[NewCircuitService],
    circuit_members: &[NewCircuitMember],
) -> QueryResult<()> {
    conn.transaction::<_, _, _>(|| {
        insert_circuit_proposal(conn, proposal)?;
        insert_proposal_vote_record(conn, proposal_votes)?;
        insert_circuit_service(conn, circuit_services)?;
        insert_circuit_member(conn, circuit_members)?;

        Ok(())
    })
}

pub fn insert_proposal_information_and_notification(
    conn: &PgConnection,
    proposal: CircuitProposal,
    proposal_votes: &[NewProposalVoteRecord],
    circuit_services: &[NewCircuitService],
    circuit_members: &[NewCircuitMember],
) -> QueryResult<()> {
    conn.transaction::<_, _, _>(|| {
        let notification = create_new_notification(
            "circuit_proposal",
            &proposal.requester,
            &proposal.circuit_id,
        );
        insert_gameroom_notification(conn, &[notification])?;
        insert_circuit_proposal(conn, proposal)?;
        insert_proposal_vote_record(conn, proposal_votes)?;
        insert_circuit_service(conn, circuit_services)?;
        insert_circuit_member(conn, circuit_members)?;

        Ok(())
    })
}

pub fn fetch_circuit_proposal_with_status(
    conn: &PgConnection,
    circuit_id: &str,
    status: &str,
) -> QueryResult<Option<CircuitProposal>> {
    circuit_proposal::table
        .select(circuit_proposal::all_columns)
        .filter(
            circuit_proposal::circuit_id
                .eq(circuit_id)
                .and(circuit_proposal::status.eq(status)),
        )
        .first(conn)
        .map(Some)
        .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
}
