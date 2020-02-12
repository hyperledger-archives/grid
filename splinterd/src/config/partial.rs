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

use std::time::Duration;

/// ConfigSource displays the source of configuration values, used to identify which of the various
/// config modules were used to create a particular PartialConfig object.
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub enum ConfigSource {
    Toml { file: String },
    Default,
    Environment,
    CommandLine,
}

/// PartialConfig is an intermediate representation of configuration values, used when combining
/// several sources. As such, all values of the PartialConfig are options as it is not necessary
/// to provide all values from a single source.
#[derive(Deserialize, Debug)]
pub struct PartialConfig {
    source: ConfigSource,
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
    state_dir: Option<String>,
}

impl PartialConfig {
    pub fn new(source: ConfigSource) -> Self {
        PartialConfig {
            source,
            storage: None,
            transport: None,
            cert_dir: None,
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
            #[cfg(feature = "database")]
            database: None,
            registry_backend: None,
            registry_file: None,
            heartbeat_interval: None,
            admin_service_coordinator_timeout: None,
            state_dir: None,
        }
    }

    pub fn source(&self) -> ConfigSource {
        self.source.clone()
    }

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

    pub fn state_dir(&self) -> Option<String> {
        self.state_dir.clone()
    }

    /// Adds a `storage` value to the PartialConfig object.
    ///
    /// # Arguments
    ///
    /// * `storage` - The type of storage that should be used to store circuit state.
    ///
    pub fn with_storage(mut self, storage: Option<String>) -> Self {
        self.storage = storage;
        self
    }

    /// Adds a `transport` value to the PartialConfig object.
    ///
    /// # Arguments
    ///
    /// * `transport` -  Which transport type this splinterd node supports.
    ///
    pub fn with_transport(mut self, transport: Option<String>) -> Self {
        self.transport = transport;
        self
    }

    /// Adds a `cert_dir` value to the PartialConfig object.
    ///
    /// # Arguments
    ///
    /// * `cert_dir` - Directory containing any certificates and keys to be used.
    ///
    pub fn with_cert_dir(mut self, cert_dir: Option<String>) -> Self {
        self.cert_dir = cert_dir;
        self
    }

    /// Adds a `ca_certs` value to the  PartialConfig object.
    ///
    /// # Arguments
    ///
    /// * `ca_certs` - List of certificate authority certificates (*.pem files).
    ///
    pub fn with_ca_certs(mut self, ca_certs: Option<String>) -> Self {
        self.ca_certs = ca_certs;
        self
    }

    /// Adds a `client_cert` value to the PartialConfig object.
    ///
    /// # Arguments
    ///
    /// * `client_cert` - A certificate signed by a certificate authority. Used by the daemon when
    ///                   it is acting as a client, sending messages.
    ///
    pub fn with_client_cert(mut self, client_cert: Option<String>) -> Self {
        self.client_cert = client_cert;
        self
    }

    /// Adds a `client_key` value to the PartialConfig object.
    ///
    /// # Arguments
    ///
    /// * `client_key` - Private key used by daemon when it is acting as a client.
    ///
    pub fn with_client_key(mut self, client_key: Option<String>) -> Self {
        self.client_key = client_key;
        self
    }

    /// Adds a `server_cert` value to the PartialConfig object.
    ///
    /// # Arguments
    ///
    /// * `server_cert` - A certificate signed by a certificate authority. Used by the daemon when
    ///                   it is acting as a server, receiving messages.
    ///
    pub fn with_server_cert(mut self, server_cert: Option<String>) -> Self {
        self.server_cert = server_cert;
        self
    }

    /// Adds a `server_key` value to the PartialConfig object.
    ///
    /// # Arguments
    ///
    /// * `server_key` - Private key used by daemon when it is acting as a server.
    ///
    pub fn with_server_key(mut self, server_key: Option<String>) -> Self {
        self.server_key = server_key;
        self
    }

    /// Adds a `service_endpoint` value to the PartialConfig object.
    ///
    /// # Arguments
    ///
    /// * `service_endpoint` - Endpoint used for service to daemon communication.
    ///
    pub fn with_service_endpoint(mut self, service_endpoint: Option<String>) -> Self {
        self.service_endpoint = service_endpoint;
        self
    }

    /// Adds a `network_endpoint` value to the PartialConfig object.
    ///
    /// # Arguments
    ///
    /// * `network_endpoint` - Endpoint used for daemon to daemon communication.
    ///
    pub fn with_network_endpoint(mut self, network_endpoint: Option<String>) -> Self {
        self.network_endpoint = network_endpoint;
        self
    }

    /// Adds a `peers` value to the PartialConfig object.
    ///
    /// # Arguments
    ///
    /// * `peers` - A list of splinter nodes the daemon will automatically connect to on start up.
    ///
    pub fn with_peers(mut self, peers: Option<Vec<String>>) -> Self {
        self.peers = peers;
        self
    }

    /// Adds a `node_id` value to the PartialConfig object.
    ///
    /// # Arguments
    ///
    /// * `node_id` - Unique ID for the node.
    ///
    pub fn with_node_id(mut self, node_id: Option<String>) -> Self {
        self.node_id = node_id;
        self
    }

    /// Adds a `bind` value to the PartialConfig object.
    ///
    /// # Arguments
    ///
    /// * `bind` - Connection endpoint for REST API.
    ///
    pub fn with_bind(mut self, bind: Option<String>) -> Self {
        self.bind = bind;
        self
    }

    #[cfg(feature = "database")]
    /// Adds a `database` value to the PartialConfig object, when the `database`
    /// feature flag is used.
    ///
    /// # Arguments
    ///
    /// * `database` - Connection endpoint for a database.
    ///
    pub fn with_database(mut self, database: Option<String>) -> Self {
        self.database = database;
        self
    }

    /// Adds a `registry_backend` value to the PartialConfig object.
    ///
    /// # Arguments
    ///
    /// * `registry_backend` - Backend type for the node registry.
    ///
    pub fn with_registry_backend(mut self, registry_backend: Option<String>) -> Self {
        self.registry_backend = registry_backend;
        self
    }

    /// Adds a `registry_file` value to the PartialConfig object.
    ///
    /// # Arguments
    ///
    /// * `registry_file` - File path to the node registry file if registry-backend is FILE.
    ///
    pub fn with_registry_file(mut self, registry_file: Option<String>) -> Self {
        self.registry_file = registry_file;
        self
    }

    /// Adds a `heartbeat_interval` value to the PartialConfig object.
    ///
    /// # Arguments
    ///
    /// * `heartbeat_interval` - How often heartbeat should be sent.
    ///
    pub fn with_heartbeat_interval(mut self, heartbeat_interval: Option<u64>) -> Self {
        self.heartbeat_interval = heartbeat_interval;
        self
    }

    /// Adds a `timeout` value to the PartialConfig object.
    ///
    /// # Arguments
    ///
    /// * `timeout` - The coordinator timeout for admin service proposals (in milliseconds).
    ///
    pub fn with_admin_service_coordinator_timeout(mut self, timeout: Option<u64>) -> Self {
        let duration: Option<Duration> = match timeout {
            Some(t) => Some(Duration::from_millis(t)),
            _ => None,
        };
        self.admin_service_coordinator_timeout = duration;
        self
    }

    /// Adds a `state_dir` value to the PartialConfig object.
    ///
    /// # Arguments
    ///
    /// * `state_dir` - The location of the storage directory when storage is YAML.
    ///
    pub fn with_state_dir(mut self, state_dir: Option<String>) -> Self {
        self.state_dir = state_dir;
        self
    }
}
