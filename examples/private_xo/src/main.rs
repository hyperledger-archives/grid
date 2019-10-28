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

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

mod error;
mod protos;
mod routes;
mod service;
mod transaction;

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::Builder;

use clap::{App, Arg};
use iron::prelude::*;
use iron::status;
use router::Router;

use splinter::consensus::two_phase::TwoPhaseEngine;
use splinter::consensus::{ConsensusEngine, StartupState};
use splinter::mesh::Mesh;
use splinter::network::Network;
use splinter::transport::{raw::RawTransport, tls::TlsTransport, Transport};

use crate::error::CliError;
use crate::routes::{batches, state, State};
use crate::service::consensus::{PrivateXoNetworkSender, PrivateXoProposalManager};
use crate::service::{start_service_loop, ServiceConfig, ServiceError};
use crate::transaction::{XoState, XoStateError};

const HEARTBEAT: u64 = 60;

fn index(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, "Private XO Server")))
}

fn main() -> Result<(), CliError> {
    let matches = configure_app_args().get_matches();
    configure_logging(&matches);
    let bind_value = matches
        .value_of("bind")
        .expect("Bind was not marked as a required attribute");

    let running = Arc::new(AtomicBool::new(true));
    configure_shutdown_handler(Arc::clone(&running))?;

    let xo_state = XoState::new()?;
    let pending_batches = Arc::new(Mutex::new(VecDeque::new()));
    let pending_proposal = Arc::new(Mutex::new(None));

    let mut transport = get_transport(&matches)?;
    let network = create_network_and_connect(
        &mut *transport,
        matches
            .value_of("connect")
            .expect("Connect was not marked as a required attribute"),
    )?;

    let service_config = get_service_config(
        network
            .peer_ids()
            .get(0)
            .cloned()
            .ok_or_else(|| CliError("Unable to connect to Splinter Node".into()))?,
        &matches,
    );

    let (send, recv) = crossbeam_channel::bounded(5);

    let (consensus_msg_tx, consensus_msg_rx) = std::sync::mpsc::channel();
    let (proposal_update_tx, proposal_update_rx) = std::sync::mpsc::channel();

    let proposal_manager = PrivateXoProposalManager::new(
        service_config.clone(),
        xo_state.clone(),
        pending_batches.clone(),
        pending_proposal.clone(),
        proposal_update_tx.clone(),
        send.clone(),
    );
    let consensus_network_sender =
        PrivateXoNetworkSender::new(service_config.clone(), send.clone());
    let startup_state = StartupState {
        id: service_config.service_id().as_bytes().into(),
        peer_ids: service_config
            .verifiers()
            .iter()
            .map(|id| id.as_bytes().into())
            .collect(),
        last_proposal: None,
    };

    let _ = Builder::new()
        .name("TwoPhaseConsensus".into())
        .spawn(move || {
            let mut two_phase_engine = TwoPhaseEngine::default();
            two_phase_engine
                .run(
                    consensus_msg_rx,
                    proposal_update_rx,
                    Box::new(consensus_network_sender),
                    Box::new(proposal_manager),
                    startup_state,
                )
                .unwrap_or_else(|err| {
                    error!("Error while running two phase consensus: {}", err);
                })
        })
        .map_err(|err| ServiceError(format!("Unable to start consensus thread: {}", err)))?;

    start_service_loop(
        service_config.clone(),
        (send.clone(), recv),
        consensus_msg_tx,
        proposal_update_tx,
        pending_proposal,
        network.clone(),
        running,
    )?;

    let (address, port) = split_endpoint(bind_value)?;

    let mut router = Router::new();
    router.get("/", index, "index");
    router.get("/batch_statuses", batches::batch_statuses, "batch_statuses");
    router.post("/batches", batches::batches, "batches");
    router.get("/state", state::list_state_with_params, "list_state");
    router.get("/state/:address", state::get_state_by_address, "get_state");

    let mut chain = Chain::new(router);

    chain.link_before(move |req: &mut Request| {
        req.set_mut(State::new(service_config.clone()));
        req.set_mut(State::new(xo_state.clone()));
        req.set_mut(State::new(pending_batches.clone()));
        req.set_mut(State::new(send.clone()));

        Ok(())
    });

    Iron::new(chain)
        .http(&format!("{}:{}", address, port))
        .map_err(|err| {
            CliError(format!(
                "Unable to start REST API on {}: {}",
                bind_value, err
            ))
        })?;

    Ok(())
}

