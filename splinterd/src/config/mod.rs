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
mod error;
mod partial;
mod toml;

#[cfg(not(feature = "config-toml"))]
pub use crate::config::toml::from_file;
#[cfg(feature = "config-toml")]
pub use crate::config::toml::TomlConfig;
#[cfg(feature = "config-builder")]
pub use builder::{ConfigBuilder, PartialConfigBuilder};
pub use error::ConfigError;
pub use partial::PartialConfig;

use std::time::Duration;

#[derive(Deserialize, Default, Debug)]
pub struct Config {
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
    admin_service_coordinator_timeout: Option<Duration>,
}

impl Config {
    pub fn storage(&self) -> Option<String> {
        self.storage.clone()
    }

    pub fn transport(&self) -> Option<String> {
        self.transport.clone()
    }

    pub fn cert_dir(&self) -> Option<String> {
        self.cert_dir.clone()
    }

    pub fn ca_certs(&self) -> Option<String> {
        self.ca_certs.clone()
    }

    pub fn client_cert(&self) -> Option<String> {
        self.client_cert.clone()
    }

    pub fn client_key(&self) -> Option<String> {
        self.client_key.clone()
    }

    pub fn server_cert(&self) -> Option<String> {
        self.server_cert.clone()
    }

    pub fn server_key(&self) -> Option<String> {
        self.server_key.clone()
    }

    pub fn service_endpoint(&self) -> Option<String> {
        self.service_endpoint.clone()
    }

    pub fn network_endpoint(&self) -> Option<String> {
        self.network_endpoint.clone()
    }

    pub fn peers(&self) -> Option<Vec<String>> {
        self.peers.clone()
    }

    pub fn node_id(&self) -> Option<String> {
        self.node_id.clone()
    }

    pub fn bind(&self) -> Option<String> {
        self.bind.clone()
    }

    #[cfg(feature = "database")]
    pub fn database(&self) -> Option<String> {
        self.database.clone()
    }

    pub fn registry_backend(&self) -> Option<String> {
        self.registry_backend.clone()
    }

    pub fn registry_file(&self) -> Option<String> {
        self.registry_file.clone()
    }

    pub fn heartbeat_interval(&self) -> Option<u64> {
        self.heartbeat_interval
    }

    pub fn admin_service_coordinator_timeout(&self) -> Option<Duration> {
        self.admin_service_coordinator_timeout
    }
}
