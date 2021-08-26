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

//! Traits and implementations useful for interacting with the REST API.

pub mod location;
pub mod pike;
pub mod product;
#[cfg(feature = "purchase-order")]
pub mod purchase_order;
pub mod schema;

use crate::error::ClientError;
use sawtooth_sdk::messages::batch::BatchList;

pub trait Client {
    /// Submits a list of batches
    ///
    /// # Arguments
    ///
    /// * `wait` - wait time in seconds
    /// * `batch_list` - The `BatchList` to be submitted
    /// * `service_id` - optional service id if running splinter
    fn post_batches(
        &self,
        wait: u64,
        batch_list: &BatchList,
        service_id: Option<&str>,
    ) -> Result<(), ClientError>;
}

pub trait ClientFactory {
    /// Retrieves a client for listing and showing locations
    fn get_location_client(&self, url: String) -> Box<dyn location::LocationClient>;

    /// Retrieves a client for listing and showing pike members
    fn get_pike_client(&self, url: String) -> Box<dyn pike::PikeClient>;

    /// Retrieves a client for listing and showing products
    fn get_product_client(&self, url: String) -> Box<dyn product::ProductClient>;

    /// Retrieves a client for listing and showing
    /// purchase orders, revisions, and versions
    #[cfg(feature = "purchase-order")]
    fn get_purchase_order_client(
        &self,
        url: String,
    ) -> Box<dyn purchase_order::PurchaseOrderClient>;

    /// Retrieves a client for listing and showing schemas
    fn get_schema_client(&self, url: String) -> Box<dyn schema::SchemaClient>;
}
