// Copyright 2019 Cargill Incorporated
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

use crate::config::ConfigBuilder;
use crate::config::ConfigError;

use serde_derive::Deserialize;

use toml;

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
    registry_backend: Option<String>,
    registry_file: Option<String>,
    heartbeat_interval: Option<u64>,
}

impl TomlConfig {
    pub fn new(toml: String) -> Result<TomlConfig, ConfigError> {
        toml::from_str::<TomlConfig>(&toml).map_err(ConfigError::from)
    }

    pub fn take_storage(&mut self) -> Option<String> {
        self.storage.take()
    }

    pub fn take_transport(&mut self) -> Option<String> {
        self.transport.take()
    }

    pub fn take_cert_dir(&mut self) -> Option<String> {
        self.cert_dir.take()
    }

    pub fn take_ca_certs(&mut self) -> Option<String> {
        self.ca_certs.take()
    }

    pub fn take_client_cert(&mut self) -> Option<String> {
        self.client_cert.take()
    }

    pub fn take_client_key(&mut self) -> Option<String> {
        self.client_key.take()
    }

    pub fn take_server_cert(&mut self) -> Option<String> {
        self.server_cert.take()
    }

    pub fn take_server_key(&mut self) -> Option<String> {
        self.server_key.take()
    }

    pub fn take_service_endpoint(&mut self) -> Option<String> {
        self.service_endpoint.take()
    }

    pub fn take_network_endpoint(&mut self) -> Option<String> {
        self.network_endpoint.take()
    }

    pub fn take_peers(&mut self) -> Option<Vec<String>> {
        self.peers.take()
    }

    pub fn take_node_id(&mut self) -> Option<String> {
        self.node_id.take()
    }

    pub fn take_bind(&mut self) -> Option<String> {
        self.bind.take()
    }

    pub fn take_registry_backend(&mut self) -> Option<String> {
        self.registry_backend.take()
    }

    pub fn take_registry_file(&mut self) -> Option<String> {
        self.registry_file.take()
    }

    pub fn take_heartbeat_interval(&mut self) -> Option<u64> {
        self.heartbeat_interval.take()
    }

    pub fn apply_to_builder(mut self, mut builder: ConfigBuilder) -> ConfigBuilder {
        if let Some(x) = self.take_storage() {
            builder = builder.with_storage(x);
        }
        if let Some(x) = self.take_transport() {
            builder = builder.with_transport(x);
        }
        if let Some(x) = self.take_cert_dir() {
            builder = builder.with_cert_dir(x);
        }
        if let Some(x) = self.take_ca_certs() {
            builder = builder.with_ca_certs(x);
        }
        if let Some(x) = self.take_client_cert() {
            builder = builder.with_client_cert(x);
        }
        if let Some(x) = self.take_client_key() {
            builder = builder.with_client_key(x);
        }
        if let Some(x) = self.take_server_cert() {
            builder = builder.with_server_cert(x);
        }
        if let Some(x) = self.take_server_key() {
            builder = builder.with_server_key(x);
        }
        if let Some(x) = self.take_service_endpoint() {
            builder = builder.with_service_endpoint(x);
        }
        if let Some(x) = self.take_network_endpoint() {
            builder = builder.with_network_endpoint(x);
        }
        if let Some(x) = self.take_peers() {
            builder = builder.with_peers(x);
        }
        if let Some(x) = self.take_node_id() {
            builder = builder.with_node_id(x);
        }
        if let Some(x) = self.take_bind() {
            builder = builder.with_bind(x);
        }
        if let Some(x) = self.take_registry_backend() {
            builder = builder.with_registry_backend(x);
        }
        if let Some(x) = self.take_registry_file() {
            builder = builder.with_registry_file(x);
        }
        if let Some(x) = self.take_heartbeat_interval() {
            builder = builder.with_heartbeat_interval(x);
        }

        builder
    }
}
