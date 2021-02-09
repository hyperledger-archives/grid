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
use crate::commits::MAX_COMMIT_NUM;
use crate::locations::store::diesel::{
    schema::{location, location_attribute},
    LocationStoreError,
};

use crate::error::InternalError;
use crate::locations::store::diesel::models::{LocationAttributeModel, LocationModel};
use crate::locations::store::{Location, LocationAttribute, LocationList};
use crate::paging::Paging;

use diesel::prelude::*;

pub(in crate::locations::store::diesel) trait LocationStoreListLocationsOperation<C: Connection> {
    fn list_locations(
        &self,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<LocationList, LocationStoreError>;
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
impl<'a> LocationStoreListLocationsOperation<diesel::pg::PgConnection>
    for LocationStoreOperations<'a, diesel::pg::PgConnection>
{
    fn list_locations(
        &self,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<LocationList, LocationStoreError> {
        self.conn
            .build_transaction()
            .read_write()
            .run::<_, LocationStoreError, _>(|| {
                let mut query = location::table
                    .into_boxed()
                    .select(location::all_columns)
                    .limit(limit)
                    .offset(offset)
                    .filter(location::end_commit_num.eq(MAX_COMMIT_NUM));

                if let Some(service_id) = service_id {
                    query = query.filter(location::service_id.eq(service_id));
                } else {
                    query = query.filter(location::service_id.is_null());
                }

                let locs = query.load::<LocationModel>(self.conn).map_err(|err| {
                    LocationStoreError::InternalError(InternalError::from_source(Box::new(err)))
                })?;

                let mut locations = Vec::new();

                for l in locs {
                    let loc: LocationModel = l;
                    let roots =
                        Self::get_root_attributes(&*self.conn, &loc.location_id, service_id)?;

                    let attrs = Self::get_attributes(&*self.conn, roots)?;

                    locations.push(Location::from((loc, attrs)));
                }

                let mut count_query = location::table.into_boxed().select(location::all_columns);

                if let Some(service_id) = service_id {
                    count_query = count_query.filter(location::service_id.eq(service_id));
                } else {
                    count_query = count_query.filter(location::service_id.is_null());
                }

                let total = count_query.count().get_result(self.conn)?;

                Ok(LocationList::new(
                    locations,
                    Paging::new(offset, limit, total),
                ))
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
                    Self::get_attributes(&conn, children)?,
                )));
            }
        }

        Ok(attrs)
    }
}

#[cfg(feature = "sqlite")]
impl<'a> LocationStoreListLocationsOperation<diesel::sqlite::SqliteConnection>
    for LocationStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn list_locations(
        &self,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<LocationList, LocationStoreError> {
        self.conn
            .immediate_transaction::<_, LocationStoreError, _>(|| {
                let mut query = location::table
                    .into_boxed()
                    .select(location::all_columns)
                    .limit(limit)
                    .offset(offset)
                    .filter(location::end_commit_num.eq(MAX_COMMIT_NUM));

                if let Some(service_id) = service_id {
                    query = query.filter(location::service_id.eq(service_id));
                } else {
                    query = query.filter(location::service_id.is_null());
                }

                let locs = query.load::<LocationModel>(self.conn).map_err(|err| {
                    LocationStoreError::InternalError(InternalError::from_source(Box::new(err)))
                })?;

                let mut locations = Vec::new();

                for l in locs {
                    let loc: LocationModel = l;
                    let roots =
                        Self::get_root_attributes(&*self.conn, &loc.location_id, service_id)?;

                    let attrs = Self::get_attributes(&*self.conn, roots)?;

                    locations.push(Location::from((loc, attrs)));
                }

                let mut count_query = location::table.into_boxed().select(location::all_columns);

                if let Some(service_id) = service_id {
                    count_query = count_query.filter(location::service_id.eq(service_id));
                } else {
                    count_query = count_query.filter(location::service_id.is_null());
                }

                let total = count_query.count().get_result(self.conn)?;

                Ok(LocationList::new(
                    locations,
                    Paging::new(offset, limit, total),
                ))
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
                    Self::get_attributes(&conn, children)?,
                )));
            }
        }

        Ok(attrs)
    }
}
