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

extern crate libsplinter;
extern crate rustls;
extern crate url;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
extern crate simple_logger;
extern crate toml;
extern crate serde;
#[macro_use] extern crate serde_derive;

mod daemon;
mod config;

use log::LogLevel;
use url::Url;
use config::{Config, ConfigError};
use std::fs::File;

use daemon::SplinterDaemon;

fn main() {
    let matches = clap_app!(splinter =>
        (version: crate_version!())
        (about: "Splinter Node")
        (@arg config: -c --config +takes_value)
        (@arg ca_file: --("ca-file") +takes_value +multiple
          "file path to the trusted ca cert")
        (@arg client_cert: --("client-cert") +takes_value
          "file path the cert for the node when connecting to a node")
        (@arg server_cert: --("server-cert") +takes_value
          "file path the cert for the node when connecting to a node")
        (@arg server_key:  --("server-key") +takes_value
          "file path key for the node when connecting to a node as sever")
        (@arg client_key:  --("client-key") +takes_value
          "file path key for the node when connecting to a node as client")
        (@arg network_endpoint: -n --("network-endpoint") +takes_value
          "endpoint to connect to the network, tcp://ip:port")
        (@arg service_endpoint: --("service-endpoint") +takes_value
          "endpoint that service will connect to, tcp://ip:port")
        (@arg peers: --peer +takes_value +multiple
          "endpoint that service will connect to, ip:port")
        (@arg verbose: -v --verbose +multiple
         "increase output verbosity")).get_matches();

    let logger = match matches.occurrences_of("verbose") {
        0 => simple_logger::init_with_level(LogLevel::Warn),
        1 => simple_logger::init_with_level(LogLevel::Info),
        _ => simple_logger::init_with_level(LogLevel::Debug),
    };

    logger.expect("Failed to create logger");

    debug!("Loading configuration file");

    let config = {
        let config_file_path = matches
            .value_of("config")
            .unwrap_or("config.toml");

        File::open(config_file_path)
            .map_err(ConfigError::from)
            .and_then(Config::from_file)
            .unwrap_or_else(|err| {
                warn!("No config file loaded {:?}", err);
                Config::default()
            })
    };

    debug!("Configuration: {:?}", config);

    let service_endpoint = matches
        .value_of("service_endpoint")
        .map(String::from)
        .or_else(|| config.service_endpoint())
        .map_or_else(|| Url::parse("tcp://127.0.0.1:8043"), |ep| Url::parse(&ep))
        .expect("Must provide a valid service endpoint");

    let network_endpoint = matches
        .value_of("network_endpoint")
        .map(String::from)
        .or_else(|| config.network_endpoint())
        .map_or_else(|| Url::parse("tcp://127.0.0.1:8044"), |ep| Url::parse(&ep))
        .expect("Must provide a valid network endpoint");

    let ca_files = matches
        .values_of("ca_file")
        .map(|values| values.map(String::from).collect::<Vec<String>>())
        .or_else(|| config.ca_certs())
        .expect("At least one ca file must be provided");

    let client_cert =  matches
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

    let initial_peers = {
        let urls = matches
            .values_of("peers")
            .map(|values| values.map(String::from).collect::<Vec<String>>())
            .or_else(|| config.peers())
            .unwrap_or(Vec::new());

        let mut peers = Vec::new();
        for url in urls {
            match Url::parse(&url) {
                Ok(u) => peers.push(u),
                Err(err) =>  {
                    error!("Invalid peer url {:?}", err);
                    std::process::exit(1);
                }
            };
        }

        peers
    };

    let mut node = match SplinterDaemon::new(
        ca_files,
        &client_cert,
        &server_cert,
        &server_key_file,
        &client_key_file,
        network_endpoint,
        service_endpoint,
        initial_peers,
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
