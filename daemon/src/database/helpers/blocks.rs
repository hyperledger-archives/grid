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

use super::models::Block;
use super::schema::{block, chain_record};
use super::MAX_BLOCK_NUM;

use diesel::{
    dsl::{delete, insert_into, update},
    pg::PgConnection,
    prelude::*,
    result::Error::NotFound,
    QueryResult,
};

const NULL_BLOCK_ID: &str = "0000000000000000";

pub fn insert_block(conn: &PgConnection, block: &Block) -> QueryResult<()> {
    insert_into(block::table)
        .values(block)
        .execute(conn)
        .map(|_| ())
}

pub fn resolve_fork(conn: &PgConnection, block_num: i64) -> QueryResult<()> {
    delete(chain_record::table)
        .filter(chain_record::start_block_num.ge(block_num))
        .execute(conn)?;

    update(chain_record::table)
        .filter(chain_record::end_block_num.ge(block_num))
        .set(chain_record::end_block_num.eq(MAX_BLOCK_NUM))
        .execute(conn)?;

    delete(block::table)
        .filter(block::block_num.ge(block_num))
        .execute(conn)?;

    Ok(())
}

pub fn get_block_by_block_num(conn: &PgConnection, block_num: i64) -> QueryResult<Option<Block>> {
    block::table
        .select(block::all_columns)
        .filter(block::block_num.eq(&block_num))
        .first(conn)
        .map(Some)
        .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
}

pub fn get_current_block_id(conn: &PgConnection) -> QueryResult<String> {
    block::table
        .select(block::block_id)
        .order_by(block::block_num.desc())
        .limit(1)
        .first(conn)
        .or_else(|err| {
            if err == NotFound {
                Ok(NULL_BLOCK_ID.into())
            } else {
                Err(err)
            }
        })
}
