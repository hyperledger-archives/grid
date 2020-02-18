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

mod builder;
#[cfg(feature = "config-command-line")]
mod command_line;
#[cfg(feature = "config-default")]
mod default;
#[cfg(feature = "config-env-var")]
mod env;
mod error;
mod partial;
#[cfg(feature = "config-toml")]
mod toml;

use std::time::Duration;

#[cfg(feature = "config-command-line")]
pub use crate::config::command_line::CommandLineConfig;
#[cfg(feature = "config-default")]
pub use crate::config::default::DefaultConfig;
#[cfg(feature = "config-env-var")]
pub use crate::config::env::EnvVarConfig;
#[cfg(feature = "config-toml")]
pub use crate::config::toml::TomlConfig;
pub use builder::{ConfigBuilder, PartialConfigBuilder};
pub use error::ConfigError;
pub use partial::{ConfigSource, PartialConfig};

/// Config is the final representation of configuration values. This final config object assembles
/// values from PartialConfig objects generated from various sources.
#[derive(Debug)]
pub struct Config {
    storage: (String, ConfigSource),
    transport: (String, ConfigSource),
    cert_dir: (String, ConfigSource),
    ca_certs: (String, ConfigSource),
    client_cert: (String, ConfigSource),
    client_key: (String, ConfigSource),
    server_cert: (String, ConfigSource),
    server_key: (String, ConfigSource),
    service_endpoint: (String, ConfigSource),
    network_endpoint: (String, ConfigSource),
    peers: (Vec<String>, ConfigSource),
    node_id: (String, ConfigSource),
    bind: (String, ConfigSource),
    #[cfg(feature = "database")]
    database: (String, ConfigSource),
    registry_backend: (String, ConfigSource),
    registry_file: (String, ConfigSource),
    heartbeat_interval: (u64, ConfigSource),
    admin_service_coordinator_timeout: (Duration, ConfigSource),
    state_dir: (String, ConfigSource),
    insecure: (bool, ConfigSource),
}

impl Config {
    pub fn storage(&self) -> &str {
        &self.storage.0
    }

    pub fn transport(&self) -> &str {
        &self.transport.0
    }

    pub fn cert_dir(&self) -> &str {
        &self.cert_dir.0
    }

    pub fn ca_certs(&self) -> &str {
        &self.ca_certs.0
    }

    pub fn client_cert(&self) -> &str {
        &self.client_cert.0
    }

    pub fn client_key(&self) -> &str {
        &self.client_key.0
    }

    pub fn server_cert(&self) -> &str {
        &self.server_cert.0
    }

    pub fn server_key(&self) -> &str {
        &self.server_key.0
    }

    pub fn service_endpoint(&self) -> &str {
        &self.service_endpoint.0
    }

    pub fn network_endpoint(&self) -> &str {
        &self.network_endpoint.0
    }

    pub fn peers(&self) -> &[String] {
        &self.peers.0
    }

    pub fn node_id(&self) -> &str {
        &self.node_id.0
    }

    pub fn bind(&self) -> &str {
        &self.bind.0
    }

    #[cfg(feature = "database")]
    pub fn database(&self) -> &str {
        &self.database.0
    }

    pub fn registry_backend(&self) -> &str {
        &self.registry_backend.0
    }

    pub fn registry_file(&self) -> &str {
        &self.registry_file.0
    }

    pub fn heartbeat_interval(&self) -> u64 {
        self.heartbeat_interval.0
    }

    pub fn admin_service_coordinator_timeout(&self) -> Duration {
        self.admin_service_coordinator_timeout.0
    }

    pub fn state_dir(&self) -> &str {
        &self.state_dir.0
    }

    pub fn insecure(&self) -> bool {
        self.insecure.0
    }

    fn storage_source(&self) -> &ConfigSource {
        &self.storage.1
    }

    fn transport_source(&self) -> &ConfigSource {
        &self.transport.1
    }

    fn cert_dir_source(&self) -> &ConfigSource {
        &self.cert_dir.1
    }

