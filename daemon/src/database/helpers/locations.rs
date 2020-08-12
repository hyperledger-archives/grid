/*
 * Copyright 2020 Cargill Incorporated
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * -----------------------------------------------------------------------------
 */

use super::models::{NewLocation, Location, NewLocationPropertyValue, LocationPropertyValue};
use super::schema::{location, location_property_value};
use super::MAX_COMMIT_NUM;

use diesel::{
    dsl::{insert_into, update},
    pg::PgConnection,
    prelude::*,
    result::Error::NotFound,
    QueryResult,
};

pub fn insert_locations(conn: &PgConnection, locations: &[NewLocation]) -> QueryResult<()> {
    for loc in locations {
        update_loc_end_commit_num(
            conn,
            &loc.location_id,
            loc.service_id.as_deref(),
            loc.start_commit_num,
        )?;
    }

    insert_into(location::table)
        .values(locations)
        .execute(conn)
        .map(|_| ())
}

pub fn insert_location_property_values(
    conn: &PgConnection,
    property_values: &[NewLocationPropertyValue],
) -> QueryResult<()> {
    for value in property_values {
        update_location_property_values (
            conn,
            &value.location_id,
            value.service_id.as_deref(),
            value.start_commit_num,
        )?;
    }

    insert_into(location_property_value::table)
        .values(property_values)
        .execute(conn)
        .map(|_| ())
}

pub fn delete_location(
    conn: &PgConnection,
    address: &str,
    current_commit_num: i64
) -> QueryResult<()> {
    update(location::table)
        .filter(
            location::location_address
                .eq(address)
                .and(location::end_commit_num.eq(MAX_COMMIT_NUM)),
        )
        .set(location::end_commit_num.eq(current_commit_num))
        .execute(conn)
        .map(|_| ())
}

pub fn delete_location_property_values(
    conn: &PgConnection,
    address: &str,
    current_commit_num: i64,
) -> QueryResult<()> {
    update(location_property_value::table)
        .filter(
            location_property_value::location_address
                .eq(address)
                .and(location_property_value::end_commit_num.eq(MAX_COMMIT_NUM)),
        )
        .set(location_property_value::end_commit_num.eq(current_commit_num))
        .execute(conn)
        .map(|_| ())
}

fn update_loc_end_commit_num(
    conn: &PgConnection,
    location_id: &str,
    service_id: Option<&str>,
    current_commit_num: i64,
) -> QueryResult<()> {
    let update = update(location::table);

    if let Some(service_id) = service_id {
        update
            .filter(
                location::location_id
                    .eq(location_id)
                    .and(location::end_commit_num.eq(MAX_COMMIT_NUM))
                    .and(location::service_id.eq(service_id)),
            )
            .set(location::end_commit_num.eq(current_commit_num))
            .execute(conn)
            .map(|_| ())
    } else {
        update
            .filter(
                location::location_id
                    .eq(location_id)
                    .and(location::end_commit_num.eq(MAX_COMMIT_NUM)),
            )
            .set(location::end_commit_num.eq(current_commit_num))
            .execute(conn)
            .map(|_| ())
    }
}

fn update_location_property_values(
    conn: &PgConnection,
    location_id: &str,
    service_id: Option<&str>,
    current_commit_num: i64,
) -> QueryResult<()> {
    let update = update(location_property_value::table);

    if let Some(service_id) = service_id {
        update
            .filter(location_property_value::location_id
                .eq(location_id)
                .and(location_property_value::end_commit_num.eq(MAX_COMMIT_NUM))
                .and(location_property_value::service_id.eq(service_id))
            )
            .set(location_property_value::end_commit_num.eq(current_commit_num))
            .execute(conn)
            .map(|_| ())
    } else {
        update
            .filter(
                location_property_value::location_id
                    .eq(location_id)
                    .and(location_property_value::end_commit_num.eq(MAX_COMMIT_NUM))
            )
            .set(location_property_value::end_commit_num.eq(current_commit_num))
            .execute(conn)
            .map(|_| ())
    }
}

pub fn list_locations(conn: &PgConnection, service_id: Option<&str>) -> QueryResult<Vec<Location>> {
    let mut query = location::table
        .into_boxed()
        .select(location::all_columns)
        .filter(location::end_commit_num.eq(MAX_COMMIT_NUM));

    if let Some(service_id) = service_id {
        query = query.filter(location::service_id.eq(service_id));
    } else {
        query = query.filter(location::service_id.is_null());
    }
    query.load::<Location>(conn)
}

pub fn list_location_property_values(
    conn: &PgConnection,
    service_id: Option<&str>,
) -> QueryResult<Vec<LocationPropertyValue>> {
    let mut query = location_property_value::table
        .into_boxed()
        .select(location_property_value::all_columns)
        .filter(location_property_value::end_commit_num.eq(MAX_COMMIT_NUM));

    if let Some(service_id) = service_id {
        query = query.filter(location_property_value::service_id.eq(service_id));
    } else {
        query = query.filter(location_property_value::service_id.is_null());
    }
    query.load::<LocationPropertyValue>(conn)
}

pub fn fetch_location(
    conn: &PgConnection,
    location_id: &str,
    service_id: Option<&str>,
) -> QueryResult<Option<Location>> {
    let mut query = location::table
        .into_boxed()
        .select(location::all_columns)
        .filter(
            location::location_id
                .eq(location_id)
                .and(location::end_commit_num.eq(MAX_COMMIT_NUM))
        );

    if let Some(service_id) = service_id {
        query = query.filter(location::service_id.eq(service_id));
    } else {
        query = query.filter(location::service_id.is_null());
    }

    query
        .first(conn)
        .map(Some)
        .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
}

pub fn fetch_location_property_values(
    conn: &PgConnection,
    location_id: &str,
    service_id: Option<&str>,
) -> QueryResult<Vec<LocationPropertyValue>> {
    let mut query = location_property_value::table
        .into_boxed()
        .select(location_property_value::all_columns)
        .filter(
            location_property_value::location_id
                .eq(location_id)
                .and(location_property_value::end_commit_num.eq(MAX_COMMIT_NUM)),
        );

    if let Some(service_id) = service_id {
        query = query.filter(location_property_value::service_id.eq(service_id));
    } else {
        query = query.filter(location_property_value::service_id.is_null());
    }
    query.load::<LocationPropertyValue>(conn)
}
