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
use ::log::{log, warn};

use crate::channel::{Receiver, RecvError};
use crate::network::Network;

// Message to send to the network message sender with the recipient and payload
#[derive(Clone, Debug)]
pub struct SendRequest {
    recipient: String,
    payload: Vec<u8>,
}

impl SendRequest {
    pub fn new(recipient: String, payload: Vec<u8>) -> Self {
        SendRequest { recipient, payload }
    }

    pub fn recipient(&self) -> &str {
        &self.recipient
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
}

// The NetworkMessageSender recv messages that should be sent over the network. The Sender side of
// the channel will be passed to handlers.
pub struct NetworkMessageSender {
    rc: Box<Receiver<SendRequest>>,
    network: Network,
}

impl NetworkMessageSender {
    pub fn new(rc: Box<Receiver<SendRequest>>, network: Network) -> Self {
        NetworkMessageSender { rc, network }
    }

    pub fn run(&self) -> Result<(), NetworkMessageSenderError> {
        loop {
            let send_request = self.rc.recv()?;
            match self
                .network
                .send(send_request.recipient().into(), send_request.payload())
            {
                Ok(_) => (),
                Err(err) => warn!("Unable to send message: {:?}", err),
            };
        }
    }
}

#[derive(Debug)]
pub enum NetworkMessageSenderError {
    RecvError(String),
}

impl From<RecvError> for NetworkMessageSenderError {
    fn from(recv_error: RecvError) -> Self {
        NetworkMessageSenderError::RecvError(format!("Recv Error: {:?}", recv_error))
    }
}

#[cfg(test)]
mod tests {
    use crate::channel::{Receiver, Sender};
    use crossbeam_channel;

    use std::sync::mpsc;
    use std::thread;

    use super::*;
    use crate::mesh::Mesh;
    use crate::network::Network;
    use crate::transport::raw::RawTransport;
    use crate::transport::Transport;

    // Test that a message can successfully be sent by passing it to the sender end of the
    // NetworkMessageSender channel, recv the message, and then send it over the network.
    fn test_network_message_sender(
        sender: Box<dyn Sender<SendRequest>>,
        receiver: Box<dyn Receiver<SendRequest>>,
    ) {
        let mut transport = RawTransport::default();
        let mut listener = transport.listen("127.0.0.1:0").unwrap();
        let endpoint = listener.endpoint();

        let mesh1 = Mesh::new(1, 1);
        let mut network1 = Network::new(mesh1.clone());

        let network_message_sender = NetworkMessageSender::new(receiver, network1.clone());

        thread::spawn(move || {
            let mesh2 = Mesh::new(1, 1);
            let mut network2 = Network::new(mesh2.clone());
            let connection = listener.accept().unwrap();
            network2.add_peer("ABC".to_string(), connection).unwrap();
            let network_message = network2.recv().unwrap();
            assert_eq!(network_message.peer_id(), "ABC".to_string());
            assert_eq!(
                network_message.payload().to_vec(),
                b"FromNetworkMessageSender".to_vec()
            );
        });

        let connection = transport.connect(&endpoint).unwrap();
        network1.add_peer("123".to_string(), connection).unwrap();

        thread::spawn(move || network_message_sender.run());

        let send_request =
            SendRequest::new("123".to_string(), b"FromNetworkMessageSender".to_vec());
        sender.send(send_request).unwrap();
    }

    // Test that a messages can successfully be sent by passing it to the sender end of the
    // NetworkMessageSender channel, recv the message, and then send it over the network.
    fn test_network_message_sender_rapid_fire(
        sender: Box<dyn Sender<SendRequest>>,
        receiver: Box<dyn Receiver<SendRequest>>,
    ) {
        let mut transport = RawTransport::default();
        let mut listener = transport.listen("127.0.0.1:0").unwrap();
        let endpoint = listener.endpoint();

        let mesh1 = Mesh::new(5, 5);
        let mut network1 = Network::new(mesh1.clone());

        let network_message_sender = NetworkMessageSender::new(receiver, network1.clone());

        thread::spawn(move || {
            let mesh2 = Mesh::new(5, 5);
            let mut network2 = Network::new(mesh2.clone());
            let connection = listener.accept().unwrap();
            network2.add_peer("ABC".to_string(), connection).unwrap();
            for _ in 0..100 {
                let network_message = network2.recv().unwrap();
                assert_eq!(network_message.peer_id(), "ABC".to_string());
                assert_eq!(
                    network_message.payload().to_vec(),
                    b"FromNetworkMessageSender".to_vec()
                );
            }
        });

        let connection = transport.connect(&endpoint).unwrap();
        network1.add_peer("123".to_string(), connection).unwrap();

        thread::spawn(move || network_message_sender.run());

        let send_request =
            SendRequest::new("123".to_string(), b"FromNetworkMessageSender".to_vec());

        for _ in 0..100 {
            sender.send(send_request.clone()).unwrap();
        }
    }

    #[test]
    fn test_receiver_crossbeam() {
        let (send, recv) = crossbeam_channel::bounded(5);
        // test that send is cloneable.
        let send_box = Box::new(send);
        let send_clone = send_box.clone();
        test_network_message_sender(send_clone, Box::new(recv));
    }

    #[test]
    fn test_receiver_mpsc() {
        let (send, recv) = mpsc::channel();
        test_network_message_sender(Box::new(send), Box::new(recv));
    }

    #[test]
    fn test_receiver_crossbeam_rapid_fire() {
        let (send, recv) = crossbeam_channel::bounded(5);
        test_network_message_sender_rapid_fire(Box::new(send), Box::new(recv));
    }

    #[test]
    fn test_receiver_mpsc_rapid_fire() {
        let (send, recv) = mpsc::channel();
        test_network_message_sender_rapid_fire(Box::new(send), Box::new(recv));
    }

}
