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

#[cfg(feature = "rest-api-endpoint-agent")]
pub mod agents;
#[cfg(feature = "rest-api-endpoint-batches")]
pub mod batches;
pub mod error;
#[cfg(feature = "rest-api-endpoint-location")]
pub mod locations;
#[cfg(feature = "rest-api-endpoint-organization")]
pub mod organizations;
pub mod paging;
#[cfg(feature = "rest-api-endpoint-product")]
pub mod products;
#[cfg(feature = "rest-api-endpoint-purchase-order")]
pub mod purchase_order;
#[cfg(feature = "rest-api-endpoint-role")]
pub mod roles;
#[cfg(feature = "rest-api-endpoint-schema")]
pub mod schemas;
#[cfg(feature = "rest-api-endpoint-submit")]
pub mod submit;
#[cfg(feature = "rest-api-endpoint-record")]
pub mod track_and_trace;
