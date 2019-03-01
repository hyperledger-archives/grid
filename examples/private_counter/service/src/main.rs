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

use std::fmt::Write as FmtWrite;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::string::ToString;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{Builder, JoinHandle};
use std::time::Duration;

use ::log::LogLevel;
use ::log::{debug, error, info, log, warn};
use clap::{App, Arg};
use crossbeam_channel;
use protobuf::{self, Message};
use sha2::{Digest, Sha256};
use threadpool::ThreadPool;
use uuid::Uuid;

use libsplinter::mesh::Mesh;
use libsplinter::network::{
    sender::{NetworkMessageSender, NetworkMessageSenderError, SendRequest},
    Network, RecvTimeoutError,
};
use libsplinter::protos::authorization::{
    AuthorizationMessage, AuthorizationMessageType, ConnectRequest, ConnectRequest_HandshakeMode,
    ConnectResponse, ConnectResponse_AuthorizationType, TrustRequest,
};
use libsplinter::protos::circuit::{
    CircuitDirectMessage, CircuitMessage, CircuitMessageType, ServiceConnectRequest,
    ServiceConnectResponse, ServiceConnectResponse_Status, ServiceDisconnectRequest,
};
use libsplinter::protos::n_phase::{
    NPhaseTransactionMessage, NPhaseTransactionMessage_Type, TransactionVerificationRequest,
    TransactionVerificationResponse, TransactionVerificationResponse_Result,
};
use libsplinter::protos::network::{NetworkMessage, NetworkMessageType};
use libsplinter::transport::{raw::RawTransport, tls::TlsTransport, Transport};

use crate::error::{HandleError, ServiceError};

// Recv timeout in secs
const TIMEOUT_SEC: u64 = 2;

#[derive(Default, Debug)]
struct ServiceState {
    counter: u32,
    proposed_increment: Option<u32>,
}

fn main() -> Result<(), ServiceError> {
    let matches = configure_args().get_matches();

    let matches2 = matches.clone();
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    let listener = TcpListener::bind(matches.value_of("bind").unwrap()).unwrap();
    ctrlc::set_handler(move || {
        info!("Received Shutdown");
        r.store(false, Ordering::SeqCst);

        // wake the listener so it can shutdown
        TcpStream::connect(matches2.value_of("bind").unwrap()).unwrap();
    })
    .expect("Error setting Ctrl-C handler");

    configure_logging(&matches);

    let state: Arc<Mutex<ServiceState>> = Default::default();
    let circuit = matches.value_of("circuit").unwrap().to_string();
    let service_id = matches.value_of("service_id").unwrap().to_string();

    let mut transport = get_transport(&matches)?;
    let network = create_network_and_connect(&mut transport, matches.value_of("connect").unwrap())?;
    let (send, recv) = crossbeam_channel::bounded(5);
    let (sender_thread, receiver_thread) = start_service_loop(
        circuit.clone(),
        service_id.clone(),
        (send.clone(), recv),
        network.clone(),
        state.clone(),
        running.clone(),
    )?;

    let workers: usize = matches.value_of("workers").unwrap().parse().unwrap();
    let pool = ThreadPool::new(workers);
    let verifiers: Vec<String> = matches
        .values_of("verifier")
        .unwrap()
        .map(ToString::to_string)
        .collect();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        debug!("Received connection");

        if !running.load(Ordering::SeqCst) {
            info!("Shutting Down");
            break;
        }

        let stream_state = state.clone();
        let peer_id = network.peer_ids()[0].clone();
        let stream_circuit = circuit.clone();
        let stream_service_id = service_id.clone();
        let stream_verifiers = verifiers.clone();
        let stream_sender = send.clone();
        pool.execute(move || {
            match handle_connection(
                stream,
                stream_state,
                peer_id,
                stream_circuit,
                stream_service_id,
                stream_verifiers,
                stream_sender,
            ) {
                Ok(_) => (),
                Err(err) => error!("Error encountered in handling connection: {}", err),
            }
        });
    }

    let _ = sender_thread.join();
    let _ = receiver_thread.join();

    Ok(())
}

