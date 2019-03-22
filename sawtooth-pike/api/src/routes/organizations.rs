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
use pike_db::models::Organization;

#[get("/organization/<id>")]
fn get_org(conn: DbConn, id: String) -> Option<Json<Organization>> {
    if let Ok(org) = db::get_org(&conn, &id) {
        Some(Json(org))
    } else {
        None
    }
}

#[get("/organization")]
fn get_orgs(conn: DbConn) -> Json<Vec<Organization>> {
    if let Ok(orgs) = db::get_orgs(&conn) {
        Json(orgs)
    } else {
        Json(vec![])
    }
}
