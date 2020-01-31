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

#[cfg(feature = "config-builder")]
mod builder;
#[cfg(feature = "config-default")]
mod default;
mod error;
mod partial;
mod toml;

#[cfg(feature = "config-default")]
pub use crate::config::default::DefaultConfig;
#[cfg(not(feature = "config-toml"))]
pub use crate::config::toml::from_file;
#[cfg(feature = "config-toml")]
pub use crate::config::toml::TomlConfig;
#[cfg(feature = "config-builder")]
pub use builder::PartialConfigBuilder;
pub use error::ConfigError;
pub use partial::PartialConfig;

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;

    /// Path to existing example config toml files from the top-level Splinterd directory.
    static TEST_TOML: &str = "sample_configs/splinterd.toml.example";

    /// Values present in the existing example config TEST_TOML file.
    static EXAMPLE_STORAGE: &str = "yaml";
    static EXAMPLE_TRANSPORT: &str = "tls";
    static EXAMPLE_CA_CERTS: &str = "certs/ca.pem";
    static EXAMPLE_CLIENT_CERT: &str = "certs/client.crt";
    static EXAMPLE_CLIENT_KEY: &str = "certs/client.key";
    static EXAMPLE_SERVER_CERT: &str = "certs/server.crt";
    static EXAMPLE_SERVER_KEY: &str = "certs/server.key";
    static EXAMPLE_SERVICE_ENDPOINT: &str = "127.0.0.1:8043";
    static EXAMPLE_NETWORK_ENDPOINT: &str = "127.0.0.1:8044";
    static EXAMPLE_NODE_ID: &str = "012";

    /// Asserts config values based on the TEST_TOML file values.
    fn assert_config_values(config: PartialConfig) {
        assert_eq!(config.storage(), Some(EXAMPLE_STORAGE.to_string()));
        assert_eq!(config.transport(), Some(EXAMPLE_TRANSPORT.to_string()));
        assert_eq!(config.cert_dir(), None);
        assert_eq!(config.ca_certs(), Some(EXAMPLE_CA_CERTS.to_string()));
        assert_eq!(config.client_cert(), Some(EXAMPLE_CLIENT_CERT.to_string()));
        assert_eq!(config.client_key(), Some(EXAMPLE_CLIENT_KEY.to_string()));
        assert_eq!(config.server_cert(), Some(EXAMPLE_SERVER_CERT.to_string()));
        assert_eq!(config.server_key(), Some(EXAMPLE_SERVER_KEY.to_string()));
        assert_eq!(
            config.service_endpoint(),
            Some(EXAMPLE_SERVICE_ENDPOINT.to_string())
        );
        assert_eq!(
            config.network_endpoint(),
            Some(EXAMPLE_NETWORK_ENDPOINT.to_string())
        );
        assert_eq!(config.peers(), Some(vec![]));
        assert_eq!(config.node_id(), Some(EXAMPLE_NODE_ID.to_string()));
        assert_eq!(config.bind(), None);
        #[cfg(feature = "database")]
        assert_eq!(config.database(), None);
        assert_eq!(config.registry_backend(), None);
        assert_eq!(config.registry_file(), None);
        assert_eq!(config.heartbeat_interval(), None);
        assert_eq!(config.admin_service_coordinator_timeout(), None);
    }

    #[cfg(not(feature = "config-toml"))]
    #[test]
    /// This test verifies that a PartialConfig object, constructed from the TEST_TOML file using
    /// PartialConfig module's `from_file` method, contains the correct values using the following
    /// steps:
    ///
    /// 1. The example config toml file, TEST_TOML, is opened.
    /// 2. A PartialConfig object is created by passing the opened file into the `from_file`
    ///    function defined in the PartialConfig module.
    ///
    /// This test then verifies the PartialConfig object built in step 2 contains the correct
    /// values by asserting each expected value.
    fn test_partial_config_from_file() {
        // Opening the toml file using the TEST_TOML path
        let config_file =
            fs::File::open(TEST_TOML).expect(&format!("Unable to load {}", TEST_TOML));
        // Use the PartialConfig module's `from_file` method to construct a PartialConfig object
        // from the config file previously opened.
        let generated_config = from_file(config_file).unwrap();
        // Compare the generated PartialConfig object against the expected values.
        assert_config_values(generated_config);
    }

    #[cfg(feature = "config-builder")]
    #[test]
    /// This test verifies that a PartialConfig object is accurately constructed by chaining the
    /// PartialConfigBuilder methods from a new TomlConfig object. The following steps are performed:
    ///
    /// 1. An empty PartialConfig object is constructed.
    /// 2. The fields of the PartialConfig object are populated by chaining the builder methods.
    ///
    /// This test then verifies the PartialConfig object built from chaining the builder methods
    /// contains the correct values by asserting each expected value.
    fn test_builder_chain() {
        // Create an empty PartialConfig object.
        let mut partial_config = PartialConfig::default();
        // Populate the PartialConfig fields by chaining the builder methods.
        partial_config = partial_config
            .with_storage(Some(EXAMPLE_STORAGE.to_string()))
            .with_transport(Some(EXAMPLE_TRANSPORT.to_string()))
            .with_cert_dir(None)
            .with_ca_certs(Some(EXAMPLE_CA_CERTS.to_string()))
            .with_client_cert(Some(EXAMPLE_CLIENT_CERT.to_string()))
            .with_client_key(Some(EXAMPLE_CLIENT_KEY.to_string()))
            .with_server_cert(Some(EXAMPLE_SERVER_CERT.to_string()))
            .with_server_key(Some(EXAMPLE_SERVER_KEY.to_string()))
            .with_service_endpoint(Some(EXAMPLE_SERVICE_ENDPOINT.to_string()))
            .with_network_endpoint(Some(EXAMPLE_NETWORK_ENDPOINT.to_string()))
            .with_peers(Some(vec![]))
            .with_node_id(Some(EXAMPLE_NODE_ID.to_string()))
            .with_bind(None)
            .with_registry_backend(None)
            .with_registry_file(None)
            .with_heartbeat_interval(None)
            .with_admin_service_coordinator_timeout(None);
        // Compare the generated PartialConfig object against the expected values.
        assert_config_values(partial_config);
    }

    #[cfg(feature = "config-builder")]
    #[test]
    /// This test verifies that a PartialConfig object is accurately constructed by separately
    /// applying the builder methods. The following steps are performed:
    ///
    /// 1. An empty PartialConfig object is constructed.
    /// 2. The fields of the PartialConfig object are populated by separately applying the builder
    ///    methods.
    ///
    /// This test then verifies the PartialConfig object built from separately applying the builder
    /// methods contains the correct values by asserting each expected value.
    fn test_builder_separate() {
        // Create a new PartialConfig object.
        let mut partial_config = PartialConfig::default();
        // Populate the PartialConfig fields by separately applying the builder methods.
        partial_config = partial_config.with_storage(Some(EXAMPLE_STORAGE.to_string()));
        partial_config = partial_config.with_transport(Some(EXAMPLE_TRANSPORT.to_string()));
        partial_config = partial_config.with_ca_certs(Some(EXAMPLE_CA_CERTS.to_string()));
        partial_config = partial_config.with_client_cert(Some(EXAMPLE_CLIENT_CERT.to_string()));
        partial_config = partial_config.with_client_key(Some(EXAMPLE_CLIENT_KEY.to_string()));
        partial_config = partial_config.with_server_cert(Some(EXAMPLE_SERVER_CERT.to_string()));
        partial_config = partial_config.with_server_key(Some(EXAMPLE_SERVER_KEY.to_string()));
        partial_config =
            partial_config.with_service_endpoint(Some(EXAMPLE_SERVICE_ENDPOINT.to_string()));
        partial_config =
            partial_config.with_network_endpoint(Some(EXAMPLE_NETWORK_ENDPOINT.to_string()));
        partial_config = partial_config.with_peers(Some(vec![]));
        partial_config = partial_config.with_node_id(Some(EXAMPLE_NODE_ID.to_string()));
        partial_config = partial_config.with_admin_service_coordinator_timeout(None);

        // Compare the generated PartialConfig object against the expected values.
        assert_config_values(partial_config);
    }

    #[cfg(feature = "config-toml")]
    #[test]
    /// This test verifies that a PartialConfig object, constructed from the TomlConfig module,
    /// contains the correct values using the following steps:
    ///
    /// 1. The example config toml file, TEST_TOML, is read and converted to a string.
    /// 2. A TomlConfig object is constructed by passing in the toml string created in the previous
    ///    step.
    /// 3. The TomlConfig object is transformed to a PartialConfig object using the `build` method.
    ///
    /// This test then verifies the PartialConfig object built from the TomlConfig object by
    /// asserting each expected value.
    fn test_toml_build() {
        // Read the TEST_TOML example file to a string.
        let toml_string =
            fs::read_to_string(TEST_TOML).expect(&format!("Unable to load {}", TEST_TOML));
        // Create a TomlConfig object from the toml string.
        let toml_builder = TomlConfig::new(toml_string)
            .expect(&format!("Unable to create TomlConfig from: {}", TEST_TOML));
        // Build a PartialConfig from the TomlConfig object created.
        let built_config = toml_builder.build();
        // Compare the generated PartialConfig object against the expected values.
        assert_config_values(built_config);
    }
}
