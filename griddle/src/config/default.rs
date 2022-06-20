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

//! `PartialGriddleConfig` builder using default values.

use crate::config::{
    error::GriddleConfigError, GriddleConfigSource, PartialGriddleConfig,
    PartialGriddleConfigBuilder,
};

const REST_API_ENDPOINT: &str = "localhost:8000";
#[cfg(feature = "proxy")]
const PROXY_FORWARD_URL: &str = "http://localhost:8080";

#[derive(Default)]
pub struct DefaultPartialGriddleConfigBuilder;

impl DefaultPartialGriddleConfigBuilder {
    pub fn new() -> Self {
        DefaultPartialGriddleConfigBuilder {}
    }
}

impl PartialGriddleConfigBuilder for DefaultPartialGriddleConfigBuilder {
    fn build(self) -> Result<PartialGriddleConfig, GriddleConfigError> {
        let mut partial_config = PartialGriddleConfig::new(GriddleConfigSource::Default);

        partial_config = partial_config
            .with_rest_api_endpoint(Some(String::from(REST_API_ENDPOINT)))
            .with_verbosity(Some(log::Level::Info));

        // If the current username is set, use this as the default signing key value
        if let Some(signing_key) =
            users::get_current_username().and_then(|os_str| os_str.into_string().ok())
        {
            partial_config = partial_config.with_signing_key(Some(signing_key));
        }

        #[cfg(feature = "proxy")]
        {
            partial_config =
                partial_config.with_proxy_forward_url(Some(String::from(PROXY_FORWARD_URL)));
        }

        Ok(partial_config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Verifies that a `PartialGriddleConfig` object is accurately constructed from default values.
    /// The test follows these steps:
    ///
    /// 1. Create a new `DefaultPartialGriddleConfigBuilder`
    /// 2. Call `build` on the new default config builder
    /// 3. Validate the default configuration values present in the `PartialGriddleConfig`
    ///
    fn test_default_config() {
        // Create a new `DefaultPartialGriddleConfigBuilder`
        let default_config_builder = DefaultPartialGriddleConfigBuilder::new();
        // Create a `PartialGriddleConfig` by calling `build`
        let generated_default_config = default_config_builder
            .build()
            .expect("Unable to build default config object");
        // Assert the default config values
        assert_eq!(
            generated_default_config.source(),
            GriddleConfigSource::Default
        );
        assert_eq!(
            generated_default_config.rest_api_endpoint(),
            Some(String::from(REST_API_ENDPOINT))
        );
        #[cfg(feature = "proxy")]
        assert_eq!(
            generated_default_config.proxy_forward_url(),
            Some(String::from(PROXY_FORWARD_URL))
        );
    }
}
