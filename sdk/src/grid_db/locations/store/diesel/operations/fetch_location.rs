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
use crate::grid_db::locations::store::diesel::{
    schema::{location, location_attribute},
    LocationStoreError,
};

use crate::grid_db::commits::MAX_COMMIT_NUM;
use crate::grid_db::locations::store::diesel::models::{LocationAttributeModel, LocationModel};
use crate::grid_db::locations::store::{Location, LocationAttribute};
use diesel::{prelude::*, result::Error::NotFound, QueryResult};

pub(in crate::grid_db::locations::store::diesel) trait LocationStoreFetchLocationOperation<
    C: Connection,
>
{
    fn fetch_location(
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
impl<'a> LocationStoreFetchLocationOperation<diesel::pg::PgConnection>
    for LocationStoreOperations<'a, diesel::pg::PgConnection>
{
    fn fetch_location(
        &self,
        location_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Location>, LocationStoreError> {
        self.conn
            .build_transaction()
            .read_write()
            .run::<_, LocationStoreError, _>(|| {
                let loc = location::table
                    .filter(
                        location::location_id
                            .eq(&location_id)
                            .and(location::service_id.eq(&service_id))
                            .and(location::end_commit_num.eq(MAX_COMMIT_NUM)),
                    )
                    .first::<LocationModel>(self.conn)
                    .map(Some)
                    .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
                    .map_err(|err| LocationStoreError::QueryError {
                        context: "Failed to fetch location for location_id".to_string(),
                        source: Box::new(err),
                    })?
                    .ok_or_else(|| {
                        LocationStoreError::NotFoundError(format!(
                            "Failed to find location: {}",
                            location_id,
                        ))
                    })?;

                let roots = Self::get_root_attributes(&*self.conn, &location_id, service_id)?;

                let attrs = Self::get_attributes(&*self.conn, roots)?;

                Ok(Some(Location::from((loc, attrs))))
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
impl<'a> LocationStoreFetchLocationOperation<diesel::sqlite::SqliteConnection>
    for LocationStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn fetch_location(
        &self,
        location_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Location>, LocationStoreError> {
        self.conn
            .immediate_transaction::<_, LocationStoreError, _>(|| {
                let loc = location::table
                    .filter(
                        location::location_id
                            .eq(&location_id)
                            .and(location::service_id.eq(&service_id))
                            .and(location::end_commit_num.eq(MAX_COMMIT_NUM)),
                    )
                    .first::<LocationModel>(self.conn)
                    .map(Some)
                    .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
                    .map_err(|err| LocationStoreError::QueryError {
                        context: "Failed to fetch location for location_id".to_string(),
                        source: Box::new(err),
                    })?
                    .ok_or_else(|| {
                        LocationStoreError::NotFoundError(format!(
                            "Failed to find location: {}",
                            location_id,
                        ))
                    })?;

                let roots = Self::get_root_attributes(&*self.conn, &location_id, service_id)?;

                let attrs = Self::get_attributes(&*self.conn, roots)?;

                Ok(Some(Location::from((loc, attrs))))
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
