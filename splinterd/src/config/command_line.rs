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
