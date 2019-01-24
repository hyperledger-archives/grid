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
use libsplinter::protos::circuit::{CircuitMessage, CircuitMessageType, ServiceConnectRequest};
use libsplinter::protos::network::{NetworkEcho, NetworkMessage, NetworkMessageType};
use protobuf::Message;
use std::env;

use splinter_client::{error::SplinterError, Certs, SplinterClient};

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

fn get_certs() -> Certs {
    let ca_certs = if let Ok(s) = env::var("SPLINTER_CA_CERTS") {
        s.to_string()
    } else {
        "ca.crt".to_string()
    };

    let client_cert = if let Ok(s) = env::var("SPLINTER_CLIENT_CERTS") {
        s.to_string()
    } else {
        "client.crt".to_string()
    };

    let client_priv = if let Ok(s) = env::var("SPLINTER_CLIENT_SECRET") {
        s.to_string()
    } else {
        "client.key".to_string()
    };

    Certs::new(ca_certs, client_cert, client_priv)
}
