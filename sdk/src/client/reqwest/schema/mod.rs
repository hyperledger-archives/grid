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

pub(crate) mod data;

use crate::client::schema::{Schema, SchemaClient};

use crate::client::reqwest::{fetch_entities_list, fetch_entity, post_batches};
use crate::client::Client;
use crate::error::ClientError;

use sawtooth_sdk::messages::batch::BatchList;

const SCHEMA_ROUTE: &str = "schema";

/// The Reqwest implementation of the Schema client
pub struct ReqwestSchemaClient {
    url: String,
}

impl ReqwestSchemaClient {
    pub fn new(url: String) -> Self {
        Self { url }
    }
}

impl Client for ReqwestSchemaClient {
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

impl SchemaClient for ReqwestSchemaClient {
    /// Fetches a single schema based on name
    ///
    /// # Arguments
    ///
    /// * `name` - the name of the schema (identifier)
    /// * `service_id` - optional - the service ID to fetch the schema from
    fn get_schema(&self, name: String, service_id: Option<&str>) -> Result<Schema, ClientError> {
        let dto = fetch_entity::<data::Schema>(
            &self.url,
            format!("{}/{}", SCHEMA_ROUTE, name),
            service_id,
        )?;
        Ok(Schema::from(&dto))
    }

    /// Fetches a list of schemas for the organization
    ///
    /// # Arguments
    ///
    /// * `service_id` - optional - the service ID to fetch the schemas from
    fn list_schemas(&self, service_id: Option<&str>) -> Result<Vec<Schema>, ClientError> {
        let dto_vec = fetch_entities_list::<data::Schema>(
            &self.url,
            SCHEMA_ROUTE.to_string(),
            service_id,
            None,
        )?;
        Ok(dto_vec.iter().map(Schema::from).collect())
    }
}
