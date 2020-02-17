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

use crate::models::{NewXoGame, XoGame};
use crate::schema::xo_games;

use diesel::{
    dsl::insert_into, pg::PgConnection, prelude::*, result::Error::NotFound, QueryResult,
};

pub fn list_xo_games(
    conn: &PgConnection,
    circuit_id: &str,
    limit: i64,
    offset: i64,
) -> QueryResult<Vec<XoGame>> {
    xo_games::table
        .filter(xo_games::circuit_id.eq(circuit_id))
        .limit(limit)
        .offset(offset)
        .load::<XoGame>(conn)
}

pub fn fetch_xo_game(
    conn: &PgConnection,
    circuit_id: &str,
    name: &str,
) -> QueryResult<Option<XoGame>> {
    xo_games::table
        .filter(
            xo_games::game_name
                .eq(name)
                .and(xo_games::circuit_id.eq(circuit_id)),
        )
        .first::<XoGame>(conn)
        .map(Some)
        .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
}

pub fn insert_xo_game(conn: &PgConnection, game: NewXoGame) -> QueryResult<()> {
    insert_into(xo_games::table)
        .values(game)
        .execute(conn)
        .map(|_| ())
}

pub fn update_xo_game(conn: &PgConnection, updated_game: XoGame) -> QueryResult<()> {
    diesel::update(
        xo_games::table.filter(
            xo_games::game_name
                .eq(&updated_game.game_name)
                .and(xo_games::circuit_id.eq(&updated_game.circuit_id)),
        ),
    )
    .set(updated_game.clone())
    .execute(conn)
    .map(|_| ())
}

pub fn get_xo_game_count(conn: &PgConnection) -> QueryResult<i64> {
    xo_games::table.count().get_result(conn)
}