fn get_service_config(peer_id: String, matches: &clap::ArgMatches) -> ServiceConfig {
    let circuit = matches
        .value_of("circuit")
        .expect("Circuit was not marked as a required attribute")
        .to_string();
    let service_id = matches
        .value_of("service_id")
        .expect("Service id was not marked as a required attribute")
        .to_string();
    let verifiers: Vec<String> = matches
        .values_of("verifier")
        .unwrap()
        .map(ToString::to_string)
        .collect();

    ServiceConfig::new(peer_id, circuit, service_id, verifiers)
}

/// Return the appropriate transport for the current arguments
fn get_transport(matches: &clap::ArgMatches) -> Result<Box<dyn Transport + Send>, CliError> {
    match matches.value_of("transport") {
        Some("tls") => {
            let ca_file = matches.value_of("ca_file").map(String::from);

            let client_cert = matches
                .value_of("client_cert")
                .map(String::from)
                .ok_or_else(|| CliError("Must provide a valid client certificate".into()))?;

            let client_key_file = matches
                .value_of("client_key")
                .map(String::from)
                .ok_or_else(|| CliError("Must provide a valid key path".into()))?;

            if ca_file.is_none() {
                warn!("No CA File provided; starting TlsTransport in insecure mode");
            }

            // Reuse the cert and key as a server cert and key, as there currently isn't a client-
            // only TlsTransport implementation.
            match TlsTransport::new(
                ca_file,
                client_key_file.clone(),
                client_cert.clone(),
                client_key_file,
                client_cert,
            ) {
                Ok(transport) => Ok(Box::new(transport)),
                Err(err) => Err(CliError(format!(
                    "An error occurred while creating {} transport: {:?}",
                    matches.value_of("transport").unwrap(),
                    err
                ))),
            }
        }
        Some("raw") => Ok(Box::new(RawTransport::default())),
        // this should have been caught by clap, so panic
        _ => panic!(
            "Transport type is not supported: {:?}",
            matches.value_of("transport")
        ),
    }
}

fn create_network_and_connect(
    transport: &mut (dyn Transport + Send),
    connect_endpoint: &str,
) -> Result<Network, CliError> {
    let mesh = Mesh::new(512, 128);
    let network = Network::new(mesh, HEARTBEAT)
        .map_err(|err| ServiceError(format!("Unable to start network: {}", err)))?;
    let connection = transport.connect(connect_endpoint).map_err(|err| {
        CliError(format!(
            "Unable to connect to {}: {:?}",
            connect_endpoint, err
        ))
    })?;

    network
        .add_connection(connection)
        .map_err(|err| CliError(format!("Unable to add connection to network: {:?}", err)))?;

    Ok(network)
}

fn configure_app_args<'a, 'b>() -> App<'a, 'b> {
    App::new(clap::crate_name!())
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about(clap::crate_description!())
        .arg(
            Arg::with_name("service_id")
                .short("N")
                .long("service-id")
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
                .value_name("bind")
                .takes_value(true)
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
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .multiple(true)
                .help("enable more verbose logging output"),
        )
}

fn configure_logging(matches: &clap::ArgMatches) {
    let logger = match matches.occurrences_of("verbose") {
        0 => simple_logger::init_with_level(log::Level::Warn),
        1 => simple_logger::init_with_level(log::Level::Info),
        _ => simple_logger::init_with_level(log::Level::Debug),
    };
    logger.expect("Failed to create logger");
}

fn configure_shutdown_handler(running: Arc<AtomicBool>) -> Result<(), CliError> {
    ctrlc::set_handler(move || {
        info!("Received Shutdown");
        running.store(false, Ordering::SeqCst);
    })
    .map_err(|err| CliError(format!("Unable to create control c handler: {}", err)))
}

fn valid_endpoint(s: String) -> Result<(), String> {
    split_endpoint(s).map(|_| ()).map_err(|err| err.to_string())
}

fn split_endpoint<S: AsRef<str>>(s: S) -> Result<(String, u16), CliError> {
    let s = s.as_ref();
    if s.is_empty() {
        return Err(CliError("Bind string must not be empty".into()));
    }
    let mut parts = s.split(':');

    let address = parts.next().unwrap();

    let port = if let Some(port_str) = parts.next() {
        match port_str.parse::<u16>() {
            Ok(port) if port > 0 => port,
            _ => return Err(CliError(
                format!(
                    "{} does not specify a valid port: must be an integer in the range 0 < port < 65535",
                    s)))
        }
    } else {
        return Err(CliError(format!("{} must specify a port", s)));
    };

    Ok((address.to_string(), port))
}

impl From<ServiceError> for CliError {
    fn from(err: ServiceError) -> Self {
        CliError(format!("Service Error: {}", err))
    }
}

impl From<XoStateError> for CliError {
    fn from(err: XoStateError) -> Self {
        CliError(err.to_string())
    }
}
