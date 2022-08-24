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

//! An intermediate representation of the configuration values used by Griddle, taken from
//! different sources into a common representation.

use crate::config::Scope;

/// Displays the source of the configuration value
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GriddleConfigSource {
    Default,
    Environment,
    CommandLine,
}

pub struct PartialGriddleConfig {
    source: GriddleConfigSource,
    signing_key: Option<String>,
    rest_api_endpoint: Option<String>,
    #[cfg(feature = "proxy")]
    proxy_forward_url: Option<String>,
    scope: Option<Scope>,
    verbosity: Option<log::Level>,
}

impl PartialGriddleConfig {
    pub fn new(source: GriddleConfigSource) -> Self {
        PartialGriddleConfig {
            source,
            signing_key: None,
            rest_api_endpoint: None,
            #[cfg(feature = "proxy")]
            proxy_forward_url: None,
            scope: None,
            verbosity: None,
        }
    }

    pub fn source(&self) -> GriddleConfigSource {
        self.source.clone()
    }

    pub fn signing_key(&self) -> Option<String> {
        self.signing_key.clone()
    }

    pub fn rest_api_endpoint(&self) -> Option<String> {
        self.rest_api_endpoint.clone()
    }

    #[cfg(feature = "proxy")]
    pub fn proxy_forward_url(&self) -> Option<String> {
        self.proxy_forward_url.clone()
    }

    pub fn scope(&self) -> Option<Scope> {
        self.scope.clone()
    }

    pub fn verbosity(&self) -> Option<log::Level> {
        self.verbosity
    }

    /// Adds a `signing_key` value to the `PartialGriddleConfig` object.
    ///
    /// # Arguments
    ///
    /// * `signing_key` - Key pair used to sign transactions in Griddle
    ///
    pub fn with_signing_key(mut self, signing_key: Option<String>) -> Self {
        self.signing_key = signing_key;
        self
    }

    /// Adds a `rest_api_endpoint` value to the `PartialGriddleConfig` object.
    ///
    /// # Arguments
    ///
    /// * `rest_api_endpoint` - Endpoint that Griddle will bind to
    ///
    pub fn with_rest_api_endpoint(mut self, rest_api_endpoint: Option<String>) -> Self {
        self.rest_api_endpoint = rest_api_endpoint;
        self
    }

    #[cfg(feature = "proxy")]
    /// Adds a `proxy_forward_url` value to the `PartialGriddleConfig` object.
    ///
    /// # Arguments
    ///
    /// * `proxy_forward_url` - URL accessed by Griddle's proxy
    ///
    pub fn with_proxy_forward_url(mut self, proxy_forward_url: Option<String>) -> Self {
        self.proxy_forward_url = proxy_forward_url;
        self
    }

    /// Adds a `scope` value to the `PartialGriddleConfig` object.
    ///
    /// # Arguments
    ///
    /// * `scope` - Indicates the scope of state for incoming requests
    ///
    pub fn with_scope(mut self, scope: Option<Scope>) -> Self {
        self.scope = scope;
        self
    }

    /// Adds a `verbosity` value to the `PartialGriddleConfig` object.
    ///
    /// # Arguments
    ///
    /// * `verbosity` - Level of logging to be performed
    ///
    pub fn with_verbosity(mut self, verbosity: Option<log::Level>) -> Self {
        self.verbosity = verbosity;
        self
    }
}
