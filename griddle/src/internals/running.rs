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
use grid_sdk::{error::InternalError, threading::lifecycle::ShutdownHandle};

#[cfg(feature = "rest-api")]
use crate::internals::GriddleRestApiVariant;
use crate::internals::{DLTBackend, GriddleBuilder, GriddleError, RunnableGriddle};
#[cfg(feature = "rest-api-actix-web-4")]
use crate::rest_api::actix_web_4::GriddleRestApi;

#[cfg(feature = "rest-api")]
pub enum RunningGriddleRestApiVariant {
    #[cfg(feature = "rest-api-actix-web-4")]
    ActixWeb4(GriddleRestApi),
}

pub struct Griddle {
    #[cfg(feature = "rest-api")]
    pub(super) rest_api: RunningGriddleRestApiVariant,
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

impl Griddle {
    pub fn stop(mut self) -> Result<RunnableGriddle, GriddleError> {
        self.signal_shutdown();

        #[cfg(feature = "rest-api")]
        let rest_api_variant = match self.rest_api {
            #[cfg(feature = "rest-api-actix-web-4")]
            RunningGriddleRestApiVariant::ActixWeb4(api) => {
                api.wait_for_shutdown()
                    .map_err(|err| GriddleError::InternalError(err.to_string()))?;

                GriddleRestApiVariant::ActixWeb4
            }
        };

        let mut griddle_builder = GriddleBuilder::default().with_dlt_backend(self.dlt_backend);

        #[cfg(feature = "rest-api")]
        {
            griddle_builder = griddle_builder
                .with_signer(self.signer)
                .with_rest_api_variant(rest_api_variant)
                .with_rest_api_endpoint(self.rest_api_endpoint);
        }

        #[cfg(feature = "proxy")]
        {
            griddle_builder = griddle_builder.with_proxy_client(self.proxy_client);
        }

        griddle_builder.build()
    }
}

impl ShutdownHandle for Griddle {
    fn signal_shutdown(&mut self) {
        #[cfg(feature = "rest-api")]
        match self.rest_api {
            #[cfg(feature = "rest-api-actix-web-4")]
            RunningGriddleRestApiVariant::ActixWeb4(ref mut rest_api) => {
                rest_api.signal_shutdown();
            }
        }
    }

    fn wait_for_shutdown(self) -> Result<(), InternalError> {
        let mut errors = vec![];

        #[cfg(feature = "rest-api")]
        match self.rest_api {
            #[cfg(feature = "rest-api-actix-web-4")]
            RunningGriddleRestApiVariant::ActixWeb4(rest_api) => {
                if let Err(err) = rest_api.wait_for_shutdown() {
                    errors.push(err);
                }
            }
        }

        match errors.len() {
            0 => Ok(()),
            1 => Err(errors.remove(0)),
            _ => Err(InternalError::with_message(format!(
                "Multiple errors occurred during shutdown: {}",
                errors
                    .into_iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ))),
        }
    }
}
