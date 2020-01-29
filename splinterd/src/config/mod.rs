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
mod toml;

use std::time::Duration;

#[cfg(feature = "config-command-line")]
pub use crate::config::command_line::CommandLineConfig;
#[cfg(feature = "config-default")]
pub use crate::config::default::DefaultConfig;
#[cfg(feature = "config-env-var")]
pub use crate::config::env::EnvVarConfig;
#[cfg(not(feature = "config-toml"))]
pub use crate::config::toml::from_file;
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
        debug!(
            "Config: cert_dir: {} (source: {:?})",
            self.cert_dir(),
            self.cert_dir_source()
        );
        debug!(
            "Config: ca_certs: {} (source: {:?})",
            self.ca_certs(),
            self.ca_certs_source()
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
    }
}
