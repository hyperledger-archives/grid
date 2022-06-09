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

#[cfg(feature = "rest-api")]
use cylinder::Signer;
#[cfg(all(feature = "proxy", feature = "rest-api"))]
use grid_sdk::proxy::ProxyClient;

#[cfg(feature = "rest-api")]
use crate::internals::RunnableGriddleRestApiVariant;
use crate::internals::{DLTBackend, GriddleError, RunnableGriddle};
#[cfg(feature = "rest-api-actix-web-4")]
use crate::rest_api::actix_web_4::GriddleRestApiBuilder;

#[derive(Default)]
pub struct GriddleBuilder {
    #[cfg(feature = "rest-api")]
    /// Name of the private key file used for signing
    signer: Option<Box<dyn Signer>>,
    #[cfg(feature = "rest-api")]
    /// REST API backend implementation
    rest_api_variant: Option<GriddleRestApiVariant>,
    #[cfg(feature = "rest-api")]
    /// Address of the Griddle REST API
    rest_api_endpoint: Option<String>,
    #[cfg(all(feature = "proxy", feature = "rest-api"))]
    /// Client used to proxy requests for Griddle
    proxy_client: Option<Box<dyn ProxyClient>>,
    /// Type of backend DLT used by Grid
    dlt_backend: Option<DLTBackend>,
}

#[cfg(feature = "rest-api")]
/// An enumeration of the various REST API backend implementations.
pub enum GriddleRestApiVariant {
    #[cfg(feature = "rest-api-actix-web-4")]
    /// Actix Web 3 as the backend implementation
    ActixWeb4,
}

impl GriddleBuilder {
    #[cfg(feature = "rest-api")]
    pub fn with_signer(mut self, signer: Box<dyn Signer>) -> Self {
        self.signer = Some(signer);
        self
    }

    #[cfg(feature = "rest-api")]
    pub fn with_rest_api_variant(mut self, variant: GriddleRestApiVariant) -> Self {
        self.rest_api_variant = Some(variant);
        self
    }

    #[cfg(feature = "rest-api")]
    pub fn with_rest_api_endpoint(mut self, url: String) -> Self {
        self.rest_api_endpoint = Some(url);
        self
    }

    #[cfg(all(feature = "proxy", feature = "rest-api"))]
    pub fn with_proxy_client(mut self, proxy_client: Box<dyn ProxyClient>) -> Self {
        self.proxy_client = Some(proxy_client);
        self
    }

    pub fn with_dlt_backend(mut self, backend: DLTBackend) -> Self {
        self.dlt_backend = Some(backend);
        self
    }

    pub fn build(self) -> Result<RunnableGriddle, GriddleError> {
        #[cfg(feature = "rest-api")]
        let signer = self
            .signer
            .ok_or_else(|| GriddleError::MissingRequiredField("signer".to_string()))?;

        #[cfg(feature = "rest-api")]
        let rest_api_endpoint = self
            .rest_api_endpoint
            .ok_or_else(|| GriddleError::MissingRequiredField("rest_api_endpoint".to_string()))?;

        #[cfg(feature = "rest-api")]
        let builder_rest_api_variant = self
            .rest_api_variant
            .ok_or_else(|| GriddleError::MissingRequiredField("rest_api_variant".to_string()))?;

        let dlt_backend = self
            .dlt_backend
            .ok_or_else(|| GriddleError::MissingRequiredField("dlt_backend".to_string()))?;

        #[cfg(all(feature = "proxy", feature = "rest-api"))]
        let proxy_client = self
            .proxy_client
            .ok_or_else(|| GriddleError::MissingRequiredField("proxy_client".to_string()))?;

        #[cfg(feature = "rest-api")]
        let rest_api_variant = match builder_rest_api_variant {
            GriddleRestApiVariant::ActixWeb4 => {
                let mut builder = GriddleRestApiBuilder::new()
                    .with_bind(rest_api_endpoint.clone())
                    .with_signer(signer.clone())
                    .with_dlt_backend(dlt_backend.clone());

                #[cfg(feature = "proxy")]
                {
                    builder = builder.with_proxy_client(proxy_client.clone());
                }
                // Need to create the resources in this arm
                RunnableGriddleRestApiVariant::ActixWeb4(
                    builder
                        .build()
                        .map_err(|err| GriddleError::InvalidArgumentError(err.to_string()))?,
                )
            }
        };

        Ok(RunnableGriddle {
            #[cfg(feature = "rest-api")]
            rest_api: rest_api_variant,
            #[cfg(feature = "rest-api")]
            rest_api_endpoint,
            #[cfg(feature = "rest-api")]
            signer,
            #[cfg(all(feature = "proxy", feature = "rest-api"))]
            proxy_client,
            dlt_backend,
        })
    }
}
