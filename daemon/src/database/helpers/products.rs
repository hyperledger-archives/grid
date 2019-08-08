/*
 * Copyright (c) 2019 Target Brands, Inc.
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

use super::models::{NewProduct, Product};
use super::schema::product;
use super::MAX_BLOCK_NUM;

use diesel::{
    dsl::{insert_into, update},
    pg::PgConnection,
    prelude::*,
    result::Error::NotFound,
    QueryResult,
};

#[allow(dead_code)]
pub fn insert_products(conn: &PgConnection, products: &[NewProduct]) -> QueryResult<()> {
    for prod in products {
        update_prod_end_block_num(conn, &prod.product_id, prod.start_block_num)?;
    }

    insert_into(product::table)
        .values(products)
        .execute(conn)
        .map(|_| ())
}

fn update_prod_end_block_num(
    conn: &PgConnection,
    product_id: &str,
    current_block_num: i64,
) -> QueryResult<()> {
    update(product::table)
        .filter(
            product::product_id
                .eq(product_id)
                .and(product::end_block_num.eq(MAX_BLOCK_NUM)),
        )
        .set(product::end_block_num.eq(current_block_num))
        .execute(conn)
        .map(|_| ())
}

#[allow(dead_code)]
pub fn list_products(conn: &PgConnection) -> QueryResult<Vec<Product>> {
    product::table
        .select(product::all_columns)
        .filter(product::end_block_num.eq(MAX_BLOCK_NUM))
        .load::<Product>(conn)
}

#[allow(dead_code)]
pub fn fetch_product(conn: &PgConnection, product_id: &str) -> QueryResult<Option<Product>> {
    product::table
        .select(product::all_columns)
        .filter(
            product::product_id
                .eq(product_id)
                .and(product::end_block_num.eq(MAX_BLOCK_NUM)),
        )
        .first(conn)
        .map(Some)
        .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
}
