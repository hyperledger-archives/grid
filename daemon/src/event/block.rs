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

use diesel::prelude::*;
use sawtooth_sdk::messages::events::{Event, Event_Attribute};

use crate::database::{models::Block, schema, ConnectionPool};

use super::{error::EventError, EventHandler};

pub struct BlockEventHandler {
    connection_pool: ConnectionPool,
}

impl BlockEventHandler {
    pub fn new(connection_pool: ConnectionPool) -> Self {
        Self { connection_pool }
    }

    fn require_attr(attributes: &[Event_Attribute], key: &str) -> Result<String, EventError> {
        attributes
            .iter()
            .find(|attr| attr.get_key() == key)
            .map(|attr| attr.get_value().to_owned())
            .ok_or_else(|| EventError(format!("Unable to find {}", key)))
    }
}

impl EventHandler for BlockEventHandler {

    fn handle_events(&self, events: &[Event]) -> Result<(), EventError> {
        let block = get_block(events)?;

        debug!(
            "Received sawtooth/block-commit ({}, {}, {})",
            block.block_id, block.block_num, block.state_root_hash
        );

        let conn = self
            .connection_pool
            .get()
            .map_err(|err| EventError(format!("Unable to connect to database: {}", err)))?;

        diesel::insert_into(schema::block::table)
            .values(&block)
            .execute(&*conn)
            .map_err(|err| EventError(format!("Unable to insert block in database: {}", err)))?;

        Ok(())
    }
}
