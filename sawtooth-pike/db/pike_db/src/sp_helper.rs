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

use schema::smartpermissions;
use schema::smartpermissions::dsl;
use models::{SmartPermission, NewSmartPermission};

use diesel;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::QueryResult;

pub fn create_smart_permission(conn: &PgConnection, sp: NewSmartPermission) -> QueryResult<SmartPermission> {
    diesel::insert_into(smartpermissions::table)
        .values(&sp)
        .get_result::<SmartPermission>(conn)
}

pub fn delete_smart_permission(conn: &PgConnection, address: &str) -> QueryResult<SmartPermission> {
    diesel::delete(smartpermissions::table)
        .filter(dsl::address.eq(address))
        .get_result::<SmartPermission>(conn)
}

pub fn get_smart_permission(conn: &PgConnection, name: &str) -> QueryResult<SmartPermission> {
    smartpermissions::table
        .select(smartpermissions::all_columns)
        .find(name)
        .first(conn)
}

pub fn get_smart_permissions(conn: &PgConnection) -> QueryResult<Vec<SmartPermission>> {
    smartpermissions::table
        .select(smartpermissions::all_columns)
        .load(conn)
}
