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

#[cfg(not(feature = "config-toml"))]
use std::fs::File;
#[cfg(not(feature = "config-toml"))]
use std::io::Read;

#[cfg(feature = "config-toml")]
use crate::config::PartialConfigBuilder;
use crate::config::{ConfigError, ConfigSource, PartialConfig};

#[cfg(feature = "config-toml")]
use serde_derive::Deserialize;

use toml;

#[cfg(feature = "config-toml")]
/// Holds configuration values defined in a toml file.
#[derive(Deserialize, Default, Debug)]
pub struct TomlConfig {
    source: Option<ConfigSource>,
    storage: Option<String>,
    transport: Option<String>,
    cert_dir: Option<String>,
    ca_certs: Option<String>,
    client_cert: Option<String>,
    client_key: Option<String>,
    server_cert: Option<String>,
    server_key: Option<String>,
    service_endpoint: Option<String>,
    network_endpoint: Option<String>,
    peers: Option<Vec<String>>,
    node_id: Option<String>,
    bind: Option<String>,
    #[cfg(feature = "database")]
    database: Option<String>,
    registry_backend: Option<String>,
    registry_file: Option<String>,
    heartbeat_interval: Option<u64>,
    admin_service_coordinator_timeout: Option<u64>,
}

#[cfg(feature = "config-toml")]
impl TomlConfig {
    pub fn new(toml: String, toml_path: String) -> Result<TomlConfig, ConfigError> {
        let mut toml_config = toml::from_str::<TomlConfig>(&toml).map_err(ConfigError::from)?;
        toml_config.source = Some(ConfigSource::Toml { file: toml_path });
        Ok(toml_config)
    }
}

#[cfg(feature = "config-toml")]
impl PartialConfigBuilder for TomlConfig {
    fn build(self) -> PartialConfig {
        let source = match self.source {
            Some(s) => s,
            None => ConfigSource::Toml {
                file: String::from(""),
            },
        };
        let partial_config = PartialConfig::new(source)
            .with_storage(self.storage)
            .with_transport(self.transport)
            .with_cert_dir(self.cert_dir)
            .with_ca_certs(self.ca_certs)
            .with_client_cert(self.client_cert)
            .with_client_key(self.client_key)
            .with_server_cert(self.server_cert)
            .with_server_key(self.server_key)
            .with_service_endpoint(self.service_endpoint)
            .with_network_endpoint(self.network_endpoint)
            .with_peers(self.peers)
            .with_node_id(self.node_id)
            .with_bind(self.bind)
            .with_registry_backend(self.registry_backend)
            .with_registry_file(self.registry_file)
            .with_heartbeat_interval(self.heartbeat_interval)
            .with_admin_service_coordinator_timeout(self.admin_service_coordinator_timeout);

        #[cfg(not(feature = "database"))]
        return partial_config;

        #[cfg(feature = "database")]
        return partial_config.with_database(self.database);
    }
}

/// Creates a new PartialConfig object from a toml file. Available to use when the `configtoml`
/// feature flag is not in use.
#[cfg(not(feature = "config-toml"))]
pub fn from_file(mut f: File) -> Result<PartialConfig, ConfigError> {
    let mut toml = String::new();
    f.read_to_string(&mut toml)?;

    let result = toml::from_str::<PartialConfig>(&toml)
        .map_err(ConfigError::from)?
        .with_source(ConfigSource::TomlFile { file: f });

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(feature = "config-toml"))]
    use std::fs;

    use toml::{map::Map, Value};

    /// Path to an example config toml file.
    static TEST_TOML: &str = "config_test.toml";

    /// Example configuration values.
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

    /// Converts a list of tuples to a toml Table Value used to write a toml file.
    fn get_toml_value() -> Value {
        let values = vec![
            ("storage".to_string(), EXAMPLE_STORAGE.to_string()),
            ("transport".to_string(), EXAMPLE_TRANSPORT.to_string()),
            ("ca_certs".to_string(), EXAMPLE_CA_CERTS.to_string()),
            ("client_cert".to_string(), EXAMPLE_CLIENT_CERT.to_string()),
            ("client_key".to_string(), EXAMPLE_CLIENT_KEY.to_string()),
            ("server_cert".to_string(), EXAMPLE_SERVER_CERT.to_string()),
            ("server_key".to_string(), EXAMPLE_SERVER_KEY.to_string()),
            (
                "service_endpoint".to_string(),
                EXAMPLE_SERVICE_ENDPOINT.to_string(),
            ),
            (
                "network_endpoint".to_string(),
                EXAMPLE_NETWORK_ENDPOINT.to_string(),
            ),
            ("node_id".to_string(), EXAMPLE_NODE_ID.to_string()),
        ];

        let mut config_values = Map::new();
        values.iter().for_each(|v| {
            config_values.insert(v.0.clone(), Value::String(v.1.clone()));
        });
        Value::Table(config_values)
    }

    #[cfg(not(feature = "config-toml"))]
    /// Creates the example toml file used to populate a PartialConfig object.
    fn create_toml_file() {
        let toml_string = toml::to_string(&get_toml_value()).expect("Could not encode TOML value");
        fs::write("config_test.toml", toml_string).expect("Could not write test toml file");
    }

    /// Asserts config values based on the example configuration values.
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
        assert_eq!(config.peers(), None);
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
    /// 1. An example config toml file, TEST_TOML, is created, and then opened.
    /// 2. A PartialConfig object is created by passing the opened file into the `from_file`
    ///    function defined in the PartialConfig module.
    ///
    /// This test then verifies the PartialConfig object built in step 2 contains the correct
    /// values by asserting each expected value. The TEST_TOML file is then removed.
    fn test_partial_config_from_file() {
        // Create example toml file.
        create_toml_file();
        // Opening the toml file using the TEST_TOML path
        let config_file =
            fs::File::open(TEST_TOML).expect(&format!("Unable to load {}", TEST_TOML));
        // Use the TomlConfig module's `from_file` method to construct a PartialConfig object
        // from the config file previously opened.
        let generated_config = from_file(config_file).unwrap();
        // Compare the generated PartialConfig object against the expected values.
        assert_config_values(generated_config);
        // Remove example file.
        fs::remove_file(TEST_TOML).expect("Unable to remove test toml file");
    }

    #[test]
    /// This test verifies that a PartialConfig object, constructed from the TomlConfig module,
    /// contains the correct values using the following steps:
    ///
    /// 1. An example config toml is string is created.
    /// 2. A TomlConfig object is constructed by passing in the toml string created in the previous
    ///    step.
    /// 3. The TomlConfig object is transformed to a PartialConfig object using the `build` method.
    ///
    /// This test then verifies the PartialConfig object built from the TomlConfig object by
    /// asserting each expected value.
    fn test_toml_build() {
        // Create an example toml string.
        let toml_string = toml::to_string(&get_toml_value()).expect("Could not encode TOML value");
        // Create a TomlConfig object from the toml string.
        let toml_builder = TomlConfig::new(toml_string, TEST_TOML.to_string())
            .expect(&format!("Unable to create TomlConfig from: {}", TEST_TOML));
        // Build a PartialConfig from the TomlConfig object created.
        let built_config = toml_builder.build();
        // Compare the generated PartialConfig object against the expected values.
        assert_config_values(built_config);
    }
}
