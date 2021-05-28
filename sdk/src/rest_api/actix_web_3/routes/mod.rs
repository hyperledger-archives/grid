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

#[cfg(feature = "rest-api-endpoint-agent")]
mod agents;
#[cfg(feature = "rest-api-endpoint-batches")]
mod batches;
#[cfg(feature = "rest-api-endpoint-location")]
mod locations;
#[cfg(feature = "rest-api-endpoint-organization")]
mod organizations;
#[cfg(feature = "rest-api-endpoint-product")]
mod products;
#[cfg(feature = "rest-api-endpoint-purchase-order")]
mod purchase_orders;
#[cfg(feature = "rest-api-endpoint-record")]
mod records;
#[cfg(feature = "rest-api-endpoint-role")]
mod roles;
#[cfg(feature = "rest-api-endpoint-schema")]
mod schemas;
#[cfg(feature = "rest-api-endpoint-submit")]
mod submit;

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

pub(in crate::rest_api) const DEFAULT_GRID_PROTOCOL_VERSION: &str = "1";
