// Copyright 2018-2022 Cargill Incorporated
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

#[cfg(feature = "rest-api-endpoint-agent")]
pub(in crate) mod agents;
#[cfg(feature = "rest-api-endpoint-batches")]
pub(in crate) mod batches;
#[cfg(feature = "rest-api-endpoint-location")]
pub(in crate) mod locations;
#[cfg(feature = "rest-api-endpoint-organization")]
pub(in crate) mod organizations;
#[cfg(feature = "rest-api-endpoint-product")]
pub(in crate) mod products;
#[cfg(feature = "rest-api-endpoint-proxy")]
mod proxy;
#[cfg(feature = "rest-api-endpoint-purchase-order")]
pub(in crate) mod purchase_orders;
#[cfg(feature = "rest-api-endpoint-record")]
pub(in crate) mod records;
#[cfg(feature = "rest-api-endpoint-role")]
pub(in crate) mod roles;
#[cfg(feature = "rest-api-endpoint-schema")]
pub(in crate) mod schemas;
#[cfg(feature = "rest-api-endpoint-submit")]
pub(in crate) mod submit;

#[cfg(feature = "rest-api-endpoint-agent")]
pub use agents::*;
#[cfg(feature = "rest-api-endpoint-batches")]
pub use batches::*;
#[cfg(feature = "rest-api-endpoint-location")]
pub use locations::*;
#[cfg(feature = "rest-api-endpoint-organization")]
pub use organizations::*;
#[cfg(feature = "rest-api-endpoint-product")]
pub use products::*;
#[cfg(feature = "rest-api-endpoint-proxy")]
pub use proxy::*;
#[cfg(feature = "rest-api-endpoint-purchase-order")]
pub use purchase_orders::*;
#[cfg(feature = "rest-api-endpoint-record")]
pub use records::*;
#[cfg(feature = "rest-api-endpoint-role")]
pub use roles::*;
#[cfg(feature = "rest-api-endpoint-schema")]
pub use schemas::*;
#[cfg(feature = "rest-api-endpoint-submit")]
pub use submit::*;

#[cfg(any(
    feature = "rest-api-endpoint-agent",
    feature = "rest-api-endpoint-batches",
    feature = "rest-api-endpoint-location",
    feature = "rest-api-endpoint-organization",
    feature = "rest-api-endpoint-product",
    feature = "rest-api-endpoint-purchase-order",
    feature = "rest-api-endpoint-record",
    feature = "rest-api-endpoint-role",
    feature = "rest-api-endpoint-schema",
    feature = "rest-api-endpoint-submit",
))]
pub(in crate::rest_api) const DEFAULT_GRID_PROTOCOL_VERSION: &str = "1";
