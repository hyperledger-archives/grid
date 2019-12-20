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

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
#[cfg(feature = "rest-api")]
extern crate serde_json;
#[macro_use]
#[cfg(feature = "database")]
extern crate diesel;
#[macro_use]
#[cfg(feature = "database")]
extern crate diesel_migrations;

#[macro_export]
macro_rules! rwlock_read_unwrap {
    ($lock:expr) => {
        match $lock.read() {
            Ok(d) => d,
            Err(e) => panic!("RwLock error: {:?}", e),
        }
    };
}

#[macro_export]
macro_rules! rwlock_write_unwrap {
    ($lock:expr) => {
        match $lock.write() {
            Ok(d) => d,
            Err(e) => panic!("RwLock error: {:?}", e),
        }
    };
}

#[macro_export]
macro_rules! mutex_lock_unwrap {
    ($lock:expr) => {
        match $lock.lock() {
            Ok(guard) => guard,
            Err(e) => panic!("Mutex error: {:?}", e),
        }
    };
}

pub mod admin;
#[cfg(feature = "biome")]
pub mod biome;
pub mod channel;
pub mod circuit;
pub mod collections;
pub mod consensus;
#[cfg(feature = "database")]
pub mod database;
#[cfg(feature = "events")]
pub mod events;
mod hex;
pub mod keys;
#[cfg(feature = "matrix")]
mod matrix;
pub mod mesh;
pub mod network;
pub mod node_registry;
pub mod orchestrator;
pub mod protos;
#[cfg(feature = "rest-api")]
pub mod rest_api;
pub mod service;
pub mod signing;
pub mod storage;
pub mod transport;

#[cfg(feature = "rest-api")]
pub use actix_web;
#[cfg(feature = "rest-api")]
pub use futures;
