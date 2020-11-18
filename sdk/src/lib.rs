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

// Required due to a bug in rust-protobuf: https://github.com/stepancheg/rust-protobuf/issues/331
#![allow(renamed_and_removed_lints)]

extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate cfg_if;
#[macro_use]
#[cfg(feature = "diesel")]
extern crate diesel;
#[macro_use]
#[cfg(feature = "diesel")]
extern crate diesel_migrations;

pub mod error;
pub mod grid_db;
mod hex;
#[macro_use]
extern crate log;
pub mod permissions;
pub mod protocol;
pub mod protos;
pub mod store;
