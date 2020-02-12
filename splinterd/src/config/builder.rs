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

use std::path::Path;

use crate::config::error::ConfigError;
use crate::config::{Config, ConfigSource, PartialConfig};

pub trait PartialConfigBuilder {
    /// Takes all values set in a config object to create a PartialConfig object.
    ///
    fn build(self) -> PartialConfig;
}

fn get_file_path(cert_dir: &str, file: &str) -> String {
    let cert_dir_path = Path::new(&cert_dir);
    let cert_file_path = cert_dir_path.join(file);
    cert_file_path
        .to_str()
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| String::from(file))
}

/// ConfigBuilder collects PartialConfig objects from various sources to be used to generate a
/// Config object.
pub struct ConfigBuilder {
    partial_configs: Vec<PartialConfig>,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        ConfigBuilder {
            partial_configs: Vec::new(),
        }
    }

    #[cfg(feature = "default")]
    /// Adds a PartialConfig to the ConfigBuilder object.
    ///
    /// # Arguments
    ///
    /// * `partial` - A PartialConfig object generated from any of the config modules.
    ///
    pub fn with_partial_config(mut self, partial: PartialConfig) -> Self {
        self.partial_configs.push(partial);
        self
    }

    /// Builds a Config object by incorporating the values from each PartialConfig object.
    ///
    pub fn build(self) -> Result<Config, ConfigError> {
        let cert_dir = self
            .partial_configs
            .iter()
            .find_map(|p| match p.cert_dir() {
                Some(v) => Some((v, p.source())),
                None => None,
            })
            .ok_or_else(|| ConfigError::MissingValue("certificate directory".to_string()))?;
        let ca_certs = self
            .partial_configs
            .iter()
            .find_map(|p| match p.ca_certs() {
                Some(v) => {
                    if p.source() != ConfigSource::Default {
                        Some((v, p.source()))
                    } else {
                        Some((get_file_path(&cert_dir.0, &v), p.source()))
                    }
                }
                None => None,
            })
            .ok_or_else(|| ConfigError::MissingValue("ca certs".to_string()))?;
        let client_cert = self
            .partial_configs
            .iter()
            .find_map(|p| match p.client_cert() {
                Some(v) => {
                    if p.source() != ConfigSource::Default {
                        Some((v, p.source()))
                    } else {
                        Some((get_file_path(&cert_dir.0, &v), p.source()))
                    }
                }
                None => None,
            })
            .ok_or_else(|| ConfigError::MissingValue("client certificate".to_string()))?;
        let client_key = self
            .partial_configs
            .iter()
            .find_map(|p| match p.client_key() {
                Some(v) => {
                    if p.source() != ConfigSource::Default {
                        Some((v, p.source()))
                    } else {
                        Some((get_file_path(&cert_dir.0, &v), p.source()))
                    }
                }
                None => None,
            })
            .ok_or_else(|| ConfigError::MissingValue("client key".to_string()))?;
        let server_cert = self
            .partial_configs
            .iter()
            .find_map(|p| match p.server_cert() {
                Some(v) => {
                    if p.source() != ConfigSource::Default {
                        Some((v, p.source()))
                    } else {
                        Some((get_file_path(&cert_dir.0, &v), p.source()))
                    }
                }
                None => None,
            })
            .ok_or_else(|| ConfigError::MissingValue("server certificate".to_string()))?;
        let server_key = self
            .partial_configs
            .iter()
            .find_map(|p| match p.server_key() {
                Some(v) => {
                    if p.source() != ConfigSource::Default {
                        Some((v, p.source()))
                    } else {
                        Some((get_file_path(&cert_dir.0, &v), p.source()))
                    }
                }
                None => None,
            })
            .ok_or_else(|| ConfigError::MissingValue("server key".to_string()))?;
        // Iterates over the list of PartialConfig objects to find the first config with a value
        // for the specific field. If no value is found, an error is returned.
        Ok(Config {
            storage: self
                .partial_configs
                .iter()
                .find_map(|p| match p.storage() {
                    Some(v) => Some((v, p.source())),
                    None => None,
                })
                .ok_or_else(|| ConfigError::MissingValue("storage".to_string()))?,
            transport: self
                .partial_configs
                .iter()
                .find_map(|p| match p.transport() {
                    Some(v) => Some((v, p.source())),
                    None => None,
                })
                .ok_or_else(|| ConfigError::MissingValue("transport".to_string()))?,
            cert_dir,
            ca_certs,
            client_cert,
            client_key,
            server_cert,
            server_key,
            service_endpoint: self
                .partial_configs
                .iter()
                .find_map(|p| match p.service_endpoint() {
                    Some(v) => Some((v, p.source())),
                    None => None,
                })
                .ok_or_else(|| ConfigError::MissingValue("service endpoint".to_string()))?,
            network_endpoint: self
                .partial_configs
                .iter()
                .find_map(|p| match p.network_endpoint() {
                    Some(v) => Some((v, p.source())),
                    None => None,
                })
                .ok_or_else(|| ConfigError::MissingValue("network endpoint".to_string()))?,
            peers: self
                .partial_configs
                .iter()
                .find_map(|p| match p.peers() {
                    Some(v) => Some((v, p.source())),
                    None => None,
                })
                .ok_or_else(|| ConfigError::MissingValue("peers".to_string()))?,
            node_id: self
                .partial_configs
                .iter()
                .find_map(|p| match p.node_id() {
                    Some(v) => Some((v, p.source())),
                    None => None,
                })
                .ok_or_else(|| ConfigError::MissingValue("node id".to_string()))?,
            bind: self
                .partial_configs
                .iter()
                .find_map(|p| match p.bind() {
                    Some(v) => Some((v, p.source())),
                    None => None,
                })
                .ok_or_else(|| ConfigError::MissingValue("bind".to_string()))?,
            #[cfg(feature = "database")]
            database: self
                .partial_configs
                .iter()
                .find_map(|p| match p.database() {
                    Some(v) => Some((v, p.source())),
                    None => None,
                })
                .ok_or_else(|| ConfigError::MissingValue("database".to_string()))?,
            registry_backend: self
                .partial_configs
                .iter()
                .find_map(|p| match p.registry_backend() {
                    Some(v) => Some((v, p.source())),
                    None => None,
                })
                .ok_or_else(|| ConfigError::MissingValue("registry backend".to_string()))?,
            registry_file: self
                .partial_configs
                .iter()
                .find_map(|p| match p.registry_file() {
                    Some(v) => Some((v, p.source())),
                    None => None,
                })
                .ok_or_else(|| ConfigError::MissingValue("registry file".to_string()))?,
            heartbeat_interval: self
                .partial_configs
                .iter()
                .find_map(|p| match p.heartbeat_interval() {
                    Some(v) => Some((v, p.source())),
                    None => None,
                })
                .ok_or_else(|| ConfigError::MissingValue("heartbeat interval".to_string()))?,
            admin_service_coordinator_timeout: self
                .partial_configs
                .iter()
                .find_map(|p| match p.admin_service_coordinator_timeout() {
                    Some(v) => Some((v, p.source())),
                    None => None,
                })
                .ok_or_else(|| {
                    ConfigError::MissingValue("admin service coordinator timeout".to_string())
                })?,

            state_dir: self
                .partial_configs
                .iter()
                .find_map(|p| match p.state_dir() {
                    Some(v) => Some((v, p.source())),
                    None => None,
                })
                .ok_or_else(|| ConfigError::MissingValue("state directory".to_string()))?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    /// Asserts the example configuration values.
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

    #[test]
    /// This test verifies that a PartialConfig object is accurately constructed by chaining the
    /// PartialConfigBuilder methods. The following steps are performed:
    ///
    /// 1. An empty PartialConfig object is constructed.
    /// 2. The fields of the PartialConfig object are populated by chaining the builder methods.
    ///
    /// This test then verifies the PartialConfig object built from chaining the builder methods
    /// contains the correct values by asserting each expected value.
    fn test_builder_chain() {
        // Create an empty PartialConfig object.
        let mut partial_config = PartialConfig::new(ConfigSource::Default);
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
        let mut partial_config = PartialConfig::new(ConfigSource::Default);
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
}
