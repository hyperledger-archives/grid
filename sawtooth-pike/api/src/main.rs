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

#![feature(plugin, decl_macro, custom_derive)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate rocket_cors;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;
extern crate serde_yaml;
extern crate serde_json;
extern crate pike_db;
extern crate sawtooth_sdk;
extern crate protobuf;
extern crate uuid;

mod openapi;
mod routes;
mod guard;
mod submit;
#[cfg(test)] mod tests;

use std::env;
use rocket::http::Method;
use rocket_cors::{AllowedOrigins, AllowedHeaders};
use rocket_contrib::Json;
use routes::{agents, organizations};
use pike_db::pools;
use routes::transactions;

use sawtooth_sdk::messaging::zmq_stream::ZmqMessageConnection;

#[get("/")]
fn hello() -> &'static str {
    "Hello, world!"
}

#[error(404)]
fn not_found(_: &rocket::Request) -> Json {
    Json(json!({
        "message": "Not Found"
    }))
}

#[error(500)]
fn internal_server_error(_: &rocket::Request) -> Json {
    Json(json!({
        "message": "Internal Server Error"
    }))
}

fn main() {
    let (allowed_origins, failed_origins) = AllowedOrigins::some(&["http://localhost:9002"]);
    assert!(failed_origins.is_empty());

    let options = rocket_cors::Cors {
        allowed_origins: allowed_origins,
        allowed_methods: vec![Method::Get, Method::Post, Method::Options].into_iter().map(From::from).collect(),
        allowed_headers: AllowedHeaders::some(&["Authorization", "Accept", "Content-Type"]),
        allow_credentials: true,
        ..Default::default()
    };

    let database_url = if let Ok(s) = env::var("DATABASE_URL") {
        s
    } else {
        "postgres://localhost:5432".into()
    };

    let validator_url = if let Ok(s) = env::var("VALIDATOR_URL") {
       s
    } else {
        "tcp://localhost:8004".into()
    };

    rocket::ignite()
        .mount("/", routes![
               hello,
               openapi::openapi_json,
               openapi::openapi_yaml,
               agents::get_agent,
               agents::get_agents,
               organizations::get_org,
               organizations::get_orgs,
               transactions::submit_txns,
               transactions::submit_txns_wait,
               transactions::get_batch_status])
        .manage(pools::init_pg_pool(database_url))
        .manage(ZmqMessageConnection::new(&validator_url))
        .attach(options)
        .catch(errors![not_found, internal_server_error])
        .launch();
}
