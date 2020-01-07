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

use super::models::{Agent, NewAgent};
use super::schema::agent;
use super::MAX_COMMIT_NUM;

use diesel::{
    dsl::{insert_into, update},
    pg::PgConnection,
    prelude::*,
    result::Error::NotFound,
    QueryResult,
};

pub fn insert_agents(conn: &PgConnection, agents: &[NewAgent]) -> QueryResult<()> {
    for agent in agents {
        update_agent_end_commit_num(conn, &agent.public_key, agent.start_commit_num)?;
    }

    insert_into(agent::table)
        .values(agents)
        .execute(conn)
        .map(|_| ())
}

fn update_agent_end_commit_num(
    conn: &PgConnection,
    public_key: &str,
    current_commit_num: i64,
) -> QueryResult<()> {
    update(agent::table)
        .filter(
            agent::public_key
                .eq(public_key)
                .and(agent::end_commit_num.eq(MAX_COMMIT_NUM)),
        )
        .set(agent::end_commit_num.eq(current_commit_num))
        .execute(conn)
        .map(|_| ())
}

pub fn get_agents(conn: &PgConnection) -> QueryResult<Vec<Agent>> {
    agent::table
        .select(agent::all_columns)
        .filter(agent::end_commit_num.eq(MAX_COMMIT_NUM))
        .load::<Agent>(conn)
}

pub fn get_agent(conn: &PgConnection, public_key: &str) -> QueryResult<Option<Agent>> {
    agent::table
        .select(agent::all_columns)
        .filter(
            agent::public_key
                .eq(public_key)
                .and(agent::end_commit_num.eq(MAX_COMMIT_NUM)),
        )
        .first(conn)
        .map(Some)
        .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
}
