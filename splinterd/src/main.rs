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
#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
extern crate simple_logger;

use libsplinter::{
    create_client_config, create_server_config, create_server_session, load_cert, load_key,
    ConfigType, Connection, ConnectionType, SessionType, Shared, SplinterError,
};
use log::LogLevel;
use std::collections::HashMap;
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::{thread, time};

struct SplinterDaemon {
    client_config: rustls::ClientConfig,
    server_config: rustls::ServerConfig,
    state: Arc<Mutex<Shared>>,
    service_endpoint: std::net::SocketAddr,
    network_endpoint: std::net::SocketAddr,
    initial_peers: Vec<String>,
}

impl SplinterDaemon {
    fn new(
        ca_files: Vec<&str>,
        client_cert: &str,
        server_cert: &str,
        server_key_file: &str,
        client_key_file: &str,
        network_endpoint: &str,
        service_endpoint: &str,
        initial_peers: Vec<String>,
    ) -> Result<SplinterDaemon, SplinterError> {
        let mut ca_certs = Vec::new();
        for ca_file in ca_files {
            let ca_cert = load_cert(ca_file)?;
            ca_certs.extend(ca_cert);
        }
        let server_key = load_key(server_key_file)?;
        let client_key = load_key(client_key_file)?;

        let client_certs = load_cert(client_cert)?;

        // This should be updated to not just be all the suites
        let cipher_suites = rustls::ALL_CIPHERSUITES.to_vec();

        let client_config =
            create_client_config(ca_certs.clone(), client_certs, client_key, cipher_suites);
        // create server config
        let server_certs = load_cert(server_cert)?;
        let server_config = create_server_config(ca_certs, server_certs, server_key)?;

        // create splinterD node
        let state = Arc::new(Mutex::new(Shared::new()));

        let service_endpoint = if let Ok(addr) = service_endpoint.parse() {
            addr
        } else {
            return Err(SplinterError::CouldNotResolveHostName);
        };

        let network_endpoint = if let Ok(addr) = network_endpoint.parse() {
            addr
        } else {
            return Err(SplinterError::CouldNotResolveHostName);
        };

        Ok(SplinterDaemon {
            client_config,
            server_config,
            state,
            service_endpoint,
            network_endpoint,
            initial_peers,
        })
    }

    fn stop() -> () {
        //also add control-c handling
        unimplemented!();
    }

    fn start(&mut self) -> Result<(), SplinterError> {

        // create peers and pass to threads
        for peer in self.initial_peers.iter() {
            let addr: std::net::SocketAddr = if let Ok(addr) = peer.parse() {
               addr 
            } else {
                return Err(SplinterError::CouldNotResolveHostName);
            };

            let mut socket = TcpStream::connect(addr.clone())?;
            socket.set_nonblocking(true)?;

            // update to use correct dns_name
            let mut connection = Connection::new(
                socket,
                ConfigType::client(self.client_config.clone()),
                self.state.clone(),
                Some("server".to_string()),
                ConnectionType::Network,
            )?;
            let handle = thread::spawn(move || connection.handle_msg());
        }

        let network_endpoint = self.network_endpoint;
        let network_server_config = self.server_config.clone();
        let network_state = self.state.clone();
        thread::spawn(move || {
            // start up a listener and accept incoming connections
            let listener = TcpListener::bind(network_endpoint)?;
            for socket in listener.incoming() {
                match socket {
                    Ok(mut socket) => {
                        socket.set_nonblocking(true)?;
                        let addr = if let Ok(addr) = socket.peer_addr() {
                            addr
                        } else {
                            return Err(SplinterError::CouldNotResolveHostName);
                        };

                        // update to use correct dns_name
                        let mut connection = Connection::new(
                            socket,
                            ConfigType::server(network_server_config.clone()),
                            network_state.clone(),
                            None,
                            ConnectionType::Network,
                        )?;
                        let handle = thread::spawn(move || connection.handle_msg());
                    }
                    Err(e) => return Err(SplinterError::from(e)),
                }
            }

            Ok(())
        });

        // start up a listener and accept incoming connections
        let listener = TcpListener::bind(self.service_endpoint)?;

        for socket in listener.incoming() {
            match socket {
                Ok(mut socket) => {
                    socket.set_nonblocking(true)?;

                    let addr = if let Ok(addr) = socket.peer_addr() {
                        addr
                    } else {
                        return Err(SplinterError::CouldNotResolveHostName);
                    };
                    // update to use correct dns_name
                    let mut connection = Connection::new(
                        socket,
                        ConfigType::server(self.server_config.clone()),
                        self.state.clone(),
                        None,
                        ConnectionType::Service,
                    )?;
                    let handle = thread::spawn(move || connection.handle_msg());
                }
                Err(e) => return Err(SplinterError::from(e)) 
            }
        }

        Ok(())
    }
}

fn main() {
    let matches = clap_app!(splinter =>
        (version: crate_version!())
        (about: "Splinter Node")
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
        (@arg network_endpoint: -n --("network-port") +takes_value
          "endpoint to connect to the network, ip:port")
        (@arg service_endpoint: --("service-port") +takes_value
          "endpoint that service will connect to, ip:port")
        (@arg peers: --peer +takes_value +multiple
          "endpoint that service will connect to, ip:port")
        (@arg verbose: -v --verbose +multiple
         "increase output verbosity")).get_matches();

    let service_endpoint = matches
        .value_of("service_endpoint")
        .unwrap_or("127.0.0.1:8043");

    let network_endpoint = matches
        .value_of("network_endpoint")
        .unwrap_or("127.0.0.1:8044");

    let ca_files = matches
        .values_of("ca_file")
        .map(|values| values.map(|v| v.into()).collect())
        .expect("At least one ca file must be provided");

    let client_cert = matches
        .value_of("client_cert")
        .expect("Must provide a valid client certifcate");

    let server_cert = matches
        .value_of("server_cert")
        .expect("Must provide a valid server certifcate");

    let server_key_file = matches
        .value_of("server_key")
        .expect("Must provide a valid key path");

    let client_key_file = matches
        .value_of("client_key")
        .expect("Must provide a valid key path");

    // need to also provide dns_name
    let initial_peers = matches
        .values_of("peers")
        .map(|values| values.map(|v| v.into()).collect())
        .unwrap_or(Vec::new());

    let logger = match matches.occurrences_of("verbose") {
        0 => simple_logger::init_with_level(LogLevel::Warn),
        1 => simple_logger::init_with_level(LogLevel::Info),
        _  => simple_logger::init_with_level(LogLevel::Debug),
    };

    logger.expect("Failed to create logger");

    let mut node = match SplinterDaemon::new(
        ca_files,
        client_cert,
        server_cert,
        server_key_file,
        client_key_file,
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
