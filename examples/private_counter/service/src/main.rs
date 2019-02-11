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

use crate::error::HandleError;

fn main() -> Result<(), String> {
    let matches = App::new(clap::crate_name!())
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about(clap::crate_description!())
        .arg(
            Arg::with_name("bind")
                .short("B")
                .long("bind")
                .value_name("bind")
                .takes_value(true)
                .default_value("localhost:8000")
                .validator(valid_bind),
        )
        .arg(
            Arg::with_name("workers")
                .short("w")
                .long("workers")
                .takes_value(true)
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
        .get_matches();

    let logger = match matches.occurrences_of("verbose") {
        0 => simple_logger::init_with_level(LogLevel::Warn),
        1 => simple_logger::init_with_level(LogLevel::Info),
        _ => simple_logger::init_with_level(LogLevel::Debug),
    };

    logger.expect("Failed to create logger");

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

fn valid_bind<S: AsRef<str>>(s: S) -> Result<(), String> {
    if s.as_ref().is_empty() {
        return Err("Bind string must not be empty".into());
    }
    let mut parts = s.as_ref().split(":");

    parts.next().unwrap();

    if let Some(port_str) = parts.next() {
        match port_str.parse::<u16>() {
            Ok(port) if port > 0 => port,
            _ => {
                return Err(format!(
                    "{} does not specify a valid port: must be an int between 0 < port < 65535",
                    s.as_ref()
                ));
            }
        }
    } else {
        return Err(format!("{} must specify a port", s.as_ref()));
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
