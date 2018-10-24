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

use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use protobuf;
use messaging::protocol::{
    Message,
    MessageType,
    CreateCircuitRequest,
    DestroyCircuitRequest,
    GossipMessageRequest
};
use libsplinter::SplinterError;
use splinter_client::{
    SplinterClient,
    Certs
};

pub fn do_create_circuit(
    url: &str,
    name: &str,
    participants: Vec<String>
) -> Result<(), SplinterError> {
    let msg = {
        let mut req = CreateCircuitRequest::new(); 
        req.set_name(name.to_string());
        req.set_participants(protobuf::RepeatedField::from_vec(participants));

        let mut m = Message::new();
        m.set_message_type(MessageType::CREATE_CIRCUIT_REQUEST);
        m.set_create_circuit_request(req);

        m
    };

    let mut conn = SplinterClient::connect(url, get_certs())?;

    conn.send(&msg).map(|_| ())
}

pub fn do_destroy_circuit(
    url: &str,
    name: &str
) -> Result<(), SplinterError> {
    let msg = {
        let mut req = DestroyCircuitRequest::new(); 
        req.set_name(name.to_string());

        let mut m = Message::new();
        m.set_message_type(MessageType::DESTROY_CIRCUIT_REQUEST);
        m.set_destroy_circuit_request(req);

        m
    };

    let mut conn = SplinterClient::connect(url, get_certs())?;

    conn.send(&msg).map(|_| ())
}

pub fn do_gossip(
    url: &str,
    name: &str,
    payload_file: &str
) -> Result<(), SplinterError> {
    let payload = {
        let mut b = Vec::new();
        File::open(payload_file)?
            .read_to_end(&mut b)?;
        b
    };

    let msg = {
        let mut req = GossipMessageRequest::new();
        req.set_name(name.to_string());
        req.set_payload(payload);

        let mut m = Message::new();
        m.set_message_type(MessageType::GOSSIP_MESSAGE_REQUEST);
        m.set_gossip_message_request(req);

        m
    };

    let mut conn = SplinterClient::connect(url, get_certs())?;

    conn.send(&msg).map(|_| ())
}

fn get_certs() -> Certs {
    let ca_certs = if let Ok(s) = env::var("SPLINTER_CA_CERTS") {
        s.split(",")
            .map(PathBuf::from)
            .collect()
    } else {
        vec![PathBuf::from("ca.crt")]
    };

    let client_cert = if let Ok(s) = env::var("SPLINTER_CLIENT_CERTS") {
        PathBuf::from(s)
    } else {
        PathBuf::from("client.crt")
    };

    let client_priv = if let Ok(s) = env::var("SPLINTER_CLIENT_SECRET") {
        PathBuf::from(s)
    } else {
        PathBuf::from("client.key")
    };

    Certs::new(ca_certs, client_cert, client_priv)
}
