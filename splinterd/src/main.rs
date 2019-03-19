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

mod config;
mod daemon;

use crate::config::{Config, ConfigError};
use crate::daemon::SplinterDaemon;
use libsplinter::transport::raw::RawTransport;
use libsplinter::transport::tls::TlsTransport;
use libsplinter::transport::Transport;

use ::log::LogLevel;
use ::log::{debug, error, log, warn};
use clap::{clap_app, crate_version};

use std::env;
use std::fs::File;

const DEFAULT_STATE_DIR: &str = "/var/lib/splinter/";
const STATE_DIR_ENV: &str = "SPLINTER_STATE_DIR";

fn main() {
    let matches = clap_app!(splinter =>
        (version: crate_version!())
        (about: "Splinter Node")
        (@arg config: -c --config +takes_value)
        (@arg node_id: --("node-id") +takes_value
          "unique id for the node ")
        (@arg storage: --("storage") +takes_value
          "storage type used for node, default yaml")
        (@arg transport: --("transport") +takes_value
          "transport type for sockets, either raw or tls")
        (@arg network_endpoint: -n --("network-endpoint") +takes_value
          "endpoint to connect to the network, tcp://ip:port")
        (@arg service_endpoint: --("service-endpoint") +takes_value
          "endpoint that service will connect to, tcp://ip:port")
        (@arg peers: --peer +takes_value +multiple
          "endpoint that service will connect to, ip:port")
        (@arg ca_file: --("ca-file") +takes_value
          "file path to the trusted ca cert")
        (@arg client_cert: --("client-cert") +takes_value
          "file path the cert for the node when connecting to a node")
        (@arg server_cert: --("server-cert") +takes_value
          "file path the cert for the node when connecting to a node")
        (@arg server_key:  --("server-key") +takes_value
          "file path key for the node when connecting to a node as sever")
        (@arg client_key:  --("client-key") +takes_value
          "file path key for the node when connecting to a node as client")
        (@arg verbose: -v --verbose +multiple
         "increase output verbosity"))
    .get_matches();

    let logger = match matches.occurrences_of("verbose") {
        0 => simple_logger::init_with_level(LogLevel::Warn),
        1 => simple_logger::init_with_level(LogLevel::Info),
        2 => simple_logger::init_with_level(LogLevel::Debug),
        _ => simple_logger::init_with_level(LogLevel::Trace),
    };

    logger.expect("Failed to create logger");

    debug!("Loading configuration file");

    let config = {
        // get provided config file or search default location
        let config_file_path = matches
            .value_of("config")
            .unwrap_or("/etc/splinter/splinterd.toml");

        File::open(config_file_path)
            .map_err(ConfigError::from)
            .and_then(Config::from_file)
            .unwrap_or_else(|err| {
                warn!("Unable to load {}: {}", config_file_path, err);
                Config::default()
            })
    };

    debug!("Configuration: {:?}", config);

    // Currently only YamlStorage is supported

    let node_id = matches
        .value_of("node_id")
        .map(String::from)
        .or_else(|| config.node_id())
        .expect("Must provide a unique node id");

    let storage_type = matches
        .value_of("storage")
        .map(String::from)
        .or_else(|| config.storage())
        .or_else(|| Some(String::from("yaml")))
        .expect("No Storage Provided");

    let transport_type = matches
        .value_of("transport")
        .map(String::from)
        .or_else(|| config.transport())
        .or_else(|| Some(String::from("raw")))
        .expect("No Transport Provided");

    let service_endpoint = matches
        .value_of("service_endpoint")
        .map(String::from)
        .or_else(|| config.service_endpoint())
        .or_else(|| Some("127.0.0.1:8043".to_string()))
        .expect("Must provide a valid service endpoint");

    let network_endpoint = matches
        .value_of("network_endpoint")
        .map(String::from)
        .or_else(|| config.network_endpoint())
        .or_else(|| Some("127.0.0.1:8044".to_string()))
        .expect("Must provide a valid network endpoint");

    let initial_peers = matches
        .values_of("peers")
        .map(|values| values.map(String::from).collect::<Vec<String>>())
        .or_else(|| config.peers())
        .unwrap_or_default();

    let transport = get_transport(&transport_type, &matches, &config);

    let location = {
        if let Ok(s) = env::var(STATE_DIR_ENV) {
            s.to_string()
        } else {
            DEFAULT_STATE_DIR.to_string()
        }
    };

    let storage_location = match &storage_type as &str {
        "yaml" => location + "/circuits.yaml",
        "memory" => "memory".to_string(),
        _ => panic!("Storage type is not supported: {}", storage_type),
    };

    let mut node = match SplinterDaemon::new(
        storage_location,
        transport,
        network_endpoint,
        service_endpoint,
        initial_peers,
        node_id,
    ) {
        Ok(node) => node,
        Err(err) => {
            error!("An error occurred while creating daemon {:?}", err);
            std::process::exit(1);
        }
    };

    if let Err(err) = node.start() {
        error!("Failed to start daemon {:?}", err);
        std::process::exit(1);
    }
}

fn get_transport(
    transport_type: &str,
    matches: &clap::ArgMatches,
    config: &Config,
) -> Box<dyn Transport + Send> {
    match transport_type {
        "tls" => {
            let ca_file = matches
                .value_of("ca_file")
                .map(String::from)
                .or_else(|| config.ca_certs())
                .expect("Must provide a valid file containing ca certs");

            let client_cert = matches
                .value_of("client_cert")
                .map(String::from)
                .or_else(|| config.client_cert())
                .expect("Must provide a valid client certifcate");

            let server_cert = matches
                .value_of("server_cert")
                .map(String::from)
                .or_else(|| config.server_cert())
                .expect("Must provide a valid server certifcate");

            let server_key_file = matches
                .value_of("server_key")
                .map(String::from)
                .or_else(|| config.server_key())
                .expect("Must provide a valid key path");

            let client_key_file = matches
                .value_of("client_key")
                .map(String::from)
                .or_else(|| config.client_key())
                .expect("Must provide a valid key path");

            let transport = match TlsTransport::new(
                ca_file,
                client_key_file,
                client_cert,
                server_key_file,
                server_cert,
            ) {
                Ok(transport) => transport,
                Err(err) => {
                    error!("An error occurred while creating transport{:?}", err);
                    std::process::exit(1);
                }
            };

            Box::new(transport)
        }
        "raw" => Box::new(RawTransport::default()),
        _ => panic!("Transport type is not supported: {}", transport_type),
    }
}
