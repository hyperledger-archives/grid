/*
 * Copyright 2019 Cargill Incorporated
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * -----------------------------------------------------------------------------
 */

use sawtooth_sdk::messaging::{
    stream::{MessageConnection, MessageReceiver, MessageSender},
    zmq_stream::{ZmqMessageConnection, ZmqMessageSender},
};

pub struct SawtoothConnection {
    sender: ZmqMessageSender,
    receiver: MessageReceiver,
}

impl SawtoothConnection {
    pub fn new(validator_address: &str) -> SawtoothConnection {
        let zmq_connection = ZmqMessageConnection::new(&validator_address);
        let (sender, receiver) = zmq_connection.create();
        SawtoothConnection { sender, receiver }
    }

    pub fn get_sender(&self) -> Box<dyn MessageSender + Send> {
        Box::new(self.sender.clone())
    }

    pub fn get_receiver(&self) -> &MessageReceiver {
        &self.receiver
    }
}