macro_rules! unwrap_or_break {
    ($result:expr) => {
        match $result {
            Ok(x) => x,
            Err(err) => {
                error!("Network Receive Failed; Terminating due to {:?}", err);
                break;
            }
        }
    };
}

fn create_network_and_connect(
    transport: &mut Box<dyn Transport + Send>,
    connect_endpoint: &str,
) -> Result<Network, ServiceError> {
    let mesh = Mesh::new(512, 128);
    let network = Network::new(mesh);
    let connection = transport.connect(connect_endpoint).map_err(|err| {
        ServiceError(format!(
            "Unable to connect to {}: {:?}",
            connect_endpoint, err
        ))
    })?;

    network
        .add_connection(connection)
        .map_err(|err| ServiceError(format!("Unable to add connection to network: {:?}", err)))?;

    Ok(network)
}

fn start_service_loop(
    circuit: String,
    service_id: String,
    channel: (
        crossbeam_channel::Sender<SendRequest>,
        crossbeam_channel::Receiver<SendRequest>,
    ),
    network: Network,
    state: Arc<Mutex<ServiceState>>,
    running: Arc<AtomicBool>,
) -> Result<
    (
        JoinHandle<Result<(), NetworkMessageSenderError>>,
        JoinHandle<()>,
    ),
    ServiceError,
> {
    info!("Starting Private Counter Service");
    let sender_network = network.clone();
    let (send, recv) = channel;

    let running_clone = running.clone();
    let sender_thread = Builder::new()
        .name("NetworkMessageSender".into())
        .spawn(move || {
            let network_sender =
                NetworkMessageSender::new(Box::new(recv), sender_network, running_clone);
            network_sender.run()
        })
        .map_err(|err| {
            ServiceError(format!(
                "Unable to start network message sender thread: {}",
                err
            ))
        })?;

    let recv_network = network.clone();
    let reply_sender = send.clone();
    let receiver_thread = Builder::new()
        .name("NetworkReceiver".into())
        .spawn(move || {
            run_service_loop(
                recv_network,
                &reply_sender,
                circuit,
                service_id,
                state,
                running,
            )
        })
        .map_err(|err| ServiceError(format!("Unable to start network receiver thread: {}", err)))?;

    let connect_request_msg_bytes = create_connect_request()
        .map_err(|err| ServiceError(format!("Unable to create connect request: {}", err)))?;
    for peer_id in network.peer_ids() {
        debug!("Sending connect request to peer {}", peer_id);
        network
            .send(&peer_id, &connect_request_msg_bytes)
            .map_err(|err| ServiceError(format!("Unable to send connect request: {:?}", err)))?;
    }

    Ok((sender_thread, receiver_thread))
}

