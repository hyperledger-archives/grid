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

use super::LocationStoreOperations;
use crate::grid_db::locations::store::diesel::{
    schema::{location, location_attribute},
    LocationStoreError,
};

use crate::grid_db::commits::MAX_COMMIT_NUM;
use diesel::{dsl::update, prelude::*};

pub(in crate::grid_db::locations::store::diesel) trait LocationStoreDeleteLocationOperation {
    fn delete_location(
        &self,
        address: &str,
        current_commit_num: i64,
    ) -> Result<(), LocationStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> LocationStoreDeleteLocationOperation
    for LocationStoreOperations<'a, diesel::pg::PgConnection>
{
    fn delete_location(
        &self,
        address: &str,
        current_commit_num: i64,
    ) -> Result<(), LocationStoreError> {
        self.conn.transaction::<_, LocationStoreError, _>(|| {
            pg::delete_location(&*self.conn, address, current_commit_num)?;
            pg::delete_location_attributes(&*self.conn, address, current_commit_num)?;

            Ok(())
        })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> LocationStoreDeleteLocationOperation
    for LocationStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn delete_location(
        &self,
        address: &str,
        current_commit_num: i64,
    ) -> Result<(), LocationStoreError> {
        self.conn.transaction::<_, LocationStoreError, _>(|| {
            sqlite::delete_location(&*self.conn, address, current_commit_num)?;
            sqlite::delete_location_attributes(&*self.conn, address, current_commit_num)?;

            Ok(())
        })
    }
}

#[cfg(feature = "postgres")]
mod pg {
    use super::*;

    pub fn delete_location(
        conn: &PgConnection,
        address: &str,
        current_commit_num: i64,
    ) -> QueryResult<()> {
        update(location::table)
            .filter(
                location::location_address
                    .eq(address)
                    .and(location::end_commit_num.eq(MAX_COMMIT_NUM)),
            )
            .set(location::end_commit_num.eq(current_commit_num))
            .execute(conn)
            .map(|_| ())
    }

    pub fn delete_location_attributes(
        conn: &PgConnection,
        address: &str,
        current_commit_num: i64,
    ) -> QueryResult<()> {
        update(location_attribute::table)
            .filter(
                location_attribute::location_address
                    .eq(address)
                    .and(location_attribute::end_commit_num.eq(MAX_COMMIT_NUM)),
            )
            .set(location_attribute::end_commit_num.eq(current_commit_num))
            .execute(conn)
            .map(|_| ())
    }
}

#[cfg(feature = "sqlite")]
mod sqlite {
    use super::*;

    pub fn delete_location(
        conn: &SqliteConnection,
        address: &str,
        current_commit_num: i64,
    ) -> QueryResult<()> {
        update(location::table)
            .filter(
                location::location_address
                    .eq(address)
                    .and(location::end_commit_num.eq(MAX_COMMIT_NUM)),
            )
            .set(location::end_commit_num.eq(current_commit_num))
            .execute(conn)
            .map(|_| ())
    }

    pub fn delete_location_attributes(
        conn: &SqliteConnection,
        address: &str,
        current_commit_num: i64,
    ) -> QueryResult<()> {
        update(location_attribute::table)
            .filter(
                location_attribute::location_address
                    .eq(address)
                    .and(location_attribute::end_commit_num.eq(MAX_COMMIT_NUM)),
            )
            .set(location_attribute::end_commit_num.eq(current_commit_num))
            .execute(conn)
            .map(|_| ())
    }
}
