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
use libsplinter::protos::circuit::{
    CircuitDirectMessage, CircuitMessage, CircuitMessageType, ServiceConnectRequest,
    ServiceDisconnectRequest,
};
use libsplinter::protos::network::{NetworkEcho, NetworkMessage, NetworkMessageType};
use protobuf::Message;
use splinter_client::{error::SplinterError, SplinterClient};

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use crate::cert::get_certs;

pub fn do_echo(url: &str, recipient: String, ttl: i32) -> Result<(), SplinterError> {
    let msg = {
        let mut echo = NetworkEcho::new();
        echo.set_payload(b"HelloWorld".to_vec());
        echo.set_recipient(recipient);
        echo.set_time_to_live(ttl);
        let echo_bytes = echo.write_to_bytes()?;

        let mut network_msg = NetworkMessage::new();
        network_msg.set_message_type(NetworkMessageType::NETWORK_ECHO);
        network_msg.set_payload(echo_bytes);

        network_msg
    };

    let mut conn = SplinterClient::connect(url, get_certs())?;

    conn.send(&msg).map(|_| ())
}

pub fn do_connect(url: &str, circuit: String, service: String) -> Result<(), SplinterError> {
    let msg = {
        let mut connect_request = ServiceConnectRequest::new();
        connect_request.set_circuit(circuit);
        connect_request.set_service_id(service);
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

    conn.send(&msg).map(|_| ())
}

pub fn do_disconnect(url: &str, circuit: String, service: String) -> Result<(), SplinterError> {
    let msg = {
        let mut disconnect_request = ServiceDisconnectRequest::new();
        disconnect_request.set_circuit(circuit);
        disconnect_request.set_service_id(service);
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

    conn.send(&msg).map(|_| ())
}

pub fn do_send(
    url: &str,
    circuit: String,
    sender: String,
    recipient: String,
    payload_path: String,
) -> Result<(), SplinterError> {
    let msg = {
        let file = File::open(&payload_path)?;
        let mut buf_reader = BufReader::new(file);
        let mut contents = Vec::new();
        buf_reader.read_to_end(&mut contents)?;

        let mut direct_msg = CircuitDirectMessage::new();
        direct_msg.set_circuit(circuit);
        direct_msg.set_sender(sender);
        direct_msg.set_recipient(recipient);
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

    conn.send(&msg).map(|_| ())
}
