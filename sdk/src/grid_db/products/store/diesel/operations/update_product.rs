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

use super::ProductStoreOperations;

use crate::grid_db::products::{
    error::ProductStoreError, store::diesel::schema::product_property_value, MAX_COMMIT_NUM,
};
use diesel::{dsl::update, prelude::*};

pub(in crate::grid_db::products) trait UpdateProductOperation {
    fn update_product(
        &self,
        product_id: &str,
        service_id: Option<&str>,
        current_commit_num: i64,
    ) -> Result<(), ProductStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> UpdateProductOperation for ProductStoreOperations<'a, diesel::pg::PgConnection> {
    fn update_product(
        &self,
        product_id: &str,
        service_id: Option<&str>,
        current_commit_num: i64,
    ) -> Result<(), ProductStoreError> {
        pg::update_product_property_values(
            &*self.conn,
            product_id,
            service_id,
            current_commit_num,
        )?;

        Ok(())
    }
}

#[cfg(feature = "sqlite")]
impl<'a> UpdateProductOperation for ProductStoreOperations<'a, diesel::sqlite::SqliteConnection> {
    fn update_product(
        &self,
        product_id: &str,
        service_id: Option<&str>,
        current_commit_num: i64,
    ) -> Result<(), ProductStoreError> {
        sqlite::update_product_property_values(
            &*self.conn,
            product_id,
            service_id,
            current_commit_num,
        )?;

        Ok(())
    }
}

#[cfg(feature = "postgres")]
mod pg {
    use super::*;

    pub fn update_product_property_values(
        conn: &PgConnection,
        product_id: &str,
        service_id: Option<&str>,
        current_commit_num: i64,
    ) -> QueryResult<()> {
        let update = update(product_property_value::table);

        if let Some(service_id) = service_id {
            update
                .filter(
                    product_property_value::product_id
                        .eq(product_id)
                        .and(product_property_value::end_commit_num.eq(MAX_COMMIT_NUM))
                        .and(product_property_value::service_id.eq(service_id)),
                )
                .set(product_property_value::end_commit_num.eq(current_commit_num))
                .execute(conn)
                .map(|_| ())
        } else {
            update
                .filter(
                    product_property_value::product_id
                        .eq(product_id)
                        .and(product_property_value::end_commit_num.eq(MAX_COMMIT_NUM)),
                )
                .set(product_property_value::end_commit_num.eq(current_commit_num))
                .execute(conn)
                .map(|_| ())
        }
    }
}

#[cfg(feature = "sqlite")]
mod sqlite {
    use super::*;

    pub fn update_product_property_values(
        conn: &SqliteConnection,
        product_id: &str,
        service_id: Option<&str>,
        current_commit_num: i64,
    ) -> QueryResult<()> {
        let update = update(product_property_value::table);

        if let Some(service_id) = service_id {
            update
                .filter(
                    product_property_value::product_id
                        .eq(product_id)
                        .and(product_property_value::end_commit_num.eq(MAX_COMMIT_NUM))
                        .and(product_property_value::service_id.eq(service_id)),
                )
                .set(product_property_value::end_commit_num.eq(current_commit_num))
                .execute(conn)
                .map(|_| ())
        } else {
            update
                .filter(
                    product_property_value::product_id
                        .eq(product_id)
                        .and(product_property_value::end_commit_num.eq(MAX_COMMIT_NUM)),
                )
                .set(product_property_value::end_commit_num.eq(current_commit_num))
                .execute(conn)
                .map(|_| ())
        }
    }
}
