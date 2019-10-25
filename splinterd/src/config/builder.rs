// Copyright 2018 Cargill Incorporated
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

use crate::config::Config;

pub struct ConfigBuilder {
    storage: Option<String>,
    transport: Option<String>,
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

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            storage: None,
            transport: None,
            ca_certs: None,
            client_cert: None,
            client_key: None,
            server_cert: None,
            server_key: None,
            service_endpoint: None,
            network_endpoint: None,
            peers: None,
            node_id: None,
            bind: None,
            registry_backend: None,
            registry_file: None,
            heartbeat_interval: None,
        }
    }

    pub fn with_storage(mut self, storage: String) -> Self {
        self.storage = Some(storage);
        self
    }

    pub fn with_transport(mut self, transport: String) -> Self {
        self.transport = Some(transport);
        self
    }

    pub fn with_ca_certs(mut self, ca_certs: String) -> Self {
        self.ca_certs = Some(ca_certs);
        self
    }

    pub fn with_client_cert(mut self, client_cert: String) -> Self {
        self.client_cert = Some(client_cert);
        self
    }

    pub fn with_client_key(mut self, client_key: String) -> Self {
        self.client_key = Some(client_key);
        self
    }

    pub fn with_server_cert(mut self, server_cert: String) -> Self {
        self.server_cert = Some(server_cert);
        self
    }

    pub fn with_server_key(mut self, server_key: String) -> Self {
        self.server_key = Some(server_key);
        self
    }

    pub fn with_service_endpoint(mut self, service_endpoint: String) -> Self {
        self.service_endpoint = Some(service_endpoint);
        self
    }

    pub fn with_network_endpoint(mut self, network_endpoint: String) -> Self {
        self.network_endpoint = Some(network_endpoint);
        self
    }

    pub fn with_peers(mut self, peers: Vec<String>) -> Self {
        self.peers = Some(peers);
        self
    }

    pub fn with_node_id(mut self, node_id: String) -> Self {
        self.node_id = Some(node_id);
        self
    }

    pub fn with_bind(mut self, bind: String) -> Self {
        self.bind = Some(bind);
        self
    }

    pub fn with_registry_backend(mut self, registry_backend: String) -> Self {
        self.registry_backend = Some(registry_backend);
        self
    }

    pub fn with_registry_file(mut self, registry_file: String) -> Self {
        self.registry_file = Some(registry_file);
        self
    }

    pub fn with_heartbeat_interval(mut self, heartbeat_interval: u64) -> Self {
        self.heartbeat_interval = Some(heartbeat_interval);
        self
    }

    pub fn build(self) -> Config {
        Config {
            storage: self.storage,
            transport: self.transport,
            ca_certs: self.ca_certs,
            client_cert: self.client_cert,
            client_key: self.client_key,
            server_cert: self.server_cert,
            server_key: self.server_key,
            service_endpoint: self.service_endpoint,
            network_endpoint: self.network_endpoint,
            peers: self.peers,
            node_id: self.node_id,
            bind: self.bind,
            registry_backend: self.registry_backend,
            registry_file: self.registry_file,
            heartbeat_interval: self.heartbeat_interval,
        }
    }
}
