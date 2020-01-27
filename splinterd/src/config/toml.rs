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
use crate::config::{ConfigError, PartialConfig};

#[cfg(feature = "config-toml")]
use serde_derive::Deserialize;

use toml;

#[cfg(feature = "config-toml")]
#[derive(Deserialize, Default, Debug)]
pub struct TomlConfig {
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
    pub fn new(toml: String) -> Result<TomlConfig, ConfigError> {
        toml::from_str::<TomlConfig>(&toml).map_err(ConfigError::from)
    }
}

#[cfg(feature = "config-toml")]
impl PartialConfigBuilder for TomlConfig {
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

    toml::from_str::<PartialConfig>(&toml).map_err(ConfigError::from)
}
