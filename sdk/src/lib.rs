// Copyright 2019 Cargill Incorporated
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

extern crate serde;
#[macro_use]
extern crate serde_derive;
#[cfg(feature = "serde_json")]
#[macro_use]
extern crate serde_json;
#[macro_use]
#[cfg(feature = "pike")]
extern crate cfg_if;
#[macro_use]
#[cfg(feature = "diesel")]
extern crate diesel;
#[macro_use]
#[cfg(feature = "diesel")]
extern crate diesel_migrations;
#[macro_use]
#[cfg(feature = "log")]
extern crate log;

#[cfg(feature = "backend")]
pub mod backend;
#[cfg(feature = "batch-processor")]
pub mod batch_processor;
#[cfg(feature = "batch-store")]
pub mod batches;
#[cfg(feature = "client")]
pub mod client;
pub mod commits;
pub mod error;
mod hex;
#[cfg(feature = "location")]
pub mod location;
pub mod migrations;
pub mod paging;
#[cfg(feature = "pike")]
pub mod permissions;
#[cfg(feature = "pike")]
pub mod pike;
#[cfg(feature = "product")]
pub mod product;
pub mod protocol;
pub mod protos;
#[cfg(feature = "purchase-order")]
pub mod purchase_order;
#[cfg(feature = "rest-api")]
pub mod rest_api;
#[cfg(feature = "schema")]
pub mod schema;
pub mod store;
#[cfg(any(feature = "test-postgres", feature = "test-sqlite"))]
pub mod testing;
#[cfg(feature = "track-and-trace")]
pub mod track_and_trace;
#[cfg(feature = "workflow")]
pub mod workflow;
