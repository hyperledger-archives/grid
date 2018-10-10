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
use pike_db::models::SmartPermission;

#[get("/smartpermission/<name>")]
fn get_smart_permission(conn: DbConn, name: String) -> Option<Json<SmartPermission>> {
    if let Ok(sp) = db::get_smart_permission(&conn, &name) {
        Some(Json(sp))
    } else {
        None
    }
}

#[get("/smartpermission")]
fn get_smart_permissions(conn: DbConn) -> Json<Vec<SmartPermission>> {
    if let Ok(sp) = db::get_smart_permissions(&conn) {
        Json(sp)
    } else {
        Json(vec![])
    }
}