    fn ca_certs_source(&self) -> &ConfigSource {
        &self.ca_certs.1
    }

    fn client_cert_source(&self) -> &ConfigSource {
        &self.client_cert.1
    }

    fn client_key_source(&self) -> &ConfigSource {
        &self.client_key.1
    }

    fn server_cert_source(&self) -> &ConfigSource {
        &self.server_cert.1
    }

    fn server_key_source(&self) -> &ConfigSource {
        &self.server_key.1
    }

    fn service_endpoint_source(&self) -> &ConfigSource {
        &self.service_endpoint.1
    }

    fn network_endpoint_source(&self) -> &ConfigSource {
        &self.network_endpoint.1
    }

    fn peers_source(&self) -> &ConfigSource {
        &self.peers.1
    }

    fn node_id_source(&self) -> &ConfigSource {
        &self.node_id.1
    }

    fn bind_source(&self) -> &ConfigSource {
        &self.bind.1
    }

    #[cfg(feature = "database")]
    fn database_source(&self) -> &ConfigSource {
        &self.database.1
    }

    fn registry_backend_source(&self) -> &ConfigSource {
        &self.registry_backend.1
    }

    fn registry_file_source(&self) -> &ConfigSource {
        &self.registry_file.1
    }

    fn heartbeat_interval_source(&self) -> &ConfigSource {
        &self.heartbeat_interval.1
    }

    fn admin_service_coordinator_timeout_source(&self) -> &ConfigSource {
        &self.admin_service_coordinator_timeout.1
    }

    fn state_dir_source(&self) -> &ConfigSource {
        &self.state_dir.1
    }

    fn insecure_source(&self) -> &ConfigSource {
        &self.insecure.1
    }

    /// Displays the configuration value along with where the value was sourced from.
    pub fn log_as_debug(&self) {
        debug!(
            "Config: storage: {} (source: {:?})",
            self.storage(),
            self.storage_source()
        );
        debug!(
            "Config: transport: {} (source: {:?})",
            self.transport(),
            self.transport_source()
        );
        if self.transport() == "tls" {
            debug!(
                "Config: ca_certs: {} (source: {:?})",
                self.ca_certs(),
                self.ca_certs_source()
            );
            debug!(
                "Config: cert_dir: {} (source: {:?})",
                self.cert_dir(),
                self.cert_dir_source()
            );
            debug!(
                "Config: client_cert: {} (source: {:?})",
                self.client_cert(),
                self.client_cert_source()
            );
            debug!(
                "Config: client_key: {} (source: {:?})",
                self.client_key(),
                self.client_key_source()
            );
            debug!(
                "Config: server_cert: {} (source: {:?})",
                self.server_cert(),
                self.server_cert_source()
            );
            debug!(
                "Config: server_key: {} (source: {:?})",
                self.server_key(),
                self.server_key_source()
            );
        }
        debug!(
            "Config: service_endpoint: {} (source: {:?})",
            self.service_endpoint(),
            self.service_endpoint_source()
        );
        debug!(
            "Config: network_endpoint: {} (source: {:?})",
            self.network_endpoint(),
            self.network_endpoint_source()
        );
        debug!(
            "Config: peers: {:?} (source: {:?})",
            self.peers(),
            self.peers_source()
        );
        debug!(
            "Config: node_id: {} (source: {:?})",
            self.node_id(),
            self.node_id_source()
        );
        debug!(
            "Config: bind: {} (source: {:?})",
            self.bind(),
            self.bind_source()
        );
        debug!(
            "Config: registry_backend: {} (source: {:?})",
            self.registry_backend(),
            self.registry_backend_source()
        );
        debug!(
            "Config: registry_file: {} (source: {:?})",
            self.registry_file(),
            self.registry_file_source()
        );
        debug!(
            "Config: state_dir: {} (source: {:?})",
            self.state_dir(),
            self.state_dir_source()
        );
        debug!(
            "Config: heartbeat_interval: {} (source: {:?})",
            self.heartbeat_interval(),
            self.heartbeat_interval_source()
        );
        debug!(
            "Config: admin_service_coordinator_timeout: {:?} (source: {:?})",
            self.admin_service_coordinator_timeout(),
            self.admin_service_coordinator_timeout_source()
        );
        #[cfg(feature = "database")]
        debug!(
            "database: {} (source: {:?})",
            self.database(),
            self.database_source(),
        );
        debug!(
            "Config: insecure: {:?} (source: {:?})",
            self.insecure(),
            self.insecure_source()
        );
    }
}

