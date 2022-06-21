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

//! `PartialGriddleConfig` builder using values from the environment.

use std::env;

use crate::config::{
    error::GriddleConfigError, GriddleConfigSource, PartialGriddleConfig,
    PartialGriddleConfigBuilder,
};

const SIGNING_KEY_ENV: &str = "GRIDDLE_KEY";
const REST_API_ENDPOINT_ENV: &str = "GRIDDLE_BIND";
#[cfg(feature = "proxy")]
const PROXY_FORWARD_URL_ENV: &str = "GRIDDLE_FORWARD_URL";

/// Trait to outline a basic read-only environment variable store
pub trait EnvStore {
    /// Returns an environment variable for a given key
    ///
    /// # Arguments
    ///
    /// * `key` - A string slice of the name of the environment variable
    fn get(&self, key: &str) -> Option<String>;
}

/// Implementation of `GriddleEnvStore` for OS environment variables
pub struct GriddleOsEnvStore;

impl EnvStore for GriddleOsEnvStore {
    fn get(&self, key: &str) -> Option<String> {
        env::var(key).ok()
    }
}

pub struct EnvPartialGriddleConfigBuilder<S: EnvStore> {
    store: S,
}

/// Implementation of the `PartialGriddleConfigBuilder` trait to create a `PartialGriddleConfig`
/// from environment variables.
impl EnvPartialGriddleConfigBuilder<GriddleOsEnvStore> {
    pub fn new() -> Self {
        EnvPartialGriddleConfigBuilder {
            store: GriddleOsEnvStore {},
        }
    }
}

impl<S: EnvStore> EnvPartialGriddleConfigBuilder<S> {
    /// Returns an `EnvPartialGriddleConfigBuilder` that will fetch data from the given store.
    ///
    /// # Arguments
    ///
    /// * `store` - An instance of `EnvStore`
    ///
    #[cfg(test)]
    pub fn from_store(store: S) -> Self {
        EnvPartialGriddleConfigBuilder { store }
    }
}

impl<S: EnvStore> PartialGriddleConfigBuilder for EnvPartialGriddleConfigBuilder<S> {
    fn build(self) -> Result<PartialGriddleConfig, GriddleConfigError> {
        let mut config = PartialGriddleConfig::new(GriddleConfigSource::Environment);

        config = config
            .with_signing_key(self.store.get(SIGNING_KEY_ENV))
            .with_rest_api_endpoint(self.store.get(REST_API_ENDPOINT_ENV));

        #[cfg(feature = "proxy")]
        {
            config = config.with_proxy_forward_url(self.store.get(PROXY_FORWARD_URL_ENV));
        }

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Implementation of `EnvStore` that supports arbitrary hashmaps
    pub(crate) struct GriddleHashmapEnvStore {
        internal: HashMap<String, String>,
    }

    impl GriddleHashmapEnvStore {
        /// Returns an `GriddleHashmapEnvStore` that will fetch an environment variable using the
        /// given hashmap.
        ///
        /// # Arguments
        ///
        /// * `internal` - The internal map to obtain values from
        ///
        pub fn new(internal: HashMap<String, String>) -> GriddleHashmapEnvStore {
            GriddleHashmapEnvStore { internal }
        }
    }

    impl EnvStore for GriddleHashmapEnvStore {
        fn get(&self, key: &str) -> Option<String> {
            self.internal.get(key).map(ToOwned::to_owned)
        }
    }

    #[test]
    /// This test verifies that a `PartialGriddleConfig` object, constructed from the
    /// `EnvPartialGriddleConfigBuilder` module, contains the correct values. The test steps are
    /// described below:
    ///
    /// 1. Create a set of mock environment variables, with unset values
    /// 2. Create a new `EnvPartialGriddleConfigBuilder` object using the empty mock environment
    /// 3. The `EnvPartialGriddleConfigBuilder` object is transformed to a `PartialGriddleConfig`
    ///    object using `build`
    /// 4. Validate the values of the partial config are not set
    ///
    /// This test verifies a `PartialGriddleConfig` object built from unset environment
    /// variables, using a `EnvPartialGriddleConfigBuilder`, will result in a `PartialGriddleConfig`
    /// object with unset values.
    fn test_empty_env_config() {
        // Create a new EnvPartialGriddleConfigBuilder object.
        let store = GriddleHashmapEnvStore::new(HashMap::new());
        let env_config_builder = EnvPartialGriddleConfigBuilder::from_store(store);

        // Build a `PartialGriddleConfig` from the `EnvPartialGriddleConfigBuilder` object created.
        let empty_generated_config = env_config_builder
            .build()
            .expect("Unable to build `EnvPartialGriddleConfigBuilder`");
        assert_eq!(
            empty_generated_config.source(),
            GriddleConfigSource::Environment
        );
        // Assert the config object's values are not set.
        assert_eq!(empty_generated_config.signing_key(), None);
        assert_eq!(empty_generated_config.rest_api_endpoint(), None);
    }

    #[test]
    /// This test verifies that a `PartialGriddleConfig` object, constructed from the
    /// `EnvPartialGriddleConfigBuilder` module, contains the correct values. The test steps are
    /// described below:
    ///
    /// 1. Create a set of mock environment variables, setting the values
    /// 2. Create a new `EnvPartialGriddleConfigBuilder` object using the mock environment vars
    /// 3. The `EnvPartialGriddleConfigBuilder` object is transformed to a `PartialGriddleConfig`
    ///    object using `build`
    /// 4. Validate the values of the partial config are set as expected
    ///
    /// This test verifies a `PartialGriddleConfig` object built from a
    /// `EnvPartialGriddleConfigBuilder` results in the expected values.
    fn test_setup_env_config() {
        // Create a new EnvPartialGriddleConfigBuilder object.
        let mut store_internals: HashMap<String, String> = HashMap::new();
        store_internals.insert(SIGNING_KEY_ENV.to_string(), "test-key".to_string());
        store_internals.insert(REST_API_ENDPOINT_ENV.to_string(), "test-url".to_string());
        let store = GriddleHashmapEnvStore::new(store_internals);
        let env_config_builder = EnvPartialGriddleConfigBuilder::from_store(store);
        // Build a `PartialGriddleConfig` from the `EnvPartialGriddleConfigBuilder` object created.
        let generated_config = env_config_builder
            .build()
            .expect("Unable to build `EnvPartialGriddleConfigBuilder`");
        assert_eq!(generated_config.source(), GriddleConfigSource::Environment);
        // Compare the generated `PartialGriddleConfig` object against the expected values.
        assert_eq!(
            generated_config.signing_key(),
            Some(String::from("test-key"))
        );
        assert_eq!(
            generated_config.rest_api_endpoint(),
            Some(String::from("test-url"))
        );
    }
}