fn run_service_loop(
    network: Network,
    reply_sender: &crossbeam_channel::Sender<SendRequest>,
    circuit: String,
    service_id: String,
    state: Arc<Mutex<ServiceState>>,
    running: Arc<AtomicBool>,
) {
    let timeout = Duration::from_secs(TIMEOUT_SEC);
    while running.load(Ordering::SeqCst) {
        match network.recv_timeout(timeout) {
            Ok(message) => {
                let msg: NetworkMessage =
                    unwrap_or_break!(protobuf::parse_from_bytes(message.payload()));

                match msg.get_message_type() {
                    NetworkMessageType::AUTHORIZATION => {
                        let auth_msg: AuthorizationMessage =
                            unwrap_or_break!(protobuf::parse_from_bytes(msg.get_payload()));
                        if unwrap_or_break!(handle_authorized_msg(
                            auth_msg,
                            message.peer_id(),
                            &service_id,
                            &reply_sender
                        )) {
                            info!("Successfully authorized with peer {}", message.peer_id());

                            unwrap_or_break!(network.send(
                                message.peer_id(),
                                &unwrap_or_break!(create_circuit_service_connect_request(
                                    &circuit,
                                    &service_id
                                ))
                            ));
                        }
                    }
                    NetworkMessageType::CIRCUIT => {
                        let circuit_msg =
                            unwrap_or_break!(protobuf::parse_from_bytes(msg.get_payload()));
                        unwrap_or_break!(handle_circuit_msg(
                            message.peer_id(),
                            circuit_msg,
                            &reply_sender,
                            &state
                        ));
                    }
                    _ => {
                        debug!("Ignoring message of type {:?}", msg.get_message_type());
                    }
                };
            }
            Err(RecvTimeoutError::Disconnected) => {
                error!("Network has disconnected");
                break;
            }
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::NoPeerError(err)) => {
                warn!("Received NoPeerError: {}", err);
            }
        }
    }
    info!("Sending disconnect request");
    let disconnect_msg = create_circuit_service_disconnect_request(&circuit, &service_id)
        .expect("Unable to create disconnect message");
    for peer_id in network.peer_ids() {
        match network.send(&peer_id, &disconnect_msg) {
            Ok(_) => (),
            Err(err) => error!(
                "Unable to send disconnect message to {}: {:?}",
                &peer_id, err
            ),
        }
    }
}

/// Handles authorization messages
fn handle_authorized_msg(
    auth_msg: AuthorizationMessage,
    source_peer_id: &str,
    identity: &str,
    sender: &crossbeam_channel::Sender<SendRequest>,
) -> Result<bool, ServiceError> {
    match auth_msg.get_message_type() {
        AuthorizationMessageType::CONNECT_RESPONSE => {
            let msg: ConnectResponse = protobuf::parse_from_bytes(auth_msg.get_payload())?;

            if msg
                .get_accepted_authorization_types()
                .iter()
                .any(|t| t == &ConnectResponse_AuthorizationType::TRUST)
            {
                let mut trust_request = TrustRequest::new();
                trust_request.set_identity(identity.to_string());
                sender.send(SendRequest::new(
                    source_peer_id.to_string(),
                    wrap_in_network_auth_envelopes(
                        AuthorizationMessageType::TRUST_REQUEST,
                        trust_request,
                    )?,
                ))?;
            }
            // send trust request
            Ok(false)
        }
        AuthorizationMessageType::AUTHORIZE => Ok(true),
        _ => Ok(false),
    }
}

fn handle_circuit_msg(
    source_peer_id: &str,
    circuit_msg: CircuitMessage,
    sender: &crossbeam_channel::Sender<SendRequest>,
    state: &Arc<Mutex<ServiceState>>,
) -> Result<(), ServiceError> {
    match circuit_msg.get_message_type() {
        CircuitMessageType::SERVICE_CONNECT_RESPONSE => {
            let msg: ServiceConnectResponse =
                protobuf::parse_from_bytes(circuit_msg.get_payload())?;
            match msg.get_status() {
                ServiceConnectResponse_Status::OK => info!(
                    "Service {} on circuit {} has connected",
                    msg.get_service_id(),
                    msg.get_circuit(),
                ),
                ServiceConnectResponse_Status::ERROR_QUEUE_FULL => warn!("Queue is full"),
                _ => {
                    return Err(ServiceError(format!(
                        "Unable to connect service {} to circuit {}: {}",
                        msg.get_service_id(),
                        msg.get_circuit(),
                        msg.get_error_message()
                    )));
                }
            }
        }
        CircuitMessageType::CIRCUIT_DIRECT_MESSAGE => {
            let mut msg: CircuitDirectMessage =
                protobuf::parse_from_bytes(circuit_msg.get_payload())?;
            handle_direct_msg(source_peer_id, &mut msg, sender, state)?;
        }
        _ => debug!("Received message {:?}", circuit_msg),
    }

    Ok(())
}

