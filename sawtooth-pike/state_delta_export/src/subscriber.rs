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

extern crate chan_signal;

use protobuf;
use uuid;

use sawtooth_sdk::messaging::stream::MessageSender;
use sawtooth_sdk::messaging::stream::MessageConnection;
use sawtooth_sdk::messaging::zmq_stream::{ZmqMessageConnection};
use sawtooth_sdk::messages::events::{EventSubscription, EventFilter, EventFilter_FilterType, EventList};
use sawtooth_sdk::messages::client_event::{ClientEventsSubscribeRequest, ClientEventsSubscribeResponse};
use sawtooth_sdk::messages::client_event::ClientEventsSubscribeResponse_Status;
use sawtooth_sdk::messages::validator::Message_MessageType::CLIENT_EVENTS_SUBSCRIBE_REQUEST;

const PIKE_NAMESPACE: &'static str = "cad11d";
const NULL_BLOCK_ID: &'static str = "0000000000000000";

#[derive(Clone, Copy)]
pub struct Subscriber {
    is_active: bool
}

impl Subscriber {
    pub fn new() -> Subscriber {
        Subscriber {
            is_active: false,
        }
    }

    pub fn start<F>(&mut self, validator: String, handler: F) where F : Fn(EventList) {

        // Connect to Validator

        info!("Establishing connection with validator {}", validator);

        let connection = ZmqMessageConnection::new(&validator);
        let (sender, receiver) = connection.create();

        // Build event subscription payload

        let mut state_delta_sub = EventSubscription::new();
        state_delta_sub.set_event_type(String::from("sawtooth/state-delta"));

        let mut block_commit_sub = EventSubscription::new();
        block_commit_sub.set_event_type(String::from("sawtooth/block-commit"));

        let mut event_filter = EventFilter::new();
        event_filter.set_key(String::from("address"));
        event_filter.set_match_string(format!("^{}.*", PIKE_NAMESPACE));
        event_filter.set_filter_type(EventFilter_FilterType::REGEX_ANY);

        state_delta_sub.set_filters(
            protobuf::RepeatedField::from_vec(vec![event_filter]));

        // Build subscription request payload

        let mut subscription_request = ClientEventsSubscribeRequest::new();

        subscription_request.set_last_known_block_ids(
            protobuf::RepeatedField::from_vec(vec![String::from(NULL_BLOCK_ID)]));
        subscription_request.set_subscriptions(
            protobuf::RepeatedField::from_vec(vec![state_delta_sub, block_commit_sub]));

        // Attempt to send state delta subscription request

        let msg_bytes = match protobuf::Message::write_to_bytes(&subscription_request) {
            Ok(b) => b,
            Err(error) => {
                error!("Error serializing request: {:?}", error);
                return;
            },
        };

        let correlation_id = match uuid::Uuid::new(uuid::UuidVersion::Random) {
            Some(cid) => cid.to_string(),
            None => {
                error!("Error generating UUID");
                return;
            },
        };

        let mut future = match sender.send(CLIENT_EVENTS_SUBSCRIBE_REQUEST, &correlation_id, &msg_bytes) {
            Ok(f) => f,
            Err(error) => {
                error!("Error unwrapping future: {:?}", error);
                return;
            },
        };

        let response_msg = match future.get() {
            Ok(m) => m,
            Err(error) => {
                error!("Error getting future: {:?}", error);
                return;
            },
        };

        let response: ClientEventsSubscribeResponse = match protobuf::parse_from_bytes(&response_msg.content) {
            Ok(r) => r,
            Err(error) => {
                error!("Error parsing response: {:?}", error);
                return;
            },
        };

        // Validate response

        if !(response.status == ClientEventsSubscribeResponse_Status::OK) {
            error!("subscription status: {:?}", response.status);
            return;
        }

        info!("Successfully subscribed to validator state delta events");

        // Listen for state delta events

        self.is_active = true;

        while self.is_active {
            let content = match receiver.recv().unwrap() {
                Ok(res) => res.content,
                Err(err) => {
                    error!("An error occurred while attempting to receive message {:?}", err);
                    continue;
                }
            };

            let event_list = match protobuf::parse_from_bytes(&content) {
                Ok(l) => l,
                Err(err) => {
                    error!("An error occured while attempting to receive message {:?}", err);
                    continue;
                }
            };

            handler(event_list);
        }
    }

    pub fn stop(&mut self) {
        if self.is_active {
            self.is_active = false;
        }
    }
}
