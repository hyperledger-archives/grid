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
        diesel::{
            models::{NewProduct, NewProductPropertyValue},
            schema::{product, product_property_value},
        },
        error::ProductStoreError,
        Product,
    },
    MAX_COMMIT_NUM,
};

use diesel::{
    dsl::{insert_into, update},
    prelude::*,
};

pub(in crate::products) trait AddProductOperation {
    fn add_product(&self, product: Product) -> Result<(), ProductStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> AddProductOperation for ProductStoreOperations<'a, diesel::pg::PgConnection> {
    fn add_product(&self, product: Product) -> Result<(), ProductStoreError> {
        let (product_model, property_models) = product.into();

        self.conn.transaction::<_, ProductStoreError, _>(|| {
            pg::insert_product(&*self.conn, &product_model)?;
            pg::insert_product_property_values(&*self.conn, &property_models)?;

            Ok(())
        })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> AddProductOperation for ProductStoreOperations<'a, diesel::sqlite::SqliteConnection> {
    fn add_product(&self, product: Product) -> Result<(), ProductStoreError> {
        let (product_model, property_models) = product.into();

        self.conn.transaction::<_, ProductStoreError, _>(|| {
            sqlite::insert_product(&*self.conn, &product_model)?;
            sqlite::insert_product_property_values(&*self.conn, &property_models)?;

            Ok(())
        })
    }
}

#[cfg(feature = "postgres")]
mod pg {
    use super::*;

    pub fn insert_product(conn: &PgConnection, product: &NewProduct) -> QueryResult<()> {
        update_prod_end_commit_num(
            conn,
            &product.product_id,
            product.service_id.as_deref(),
            product.start_commit_num,
        )?;

        insert_into(product::table)
            .values(product)
            .execute(conn)
            .map(|_| ())
    }

    pub fn insert_product_property_values(
        conn: &PgConnection,
        property_values: &[NewProductPropertyValue],
    ) -> QueryResult<()> {
        for value in property_values {
            update_prod_property_values(
                conn,
                &value.product_id,
                value.service_id.as_deref(),
                value.start_commit_num,
            )?;
        }

        insert_into(product_property_value::table)
            .values(property_values)
            .execute(conn)
            .map(|_| ())
    }
    fn update_prod_end_commit_num(
        conn: &PgConnection,
        product_id: &str,
        service_id: Option<&str>,
        current_commit_num: i64,
    ) -> QueryResult<()> {
        let update = update(product::table);

        if let Some(service_id) = service_id {
            update
                .filter(
                    product::product_id
                        .eq(product_id)
                        .and(product::end_commit_num.eq(MAX_COMMIT_NUM))
                        .and(product::service_id.eq(service_id)),
                )
                .set(product::end_commit_num.eq(current_commit_num))
                .execute(conn)
                .map(|_| ())
        } else {
            update
                .filter(
                    product::product_id
                        .eq(product_id)
                        .and(product::end_commit_num.eq(MAX_COMMIT_NUM)),
                )
                .set(product::end_commit_num.eq(current_commit_num))
                .execute(conn)
                .map(|_| ())
        }
    }

    fn update_prod_property_values(
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

    pub fn insert_product(conn: &SqliteConnection, product: &NewProduct) -> QueryResult<()> {
        update_prod_end_commit_num(
            conn,
            &product.product_id,
            product.service_id.as_deref(),
            product.start_commit_num,
        )?;

        insert_into(product::table)
            .values(product)
            .execute(conn)
            .map(|_| ())
    }

    pub fn insert_product_property_values(
        conn: &SqliteConnection,
        property_values: &[NewProductPropertyValue],
    ) -> QueryResult<()> {
        for value in property_values {
            update_prod_property_values(
                conn,
                &value.product_id,
                value.service_id.as_deref(),
                value.start_commit_num,
            )?;
        }

        insert_into(product_property_value::table)
            .values(property_values)
            .execute(conn)
            .map(|_| ())
    }

    fn update_prod_end_commit_num(
        conn: &SqliteConnection,
        product_id: &str,
        service_id: Option<&str>,
        current_commit_num: i64,
    ) -> QueryResult<()> {
        let update = update(product::table);

        if let Some(service_id) = service_id {
            update
                .filter(
                    product::product_id
                        .eq(product_id)
                        .and(product::end_commit_num.eq(MAX_COMMIT_NUM))
                        .and(product::service_id.eq(service_id)),
                )
                .set(product::end_commit_num.eq(current_commit_num))
                .execute(conn)
                .map(|_| ())
        } else {
            update
                .filter(
                    product::product_id
                        .eq(product_id)
                        .and(product::end_commit_num.eq(MAX_COMMIT_NUM)),
                )
                .set(product::end_commit_num.eq(current_commit_num))
                .execute(conn)
                .map(|_| ())
        }
    }

    fn update_prod_property_values(
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
