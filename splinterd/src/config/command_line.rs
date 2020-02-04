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

use crate::config::{ConfigError, PartialConfig, PartialConfigBuilder};
use clap::{ArgMatches, ErrorKind};

/// Holds configuration values from command line arguments, represented by clap ArgMatches.
pub struct CommandLineConfig {
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
}

fn parse_value(matches: &ArgMatches) -> Result<Option<u64>, ConfigError> {
    match value_t!(matches.value_of("heartbeat_interval"), u64) {
        Ok(v) => Ok(Some(v)),
        Err(e) => match e.kind {
            ErrorKind::ValueValidation => Err(ConfigError::InvalidArgument(e)),
            _ => Ok(None),
        },
    }
}

impl CommandLineConfig {
    #[allow(dead_code)]
    pub fn new(matches: ArgMatches) -> Result<Self, ConfigError> {
        Ok(CommandLineConfig {
            storage: matches.value_of("storage").map(String::from),
            transport: matches.value_of("transport").map(String::from),
            cert_dir: matches.value_of("cert_dir").map(String::from),
            ca_certs: matches.value_of("ca_file").map(String::from),
            client_cert: matches.value_of("client_cert").map(String::from),
            client_key: matches.value_of("client_key").map(String::from),
            server_cert: matches.value_of("server_cert").map(String::from),
            server_key: matches.value_of("server_key").map(String::from),
            service_endpoint: matches.value_of("service_endpoint").map(String::from),
            network_endpoint: matches.value_of("network_endpoint").map(String::from),
            peers: matches
                .values_of("peers")
                .map(|values| values.map(String::from).collect::<Vec<String>>()),
            node_id: matches.value_of("node_id").map(String::from),
            bind: matches.value_of("bind").map(String::from),
            #[cfg(feature = "database")]
            database: matches.value_of("database").map(String::from),
            registry_backend: matches.value_of("registry_backend").map(String::from),
            registry_file: matches.value_of("registry_file").map(String::from),
            heartbeat_interval: parse_value(&matches)?,
        })
    }
}

impl PartialConfigBuilder for CommandLineConfig {
    fn build(self) -> PartialConfig {
        let partial_config = PartialConfig::default()
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
            .with_heartbeat_interval(self.heartbeat_interval);

        #[cfg(not(feature = "database"))]
        return partial_config;

        #[cfg(feature = "database")]
        return partial_config.with_database(self.database);
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    use clap::ArgMatches;

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

    /// Asserts config values based on the example values.
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
            (@arg registry_file: --("registry-file") +takes_value))
        .get_matches_from(args)
    }

    #[test]
    /// This test verifies that a PartialConfig object, constructed from the CommandLineConfig module,
    /// contains the correct values using the following steps:
    ///
    /// 1. An example ArgMatches object is created using `create_arg_matches`.
    /// 2. A CommandLineConfig object is constructed by passing in the example ArgMatches created
    ///    in the previous step.
    /// 3. The CommandLineConfig object is transformed to a PartialConfig object using the `build`.
    ///
    /// This test then verifies the PartialConfig object built from the CommandLineConfig object by
    /// asserting each expected value.
    fn test_command_line_config() {
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
        ];
        // Create an example ArgMatches object to initialize the CommandLineConfig.
        let matches = create_arg_matches(args);
        // Create a new CommandLiine object from the arg matches.
        let command_config = CommandLineConfig::new(matches)
            .expect("Unable to create new CommandLineConfig object.");
        // Build a PartialConfig from the TomlConfig object created.
        let built_config = command_config.build();
        // Compare the generated PartialConfig object against the expected values.
        assert_config_values(built_config);
    }
}
