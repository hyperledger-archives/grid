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

use crate::config::{ConfigSource, PartialConfig, PartialConfigBuilder};

const DEFAULT_CERT_DIR: &str = "/etc/splinter/certs/";
const DEFAULT_STATE_DIR: &str = "/var/lib/splinter/";

const CLIENT_CERT: &str = "client.crt";
const CLIENT_KEY: &str = "private/client.key";
const SERVER_CERT: &str = "server.crt";
const SERVER_KEY: &str = "private/server.key";
const CA_PEM: &str = "ca.pem";
const HEARTBEAT_DEFAULT: u64 = 30;
const DEFAULT_ADMIN_SERVICE_COORDINATOR_TIMEOUT_MILLIS: u64 = 30000;

/// Holds the default configuration values.
pub struct DefaultConfig {
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
    state_dir: Option<String>,
}

impl DefaultConfig {
    pub fn new() -> Self {
        DefaultConfig {
            storage: Some(String::from("yaml")),
            transport: Some(String::from("raw")),
            cert_dir: Some(String::from(DEFAULT_CERT_DIR)),
            ca_certs: Some(String::from(CA_PEM)),
            client_cert: Some(String::from(CLIENT_CERT)),
            client_key: Some(String::from(CLIENT_KEY)),
            server_cert: Some(String::from(SERVER_CERT)),
            server_key: Some(String::from(SERVER_KEY)),
            service_endpoint: Some(String::from("127.0.0.1:8043")),
            network_endpoint: Some(String::from("127.0.0.1:8044")),
            peers: Some(vec![]),
            node_id: None,
            bind: Some(String::from("127.0.0.1:8080")),
            #[cfg(feature = "database")]
            database: Some(String::from("127.0.0.1:5432")),
            registry_backend: Some(String::from("FILE")),
            registry_file: Some(String::from("/etc/splinter/nodes.yaml")),
            heartbeat_interval: Some(HEARTBEAT_DEFAULT),
            admin_service_coordinator_timeout: Some(
                DEFAULT_ADMIN_SERVICE_COORDINATOR_TIMEOUT_MILLIS,
            ),
            state_dir: Some(String::from(DEFAULT_STATE_DIR)),
        }
    }
}

impl PartialConfigBuilder for DefaultConfig {
    fn build(self) -> PartialConfig {
        let partial_config = PartialConfig::new(ConfigSource::Default)
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
            .with_admin_service_coordinator_timeout(self.admin_service_coordinator_timeout)
            .with_state_dir(self.state_dir);

        #[cfg(not(feature = "database"))]
        return partial_config;

        #[cfg(feature = "database")]
        return partial_config.with_database(self.database);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::time::Duration;

    /// Asserts config values based on the default values.
    fn assert_default_values(config: PartialConfig) {
        assert_eq!(config.storage(), Some(String::from("yaml")));
        assert_eq!(config.transport(), Some(String::from("raw")));
        assert_eq!(config.cert_dir(), Some(String::from(DEFAULT_CERT_DIR)));
        assert_eq!(config.ca_certs(), Some(String::from(CA_PEM)));
        assert_eq!(config.client_cert(), Some(String::from(CLIENT_CERT)));
        assert_eq!(config.client_key(), Some(String::from(CLIENT_KEY)));
        assert_eq!(config.server_cert(), Some(String::from(SERVER_CERT)));
        assert_eq!(config.server_key(), Some(String::from(SERVER_KEY)));
        assert_eq!(
            config.service_endpoint(),
            Some(String::from("127.0.0.1:8043"))
        );
        assert_eq!(
            config.network_endpoint(),
            Some(String::from("127.0.0.1:8044"))
        );
        assert_eq!(config.peers(), Some(vec![]));
        assert_eq!(config.node_id(), None);
        assert_eq!(config.bind(), Some(String::from("127.0.0.1:8080")));
        #[cfg(feature = "database")]
        assert_eq!(config.database(), Some(String::from("127.0.0.1:5432")));
        assert_eq!(config.registry_backend(), Some(String::from("FILE")));
        assert_eq!(
            config.registry_file(),
            Some(String::from("/etc/splinter/nodes.yaml"))
        );
        assert_eq!(config.heartbeat_interval(), Some(HEARTBEAT_DEFAULT));
        assert_eq!(
            config.admin_service_coordinator_timeout(),
            Some(Duration::from_millis(
                DEFAULT_ADMIN_SERVICE_COORDINATOR_TIMEOUT_MILLIS
            ))
        );
        assert_eq!(config.state_dir(), Some(String::from(DEFAULT_STATE_DIR)));
        // Assert the source is correctly identified for this PartialConfig object.
        assert_eq!(config.source(), ConfigSource::Default);
    }

    #[test]
    /// This test verifies that a PartialConfig object is accurately constructed by using the `build`
    /// method implemented by the DefaultConfig module. The following steps are performed:
    ///
    /// 1. An empty DefaultConfig object is constructed, which implements the PartialConfigBuilder
    ///    trait.
    /// 2. A PartialConfig object is created by calling the `build` method of the DefaultConfig object.
    ///
    /// This test then verifies the PartialConfig object built from the DefaulConfig object has
    /// the correct values by asserting each expected value.
    fn test_default_builder() {
        // Create a new DefaultConfig object, which implements the PartialConfigBuilder trait.
        let default_config = DefaultConfig::new();
        // Create a PartialConfig object using the `build` method.
        let partial_config = default_config.build();
        // Compare the generated PartialConfig object against the expected values.
        assert_default_values(partial_config);
    }
}
