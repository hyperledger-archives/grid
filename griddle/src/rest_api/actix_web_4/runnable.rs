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

//! Contains the implementation of `RunnableRestApi`

use cylinder::Signer;
#[cfg(feature = "proxy")]
use grid_sdk::proxy::ProxyClient;

use crate::internals::DLTBackend;
use crate::rest_api::{
    actix_web_4::{GriddleResourceProvider, GriddleRestApi},
    error::GriddleRestApiServerError,
};

/// A configured REST API which may best started with `run` function.
pub struct RunnableGriddleRestApi {
    pub(super) resource_providers: Vec<Box<dyn GriddleResourceProvider>>,
    pub(super) bind: String,
    #[cfg(feature = "proxy")]
    pub(super) proxy_client: Box<dyn ProxyClient>,
    pub(super) signer: Box<dyn Signer>,
    pub(super) dlt_backend: DLTBackend,
}

impl RunnableGriddleRestApi {
    /// Start the REST API and finish any necessary setup such as binding to ports, adding resource
    /// endpoints, etc.
    pub fn run(self) -> Result<GriddleRestApi, GriddleRestApiServerError> {
        let RunnableGriddleRestApi {
            resource_providers,
            bind,
            #[cfg(feature = "proxy")]
            proxy_client,
            signer,
            dlt_backend,
        } = self;

        GriddleRestApi::new(
            bind,
            resource_providers,
            #[cfg(feature = "proxy")]
            proxy_client,
            signer,
            dlt_backend,
        )
    }
}