#[cfg(feature = "default")]
#[cfg(test)]
mod tests {
    use super::*;

    use std::env;
    use std::time::Duration;

    use ::toml::{map::Map, to_string, Value};
    use clap::ArgMatches;

    use crate::config::{CommandLineConfig, DefaultConfig, EnvVarConfig, TomlConfig};

    /// Path to example config toml file.
    static TEST_TOML: &str = "config_test.toml";

    /// Values present in the example config TEST_TOML file.
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

    static DEFAULT_CLIENT_CERT: &str = "client.crt";
    static DEFAULT_CLIENT_KEY: &str = "private/client.key";
    static DEFAULT_SERVER_CERT: &str = "server.crt";
    static DEFAULT_SERVER_KEY: &str = "private/server.key";
    static DEFAULT_CA_CERT: &str = "ca.pem";

    /// Converts a list of tuples to a toml Table Value used to write a toml file.
    pub fn get_toml_value() -> Value {
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

    /// Creates an ArgMatches object to be used to construct a CommandLineConfig object.
    fn create_arg_matches(args: Vec<&str>) -> ArgMatches<'static> {
        clap_app!(configtest =>
        (version: crate_version!())
        (about: "Config-Test")
        (@arg config: -c --config +takes_value)
        (@arg node_id: --("node-id") +takes_value)
        (@arg storage: --("storage") +takes_value)
        (@arg transport: --("transport") +takes_value)
        (@arg network_endpoint: -n --("network-endpoint") +takes_value)
        (@arg service_endpoint: --("service-endpoint") +takes_value)
        (@arg peers: --peer +takes_value +multiple)
        (@arg ca_file: --("ca-file") +takes_value)
        (@arg cert_dir: --("cert-dir") +takes_value)
        (@arg client_cert: --("client-cert") +takes_value)
        (@arg server_cert: --("server-cert") +takes_value)
        (@arg server_key:  --("server-key") +takes_value)
        (@arg client_key:  --("client-key") +takes_value)
        (@arg bind: --("bind") +takes_value)
        (@arg registry_backend: --("registry-backend") +takes_value)
        (@arg registry_file: --("registry-file") +takes_value)
        (@arg insecure: --("insecure")))
        .get_matches_from(args)
    }

    #[test]
    /// This test verifies that a finalized Config object constructed from just a DefaultConfig
    /// object will be unsuccessful because of the missing values, in the following steps:
    ///
    /// 1. An empty ConfigBuilder object is created.
    /// 2. A PartialConfig built from a DefaultConfig is added to the ConfigBuilder.
    ///
    /// This test then verifies the final Config object built from the ConfigBuilder object has
    /// resulted in an error because of the missing values.
    fn test_default_final_config_err() {
        // Create a new ConfigBuilder object.
        let mut builder = ConfigBuilder::new();
        // Add a PartialConfig built from a DefaultConfig object to the ConfigBuilder.
        builder = builder.with_partial_config(DefaultConfig::new().build());
        // Build the final Config object.
        let final_config = builder.build();
        // Asserts the final Config was not successfully built.
        assert!(final_config.is_err());
    }

