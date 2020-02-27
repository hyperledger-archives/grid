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

use super::super::models::KeyModel;
use super::super::schema::keys;

use diesel::{dsl::insert_into, pg::PgConnection, prelude::*, QueryResult};

pub fn insert_key(conn: &PgConnection, key: &KeyModel) -> QueryResult<usize> {
    insert_into(keys::table).values(vec![key]).execute(conn)
}

pub fn update_key(
    conn: &PgConnection,
    user_id: &str,
    public_key: &str,
    display_name: &str,
) -> QueryResult<usize> {
    diesel::update(keys::table.find((public_key, user_id)))
        .set((keys::display_name.eq(display_name),))
        .execute(conn)
}
