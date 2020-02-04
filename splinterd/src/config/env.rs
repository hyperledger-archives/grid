// Copyright 2018-2020 Cargill Incorporated
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

use std::env;

use crate::config::{PartialConfig, PartialConfigBuilder};

const STATE_DIR_ENV: &str = "SPLINTER_STATE_DIR";
const CERT_DIR_ENV: &str = "SPLINTER_CERT_DIR";

/// Holds configuration values defined as environment variables.
pub struct EnvVarConfig {
    state_dir: Option<String>,
    cert_dir: Option<String>,
}

impl EnvVarConfig {
    #[allow(dead_code)]
    pub fn new() -> Self {
        EnvVarConfig {
            state_dir: env::var(STATE_DIR_ENV).ok(),
            cert_dir: env::var(CERT_DIR_ENV).ok(),
        }
    }
}

impl PartialConfigBuilder for EnvVarConfig {
    fn build(self) -> PartialConfig {
        PartialConfig::default()
            .with_cert_dir(self.cert_dir)
            .with_state_dir(self.state_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// This test verifies that a PartialConfig object, constructed from the EnvVarConfig module,
    /// contains the correct values when the environment variables are not set using the following
    /// steps:
    ///
    /// 1. Remove any existing environment variables which may be set.
    /// 2. A new EnvVarConfig object is created.
    /// 3. The EnvVarConfig object is transformed to a PartialConfig object using the `build`.
    ///
    /// This test then verifies the PartialConfig object built from the EnvVarConfig object by
    /// asserting each expected value. As the environment variables were unset, the configuration
    /// values should be set to None.
    fn test_environment_var_unset_config() {
        // Remove any existing environment variables.
        env::remove_var(STATE_DIR_ENV);
        env::remove_var(CERT_DIR_ENV);
        // Create a new EnvVarConfig object from the arg matches.
        let env_var_config = EnvVarConfig::new();
        // Build a PartialConfig from the EnvVarConfig object created.
        let built_config = env_var_config.build();
        // Compare the generated PartialConfig object against the expected values.
        assert_eq!(built_config.state_dir(), None);
        assert_eq!(built_config.cert_dir(), None);
    }

    #[test]
    /// This test verifies that a PartialConfig object, constructed from the EnvVarConfig module,
    /// contains the correct values using the following steps:
    ///
    /// 1. Set the environment variables for both the state and cert directories.
    /// 2. A new EnvVarConfig object is created.
    /// 3. The EnvVarConfig object is transformed to a PartialConfig object using the `build`.
    ///
    /// This test then verifies the PartialConfig object built from the EnvVarConfig object by
    /// asserting each expected value. As the environment variables were set, the configuration
    /// values should reflect those values.
    fn test_environment_var_set_config() {
        // Set the environment variables.
        env::set_var(STATE_DIR_ENV, "state/test/config");
        env::set_var(CERT_DIR_ENV, "cert/test/config");
        // Create a new EnvVarConfig object from the arg matches.
        let env_var_config = EnvVarConfig::new();
        // Build a PartialConfig from the EnvVarConfig object created.
        let built_config = env_var_config.build();
        // Compare the generated PartialConfig object against the expected values.
        assert_eq!(
            built_config.state_dir(),
            Some(String::from("state/test/config"))
        );
        assert_eq!(
            built_config.cert_dir(),
            Some(String::from("cert/test/config"))
        );
    }
}
