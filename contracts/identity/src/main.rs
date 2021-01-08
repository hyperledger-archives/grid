// Copyright 2021 Cargill Incorporated
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

#[macro_use]
extern crate cfg_if;
extern crate crypto;
extern crate grid_sdk;
extern crate protobuf;

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        #[macro_use]
        extern crate sabre_sdk;
    } else {
        #[macro_use]
        extern crate clap;
        extern crate log;
        extern crate sawtooth_sdk;
        extern crate flexi_logger;

        mod error;

        use flexi_logger::{LogSpecBuilder, Logger};
        use sawtooth_sdk::processor::TransactionProcessor;
        use handler::IdentityTransactionHandler;
        use error::CliError;
    }
}

pub mod addressing;
pub mod handler;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), CliError> {
    let matches = clap_app!(wasm_store_tp =>
        (version: crate_version!())
        (about: "Implements the Grid Identity feature")
        (@arg connect: -C --connect +takes_value
        "connection endpoint for validator")
        (@arg verbose: -v --verbose +multiple
        "increase output verbosity"))
    .get_matches();

    let log_level = match matches.occurrences_of("verbose") {
        0 => log::LevelFilter::Warn,
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };

    let mut log_spec_builder = LogSpecBuilder::new();
    log_spec_builder.default(log_level);
    Logger::with(log_spec_builder.build()).start()?;

    let connect = matches
        .value_of("connect")
        .unwrap_or("tcp://localhost:4004");

    let handler = IdentityTransactionHandler::new();
    let mut processor = TransactionProcessor::new(connect);

    processor.add_handler(&handler);
    processor.start();

    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {}
