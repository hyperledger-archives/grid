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

use std::i64;

use super::models;
use super::schema;

mod agents;
mod blocks;
mod grid_schemas;
mod organizations;

pub const MAX_BLOCK_NUM: i64 = i64::MAX;

pub use agents::*;
pub use blocks::*;
pub use grid_schemas::*;
pub use organizations::*;
