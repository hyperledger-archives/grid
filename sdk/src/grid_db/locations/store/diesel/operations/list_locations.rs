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
use crate::grid_db::commits::MAX_COMMIT_NUM;
use crate::grid_db::locations::store::diesel::{
    schema::{location, location_attribute},
    LocationStoreError,
};

use crate::grid_db::locations::store::diesel::models::{LocationAttributeModel, LocationModel};
use crate::grid_db::locations::store::{Location, LocationAttribute};
use diesel::prelude::*;

pub(in crate::grid_db::locations::store::diesel) trait LocationStoreListLocationsOperation<
    C: Connection,
>
{
    fn list_locations(&self, service_id: Option<&str>)
        -> Result<Vec<Location>, LocationStoreError>;
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
    ) -> Result<Vec<Location>, LocationStoreError> {
        self.conn
            .build_transaction()
            .read_write()
            .run::<_, LocationStoreError, _>(|| {
                let locs: Vec<LocationModel> = location::table
                    .select(location::all_columns)
                    .filter(
                        location::service_id
                            .eq(&service_id)
                            .and(location::end_commit_num.eq(MAX_COMMIT_NUM)),
                    )
                    .load::<LocationModel>(self.conn)
                    .map(Some)
                    .map_err(|err| LocationStoreError::OperationError {
                        context: "Failed to fetch locations".to_string(),
                        source: Some(Box::new(err)),
                    })?
                    .ok_or_else(|| {
                        LocationStoreError::NotFoundError(
                            "Could not get all locations from storage".to_string(),
                        )
                    })?
                    .into_iter()
                    .collect();

                let mut locations = Vec::new();

                for l in locs {
                    let loc: LocationModel = l;
                    let roots =
                        Self::get_root_attributes(&*self.conn, &loc.location_id, service_id)?;

                    let attrs = Self::get_attributes(&*self.conn, roots)?;

                    locations.push(Location::from((loc, attrs)));
                }

                Ok(locations)
            })
    }

    fn get_root_attributes(
        conn: &PgConnection,
        location_id: &str,
        service_id: Option<&str>,
    ) -> QueryResult<Vec<LocationAttributeModel>> {
        location_attribute::table
            .select(location_attribute::all_columns)
            .filter(
                location_attribute::location_id
                    .eq(location_id)
                    .and(location_attribute::parent_property_name.is_null())
                    .and(location_attribute::end_commit_num.eq(MAX_COMMIT_NUM))
                    .and(location_attribute::service_id.eq(&service_id)),
            )
            .load::<LocationAttributeModel>(conn)
    }

    fn get_attributes(
        conn: &PgConnection,
        attributes: Vec<LocationAttributeModel>,
    ) -> Result<Vec<LocationAttribute>, LocationStoreError> {
        let mut attrs = Vec::new();

        for attr in attributes {
            let children = location_attribute::table
                .select(location_attribute::all_columns)
                .filter(
                    location_attribute::parent_property_name
                        .eq(&attr.parent_property_name)
                        .and(location_attribute::end_commit_num.eq(MAX_COMMIT_NUM))
                        .and(location_attribute::service_id.eq(&attr.service_id)),
                )
                .load(conn)?;

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
    ) -> Result<Vec<Location>, LocationStoreError> {
        self.conn
            .immediate_transaction::<_, LocationStoreError, _>(|| {
                let locs: Vec<LocationModel> = location::table
                    .select(location::all_columns)
                    .filter(
                        location::service_id
                            .eq(&service_id)
                            .and(location::end_commit_num.eq(MAX_COMMIT_NUM)),
                    )
                    .load::<LocationModel>(self.conn)
                    .map(Some)
                    .map_err(|err| LocationStoreError::OperationError {
                        context: "Failed to fetch locations".to_string(),
                        source: Some(Box::new(err)),
                    })?
                    .ok_or_else(|| {
                        LocationStoreError::NotFoundError(
                            "Could not get all locations from storage".to_string(),
                        )
                    })?
                    .into_iter()
                    .collect();

                let mut locations = Vec::new();

                for l in locs {
                    let loc: LocationModel = l;
                    let roots =
                        Self::get_root_attributes(&*self.conn, &loc.location_id, service_id)?;

                    let attrs = Self::get_attributes(&*self.conn, roots)?;

                    locations.push(Location::from((loc, attrs)));
                }

                Ok(locations)
            })
    }

    fn get_root_attributes(
        conn: &SqliteConnection,
        location_id: &str,
        service_id: Option<&str>,
    ) -> QueryResult<Vec<LocationAttributeModel>> {
        location_attribute::table
            .select(location_attribute::all_columns)
            .filter(
                location_attribute::location_id
                    .eq(location_id)
                    .and(location_attribute::parent_property_name.is_null())
                    .and(location_attribute::end_commit_num.eq(MAX_COMMIT_NUM))
                    .and(location_attribute::service_id.eq(&service_id)),
            )
            .load::<LocationAttributeModel>(conn)
    }

    fn get_attributes(
        conn: &SqliteConnection,
        attributes: Vec<LocationAttributeModel>,
    ) -> Result<Vec<LocationAttribute>, LocationStoreError> {
        let mut attrs = Vec::new();

        for attr in attributes {
            let children = location_attribute::table
                .select(location_attribute::all_columns)
                .filter(
                    location_attribute::parent_property_name
                        .eq(&attr.parent_property_name)
                        .and(location_attribute::end_commit_num.eq(MAX_COMMIT_NUM))
                        .and(location_attribute::service_id.eq(&attr.service_id)),
                )
                .load(conn)?;

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
