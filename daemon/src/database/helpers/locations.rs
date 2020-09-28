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

use super::models::{Location, LocationPropertyValue};
use super::schema::{location, location_property_value};
use super::MAX_COMMIT_NUM;

use diesel::{pg::PgConnection, prelude::*, result::Error::NotFound, QueryResult};

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
                .and(location::end_commit_num.eq(MAX_COMMIT_NUM)),
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
