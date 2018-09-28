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
    ConfigType, Connection, SessionType, Shared,
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

// TODO handle cipher suites
impl SplinterDaemon {
    fn new(
        ca_files: &str,
        client_cert: &str,
        server_cert: &str,
        server_key_file: &str,
        client_key_file: &str,
        network_endpoint: &str,
        service_endpoint: &str,
        initial_peers: Vec<String>,
    ) -> SplinterDaemon {
        // TODO make multiple ca certs
        let ca_cert = load_cert(ca_files);
        let server_key = load_key(server_key_file);
        let client_key = load_key(client_key_file);

        let client_certs = load_cert(client_cert);

        // TODO get cipher suite from string
        // just picked first one, need to pick the correct ones
        let cipher_suite = rustls::ALL_CIPHERSUITES.to_vec()[0];

        let client_config =
            create_client_config(ca_cert.clone(), client_certs, client_key, cipher_suite);
        // create server config
        let server_certs = load_cert(server_cert);
        let server_config = create_server_config(ca_cert, server_certs, server_key);

        // create splinterD node
        let state = Arc::new(Mutex::new(Shared::new()));
        SplinterDaemon {
            client_config,
            server_config,
            state,
            service_endpoint: service_endpoint.parse().unwrap(),
            network_endpoint: network_endpoint.parse().unwrap(),
            initial_peers,
        }
    }

    fn stop() -> () {
        //TODO also add control-c handling
        unimplemented!();
    }

    fn start(&mut self) -> () {
        // create peers and pass to threads
        for peer in self.initial_peers.iter() {
            let addr: std::net::SocketAddr = peer.parse().unwrap();
            let mut socket = TcpStream::connect(addr.clone()).expect("Cannot connect stream");
            socket.set_nonblocking(true);
            // update to use correct dns_name
            let mut connection = Connection::new(
                socket,
                ConfigType::client(self.client_config.clone()),
                self.state.clone(),
                Some("server".to_string()),
            );
            let handle = thread::spawn(move || connection.handle_msg());
        }

        // start up a listener and accept incoming connections
        let listener = TcpListener::bind(self.service_endpoint).expect("Cannot listen on port");
        for socket in listener.incoming() {
            match socket {
                Ok(mut socket) => {
                    socket.set_nonblocking(true);
                    let addr = socket.peer_addr().unwrap();
                    // update to use correct dns_name
                    let mut connection = Connection::new(
                        socket,
                        ConfigType::server(self.server_config.clone()),
                        self.state.clone(),
                        None,
                    );
                    let handle = thread::spawn(move || connection.handle_msg());
                }
                Err(e) => panic!("Error {}", e),
            }
        }
    }
}

fn main() {
    let matches = clap_app!(splinter =>
        (version: crate_version!())
        (about: "Splinter Node")
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

    // TODO make multiple
    let ca_files = matches
        .value_of("ca_file")
        .expect("Must provide a valid ca certifcate");

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

    // TODO update to provide dns_name
    let initial_peers = matches
        .values_of("peers")
        .map(|values| values.map(|v| v.into()).collect())
        .unwrap_or(Vec::new());

    let logger = match matches.occurrences_of("verbose") {
        1 => simple_logger::init_with_level(LogLevel::Info),
        2 | _ => simple_logger::init_with_level(LogLevel::Debug),
        0 => simple_logger::init_with_level(LogLevel::Warn),
    };

    logger.expect("Failed to create logger");

    let mut node = SplinterDaemon::new(
        ca_files,
        client_cert,
        server_cert,
        server_key_file,
        client_key_file,
        network_endpoint,
        service_endpoint,
        initial_peers,
    );

    node.start();
}
