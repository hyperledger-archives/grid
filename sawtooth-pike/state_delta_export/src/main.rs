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

#[macro_use] extern crate clap;
#[macro_use] extern crate log;
#[macro_use] extern crate serde_json;
extern crate sawtooth_sdk;
extern crate pike_db;
extern crate addresser;
extern crate simple_logger;
extern crate protobuf;
extern crate uuid;
extern crate chan_signal;
extern crate regex;

mod subscriber;
mod database;
mod protos;

use std::{thread, env};
use chan_signal::Signal;
use subscriber::Subscriber;
use log::LogLevel;
use regex::Regex;
use database::apply_state_change;
use pike_db::pools::init_pg_pool;

use sawtooth_sdk::messages::transaction_receipt::StateChangeList;

const APP_NAME: &'static str = env!("CARGO_PKG_NAME");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const AUTHOR: &'static str = "Cargill";

fn url_is_valid(url: &str) -> bool {
    Regex::new(r"^[a-zA-Z]+://(.+:.+@)?.+:[0-9]{2,5}/?.*$")
        .unwrap()
        .is_match(url)
}

fn main() {
    let matches = clap_app!(app =>
        (name: APP_NAME)
        (version: VERSION)
        (author: AUTHOR)
        (about: "State delta export service for pike")
        (@arg verbose: -v +multiple "Log verbosely")
        (@arg connect: -c  --connect +takes_value "Validator to connect to")
        (@arg db: -d --database +takes_value "Full url to database")
    ).get_matches();

    let logger = match matches.occurrences_of("verbose") {
        1 => simple_logger::init_with_level(LogLevel::Info),
        2 => simple_logger::init_with_level(LogLevel::Debug),
        0 | _  => simple_logger::init_with_level(LogLevel::Warn)
    };

    logger.expect("Failed to create logger");

    let connect = String::from(matches
        .value_of("connect")
        .unwrap_or("tcp://localhost:4004"));

    let db_host = match matches.value_of("db") {
        Some(host) => host.to_string(),
        None => if let Ok(s) = env::var("DATABASE_URL") {
            s
        } else {
            "postgres://localhost:5432".into()
        }
    };

    if !url_is_valid(&connect) {
        error!("{} is not a valid url", connect);
        std::process::exit(1);
    }

    if !url_is_valid(&db_host) {
        error!("{} is not a valid database url", db_host);
        std::process::exit(1);
    }

    let signal = chan_signal::notify(&[Signal::INT, Signal::TERM]);

    let mut subscriber = Subscriber::new();

    let db_conn = init_pg_pool(db_host)
        .get()
        .expect("Failed to create postgress database connection");

    thread::spawn(move || subscriber.start(connect, |e| {
        e.events
            .iter()
            .filter(|x| "sawtooth/state-delta" == x.event_type)
            .filter_map(|x| -> Option<StateChangeList> {
                protobuf::parse_from_bytes(&x.data).ok()
            })
            .flat_map(|x| x.state_changes.into_iter())
            .for_each(|x| {
                info!("Applying state change {:?}", x);
                apply_state_change(&db_conn, &x)
                    .and_then(|_| {
                        info!("State change applied successfully");
                        Ok(())
                    })
                    .unwrap_or_else(|err| error!("{:?}", err))
            })
    }));

    signal
        .recv()
        .expect("Failed to create ctrl-c handler");

    subscriber.stop();
}
