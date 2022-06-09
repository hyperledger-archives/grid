// Copyright 2022 Cargill Incorporated
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

#[cfg(feature = "rest-api")]
use cylinder::Signer;
#[cfg(all(feature = "proxy", feature = "rest-api"))]
use grid_sdk::proxy::ProxyClient;

use crate::internals::DLTBackend;
#[cfg(feature = "rest-api-actix-web-4")]
use crate::rest_api::actix_web_4::RunnableGriddleRestApi;

#[cfg(feature = "rest-api")]
pub enum RunnableGriddleRestApiVariant {
    #[cfg(feature = "rest-api-actix-web-4")]
    ActixWeb4(RunnableGriddleRestApi),
}

/// A fully configured and runnable version of Griddle
pub struct RunnableGriddle {
    #[cfg(feature = "rest-api")]
    pub(super) rest_api: RunnableGriddleRestApiVariant,
    #[cfg(feature = "rest-api")]
    pub(super) rest_api_endpoint: String,
    #[cfg(feature = "rest-api")]
    pub(super) signer: Box<dyn Signer>,
    #[cfg(all(feature = "proxy", feature = "rest-api"))]
    /// Client used to proxy requests for Griddle
    pub(super) proxy_client: Box<dyn ProxyClient>,
    /// Type of backend DLT used by Grid
    pub(super) dlt_backend: DLTBackend,
}