/// This handles the messages that are specifically targeting this service.
fn handle_direct_msg(
    source_peer_id: &str,
    circuit_msg: &mut CircuitDirectMessage,
    reply_sender: &crossbeam_channel::Sender<SendRequest>,
    state: &Arc<Mutex<ServiceState>>,
) -> Result<(), ServiceError> {
    let mut nphase_transaction_msg: NPhaseTransactionMessage =
        protobuf::parse_from_bytes(circuit_msg.get_payload())?;

    match nphase_transaction_msg.get_message_type() {
        NPhaseTransactionMessage_Type::TRANSACTION_VERIFICATION_REQUEST => {
            let mut verification_request =
                nphase_transaction_msg.take_transaction_verification_request();

            let increment = read_u32(verification_request.get_transaction_payload())?;

            debug!("Received proposed increment of {}", increment);

            let response = {
                let mut state = state.lock().expect("Counter lock has been poisoned");
                let check_result = { hash(&write_u32(state.counter + increment)?) };
                let mut response = TransactionVerificationResponse::new();
                response.set_correlation_id(verification_request.take_correlation_id());

                if check_result != verification_request.get_expected_output_hash() {
                    debug!(
                        "Hash mismatch: expected {} but was {}",
                        to_hex(verification_request.get_expected_output_hash()),
                        to_hex(&check_result)
                    );
                    debug!(
                        "In our state: {} + {} = {}",
                        state.counter,
                        increment,
                        state.counter + increment
                    );
                    response.set_result(TransactionVerificationResponse_Result::MISMATCHED_OUTPUT);
                    response.set_output_hash(check_result);
                } else {
                    let prev = state.counter;
                    state.counter += increment;
                    debug!("Committed count increment: {} -> {}", prev, state.counter);

                    response.set_result(TransactionVerificationResponse_Result::VERIFIED);
                }

                response
            };
            let mut nphase_msg = NPhaseTransactionMessage::new();
            nphase_msg
                .set_message_type(NPhaseTransactionMessage_Type::TRANSACTION_VERIFICATION_RESPONSE);
            nphase_msg.set_transaction_verification_response(response);

            reply_sender.send(SendRequest::new(
                source_peer_id.to_string(),
                create_circuit_direct_msg(
                    circuit_msg.take_circuit(),
                    // The recipient was us, so set it as the sender
                    circuit_msg.take_recipient(),
                    // and vice-versa on the recipient of this message
                    circuit_msg.take_sender(),
                    nphase_msg.write_to_bytes()?,
                    circuit_msg.take_correlation_id(),
                )?,
            ))?;
        }
        NPhaseTransactionMessage_Type::TRANSACTION_VERIFICATION_RESPONSE => {
            let verification_response =
                nphase_transaction_msg.take_transaction_verification_response();

            let mut state = state.lock().expect("Counter lock has been poisoned");
            if let Some(increment) = state.proposed_increment.take() {
                if verification_response.get_result()
                    == TransactionVerificationResponse_Result::VERIFIED
                {
                    let prev = state.counter;
                    state.counter += increment;
                    debug!("Committed count increment: {} -> {}", prev, state.counter);
                } else {
                    warn!("Counter increment failed verification");
                }
            } else {
                warn!("Received verification when no pending transaction existed");
            }
        }
        NPhaseTransactionMessage_Type::UNSET_NPHASE_TRANSACTION_MESSAGE_TYPE => warn!(
            "Ignoring improperly specified n-phase message from {}",
            circuit_msg.get_recipient()
        ),
    }

    Ok(())
}

fn read_u32(bytes: &[u8]) -> Result<u32, ServiceError> {
    let mut input = protobuf::CodedInputStream::from_bytes(bytes);
    input.read_raw_varint32().map_err(ServiceError::from)
}

fn write_u32(value: u32) -> Result<Vec<u8>, ServiceError> {
    let mut buffer: Vec<u8> = Vec::new();
    let mut output = protobuf::CodedOutputStream::vec(&mut buffer);
    output.write_raw_varint32(value)?;

    Ok(buffer)
}

