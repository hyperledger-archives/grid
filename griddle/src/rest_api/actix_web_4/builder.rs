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

//! Contains the implementation of `GriddleRestApiBuilder`

use cylinder::Signer;

use grid_sdk::error::InvalidArgumentError;
#[cfg(feature = "proxy")]
use grid_sdk::proxy::ProxyClient;

use crate::rest_api::{
    actix_web_4::{GriddleResourceProvider, RunnableGriddleRestApi},
    GriddleRestApiServerError, Scope,
};

/// Builds a `RunnableGriddleRestApi`.
///
/// This builder's primary function is to create the runnable Griddle REST API in a valid state.
#[derive(Default)]
pub struct GriddleRestApiBuilder {
    resource_providers: Vec<Box<dyn GriddleResourceProvider>>,
    bind: Option<String>,
    #[cfg(feature = "proxy")]
    proxy_client: Option<Box<dyn ProxyClient>>,
    signer: Option<Box<dyn Signer>>,
    scope: Option<Scope>,
}

impl GriddleRestApiBuilder {
    /// Construct a new `GriddleRestApiBuilder`
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_resource_provider(
        mut self,
        resource_provider: Box<dyn GriddleResourceProvider>,
    ) -> Self {
        self.resource_providers.push(resource_provider);
        self
    }

    pub fn with_bind(mut self, value: String) -> Self {
        self.bind = Some(value);
        self
    }

    #[cfg(feature = "proxy")]
    pub fn with_proxy_client(mut self, client: Box<dyn ProxyClient>) -> Self {
        self.proxy_client = Some(client);
        self
    }

    pub fn with_signer(mut self, signer: Box<dyn Signer>) -> Self {
        self.signer = Some(signer);
        self
    }

    pub fn with_scope(mut self, service_scope: Scope) -> Self {
        self.scope = Some(service_scope);
        self
    }

    pub fn build(self) -> Result<RunnableGriddleRestApi, GriddleRestApiServerError> {
        let bind = self.bind.ok_or_else(|| {
            GriddleRestApiServerError::InvalidArgument(InvalidArgumentError::new(
                "bind".to_string(),
                "Missing required field".to_string(),
            ))
        })?;

        #[cfg(feature = "proxy")]
        let proxy_client = self.proxy_client.ok_or_else(|| {
            GriddleRestApiServerError::InvalidArgument(InvalidArgumentError::new(
                "proxy_client".to_string(),
                "Missing required field".to_string(),
            ))
        })?;

        let signer = self.signer.ok_or_else(|| {
            GriddleRestApiServerError::InvalidArgument(InvalidArgumentError::new(
                "signer".to_string(),
                "Missing required field".to_string(),
            ))
        })?;

        let scope = self.scope.ok_or_else(|| {
            GriddleRestApiServerError::InvalidArgument(InvalidArgumentError::new(
                "service_scope".to_string(),
                "Missing required field".to_string(),
            ))
        })?;

        Ok(RunnableGriddleRestApi {
            bind,
            resource_providers: self.resource_providers,
            #[cfg(feature = "proxy")]
            proxy_client,
            signer,
            scope,
        })
    }
}
