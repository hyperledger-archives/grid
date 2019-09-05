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
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use clap::ArgMatches;
use libsplinter::protos::circuit::{
    CircuitDirectMessage, CircuitMessage, CircuitMessageType, ServiceConnectRequest,
    ServiceDisconnectRequest,
};
use libsplinter::protos::network::{NetworkMessage, NetworkMessageType};
use protobuf::Message;
use splinter_client::SplinterClient;

use crate::cert::get_certs;
use crate::error::CliError;

use super::Action;

pub struct ConnectAction;

impl Action for ConnectAction {
    fn run<'a>(&mut self, arg_matches: Option<&ArgMatches<'a>>) -> Result<(), CliError> {
        let matches = arg_matches.ok_or_else(|| CliError::RequiresArgs)?;
        let url = matches.value_of("url").unwrap_or("tcp://localhost:8045");
        let circuit = matches
            .value_of("circuit")
            .ok_or_else(|| CliError::MissingArg("circuit".into()))?;
        let service = matches
            .value_of("service")
            .ok_or_else(|| CliError::MissingArg("service".into()))?;

        let msg = {
            let mut connect_request = ServiceConnectRequest::new();
            connect_request.set_circuit(circuit.into());
            connect_request.set_service_id(service.into());
            let connect_bytes = connect_request.write_to_bytes().unwrap();

            let mut circuit_msg = CircuitMessage::new();
            circuit_msg.set_message_type(CircuitMessageType::SERVICE_CONNECT_REQUEST);
            circuit_msg.set_payload(connect_bytes);
            let circuit_bytes = circuit_msg.write_to_bytes().unwrap();

            let mut network_msg = NetworkMessage::new();
            network_msg.set_message_type(NetworkMessageType::CIRCUIT);
            network_msg.set_payload(circuit_bytes);

            network_msg
        };

        let mut conn = SplinterClient::connect(url, get_certs())?;

        conn.send(&msg).map(|_| ()).map_err(CliError::from)
    }
}

pub struct DisconnectAction;

impl Action for DisconnectAction {
    fn run<'a>(&mut self, arg_matches: Option<&ArgMatches<'a>>) -> Result<(), CliError> {
        let matches = arg_matches.ok_or_else(|| CliError::RequiresArgs)?;
        let url = matches.value_of("url").unwrap_or("tcp://localhost:8045");
        let circuit = matches
            .value_of("circuit")
            .ok_or_else(|| CliError::MissingArg("circuit".into()))?;
        let service = matches
            .value_of("service")
            .ok_or_else(|| CliError::MissingArg("service".into()))?;

        let msg = {
            let mut disconnect_request = ServiceDisconnectRequest::new();
            disconnect_request.set_circuit(circuit.into());
            disconnect_request.set_service_id(service.into());
            let disconnect_bytes = disconnect_request.write_to_bytes().unwrap();

            let mut circuit_msg = CircuitMessage::new();
            circuit_msg.set_message_type(CircuitMessageType::SERVICE_DISCONNECT_REQUEST);
            circuit_msg.set_payload(disconnect_bytes);
            let circuit_bytes = circuit_msg.write_to_bytes().unwrap();

            let mut network_msg = NetworkMessage::new();
            network_msg.set_message_type(NetworkMessageType::CIRCUIT);
            network_msg.set_payload(circuit_bytes);

            network_msg
        };

        let mut conn = SplinterClient::connect(url, get_certs())?;

        conn.send(&msg).map(|_| ()).map_err(CliError::from)
    }
}

pub struct SendAction;

impl Action for SendAction {
    fn run<'a>(&mut self, arg_matches: Option<&ArgMatches<'a>>) -> Result<(), CliError> {
        let matches = arg_matches.ok_or_else(|| CliError::RequiresArgs)?;
        let url = matches.value_of("url").unwrap_or("tcp://localhost:8045");
        let circuit = matches
            .value_of("circuit")
            .ok_or_else(|| CliError::MissingArg("circuit".into()))?;
        let sender = matches
            .value_of("sender")
            .ok_or_else(|| CliError::MissingArg("sender".into()))?;
        let recipient = matches
            .value_of("recipient")
            .ok_or_else(|| CliError::MissingArg("recipient".into()))?;
        let payload_path = matches
            .value_of("payload")
            .ok_or_else(|| CliError::MissingArg("payload".into()))?;

        let msg = {
            let file = File::open(payload_path).map_err(|err| {
                CliError::ActionError(format!("Unable to open {}: {}", payload_path, err))
            })?;
            let mut buf_reader = BufReader::new(file);
            let mut contents = Vec::new();
            buf_reader.read_to_end(&mut contents).map_err(|err| {
                CliError::ActionError(format!("Unable to read {}: {}", payload_path, err))
            })?;

            let mut direct_msg = CircuitDirectMessage::new();
            direct_msg.set_circuit(circuit.into());
            direct_msg.set_sender(sender.into());
            direct_msg.set_recipient(recipient.into());
            direct_msg.set_payload(contents);

            let connect_bytes = direct_msg.write_to_bytes().unwrap();

            let mut circuit_msg = CircuitMessage::new();
            circuit_msg.set_message_type(CircuitMessageType::CIRCUIT_DIRECT_MESSAGE);
            circuit_msg.set_payload(connect_bytes);
            let circuit_bytes = circuit_msg.write_to_bytes().unwrap();

            let mut network_msg = NetworkMessage::new();
            network_msg.set_message_type(NetworkMessageType::CIRCUIT);
            network_msg.set_payload(circuit_bytes);

            network_msg
        };

        let mut conn = SplinterClient::connect(url, get_certs())?;

        conn.send(&msg).map(|_| ()).map_err(CliError::from)
    }
}
