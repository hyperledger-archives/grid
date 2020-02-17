// Copyright 2018-2020 Cargill Incorporated
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
use crate::channel::Sender;
use crate::network::dispatch::{DispatchError, Handler, MessageContext};
use crate::network::sender::SendRequest;
use crate::protos::network::{NetworkEcho, NetworkHeartbeat, NetworkMessage, NetworkMessageType};

use protobuf::Message;

// Implements a handler that handles NetworkEcho Messages
pub struct NetworkEchoHandler {
    node_id: String,
}

impl Handler<NetworkMessageType, NetworkEcho> for NetworkEchoHandler {
    fn handle(
        &self,
        mut msg: NetworkEcho,
        context: &MessageContext<NetworkMessageType>,
        sender: &dyn Sender<SendRequest>,
    ) -> Result<(), DispatchError> {
        debug!("ECHO: {:?}", msg);

        let recipient = {
            // if the recipient is us forward back to sender else forward on to the intended
            // recipient
            if msg.get_recipient() == self.node_id {
                context.source_peer_id().to_string()
            } else {
                msg.get_recipient().to_string()
            }
        };

        msg.set_time_to_live(msg.get_time_to_live() - 1);
        if msg.get_time_to_live() <= 0 {
            return Ok(());
        };

        let echo_bytes = msg.write_to_bytes().unwrap();

        let mut network_msg = NetworkMessage::new();
        network_msg.set_message_type(NetworkMessageType::NETWORK_ECHO);
        network_msg.set_payload(echo_bytes);
        let network_msg_bytes = network_msg.write_to_bytes().unwrap();

        let send_request = SendRequest::new(recipient, network_msg_bytes);
        sender.send(send_request)?;
        Ok(())
    }
}

impl NetworkEchoHandler {
    pub fn new(node_id: String) -> Self {
        NetworkEchoHandler { node_id }
    }
}

// Implements a handler that handles NetworkHeartbeat Messages
#[derive(Default)]
pub struct NetworkHeartbeatHandler {}

impl Handler<NetworkMessageType, NetworkHeartbeat> for NetworkHeartbeatHandler {
    fn handle(
        &self,
        _msg: NetworkHeartbeat,
        context: &MessageContext<NetworkMessageType>,
        _sender: &dyn Sender<SendRequest>,
    ) -> Result<(), DispatchError> {
        trace!("Received Heartbeat from {}", context.source_peer_id());
        Ok(())
    }
}

impl NetworkHeartbeatHandler {
    pub fn new() -> Self {
        NetworkHeartbeatHandler {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channel::mock::MockSender;
    use crate::network::dispatch::Dispatcher;
    use crate::protos::network::{NetworkEcho, NetworkMessageType};

    #[test]
    fn dispatch_to_handler() {
        let sender = Box::new(MockSender::default());
        let mut dispatcher = Dispatcher::new(sender.box_clone());

        let handler = NetworkEchoHandler::new("TestPeer".to_string());

        dispatcher.set_handler(NetworkMessageType::NETWORK_ECHO, Box::new(handler));

        let msg = {
            let mut echo = NetworkEcho::new();
            echo.set_payload(b"HelloWorld".to_vec());
            echo.set_recipient("TestPeer".to_string());
            echo.set_time_to_live(3);
            echo
        };

        let outgoing_message_bytes = msg.write_to_bytes().unwrap();

        assert_eq!(
            Ok(()),
            dispatcher.dispatch(
                "OTHER_PEER",
                &NetworkMessageType::NETWORK_ECHO,
                outgoing_message_bytes.clone()
            )
        );

        let send_request = sender.sent().get(0).unwrap().clone();

        assert_eq!(send_request.recipient(), "OTHER_PEER");
        let network_msg: NetworkMessage =
            protobuf::parse_from_bytes(send_request.payload()).unwrap();
        let echo: NetworkEcho = protobuf::parse_from_bytes(network_msg.get_payload()).unwrap();

        assert_eq!(echo.get_recipient(), "TestPeer");
        assert_eq!(echo.get_time_to_live(), 2);
        assert_eq!(echo.get_payload().to_vec(), b"HelloWorld".to_vec());
    }
}
