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

pub mod consensus;
mod error;

use std::convert::TryFrom;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread::Builder;
use std::time::Duration;

use crossbeam_channel;
use protobuf::Message;

use libsplinter::consensus::{ConsensusMessage, Proposal, ProposalUpdate};
use libsplinter::network::{
    sender::{NetworkMessageSender, SendRequest},
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
use libsplinter::protos::network::{NetworkMessage, NetworkMessageType};
use transact::protos::batch::Batch;

use crate::protos::private_xo::{PrivateXoMessage, PrivateXoMessage_Type};
pub use crate::service::error::ServiceError;

// Recv timeout in secs
const TIMEOUT_SEC: u64 = 2;

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

#[derive(Debug, Clone)]
pub struct ServiceConfig {
    peer_id: String,
    circuit: String,
    service_id: String,
    verifiers: Vec<String>,
}

impl ServiceConfig {
    pub fn new(
        peer_id: String,
        circuit: String,
        service_id: String,
        verifiers: Vec<String>,
    ) -> Self {
        ServiceConfig {
            peer_id,
            circuit,
            service_id,
            verifiers,
        }
    }

    pub fn peer_id(&self) -> &str {
        &self.peer_id
    }

    pub fn circuit(&self) -> &str {
        &self.circuit
    }

    pub fn service_id(&self) -> &str {
        &self.service_id
    }

    pub fn verifiers(&self) -> &[String] {
        &self.verifiers
    }
}

pub fn start_service_loop(
    service_config: ServiceConfig,
    channel: (
        crossbeam_channel::Sender<SendRequest>,
        crossbeam_channel::Receiver<SendRequest>,
    ),
    consensus_msg_sender: Sender<ConsensusMessage>,
    proposal_update_sender: Sender<ProposalUpdate>,
    pending_proposal: Arc<Mutex<Option<(Proposal, Batch)>>>,
    network: Network,
    running: Arc<AtomicBool>,
) -> Result<(), ServiceError> {
    info!("Starting Private Counter Service");
    let sender_network = network.clone();
    let (send, recv) = channel;

    let network_sender_run_flag = running.clone();
    let _ = Builder::new()
        .name("NetworkMessageSender".into())
        .spawn(move || {
            let network_sender =
                NetworkMessageSender::new(Box::new(recv), sender_network, network_sender_run_flag);
            network_sender.run()
        });

    let recv_network = network.clone();
    let reply_sender = send.clone();
    let _ = Builder::new()
        .name("NetworkReceiver".into())
        .spawn(move || {
            run_service_loop(
                recv_network,
                &reply_sender,
                consensus_msg_sender,
                proposal_update_sender,
                pending_proposal,
                service_config,
                running,
            )
        });

    let connect_request_msg_bytes = create_connect_request()
        .map_err(|err| ServiceError(format!("Unable to create connect request: {}", err)))?;
    for peer_id in network.peer_ids() {
        debug!("Sending connect request to peer {}", peer_id);
        network
            .send(&peer_id, &connect_request_msg_bytes)
            .map_err(|err| ServiceError(format!("Unable to send connect request: {:?}", err)))?;
    }

    Ok(())
}

#[allow(clippy::cognitive_complexity)]
fn run_service_loop(
    network: Network,
    reply_sender: &crossbeam_channel::Sender<SendRequest>,
    consensus_msg_sender: Sender<ConsensusMessage>,
    proposal_update_sender: Sender<ProposalUpdate>,
    pending_proposal: Arc<Mutex<Option<(Proposal, Batch)>>>,
    service_config: ServiceConfig,
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
                            &service_config.service_id,
                            &reply_sender
                        )) {
                            info!("Successfully authorized with peer {}", message.peer_id());

                            unwrap_or_break!(network.send(
                                message.peer_id(),
                                &unwrap_or_break!(create_circuit_service_connect_request(
                                    &service_config.circuit,
                                    &service_config.service_id
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
                            &consensus_msg_sender,
                            &proposal_update_sender,
                            &pending_proposal,
                        ));
                    }
                    _ => {
                        debug!("Ignoring message of type {:?}", msg.get_message_type());
                    }
                };
            }
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => {
                warn!("Network disconnected");
                break;
            }
            Err(err) => debug!("Error: {:?}", err),
        }
    }

    let disconnect_msg = create_circuit_service_disconnect_request(
        &service_config.circuit,
        &service_config.service_id,
    )
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

    // This is a bit of a gross work-around the fact that Rocket can't be signaled to shutdown
    // gracefully.  We'll give the disconnect message a little time to arive in the splinter node
    // and exit the process.
    std::thread::sleep(std::time::Duration::from_millis(100));
    std::process::exit(0);
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
    consensus_msg_sender: &Sender<ConsensusMessage>,
    proposal_update_sender: &Sender<ProposalUpdate>,
    pending_proposal: &Arc<Mutex<Option<(Proposal, Batch)>>>,
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
            handle_direct_msg(
                source_peer_id,
                &mut msg,
                consensus_msg_sender,
                proposal_update_sender,
                pending_proposal,
            )?;
        }
        _ => debug!("Received message {:?}", circuit_msg),
    }
    Ok(())
}