fn hash(bytes: &[u8]) -> Vec<u8> {
    Sha256::digest(bytes).as_slice().to_vec()
}

fn to_hex(bytes: &[u8]) -> String {
    let mut buf = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        write!(&mut buf, "{:0x}", b).unwrap(); // this can't fail
    }
    buf
}

fn create_connect_request() -> Result<Vec<u8>, ServiceError> {
    let mut connect_request = ConnectRequest::new();
    connect_request.set_handshake_mode(ConnectRequest_HandshakeMode::UNIDIRECTIONAL);
    wrap_in_network_auth_envelopes(AuthorizationMessageType::CONNECT_REQUEST, connect_request)
}

fn create_circuit_service_connect_request(
    circuit: &str,
    service_id: &str,
) -> Result<Vec<u8>, ServiceError> {
    let mut connect_request = ServiceConnectRequest::new();
    connect_request.set_circuit(circuit.to_string());
    connect_request.set_service_id(service_id.to_string());
    wrap_in_circuit_envelopes(CircuitMessageType::SERVICE_CONNECT_REQUEST, connect_request)
}

fn create_circuit_service_disconnect_request(
    circuit: &str,
    service_id: &str,
) -> Result<Vec<u8>, ServiceError> {
    let mut disconnect_request = ServiceDisconnectRequest::new();
    disconnect_request.set_circuit(circuit.to_string());
    disconnect_request.set_service_id(service_id.to_string());
    wrap_in_circuit_envelopes(
        CircuitMessageType::SERVICE_DISCONNECT_REQUEST,
        disconnect_request,
    )
}

fn create_circuit_direct_msg(
    circuit: String,
    sender: String,
    recipient: String,
    payload: Vec<u8>,
    correlation_id: String,
) -> Result<Vec<u8>, ServiceError> {
    let mut direct_msg = CircuitDirectMessage::new();
    direct_msg.set_circuit(circuit);
    direct_msg.set_sender(sender);
    direct_msg.set_recipient(recipient);
    direct_msg.set_payload(payload);
    direct_msg.set_correlation_id(correlation_id);

    wrap_in_circuit_envelopes(CircuitMessageType::CIRCUIT_DIRECT_MESSAGE, direct_msg)
}

fn wrap_in_circuit_envelopes<M: protobuf::Message>(
    msg_type: CircuitMessageType,
    msg: M,
) -> Result<Vec<u8>, ServiceError> {
    let mut circuit_msg = CircuitMessage::new();
    circuit_msg.set_message_type(msg_type);
    circuit_msg.set_payload(msg.write_to_bytes()?);

    wrap_in_network_msg(NetworkMessageType::CIRCUIT, circuit_msg)
}

fn wrap_in_network_auth_envelopes<M: protobuf::Message>(
    msg_type: AuthorizationMessageType,
    auth_msg: M,
) -> Result<Vec<u8>, ServiceError> {
    let mut auth_msg_env = AuthorizationMessage::new();
    auth_msg_env.set_message_type(msg_type);
    auth_msg_env.set_payload(auth_msg.write_to_bytes()?);

    wrap_in_network_msg(NetworkMessageType::AUTHORIZATION, auth_msg_env)
}

fn wrap_in_network_msg<M: protobuf::Message>(
    msg_type: NetworkMessageType,
    msg: M,
) -> Result<Vec<u8>, ServiceError> {
    let mut network_msg = NetworkMessage::new();
    network_msg.set_message_type(msg_type);
    network_msg.set_payload(msg.write_to_bytes()?);

    network_msg.write_to_bytes().map_err(ServiceError::from)
}

