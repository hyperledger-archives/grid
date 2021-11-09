// Copyright 2021 Cargill Incorporated
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

mod data;

use crate::client::location::{Location, LocationClient};
use crate::client::reqwest::{fetch_entities_list, fetch_entity, post_batches};
use crate::client::Client;
use crate::error::ClientError;

use sawtooth_sdk::messages::batch::BatchList;

const LOCATION_ROUTE: &str = "location";

/// The Reqwest implementation of the Location client
pub struct ReqwestLocationClient {
    url: String,
}

impl ReqwestLocationClient {
    pub fn new(url: String) -> Self {
        Self { url }
    }
}

impl Client for ReqwestLocationClient {
    /// Submits a list of batches
    ///
    /// # Arguments
    ///
    /// * `wait` - wait time in seconds
    /// * `batch_list` - The `BatchList` to be submitted
    /// * `service_id` - optional - the service ID to post batches to if running splinter
    fn post_batches(
        &self,
        wait: u64,
        batch_list: &BatchList,
        service_id: Option<&str>,
    ) -> Result<(), ClientError> {
        post_batches(&self.url, wait, batch_list, service_id)
    }
}

impl LocationClient for ReqwestLocationClient {
    /// Fetches a location based on its identifier
    ///
    /// # Arguments
    ///
    /// * `id` - the location's identifier
    /// * `service_id` - optional - the service ID to fetch the location from
    fn get_location(&self, id: String, service_id: Option<&str>) -> Result<Location, ClientError> {
        let dto = fetch_entity::<data::Location>(
            &self.url,
            format!("{}/{}", LOCATION_ROUTE, id),
            service_id,
        )?;
        Ok(Location::from(&dto))
    }

    /// Fetches locations
    ///
    /// # Arguments
    ///
    /// * `service_id` - optional - the service ID to fetch locations from
    fn list_locations(&self, service_id: Option<&str>) -> Result<Vec<Location>, ClientError> {
        let dto_vec = fetch_entities_list::<data::Location>(
            &self.url,
            LOCATION_ROUTE.to_string(),
            service_id,
            None,
        )?;
        Ok(dto_vec.iter().map(Location::from).collect())
    }
}
