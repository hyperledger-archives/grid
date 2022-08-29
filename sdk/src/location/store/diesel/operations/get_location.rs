// Copyright 2018-2020 Cargill Incorporated
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

use super::LocationStoreOperations;
use crate::location::store::diesel::{
    schema::{location, location_attribute},
    LocationStoreError,
};

use crate::commits::MAX_COMMIT_NUM;
use crate::error::InternalError;
use crate::location::store::diesel::models::{LocationAttributeModel, LocationModel};
use crate::location::store::{Location, LocationAttribute};
use diesel::{prelude::*, result::Error::NotFound, QueryResult};

pub(in crate::location::store::diesel) trait LocationStoreGetLocationOperation<C: Connection> {
    fn get_location(
        &self,
        location_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Location>, LocationStoreError>;
    fn get_root_attributes(
        conn: &C,
        location_id: &str,
        service_id: Option<&str>,
    ) -> QueryResult<Vec<LocationAttributeModel>>;
    fn get_attributes(
        conn: &C,
        attributes: Vec<LocationAttributeModel>,
    ) -> Result<Vec<LocationAttribute>, LocationStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> LocationStoreGetLocationOperation<diesel::pg::PgConnection>
    for LocationStoreOperations<'a, diesel::pg::PgConnection>
{
    fn get_location(
        &self,
        location_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Location>, LocationStoreError> {
        self.conn.transaction::<_, LocationStoreError, _>(|| {
            let mut query = location::table
                .into_boxed()
                .select(location::all_columns)
                .filter(
                    location::location_id
                        .eq(&location_id)
                        .and(location::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

            if let Some(service_id) = service_id {
                query = query.filter(location::service_id.eq(service_id));
            } else {
                query = query.filter(location::service_id.is_null());
            }

            let loc = query
                .first::<LocationModel>(self.conn)
                .map(Some)
                .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
                .map_err(|err| {
                    LocationStoreError::InternalError(InternalError::from_source(Box::new(err)))
                })?;

            let roots = Self::get_root_attributes(self.conn, location_id, service_id)?;

            let attrs = Self::get_attributes(self.conn, roots)?;

            Ok(loc.map(|loc| Location::from((loc, attrs))))
        })
    }

    fn get_root_attributes(
        conn: &PgConnection,
        location_id: &str,
        service_id: Option<&str>,
    ) -> QueryResult<Vec<LocationAttributeModel>> {
        let mut query = location_attribute::table
            .into_boxed()
            .select(location_attribute::all_columns)
            .filter(
                location_attribute::location_id
                    .eq(location_id)
                    .and(location_attribute::parent_property_name.is_null())
                    .and(location_attribute::end_commit_num.eq(MAX_COMMIT_NUM)),
            );

        if let Some(service_id) = service_id {
            query = query.filter(location_attribute::service_id.eq(service_id));
        } else {
            query = query.filter(location_attribute::service_id.is_null());
        }

        query.load::<LocationAttributeModel>(conn)
    }

    fn get_attributes(
        conn: &PgConnection,
        attributes: Vec<LocationAttributeModel>,
    ) -> Result<Vec<LocationAttribute>, LocationStoreError> {
        let mut attrs = Vec::new();

        for attr in attributes {
            let mut query = location_attribute::table
                .into_boxed()
                .select(location_attribute::all_columns)
                .filter(
                    location_attribute::parent_property_name
                        .eq(&attr.parent_property_name)
                        .and(location_attribute::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

            if let Some(ref service_id) = attr.service_id {
                query = query.filter(location_attribute::service_id.eq(service_id));
            } else {
                query = query.filter(location_attribute::service_id.is_null());
            }

            let children = query.load(conn)?;

            if children.is_empty() {
                attrs.push(LocationAttribute::from(attr));
            } else {
                attrs.push(LocationAttribute::from((
                    attr,
                    Self::get_attributes(conn, children)?,
                )));
            }
        }

        Ok(attrs)
    }
}

#[cfg(feature = "sqlite")]
impl<'a> LocationStoreGetLocationOperation<diesel::sqlite::SqliteConnection>
    for LocationStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn get_location(
        &self,
        location_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Location>, LocationStoreError> {
        self.conn.transaction::<_, LocationStoreError, _>(|| {
            let mut query = location::table
                .into_boxed()
                .select(location::all_columns)
                .filter(
                    location::location_id
                        .eq(&location_id)
                        .and(location::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

            if let Some(service_id) = service_id {
                query = query.filter(location::service_id.eq(service_id));
            } else {
                query = query.filter(location::service_id.is_null());
            }

            let loc = query
                .first::<LocationModel>(self.conn)
                .map(Some)
                .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
                .map_err(|err| {
                    LocationStoreError::InternalError(InternalError::from_source(Box::new(err)))
                })?;

            let roots = Self::get_root_attributes(self.conn, location_id, service_id)?;

            let attrs = Self::get_attributes(self.conn, roots)?;

            Ok(loc.map(|loc| Location::from((loc, attrs))))
        })
    }

    fn get_root_attributes(
        conn: &SqliteConnection,
        location_id: &str,
        service_id: Option<&str>,
    ) -> QueryResult<Vec<LocationAttributeModel>> {
        let mut query = location_attribute::table
            .into_boxed()
            .select(location_attribute::all_columns)
            .filter(
                location_attribute::location_id
                    .eq(location_id)
                    .and(location_attribute::parent_property_name.is_null())
                    .and(location_attribute::end_commit_num.eq(MAX_COMMIT_NUM)),
            );

        if let Some(service_id) = service_id {
            query = query.filter(location_attribute::service_id.eq(service_id));
        } else {
            query = query.filter(location_attribute::service_id.is_null());
        }

        query.load::<LocationAttributeModel>(conn)
    }

    fn get_attributes(
        conn: &SqliteConnection,
        attributes: Vec<LocationAttributeModel>,
    ) -> Result<Vec<LocationAttribute>, LocationStoreError> {
        let mut attrs = Vec::new();

        for attr in attributes {
            let mut query = location_attribute::table
                .into_boxed()
                .select(location_attribute::all_columns)
                .filter(
                    location_attribute::parent_property_name
                        .eq(&attr.parent_property_name)
                        .and(location_attribute::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

            if let Some(ref service_id) = attr.service_id {
                query = query.filter(location_attribute::service_id.eq(service_id));
            } else {
                query = query.filter(location_attribute::service_id.is_null());
            }

            let children = query.load(conn)?;

            if children.is_empty() {
                attrs.push(LocationAttribute::from(attr));
            } else {
                attrs.push(LocationAttribute::from((
                    attr,
                    Self::get_attributes(conn, children)?,
                )));
            }
        }

        Ok(attrs)
    }
}
