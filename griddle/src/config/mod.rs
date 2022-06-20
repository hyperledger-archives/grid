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

//! Configuration to provide the necessary values to start up the Griddle daemon.
//!
//! These values may be sourced from command line arguments, environment variables or pre-defined
//! defaults. This module allows for configuration values from each of these sources to be combined
//! into a final `GriddleConfig` object.

#[cfg(feature = "config-builder")]
mod builder;
pub mod error;
mod partial;

pub use partial::{GriddleConfigSource, PartialGriddleConfig};

#[derive(Clone, Debug, PartialEq)]
/// Placeholder for indicating the scope of the requests, will be used to determine if requests
/// to Griddle should include a scope ID and what format ID to expect
pub enum Scope {
    Global,
    Service,
}

#[derive(Debug)]
pub struct GriddleConfig {
    signing_key: (String, GriddleConfigSource),
    rest_api_endpoint: (String, GriddleConfigSource),
    #[cfg(feature = "proxy")]
    proxy_forward_url: (String, GriddleConfigSource),
    scope: (Scope, GriddleConfigSource),
    verbosity: (log::Level, GriddleConfigSource),
}

impl GriddleConfig {
    pub fn signing_key(&self) -> &str {
        &self.signing_key.0
    }

    pub fn signing_key_source(&self) -> &GriddleConfigSource {
        &self.signing_key.1
    }

    pub fn rest_api_endpoint(&self) -> &str {
        &self.rest_api_endpoint.0
    }

    pub fn rest_api_endpoint_source(&self) -> &GriddleConfigSource {
        &self.rest_api_endpoint.1
    }

    #[cfg(feature = "proxy")]
    pub fn proxy_forward_url(&self) -> &str {
        &self.proxy_forward_url.0
    }

    #[cfg(feature = "proxy")]
    pub fn proxy_forward_url_source(&self) -> &GriddleConfigSource {
        &self.proxy_forward_url.1
    }

    pub fn scope(&self) -> &Scope {
        &self.scope.0
    }

    pub fn scope_source(&self) -> &GriddleConfigSource {
        &self.scope.1
    }

    pub fn verbosity(&self) -> log::Level {
        self.verbosity.0
    }

    pub fn verbosity_source(&self) -> &GriddleConfigSource {
        &self.verbosity.1
    }

    pub fn log_as_debug(&self) {
        debug!(
            "Griddle Config: signing_key: {} (source: {:?})",
            self.signing_key(),
            self.signing_key_source(),
        );
        debug!(
            "Griddle Config: rest_api_endpoint: {} (source: {:?})",
            self.rest_api_endpoint(),
            self.rest_api_endpoint_source(),
        );
        debug!(
            "Griddle Config: scope: {:?} (source: {:?})",
            self.scope(),
            self.scope_source(),
        );
        debug!(
            "Griddle Config: verbosity: {} (source: {:?})",
            self.verbosity(),
            self.verbosity_source(),
        );
        #[cfg(feature = "proxy")]
        {
            debug!(
                "Griddle Config: proxy_forward_url: {} (source: {:?})",
                self.proxy_forward_url(),
                self.proxy_forward_url_source(),
            );
        }
    }
}
