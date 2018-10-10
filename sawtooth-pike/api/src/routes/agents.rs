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

extern crate rocket;

use rocket_contrib::Json;
use guard::db_conn::DbConn;

use pike_db as db;
use pike_db::models::Agent;

#[get("/agent/<publickey>")]
fn get_agent(conn: DbConn, publickey: String) -> Option<Json<Agent>> {
    if let Ok(agent) = db::get_agent(&conn, &publickey) {
        Some(Json(agent))
    } else {
        None
    }
}

#[get("/agent")]
fn get_agents(conn: DbConn) -> Json<Vec<Agent>> {
    if let Ok(agents) = db::get_agents(&conn) {
        Json(agents)
    } else {
        Json(vec![])
    }
}
