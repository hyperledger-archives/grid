// Copyright 2018-2021 Cargill Incorporated
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

#[cfg(feature = "diesel")]
pub(in crate) mod diesel;
mod error;

use crate::paging::Paging;

#[cfg(feature = "diesel")]
pub use self::diesel::{DieselConnectionLocationStore, DieselLocationStore};
pub use error::LocationStoreError;

/// Represents a Grid Location
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Location {
    pub location_id: String,
    pub location_address: String,
    pub location_namespace: String,
    pub owner: String,
    pub attributes: Vec<LocationAttribute>,
    // The indicators of the start and stop for the slowly-changing dimensions.
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
    pub last_updated: Option<i64>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct LocationList {
    pub data: Vec<Location>,
    pub paging: Paging,
}

impl LocationList {
    pub fn new(data: Vec<Location>, paging: Paging) -> Self {
        Self { data, paging }
    }
}

/// Represents a Grid Location Attribute
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct LocationAttribute {
    pub location_id: String,
    pub location_address: String,
    pub property_name: String,
    pub data_type: String,
    pub bytes_value: Option<Vec<u8>>,
    pub boolean_value: Option<bool>,
    pub number_value: Option<i64>,
    pub string_value: Option<String>,
    pub enum_value: Option<i32>,
    pub struct_values: Option<Vec<LocationAttribute>>,
    pub lat_long_value: Option<LatLongValue>,
    // The indicators of the start and stop for the slowly-changing dimensions.
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct LatLong;

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct LatLongValue(pub i64, pub i64);

pub trait LocationStore {
    /// Adds a location to the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `location` - The location to be added
    fn add_location(&self, location: Location) -> Result<(), LocationStoreError>;

    /// Fetches a location from the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `location_id` - The ID of the location to be fetched
    ///  * `service_id` - optional - The service ID to fetch the location from
    fn get_location(
        &self,
        location_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Location>, LocationStoreError>;

    /// Gets locations from the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `service_id` - optional - The service ID to get the locations for
    ///  * `offset` - The index of the first in storage to retrieve
    ///  * `limit` - The number of items to retrieve from the offset
    fn list_locations(
        &self,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<LocationList, LocationStoreError>;

    /// Deletes a location from the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `address` - The address of the record to be deleted
    ///  * `current_commit_num` - The current commit height
    fn delete_location(
        &self,
        address: &str,
        current_commit_num: i64,
    ) -> Result<(), LocationStoreError>;
}

impl<LS> LocationStore for Box<LS>
where
    LS: LocationStore + ?Sized,
{
    fn add_location(&self, location: Location) -> Result<(), LocationStoreError> {
        (**self).add_location(location)
    }

    fn get_location(
        &self,
        location_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Location>, LocationStoreError> {
        (**self).get_location(location_id, service_id)
    }

    fn list_locations(
        &self,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<LocationList, LocationStoreError> {
        (**self).list_locations(service_id, offset, limit)
    }

    fn delete_location(
        &self,
        address: &str,
        current_commit_num: i64,
    ) -> Result<(), LocationStoreError> {
        (**self).delete_location(address, current_commit_num)
    }
}