    #[test]
    /// This test verifies that a finalized Config object constructed from just a TomlConfig
    /// object will be unsuccessful because of the missing values, in the following steps:
    ///
    /// 1. An empty ConfigBuilder object is created.
    /// 2. The example config toml, TEST_TOML, is created, read and converted to a string.
    /// 3. A TomlConfig object is constructed by passing in the toml string created in the previous
    ///    step.
    /// 4. The TomlConfig object is added to the ConfigBuilder.
    ///
    /// This test then verifies the final Config object built from the ConfigBuilder object has
    /// resulted in an error because of the missing values.
    fn test_final_config_toml_err() {
        // Create a new ConfigBuilder object.
        let mut builder = ConfigBuilder::new();
        // Create an example toml string.
        let toml_string = to_string(&get_toml_value()).expect("Could not encode TOML value");
        // Create a TomlConfig object from the toml string.
        let toml_builder = TomlConfig::new(toml_string, TEST_TOML.to_string())
            .expect(&format!("Unable to create TomlConfig from: {}", TEST_TOML));
        // Add a PartialConfig built from a DefaultConfig object to the ConfigBuilder.
        builder = builder.with_partial_config(toml_builder.build());
        // Build the final Config object.
        let final_config = builder.build();
        // Asserts the final Config was not successfully built.
        assert!(final_config.is_err());
    }

    #[test]
    /// This test verifies that a Config object, constructed from just a CommandLineConfig object,
    /// is unsuccessful because of the missing values, in the following steps:
    ///
    /// 1. An empty ConfigBuilder object is created.
    /// 2. An example ArgMatches object is created using `create_arg_matches`.
    /// 3. A CommandLineConfig object is constructed by passing in the example ArgMatches created
    ///    in the previous step.
    /// 4. A PartialConfig built from the CommandLineConfig is added to the ConfigBuilder.
    ///
    /// This test then verifies the Config object built from the CommandLineConfig has resulted
    /// in an error because of the missing values.
    fn test_command_line_final_config_err() {
        // Create a new ConfigBuilder object.
        let mut builder = ConfigBuilder::new();
        let args = vec![
            "configtest",
            "--node-id",
            EXAMPLE_NODE_ID,
            "--storage",
            EXAMPLE_STORAGE,
            "--transport",
            EXAMPLE_TRANSPORT,
            "--network-endpoint",
            EXAMPLE_NETWORK_ENDPOINT,
            "--service-endpoint",
            EXAMPLE_SERVICE_ENDPOINT,
            "--ca-file",
            EXAMPLE_CA_CERTS,
            "--client-cert",
            EXAMPLE_CLIENT_CERT,
            "--client-key",
            EXAMPLE_CLIENT_KEY,
            "--server-cert",
            EXAMPLE_SERVER_CERT,
            "--server-key",
            EXAMPLE_SERVER_KEY,
            "--registry-backend",
            "FILE",
            "--registry-file",
            "/etc/splinter/test.yaml",
            "--insecure",
        ];
        // Create an example ArgMatches object to initialize the CommandLineConfig.
        let matches = create_arg_matches(args);
        // Create a new CommandLiine object from the arg matches.
        let command_config = CommandLineConfig::new(matches)
            .expect("Unable to create new CommandLineConfig object.");
        // Add a PartialConfig built from a DefaultConfig object to the ConfigBuilder.
        builder = builder.with_partial_config(command_config.build());
        let final_config = builder.build();
        // Assert the Config object was not successfully built.
        assert!(final_config.is_err());
    }

