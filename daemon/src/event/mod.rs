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

pub mod block;
mod error;

use std::cell::RefCell;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use protobuf::Message as _;

use sawtooth_sdk::{
    messages::client_event::{
        ClientEventsSubscribeRequest, ClientEventsSubscribeResponse,
        ClientEventsSubscribeResponse_Status,
    },
    messages::events::{Event, EventFilter, EventFilter_FilterType, EventList, EventSubscription},
    messages::validator::{Message, Message_MessageType},
    messaging::stream::{MessageSender, ReceiveError, SendError},
};

use crate::sawtooth_connection::SawtoothConnection;

pub use super::event::error::{EventError, EventProcessorError};

const PIKE_NAMESPACE: &str = "cad11d";
const GRID_NAMESPACE: &str = "621dee";

const SHUTDOWN_TIMEOUT: u64 = 2;

pub trait EventHandler: Send {
    fn handle_events(&self, events: &[Event]) -> Result<(), EventError>;
}

#[macro_export]
macro_rules! event_handlers {
    [$($handler:expr),*] => {
        vec![$(Box::new($handler),)*]
    };
}

pub struct EventProcessor {
    join_handle: thread::JoinHandle<Result<(), EventProcessorError>>,
    message_sender: Box<dyn MessageSender + Send>,
}

pub struct EventProcessorShutdownHandle {
    message_sender: RefCell<Box<dyn MessageSender + Send>>,
}

impl EventProcessorShutdownHandle {
    pub fn shutdown(&self) -> Result<(), EventProcessorError> {
        let mut message_sender = self.message_sender.borrow_mut();

        debug!("Sending unsubscribe request");
        match message_sender
            .send(
                Message_MessageType::CLIENT_EVENTS_UNSUBSCRIBE_REQUEST,
                &correlation_id(),
                &[], // An unsubscribe request has no content
            )
            .map_err(|err| {
                EventProcessorError(format!("Unable to send unsubscribe request: {}", err))
            })?
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
            Err(err) => return Err(EventProcessorError::from(err)),
        }

        debug!("Closing message sender");
        message_sender.close();

        Ok(())
    }
}

impl EventProcessor {
    pub fn start(
        sawtooth_connection: SawtoothConnection,
        last_known_block_id: &str,
        event_handlers: Vec<Box<dyn EventHandler>>,
    ) -> Result<Self, EventProcessorError> {
        let message_sender = sawtooth_connection.get_sender();

        let last_known_block_id = last_known_block_id.to_owned();
        let request = create_subscription_request(last_known_block_id);
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
            return Err(EventProcessorError(format!(
                "Failed to subscribe for events: {:?} {}",
                response.get_status(),
                response.get_response_message()
            )));
        }

        let join_handle = thread::Builder::new()
            .name("EventProcessor".into())
            .spawn(move || {
                while let Ok(msg_result) = sawtooth_connection.get_receiver().recv() {
                    match msg_result {
                        Ok(msg) => handle_message(msg, &event_handlers)?,
                        Err(ReceiveError::DisconnectedError) => break,
                        Err(err) => {
                            return Err(EventProcessorError(format!(
                                "Failed to receive events; aborting: {}",
                                err
                            )));
                        }
                    }
                }

                info!("Disconnected from validator; terminating Event Processor");
                Ok(())
            })
            .map_err(|err| {
                EventProcessorError(format!("Unable to start EventProcessor thread: {}", err))
            })?;

        Ok(Self {
            join_handle,
            message_sender,
        })
    }

    pub fn take_shutdown_controls(
        self,
    ) -> (
        EventProcessorShutdownHandle,
        thread::JoinHandle<Result<(), EventProcessorError>>,
    ) {
        (
            EventProcessorShutdownHandle {
                message_sender: RefCell::new(self.message_sender),
            },
            self.join_handle,
        )
    }
}

fn handle_message(
    msg: Message,
    event_handlers: &[Box<dyn EventHandler>],
) -> Result<(), EventProcessorError> {
    if msg.get_message_type() != Message_MessageType::CLIENT_EVENTS {
        warn!("Received unexpected message: {:?}", msg.get_message_type());
        return Ok(());
    }

    let event_list: EventList = match protobuf::parse_from_bytes(msg.get_content()) {
        Ok(event_list) => event_list,
        Err(err) => {
            warn!("Unable to parse event list; ignoring: {}", err);
            return Ok(());
        }
    };

    for handler in event_handlers {
        if let Err(err) = handler.handle_events(&event_list.get_events()) {
            error!("An error occured while handling events: {}", err);
        }
    }

    Ok(())
}

fn content_of_type<M: protobuf::Message>(
    expected_type: Message_MessageType,
    msg: Message,
) -> Result<M, EventProcessorError> {
    if msg.get_message_type() != expected_type {
        return Err(EventProcessorError(format!(
            "Unexpected message type: expected {:?} but was {:?}",
            expected_type,
            msg.get_message_type()
        )));
    }

    protobuf::parse_from_bytes(msg.get_content())
        .map_err(|err| EventProcessorError(format!("Unable to parse message content: {}", err)))
}

fn create_subscription_request(last_known_block_id: String) -> ClientEventsSubscribeRequest {
    let mut block_info_subscription = EventSubscription::new();
    block_info_subscription.set_event_type("sawtooth/block-commit".into());

    // Event subscription for Grid
    let mut grid_state_filter = EventFilter::new();
    grid_state_filter.set_filter_type(EventFilter_FilterType::REGEX_ANY);
    grid_state_filter.set_key("address".into());
    grid_state_filter.set_match_string(format!("^{}.*", GRID_NAMESPACE));

    let mut grid_state_delta_subscription = EventSubscription::new();
    grid_state_delta_subscription.set_event_type("sawtooth/state-delta".into());
    grid_state_delta_subscription
        .mut_filters()
        .push(grid_state_filter);

    // Event subscription for Pike
    let mut pike_state_filter = EventFilter::new();
    pike_state_filter.set_filter_type(EventFilter_FilterType::REGEX_ANY);
    pike_state_filter.set_key("address".into());
    pike_state_filter.set_match_string(format!("^{}.*", PIKE_NAMESPACE));

    let mut pike_state_delta_subscription = EventSubscription::new();
    pike_state_delta_subscription.set_event_type("sawtooth/state-delta".into());
    pike_state_delta_subscription
        .mut_filters()
        .push(pike_state_filter);

    let mut request = ClientEventsSubscribeRequest::new();
    request.mut_subscriptions().push(block_info_subscription);
    request
        .mut_subscriptions()
        .push(pike_state_delta_subscription);
    request
        .mut_subscriptions()
        .push(grid_state_delta_subscription);
    request.mut_last_known_block_ids().push(last_known_block_id);

    request
}

fn correlation_id() -> String {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    since_the_epoch.as_millis().to_string()
}

impl From<protobuf::ProtobufError> for EventProcessorError {
    fn from(err: protobuf::ProtobufError) -> Self {
        EventProcessorError(format!("Wire protocol error: {}", &err))
    }
}

impl From<ReceiveError> for EventProcessorError {
    fn from(err: ReceiveError) -> Self {
        EventProcessorError(format!("Unable to receive message: {}", &err))
    }
}

impl From<SendError> for EventProcessorError {
    fn from(err: SendError) -> Self {
        EventProcessorError(format!("Unable to send message: {}", &err))
    }
}
