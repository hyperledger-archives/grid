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

use super::super::models::{NewUserCredentialsModel, UserCredentialsModel};
use super::super::schema::user_credentials;

use diesel::{
    dsl::insert_into, pg::PgConnection, prelude::*, result::Error::NotFound, QueryResult,
};

pub fn fetch_credential_by_username(
    conn: &PgConnection,
    username: &str,
) -> QueryResult<Option<UserCredentialsModel>> {
    user_credentials::table
        .filter(user_credentials::username.eq(username))
        .first::<UserCredentialsModel>(conn)
        .map(Some)
        .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
}

pub fn insert_credential(
    conn: &PgConnection,
    credential: NewUserCredentialsModel,
) -> QueryResult<()> {
    insert_into(user_credentials::table)
        .values(&vec![credential])
        .execute(conn)
        .map(|_| ())
}
