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

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use protobuf::Message as _;

use sawtooth_sdk::{
    messages::{
        client_event::{
            ClientEventsSubscribeRequest, ClientEventsSubscribeResponse,
            ClientEventsSubscribeResponse_Status,
        },
        events::{Event, EventFilter, EventFilter_FilterType, EventList, EventSubscription},
        validator::{Message, Message_MessageType},
    },
    messaging::{
        stream::{MessageSender, ReceiveError, SendError},
        zmq_stream::ZmqMessageSender,
    },
};

use crate::event::{EventConnection, EventConnectionUnsubscriber, EventIoError};

use super::connection::SawtoothConnection;

const SHUTDOWN_TIMEOUT: u64 = 2;

impl EventConnection for SawtoothConnection {
    type Unsubscriber = SawtoothEventUnsubscriber;

    fn name(&self) -> &str {
        "sawtooth-validator"
    }

    fn subscribe(
        &self,
        namespaces: &[&str],
        last_commit_id: &str,
    ) -> Result<Self::Unsubscriber, EventIoError> {
        let message_sender = self.get_sender();

        let request = create_subscription_request(last_commit_id, namespaces);
        let mut future = message_sender.send(
            Message_MessageType::CLIENT_EVENTS_SUBSCRIBE_REQUEST,
            &correlation_id(),
            &request.write_to_bytes()?,
        )?;

        let response: ClientEventsSubscribeResponse = content_of_type(
            Message_MessageType::CLIENT_EVENTS_SUBSCRIBE_RESPONSE,
            future.get()?,
        )?;

        if response.get_status() != ClientEventsSubscribeResponse_Status::OK {
            return Err(EventIoError(format!(
                "Failed to subscribe for events: {:?} {}",
                response.get_status(),
                response.get_response_message()
            )));
        }
        Ok(SawtoothEventUnsubscriber { message_sender })
    }

    fn recv(&self) -> Result<Vec<Event>, EventIoError> {
        match self.get_receiver().recv() {
            Ok(Ok(msg)) => extract_events(msg),
            Ok(Err(ReceiveError::DisconnectedError)) => {
                Err(EventIoError(format!("{} has disconnected", self.name())))
            }
            Ok(Err(err)) => Err(EventIoError(err.to_string())),
            Err(err) => Err(EventIoError(err.to_string())),
        }
    }

    fn close(self) -> Result<(), EventIoError> {
        self.get_sender().close();

        Ok(())
    }
}

pub struct SawtoothEventUnsubscriber {
    message_sender: ZmqMessageSender,
}

impl EventConnectionUnsubscriber for SawtoothEventUnsubscriber {
    fn unsubscribe(self) -> Result<(), EventIoError> {
        let correlation_id = correlation_id();
        debug!(
            "Sending event unsubscribe request to sawtooth-validator ({})",
            correlation_id
        );
        match self
            .message_sender
            .send(
                Message_MessageType::CLIENT_EVENTS_UNSUBSCRIBE_REQUEST,
                &correlation_id,
                &[], // An unsubscribe request has no content
            )
            .map_err(|err| EventIoError(format!("Unable to send unsubscribe request: {}", err)))?
            .get_timeout(Duration::from_secs(SHUTDOWN_TIMEOUT))
        {
            Ok(msg) => {
                if msg.get_message_type() == Message_MessageType::CLIENT_EVENTS_UNSUBSCRIBE_RESPONSE
                {
                    debug!("Successfully unsubscribed");
                } else {
                    debug!("During unsubscribe, received {:?}", msg.get_message_type());
                }
            }
            Err(ReceiveError::TimeoutError) => {
                debug!("Timeout occurred while waiting for unsubscribe response; ignoring")
            }
            Err(err) => return Err(EventIoError::from(err)),
        }

        Ok(())
    }
}

fn content_of_type<M: protobuf::Message>(
    expected_type: Message_MessageType,
    msg: Message,
) -> Result<M, EventIoError> {
    if msg.get_message_type() != expected_type {
        return Err(EventIoError(format!(
            "Unexpected message type: expected {:?} but was {:?}",
            expected_type,
            msg.get_message_type()
        )));
    }

    protobuf::parse_from_bytes(msg.get_content()).map_err(EventIoError::from)
}

fn create_subscription_request(
    last_known_block_id: &str,
    namespace_filters: &[&str],
) -> ClientEventsSubscribeRequest {
    let mut block_info_subscription = EventSubscription::new();
    block_info_subscription.set_event_type("sawtooth/block-commit".into());
    let mut request = ClientEventsSubscribeRequest::new();
    request.mut_subscriptions().push(block_info_subscription);

    for namespace in namespace_filters {
        request
            .mut_subscriptions()
            .push(make_event_filter(namespace));
    }

    request
        .mut_last_known_block_ids()
        .push(last_known_block_id.into());

    request
}

fn make_event_filter(namespace: &str) -> EventSubscription {
    let mut filter = EventFilter::new();
    filter.set_filter_type(EventFilter_FilterType::REGEX_ANY);
    filter.set_key("address".into());
    filter.set_match_string(format!("^{}.*", namespace));

    let mut event_subscription = EventSubscription::new();
    event_subscription.set_event_type("sawtooth/state-delta".into());
    event_subscription.mut_filters().push(filter);

    event_subscription
}

fn extract_events(msg: Message) -> Result<Vec<Event>, EventIoError> {
    if msg.get_message_type() != Message_MessageType::CLIENT_EVENTS {
        warn!("Received unexpected message: {:?}", msg.get_message_type());
        return Ok(vec![]);
    }

    match protobuf::parse_from_bytes::<EventList>(msg.get_content()) {
        Ok(mut event_list) => Ok(event_list.take_events().to_vec()),
        Err(err) => {
            warn!("Unable to parse event list; ignoring: {}", err);
            Ok(vec![])
        }
    }
}

fn correlation_id() -> String {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    since_the_epoch.as_millis().to_string()
}

impl From<protobuf::ProtobufError> for EventIoError {
    fn from(err: protobuf::ProtobufError) -> Self {
        EventIoError(format!("Wire protocol error: {}", &err))
    }
}

impl From<ReceiveError> for EventIoError {
    fn from(err: ReceiveError) -> Self {
        EventIoError(format!("Unable to receive message: {}", &err))
    }
}

impl From<SendError> for EventIoError {
    fn from(err: SendError) -> Self {
        EventIoError(format!("Unable to send message: {}", &err))
    }
}
