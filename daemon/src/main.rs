/*
 * Copyright 2019 Bitwise IO, Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * -----------------------------------------------------------------------------
 */

#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;

mod config;
mod error;
mod event;
mod rest_api;

use simple_logger;

use crate::config::GridConfigBuilder;
use crate::error::DaemonError;
use crate::event::{block::BlockEventHandler, EventProcessor};

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn run() -> Result<(), DaemonError> {
    let matches = clap_app!(myapp =>
        (name: APP_NAME)
        (version: VERSION)
        (author: "Contributors to Hyperledger Grid")
        (about: "Daemon Package for Hyperledger Grid")
        (@arg connect: -C --connect +takes_value "connection endpoint for validator")
        (@arg verbose: -v +multiple "Log verbosely")
        (@arg bind: -b --bind +takes_value "connection endpoint for rest API")
    )
    .get_matches();

    let config = GridConfigBuilder::default()
        .with_cli_args(&matches)
        .build()?;

    simple_logger::init_with_level(config.log_level())?;

    let (rest_api_shutdown_handle, rest_api_join_handle) =
        rest_api::run(config.rest_api_endpoint())?;

    let evt_processor = EventProcessor::start(
        config.validator_endpoint(),
        "0000000000000000",
        event_handlers![BlockEventHandler::new()],
    )
    .map_err(|err| DaemonError::EventProcessorError(Box::new(err)))?;

    let (event_processor_shutdown_handle, event_processor_join_handle) =
        evt_processor.take_shutdown_controls();

    ctrlc::set_handler(move || {
        if let Err(err) = rest_api_shutdown_handle.shutdown() {
            error!("Unable to cleanly shutdown REST API server: {}", err);
        }
        if let Err(err) = event_processor_shutdown_handle.shutdown() {
            error!("Unable to gracefully shutdown Event Processor: {}", err);
        }
    })
    .map_err(|err| DaemonError::StartUpError(Box::new(err)))?;

    rest_api_join_handle
        .join()
        .map_err(|_| {
            DaemonError::ShutdownError("Unable to cleanly join the REST API thread".into())
        })
        .and_then(|res| res.map_err(DaemonError::from))?;

    event_processor_join_handle
        .join()
        .map_err(|_| {
            DaemonError::ShutdownError("Unable to cleanly join the event processor".into())
        })
        .and_then(|res| res.map_err(DaemonError::from))?;

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        error!("{:?}", e);
        std::process::exit(1);
    }
}
