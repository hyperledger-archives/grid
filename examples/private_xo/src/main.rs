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

#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde_derive;

mod routes;

use clap::{App, Arg};
use rocket::config::{Config, Environment};

use crate::routes::batches;

#[get("/")]
fn index() -> &'static str {
    "Private XO Server"
}

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
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .multiple(true)
                .help("enable more verbose logging output"),
        )
        .get_matches();

    let (address, port) = split_bind(matches.value_of("bind").unwrap())?;

    rocket::custom(
        Config::build(Environment::Production)
            .address(address)
            .port(port)
            .finalize()
            .map_err(|err| format!("Invalid configuration: {:?}", err))?,
    )
    .mount(
        "/",
        routes![
            index,
            batches::batches,
            batches::batch_statuses,
        ],
    )
    .launch();

    Ok(())
}

fn valid_bind(s: String) -> Result<(), String> {
    split_bind(s).map(|_| ())
}

fn split_bind<S: AsRef<str>>(s: S) -> Result<(String, u16), String> {
    if s.as_ref().is_empty() {
        return Err("Bind string must not be empty".into());
    }
    let mut parts = s.as_ref().split(":");

    let address = parts.next().unwrap();

    let port = if let Some(port_str) = parts.next() {
        match port_str.parse::<u16>() {
            Ok(port) if port > 0 => port,
            _ => return Err(
                format!(
                    "{} does not specify a valid port: must be an integer in the range 0 < port < 65535",
                    s.as_ref()))
        }
    } else {
        return Err(format!("{} must specify a port", s.as_ref()));
    };

    Ok((address.to_string(), port))
}
