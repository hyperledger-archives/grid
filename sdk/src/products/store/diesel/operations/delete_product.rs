// Copyright 2018-2021 Cargill Incorporated
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

use super::ProductStoreOperations;

use crate::products::{
    store::{
        diesel::schema::{product, product_property_value},
        error::ProductStoreError,
    },
    MAX_COMMIT_NUM,
};
use diesel::{dsl::update, prelude::*};

pub(in crate::products) trait DeleteProductOperation {
    fn delete_product(
        &self,
        address: &str,
        current_commit_num: i64,
    ) -> Result<(), ProductStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> DeleteProductOperation for ProductStoreOperations<'a, diesel::pg::PgConnection> {
    fn delete_product(
        &self,
        address: &str,
        current_commit_num: i64,
    ) -> Result<(), ProductStoreError> {
        self.conn.transaction::<_, ProductStoreError, _>(|| {
            pg::delete_product(&*self.conn, address, current_commit_num)?;
            pg::delete_product_property_values(&*self.conn, address, current_commit_num)?;

            Ok(())
        })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> DeleteProductOperation for ProductStoreOperations<'a, diesel::sqlite::SqliteConnection> {
    fn delete_product(
        &self,
        address: &str,
        current_commit_num: i64,
    ) -> Result<(), ProductStoreError> {
        self.conn.transaction::<_, ProductStoreError, _>(|| {
            sqlite::delete_product(&*self.conn, address, current_commit_num)?;
            sqlite::delete_product_property_values(&*self.conn, address, current_commit_num)?;

            Ok(())
        })
    }
}

#[cfg(feature = "postgres")]
mod pg {
    use super::*;

    pub fn delete_product(
        conn: &PgConnection,
        address: &str,
        current_commit_num: i64,
    ) -> QueryResult<()> {
        update(product::table)
            .filter(
                product::product_address
                    .eq(address)
                    .and(product::end_commit_num.eq(MAX_COMMIT_NUM)),
            )
            .set(product::end_commit_num.eq(current_commit_num))
            .execute(conn)
            .map(|_| ())
    }

    pub fn delete_product_property_values(
        conn: &PgConnection,
        address: &str,
        current_commit_num: i64,
    ) -> QueryResult<()> {
        update(product_property_value::table)
            .filter(
                product_property_value::product_address
                    .eq(address)
                    .and(product_property_value::end_commit_num.eq(MAX_COMMIT_NUM)),
            )
            .set(product_property_value::end_commit_num.eq(current_commit_num))
            .execute(conn)
            .map(|_| ())
    }
}

#[cfg(feature = "sqlite")]
mod sqlite {
    use super::*;

    pub fn delete_product(
        conn: &SqliteConnection,
        address: &str,
        current_commit_num: i64,
    ) -> QueryResult<()> {
        update(product::table)
            .filter(
                product::product_address
                    .eq(address)
                    .and(product::end_commit_num.eq(MAX_COMMIT_NUM)),
            )
            .set(product::end_commit_num.eq(current_commit_num))
            .execute(conn)
            .map(|_| ())
    }

    pub fn delete_product_property_values(
        conn: &SqliteConnection,
        address: &str,
        current_commit_num: i64,
    ) -> QueryResult<()> {
        update(product_property_value::table)
            .filter(
                product_property_value::product_address
                    .eq(address)
                    .and(product_property_value::end_commit_num.eq(MAX_COMMIT_NUM)),
            )
            .set(product_property_value::end_commit_num.eq(current_commit_num))
            .execute(conn)
            .map(|_| ())
    }
}