    #[test]
    /// This test verifies that a Config object, constructed from multiple config modules,
    /// contains the correct values, giving CommandLineConfig values ultimate precedence,
    /// using the following steps:
    ///
    /// 1. An empty ConfigBuilder object is created.
    /// 2. A PartialConfig is created from the EnvVarConfig module.
    /// 3. A PartialConfig is created from the DefaultConfig module.
    /// 4. A PartialConfig is created from the TomlConfig module, using the TEST_TOML string.
    /// 5. An example ArgMatches object is created using `create_arg_matches`.
    /// 6. A CommandLineConfig object is constructed by passing in the example ArgMatches created
    ///    in the previous step.
    /// 7. All PartialConfig objects are added to the ConfigBuilder and the final Config object is
    ///    built.
    ///
    /// This test then verifies the Config object built from the ConfigBuilder object by
    /// asserting each expected value.
    fn test_final_config_precedence() {
        // Set the environment variables to populate the EnvVarConfig object.
        env::set_var("SPLINTER_STATE_DIR", "/state/test/config/");
        env::set_var("SPLINTER_CERT_DIR", "/cert/test/config/");
        // Create a new ConfigBuilder object.
        let builder = ConfigBuilder::new();
        // Arguments to be used to create a CommandLineConfig object.
        let args = vec![
            "configtest",
            "--node-id",
            "123",
            "--registry-file",
            "/etc/splinter/test.yaml",
        ];
        // Create an example ArgMatches object to initialize the CommandLineConfig.
        let matches = create_arg_matches(args);
        // Create a new CommandLine object from the arg matches.
        let command_config = CommandLineConfig::new(matches)
            .expect("Unable to create new CommandLineConfig object.")
            .build();

        // Create an example toml string.
        let toml_string = to_string(&get_toml_value()).expect("Could not encode TOML value");
        // Create a TomlConfig object from the toml string.
        let toml_config = TomlConfig::new(toml_string, TEST_TOML.to_string())
            .expect(&format!("Unable to create TomlConfig from: {}", TEST_TOML))
            .build();

        // Create a PartialConfig from the EnvVarConfig module.
        let env_config = EnvVarConfig::new().build();

        // Create a PartialConfig from the DefaultConfig module.
        let default_config = DefaultConfig::new().build();

        // Add the PartialConfigs to the final ConfigBuilder in the order of precedence.
        let final_config = builder
            .with_partial_config(command_config)
            .with_partial_config(toml_config)
            .with_partial_config(env_config)
            .with_partial_config(default_config)
            .build()
            .expect("Unable to build final Config.");

        // Assert the final configuration values.
        // Both the DefaultConfig and TomlConfig had values for `storage`, but the TomlConfig
        // value should have precedence (source should be Toml).
        assert_eq!(
            (final_config.storage(), final_config.storage_source()),
            (
                EXAMPLE_STORAGE,
                &ConfigSource::Toml {
                    file: TEST_TOML.to_string()
                }
            )
        );
        // Both the DefaultConfig and TomlConfig had values for `transport`, but the TomlConfig
        // value should have precedence (source should be Toml).
        assert_eq!(
            (final_config.transport(), final_config.transport_source()),
            (
                EXAMPLE_TRANSPORT,
                &ConfigSource::Toml {
                    file: TEST_TOML.to_string()
                }
            )
        );
        // The DefaultConfig and EnvVarConfig had values for `cert_dir`, but the EnvVarConfig value
        // should have precedence (source should be Environment).
        assert_eq!(
            (final_config.cert_dir(), final_config.cert_dir_source()),
            ("/cert/test/config/", &ConfigSource::Environment)
        );
        // Both the DefaultConfig and TomlConfig had values for `ca_certs`, but the TomlConfig
        // value should have precedence (source should be Toml).
        assert_eq!(
            (final_config.ca_certs(), final_config.ca_certs_source()),
            (
                EXAMPLE_CA_CERTS,
                &ConfigSource::Toml {
                    file: TEST_TOML.to_string()
                },
            )
        );
        // Both the DefaultConfig and TomlConfig had values for `client_cert`, but the TomlConfig
        // value should have precedence (source should be Toml).
        assert_eq!(
            (
                final_config.client_cert(),
                final_config.client_cert_source()
            ),
            (
                EXAMPLE_CLIENT_CERT,
                &ConfigSource::Toml {
                    file: TEST_TOML.to_string()
                }
            )
        );
        // Both the DefaultConfig and TomlConfig had values for `client_key`, but the TomlConfig
        // value should have precedence (source should be Toml).
        assert_eq!(
            (final_config.client_key(), final_config.client_key_source()),
            (
                EXAMPLE_CLIENT_KEY,
                &ConfigSource::Toml {
                    file: TEST_TOML.to_string()
                },
            )
        );
        // Both the DefaultConfig and TomlConfig had values for `server_cert`, but the TomlConfig
        // value should have precedence (source should be Toml).
        assert_eq!(
            (
                final_config.server_cert(),
                final_config.server_cert_source()
            ),
            (
                EXAMPLE_SERVER_CERT,
                &ConfigSource::Toml {
                    file: TEST_TOML.to_string()
                }
            )
        );
        // Both the DefaultConfig and TomlConfig had values for `server_key`, but the TomlConfig
        // value should have precedence (source should be Toml).
        assert_eq!(
            (final_config.server_key(), final_config.server_key_source()),
            (
                EXAMPLE_SERVER_KEY,
                &ConfigSource::Toml {
                    file: TEST_TOML.to_string()
                }
            )
        );
        // Both the DefaultConfig and TomlConfig had values for `service_endpoint`, but the
        // TomlConfig value should have precedence (source should be Toml).
        assert_eq!(
            (
                final_config.service_endpoint(),
                final_config.service_endpoint_source()
            ),
            (
                EXAMPLE_SERVICE_ENDPOINT,
                &ConfigSource::Toml {
                    file: TEST_TOML.to_string()
                }
            )
        );
        // Both the DefaultConfig and TomlConfig had values for `network_endpoint`, but the
        // TomlConfig value should have precedence (source should be Toml).
        assert_eq!(
            (
                final_config.network_endpoint(),
                final_config.network_endpoint_source()
            ),
            (
                EXAMPLE_NETWORK_ENDPOINT,
                &ConfigSource::Toml {
                    file: TEST_TOML.to_string()
                }
            )
        );
        // The DefaultConfig is the only config with a value for `database` (source should be Default).
        assert_eq!(
            (final_config.peers(), final_config.peers_source()),
            (&[] as &[String], &ConfigSource::Default,)
        );
        // Both the TomlConfig and CommandLineConfig had values for `node_id`, but the
        // CommandLineConfig value should have precedence (source should be CommandLine).
        assert_eq!(
            (final_config.node_id(), final_config.node_id_source()),
            ("123", &ConfigSource::CommandLine)
        );
        // The DefaultConfig is the only config with a value for `bind` (source should be Default).
        assert_eq!(
            (final_config.bind(), final_config.bind_source()),
            ("127.0.0.1:8080", &ConfigSource::Default)
        );
        #[cfg(feature = "database")]
        // The DefaultConfig is the only config with a value for `database` (source should be Default).
        assert_eq!(
            (final_config.database(), final_config.database_source()),
            ("127.0.0.1:5432", &ConfigSource::Default)
        );
        // The DefaultConfig is the only config with a value for `registry_backend` (source should
        // be Default).
        assert_eq!(
            (
                final_config.registry_backend(),
                final_config.registry_backend_source()
            ),
            ("FILE", &ConfigSource::Default)
        );
        // Both the DefaultConfig and CommandLineConfig had values for `registry_file`, but the
        // CommandLineConfig value should have precedence (source should be CommandLine).
        assert_eq!(
            (
                final_config.registry_file(),
                final_config.registry_file_source()
            ),
            ("/etc/splinter/test.yaml", &ConfigSource::CommandLine,)
        );
        // The DefaultConfig is the only config with a value for `registry_backend` (source should
        // be Default).
        assert_eq!(
            (
                final_config.heartbeat_interval(),
                final_config.heartbeat_interval_source()
            ),
            (30, &ConfigSource::Default)
        );
        // The DefaultConfig is the only config with a value for `registry_backend` (source should
        // be Default).
        assert_eq!(
            (
                final_config.admin_service_coordinator_timeout(),
                final_config.admin_service_coordinator_timeout_source()
            ),
            (Duration::from_millis(30000), &ConfigSource::Default)
        );
        // Both the DefaultConfig and EnvVarConfig had values for `state_dir`, but the
        // EnvVarConfig value should have precedence (source should be EnvVarConfig).
        assert_eq!(
            (final_config.state_dir(), final_config.state_dir_source()),
            ("/state/test/config/", &ConfigSource::Environment)
        );
    }

