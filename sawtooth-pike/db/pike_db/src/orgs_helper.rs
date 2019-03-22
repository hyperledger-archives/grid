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

use schema::organizations;
use schema::organizations::dsl;
use models::{NewOrganization, Organization};

use diesel;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::QueryResult;

pub fn create_organization(conn: &PgConnection, org: NewOrganization) -> QueryResult<Organization> {
    diesel::insert_into(organizations::table)
        .values(&org)
        .get_result::<Organization>(conn)
}

pub fn update_organization(conn: &PgConnection, id: &str, org: NewOrganization) -> QueryResult<Organization> {
    diesel::update(organizations::table)
        .filter(dsl::id.eq(id))
        .set((
            dsl::name.eq(org.name),
            dsl::address.eq(org.address),
        ))
        .get_result::<Organization>(conn)
}

pub fn get_org(conn: &PgConnection, id: &str) -> QueryResult<Organization> {
    organizations::table
        .select(organizations::all_columns)
        .find(id)
        .first(conn)
}

pub fn get_orgs(conn: &PgConnection) -> QueryResult<Vec<Organization>> {
    organizations::table
        .select(organizations::all_columns)
        .load(conn)
}