/// This handles the messages that are specifically targeting this service.
fn handle_direct_msg(
    source_peer_id: &str,
    circuit_msg: &mut CircuitDirectMessage,
    consensus_msg_sender: &Sender<ConsensusMessage>,
    proposal_update_sender: &Sender<ProposalUpdate>,
    pending_proposal: &Arc<Mutex<Option<(Proposal, Batch)>>>,
) -> Result<(), ServiceError> {
    let private_xo_message: PrivateXoMessage =
        protobuf::parse_from_bytes(circuit_msg.get_payload())?;

    match private_xo_message.get_message_type() {
        PrivateXoMessage_Type::CONSENSUS_MESSAGE => {
            consensus_msg_sender.send(ConsensusMessage::try_from(
                private_xo_message.get_consensus_message(),
            )?)?;
        }
        PrivateXoMessage_Type::PROPOSED_BATCH => {
            let proposed_batch = private_xo_message.get_proposed_batch();

            let expected_hash = proposed_batch.get_expected_hash().to_vec();
            let batch: Batch = protobuf::parse_from_bytes(proposed_batch.get_batch())?;

            let mut proposal = Proposal::default();
            proposal.id = expected_hash.into();

            *pending_proposal
                .lock()
                .expect("pending proposal lock poisoned") = Some((proposal.clone(), batch));

            proposal_update_sender.send(ProposalUpdate::ProposalReceived(
                proposal,
                circuit_msg.get_sender().as_bytes().into(),
            ))?;
        }
        PrivateXoMessage_Type::UNSET => warn!(
            "ignoring improperly specified private xo message from {:?}",
            source_peer_id,
        ),
    }

    Ok(())
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

pub fn create_circuit_direct_msg<M: protobuf::Message>(
    circuit: String,
    sender: String,
    recipient: String,
    payload: &M,
    correlation_id: String,
) -> Result<Vec<u8>, ServiceError> {
    let mut direct_msg = CircuitDirectMessage::new();
    direct_msg.set_circuit(circuit);
    direct_msg.set_sender(sender);
    direct_msg.set_recipient(recipient);
    direct_msg.set_payload(payload.write_to_bytes()?);
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

impl From<protobuf::ProtobufError> for ServiceError {
    fn from(err: protobuf::ProtobufError) -> Self {
        ServiceError(format!("Protocol Buffer Error: {}", err))
    }
}

impl<T> From<crossbeam_channel::SendError<T>> for ServiceError {
    fn from(err: crossbeam_channel::SendError<T>) -> Self {
        ServiceError(format!("Unable to send: {}", err))
    }
}