    #[test]
    /// This test verifies that a Config object, created from a DefaultConfig and CommandLineConfig
    /// object holds the correct file paths, using the following steps:
    ///
    /// 1. An empty ConfigBuilder object is created.
    /// 2. A PartialConfig is created from the DefaultConfig module.
    /// 3. An example ArgMatches object is created using `create_arg_matches`.
    /// 4. A CommandLineConfig object is constructed by passing in the example ArgMatches created
    ///    in the previous step.
    /// 5. All PartialConfig objects are added to the ConfigBuilder and the final Config object is
    ///    built.
    ///
    /// This test then verifies the Config object built holds the correct file paths. The cert_dir
    /// value passed into the CommandLineConfig object should be appended to the default file names
    /// for the certificate files.
    fn test_final_config_file_paths() {
        // Create a new ConfigBuilder object.
        let builder = ConfigBuilder::new();
        // Arguments to be used to create a CommandLineConfig object, passing in a cert_dir.
        let args = vec![
            "configtest",
            "--node-id",
            "123",
            "--cert-dir",
            "/my_files/",
            "--registry-file",
            "/etc/splinter/test.yaml",
        ];
        // Create an example ArgMatches object to initialize the CommandLineConfig.
        let matches = create_arg_matches(args);
        // Create a new CommandLine object from the arg matches.
        let command_config = CommandLineConfig::new(matches)
            .expect("Unable to create new CommandLineConfig object.")
            .build();

        // Create a PartialConfig from the DefaultConfig module.
        let default_config = DefaultConfig::new().build();

        // Add the PartialConfigs to the final ConfigBuilder in the order of precedence.
        let final_config = builder
            .with_partial_config(command_config)
            .with_partial_config(default_config)
            .build()
            .expect("Unable to build final Config.");

        // The DefaultConfig and EnvVarConfig had values for `cert_dir`, but the EnvVarConfig value
        // should have precedence (source should be Environment).
        assert_eq!(
            (final_config.cert_dir(), final_config.cert_dir_source()),
            ("/my_files/", &ConfigSource::CommandLine)
        );
        // The DefaultConfig had a value for the ca_certs, and since the cert_dir value was provided
        // to the CommandLineConfig, the cert_dir value should be appended to the default file name.
        assert_eq!(
            (final_config.ca_certs(), final_config.ca_certs_source()),
            (
                format!("{}{}", "/my_files/", DEFAULT_CA_CERT).as_str(),
                &ConfigSource::Default,
            )
        );
        // The DefaultConfig had a value for the client_cert, and since the cert_dir value was provided
        // to the CommandLineConfig, the cert_dir value should be appended to the default file name.
        assert_eq!(
            (
                final_config.client_cert(),
                final_config.client_cert_source()
            ),
            (
                format!("{}{}", "/my_files/", DEFAULT_CLIENT_CERT).as_str(),
                &ConfigSource::Default,
            )
        );
        // The DefaultConfig had a value for the client_key, and since the cert_dir value was provided
        // to the CommandLineConfig, the cert_dir value should be appended to the default file name.
        assert_eq!(
            (final_config.client_key(), final_config.client_key_source()),
            (
                format!("{}{}", "/my_files/", DEFAULT_CLIENT_KEY).as_str(),
                &ConfigSource::Default,
            )
        );
        // The DefaultConfig had a value for the server_cert, and since the cert_dir value was provided
        // to the CommandLineConfig, the cert_dir value should be appended to the default file name.
        assert_eq!(
            (
                final_config.server_cert(),
                final_config.server_cert_source()
            ),
            (
                format!("{}{}", "/my_files/", DEFAULT_SERVER_CERT).as_str(),
                &ConfigSource::Default,
            )
        );
        // The DefaultConfig had a value for the server_key, and since the cert_dir value was provided
        // to the CommandLineConfig, the cert_dir value should be appended to the default file name.
        assert_eq!(
            (final_config.server_key(), final_config.server_key_source()),
            (
                format!("{}{}", "/my_files/", DEFAULT_SERVER_KEY).as_str(),
                &ConfigSource::Default,
            )
        );
    }
}