/// Return the appropriate transport for the current arguments
fn get_transport(matches: &clap::ArgMatches) -> Result<Box<dyn Transport + Send>, ServiceError> {
    match matches.value_of("transport") {
        Some("tls") => {
            let ca_file = matches
                .value_of("ca_file")
                .map(String::from)
                .ok_or_else(|| "Must provide a valid file containing ca certs".to_string())?;

            let client_cert = matches
                .value_of("client_cert")
                .map(String::from)
                .ok_or_else(|| "Must provide a valid client certificate".to_string())?;

            let client_key_file = matches
                .value_of("client_key")
                .map(String::from)
                .ok_or_else(|| "Must provide a valid key path".to_string())?;

            match TlsTransport::new(
                ca_file,
                client_key_file.clone(),
                client_cert.clone(),
                client_key_file,
                client_cert,
            ) {
                Ok(transport) => Ok(Box::new(transport)),
                Err(err) => Err(ServiceError(format!(
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

/// Validate that the given string is a properly formatted endpoint
fn valid_endpoint<S: AsRef<str>>(s: S) -> Result<(), String> {
    let s = s.as_ref();

    if s.is_empty() {
        return Err("Bind string must not be empty".into());
    }
    let mut parts = s.split(':');

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

/// Handle HTTP calls on the given stream
fn handle_connection(
    mut stream: TcpStream,
    state: Arc<Mutex<ServiceState>>,
    peer_id: String,
    circuit: String,
    service_id: String,
    verifiers: Vec<String>,
    sender: crossbeam_channel::Sender<SendRequest>,
) -> Result<(), HandleError> {
    let mut buffer = [0; 512];

    let _ = stream.read(&mut buffer)?;
    let request = String::from_utf8_lossy(&buffer[..]);

    let response = if request.starts_with("GET / ") {
        respond(200, "OK", Some("Private Counter Server"))
    } else if request.starts_with("GET /add/") {
        // get number to add to current value
        let addition = &request["GET /add/".len()..];
        if let Some(end) = addition.find(' ') {
            let addition = &addition[..end];
            // check that the value can be parsed into a u32
            if let Ok(i) = addition.parse::<u32>() {
                let mut state = state.lock().expect("Counter lock was poisoned");

                if state.proposed_increment.is_some() {
                    respond(
                        409,
                        "CONFLICT",
                        Some("There is already a pending transaction"),
                    )
                } else {
                    debug!("Proposing increment {}", i);

                    state.proposed_increment = Some(i);

                    let correlation_id = Uuid::new_v4().to_string();

                    let mut request = TransactionVerificationRequest::new();
                    request.set_correlation_id(correlation_id.clone());
                    request.set_transaction_payload(write_u32(i)?);
                    request.set_expected_output_hash(hash(&write_u32(state.counter + i)?));

                    let mut nphase_msg = NPhaseTransactionMessage::new();
                    nphase_msg.set_message_type(
                        NPhaseTransactionMessage_Type::TRANSACTION_VERIFICATION_REQUEST,
                    );
                    nphase_msg.set_transaction_verification_request(request);

                    for verifier in verifiers {
                        sender
                            .send(SendRequest::new(
                                peer_id.clone(),
                                create_circuit_direct_msg(
                                    circuit.clone(),
                                    service_id.clone(),
                                    verifier.clone(),
                                    nphase_msg.write_to_bytes().map_err(ServiceError::from)?,
                                    correlation_id.clone(),
                                )?,
                            ))
                            .map_err(ServiceError::from)?;
                    }
                    respond(204, "NO CONTENT", None)
                }
            } else {
                respond(400, "BAD REQUEST", None)
            }
        } else {
            respond(400, "BAD REQUEST", None)
        }
    } else if request.starts_with("GET /show") {
        // return current value
        let state = state.lock().expect("Counter lock was poisoned");
        respond(200, "OK", Some(&state.counter.to_string()))
    } else {
        respond(404, "NOT FOUND", None)
    };
    stream.write_all(response.as_bytes())?;
    stream.flush()?;

    Ok(())
}

fn respond(status_code: u16, status_msg: &str, content: Option<&str>) -> String {
    format!(
        "HTTP/1.1 {} {}\r\n\r\n{}",
        status_code,
        status_msg,
        content.unwrap_or("")
    )
}

fn configure_args<'a, 'b>() -> App<'a, 'b> {
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
