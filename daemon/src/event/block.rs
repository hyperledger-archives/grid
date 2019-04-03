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

use sawtooth_sdk::messages::events::{Event, Event_Attribute};

use super::{error::EventError, EventHandler};

pub struct BlockEventHandler {}

impl BlockEventHandler {
    pub fn new() -> Self {
        Self {}
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
    fn event_type(&self) -> &str {
        "sawtooth/block-commit"
    }

    fn handle_event(&self, event: &Event) -> Result<(), EventError> {
        let attributes = event.get_attributes();

        let block_id = Self::require_attr(attributes, "block_id")?;
        let block_num = Self::require_attr(attributes, "block_num")?
            .parse::<u64>()
            .map_err(|err| EventError(format!("block_num was not a valid number: {}", err)))?;
        let state_root_hash = Self::require_attr(attributes, "state_root_hash")?;

        info!(
            "Received sawtooth/block-commit ({}, {}, {})",
            block_id, block_num, state_root_hash
        );

        Ok(())
    }
}
