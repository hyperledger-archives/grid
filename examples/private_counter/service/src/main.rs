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

mod error;

use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

use ::log::LogLevel;
use ::log::{debug, error, log};
use clap::{App, Arg};
use threadpool::ThreadPool;

use crate::error::{HandleError, ServiceError};


fn main() -> Result<(), ServiceError> {
    let matches = configure_args().get_matches();
    configure_logging(&matches);

    let listener = TcpListener::bind(matches.value_of("bind").unwrap()).unwrap();
    let workers: usize = matches.value_of("workers").unwrap().parse().unwrap();
    let pool = ThreadPool::new(workers);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        debug!("Received connection");
        pool.execute(move || match handle_connection(stream) {
            Ok(_) => (),
            Err(err) => error!("Error encoutered in handling connection: {}", err),
        });
    }

    Ok(())
}

/// Validate that the given string is a properly formatted endpoint
fn valid_endpoint<S: AsRef<str>>(s: S) -> Result<(), String> {
    let s = s.as_ref();

    if s.is_empty() {
        return Err("Bind string must not be empty".into());
    }
    let mut parts = s.split(":");

    parts.next().unwrap();

    if let Some(port_str) = parts.next() {
        match port_str.parse::<u16>() {
            Ok(port) if port > 0 => port,
            _ => {
                return Err(format!(
                    "{} does not specify a valid port: must be an int between 0 < port < 65535",
                    s
                ));
            }
        }
    } else {
        return Err(format!("{} must specify a port", s));
    };

    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> Result<(), HandleError> {
    let mut buffer = [0; 512];

    stream.read(&mut buffer)?;
    let request = String::from_utf8_lossy(&buffer[..]);

    let mut response = "HTTP/1.1".to_string();

    if request.starts_with("GET / ") {
        // Return name of service
        response = response + "200 OK\r\n\r\nPrivate Counter Server";
    } else if request.starts_with("GET /add/") {
        // get number to add to current value
        let addition = &request["GET /add/".len()..];
        if let Some(end) = addition.find(" ") {
            let addition = &addition[..end];

            // check that the value can be parsed into a u32
            if addition.parse::<u32>().is_err() {
                response = response + " 400 BAD REQUEST\r\n\r\n";
            } else {
                // return 204 NO CONTENT
                response = response + " 204 NO CONTENT\r\n\r\n";
            }
        } else {
            response = response + " 400 BAD REQUEST\r\n\r\n";
        }
    } else if request.starts_with("GET /show") {
        // return current value
        response = response + " 200 OK\r\n\r\n0";
    } else {
        // cannot handle endpoint, return 404
        response = response + " 404 NOT FOUND\r\n\r\n";
    }
    stream.write(response.as_bytes())?;
    stream.flush()?;

    Ok(())
}

fn configure_args<'a, 'b>() -> App<'a, 'b> {
    App::new(clap::crate_name!())
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about(clap::crate_description!())
        .arg(
            Arg::with_name("service_id")
                .short("N")
                .long("service_id")
                .takes_value(true)
                .value_name("ID")
                .required(true)
                .help("the name of this service, as presented to the network"),
        )
        .arg(
            Arg::with_name("circuit")
                .short("c")
                .long("circuit")
                .takes_value(true)
                .value_name("CIRCUIT NAME")
                .required(true)
                .help("the name of the circuit to connect to"),
        )
        .arg(
            Arg::with_name("verifier")
                .short("V")
                .long("verifier")
                .takes_value(true)
                .value_name("SERVICE_ID")
                .required(true)
                .multiple(true)
                .help("the name of a service that will validate a counter increment"),
        )
        .arg(
            Arg::with_name("bind")
                .short("B")
                .long("bind")
                .value_name("BIND")
                .default_value("localhost:8000")
                .validator(valid_endpoint)
                .help("endpoint to receive HTTP requests, ip:port"),
        )
        .arg(
            Arg::with_name("connect")
                .short("C")
                .long("connect")
                .value_name("CONNECT")
                .default_value("localhost:8043")
                .validator(valid_endpoint)
                .help("the service endpoint of a splinterd node, ip:port"),
        )
        .arg(
            Arg::with_name("transport")
                .long("transport")
                .default_value("raw")
                .value_name("TRANSPORT")
                .possible_values(&["raw", "tls"])
                .help("transport type for sockets, either raw or tls"),
        )
        .arg(
            Arg::with_name("ca_file")
                .long("ca-file")
                .takes_value(true)
                .value_name("FILE")
                .requires_if("transport", "tls")
                .help("file path to the trusted ca cert"),
        )
        .arg(
            Arg::with_name("client_key")
                .long("client-key")
                .takes_value(true)
                .value_name("FILE")
                .requires_if("transport", "tls")
                .help("file path for the TLS key used to connect to a splinterd node"),
        )
        .arg(
            Arg::with_name("client_cert")
                .long("client-cert")
                .takes_value(true)
                .value_name("FILE")
                .requires_if("transport", "tls")
                .help("file path the cert used to connect to a splinterd node"),
        )
        .arg(
            Arg::with_name("workers")
                .short("w")
                .long("workers")
                .takes_value(true)
                .value_name("FILE")
                .default_value("5")
                .help("number of workers in the threadpool"),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .multiple(true)
                .help("enable more verbose logging output"),
        )
}

fn configure_logging(matches: &clap::ArgMatches) {
    let logger = match matches.occurrences_of("verbose") {
        0 => simple_logger::init_with_level(LogLevel::Warn),
        1 => simple_logger::init_with_level(LogLevel::Info),
        _ => simple_logger::init_with_level(LogLevel::Debug),
    };
    logger.expect("Failed to create logger");
}
