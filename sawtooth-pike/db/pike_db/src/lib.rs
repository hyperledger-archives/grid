// Copyright 2018 Cargill Incorporated
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

#![recursion_limit="128"]

#[macro_use] extern crate diesel;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate r2d2_diesel;
extern crate r2d2;
extern crate postgres;

use diesel::result::Error;

mod schema;
mod agents_helper;
mod orgs_helper;

pub mod pools;
pub mod models;

pub use orgs_helper::*;
pub use agents_helper::*;

pub use Error::NotFound;
pub use diesel::pg::PgConnection;
pub use r2d2_diesel::ConnectionManager;
pub use r2d2::PooledConnection;

pub type QueryError = Error;
