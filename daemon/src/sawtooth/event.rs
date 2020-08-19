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

use std::convert::TryInto;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use protobuf::Message as _;

use sawtooth_sdk::{
    messages::{
        client_event::{
            ClientEventsSubscribeRequest, ClientEventsSubscribeResponse,
            ClientEventsSubscribeResponse_Status,
        },
        events::{
            Event as SawtoothEvent, EventFilter, EventFilter_FilterType,
            EventList as SawtoothEventList, EventSubscription,
        },
        network::PingResponse as SawtoothPingResponse,
        transaction_receipt::{StateChange as SawtoothStateChange, StateChangeList},
        validator::{Message, Message_MessageType},
    },
    messaging::{
        stream::{MessageSender, ReceiveError},
        zmq_stream::ZmqMessageSender,
    },
};

use crate::event::{EventConnection, EventConnectionUnsubscriber};
use grid_sdk::grid_db::commits::store::NULL_BLOCK_ID;
use grid_sdk::grid_db::commits::store::{CommitEvent, EventIoError, StateChange};

use super::connection::SawtoothConnection;

const BLOCK_COMMIT_EVENT_TYPE: &str = "sawtooth/block-commit";
const STATE_CHANGE_EVENT_TYPE: &str = "sawtooth/state-delta";
const BLOCK_ID_ATTR: &str = "block_id";
const BLOCK_NUM_ATTR: &str = "block_num";

const SHUTDOWN_TIMEOUT: u64 = 2;

impl EventConnection for SawtoothConnection {
    type Unsubscriber = SawtoothEventUnsubscriber;

    fn name(&self) -> &str {
        "sawtooth-validator"
    }

    fn subscribe(
        &mut self,
        namespaces: &[&str],
        last_commit_id: Option<&str>,
    ) -> Result<Self::Unsubscriber, EventIoError> {
        let message_sender = self.get_sender();

        let request =
            create_subscription_request(last_commit_id.unwrap_or(NULL_BLOCK_ID), namespaces);
        let mut future = message_sender.send(
            Message_MessageType::CLIENT_EVENTS_SUBSCRIBE_REQUEST,
            &correlation_id(),
            &request.write_to_bytes().map_err(|err| {
                EventIoError::ConnectionError(format!(
                    "Failed to serialize subscription request: {}",
                    err
                ))
            })?,
        )?;

        let response: ClientEventsSubscribeResponse = content_of_type(
            Message_MessageType::CLIENT_EVENTS_SUBSCRIBE_RESPONSE,
            future.get()?,
        )?;

        if response.get_status() != ClientEventsSubscribeResponse_Status::OK {
            return Err(EventIoError::ConnectionError(format!(
                "Failed to subscribe for events: {:?} {}",
                response.get_status(),
                response.get_response_message()
            )));
        }
        Ok(SawtoothEventUnsubscriber { message_sender })
    }

    fn recv(&self) -> Result<CommitEvent, EventIoError> {
        loop {
            match self.get_receiver().recv() {
                Ok(Ok(msg)) if msg.get_message_type() == Message_MessageType::CLIENT_EVENTS => {
                    break extract_event(msg)
                }
                Ok(Ok(msg)) if msg.get_message_type() == Message_MessageType::PING_REQUEST => {
                    self.get_sender().send(
                        Message_MessageType::PING_RESPONSE,
                        msg.get_correlation_id(),
                        &SawtoothPingResponse::new()
                            .write_to_bytes()
                            .map_err(|err| {
                                EventIoError::ConnectionError(format!(
                                    "Failed to serialize subscription request: {}",
                                    err
                                ))
                            })?,
                    )?;
                    trace!("Received ping request and sent reply");
                }
                Ok(Ok(msg)) => {
                    break Err(EventIoError::InvalidMessage(format!(
                        "Received unexpected message: {:?}",
                        msg.get_message_type()
                    )));
                }
                Ok(Err(ReceiveError::DisconnectedError)) => {
                    break Err(EventIoError::ConnectionError(format!(
                        "{} has disconnected",
                        self.name()
                    )))
                }
                Ok(Err(err)) => break Err(EventIoError::ConnectionError(err.to_string())),
                Err(err) => break Err(EventIoError::ConnectionError(err.to_string())),
            }
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
            .map_err(|err| {
                EventIoError::ConnectionError(format!(
                    "Unable to send unsubscribe request: {}",
                    err
                ))
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
        return Err(EventIoError::ConnectionError(format!(
            "Unexpected message type: expected {:?} but was {:?}",
            expected_type,
            msg.get_message_type()
        )));
    }

    protobuf::parse_from_bytes(msg.get_content())
        .map_err(|err| EventIoError::ConnectionError(err.to_string()))
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

fn extract_event(msg: Message) -> Result<CommitEvent, EventIoError> {
    let sawtooth_events = protobuf::parse_from_bytes::<SawtoothEventList>(msg.get_content())
        .map_err(|err| {
            EventIoError::InvalidMessage(format!("Unable to parse event list: {}", err))
        })?
        .take_events()
        .to_vec();

    sawtooth_event_to_commit_event(sawtooth_events.as_slice())
}

fn sawtooth_event_to_commit_event(events: &[SawtoothEvent]) -> Result<CommitEvent, EventIoError> {
    let (id, height) = get_id_and_height(events)?;
    let state_changes = get_state_changes(events)?;

    Ok(CommitEvent {
        service_id: None,
        id,
        height,
        state_changes,
    })
}

fn get_id_and_height(events: &[SawtoothEvent]) -> Result<(String, Option<u64>), EventIoError> {
    let block_event = get_block_event(events)?;
    let block_id = get_required_attribute_from_event(block_event, BLOCK_ID_ATTR)?;
    let block_num = get_required_attribute_from_event(block_event, BLOCK_NUM_ATTR)?
        .parse::<u64>()
        .map_err(|err| {
            EventIoError::InvalidMessage(format!("block_num was not a valid u64: {}", err))
        })?;
    Ok((block_id, Some(block_num)))
}

fn get_block_event(events: &[SawtoothEvent]) -> Result<&SawtoothEvent, EventIoError> {
    events
        .iter()
        .find(|event| event.get_event_type() == BLOCK_COMMIT_EVENT_TYPE)
        .ok_or_else(|| EventIoError::InvalidMessage("no block event found".into()))
}

fn get_required_attribute_from_event(
    event: &SawtoothEvent,
    required_attr_key: &str,
) -> Result<String, EventIoError> {
    event
        .get_attributes()
        .iter()
        .find(|attr| attr.get_key() == required_attr_key)
        .map(|attr| attr.get_value().to_string())
        .ok_or_else(|| {
            EventIoError::InvalidMessage(format!(
                "required attribute not in event: {}",
                required_attr_key
            ))
        })
}

fn get_state_changes(events: &[SawtoothEvent]) -> Result<Vec<StateChange>, EventIoError> {
    Ok(events
        .iter()
        .filter(|event| event.get_event_type() == STATE_CHANGE_EVENT_TYPE)
        .map(|event| {
            get_sawtooth_state_changes_from_sawtooth_event(&event)
                .and_then(sawtooth_state_changes_into_native_state_changes)
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .filter(|state_change| state_change.is_grid_state_change())
        .collect())
}

fn get_sawtooth_state_changes_from_sawtooth_event(
    sawtooth_event: &SawtoothEvent,
) -> Result<Vec<SawtoothStateChange>, EventIoError> {
    protobuf::parse_from_bytes::<StateChangeList>(&sawtooth_event.data)
        .map(|mut list| list.take_state_changes().to_vec())
        .map_err(|err| {
            EventIoError::InvalidMessage(format!(
                "failed to parse state change list from state change event: {}",
                err
            ))
        })
}

fn sawtooth_state_changes_into_native_state_changes(
    sawtooth_state_changes: Vec<SawtoothStateChange>,
) -> Result<Vec<StateChange>, EventIoError> {
    sawtooth_state_changes
        .into_iter()
        .map(|sawtooth_state_change| sawtooth_state_change.try_into())
        .collect()
}

fn correlation_id() -> String {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    since_the_epoch.as_millis().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    use sawtooth_sdk::messages::events::Event_Attribute;
    use sawtooth_sdk::messages::transaction_receipt::StateChange_Type as SawtoothStateChange_Type;

    const PIKE_NAMESPACE: &str = "cad11d";
    const GRID_NAMESPACE: &str = "621dee";
    const TRACK_AND_TRACE_NAMESPACE: &str = "a43b46";

    /// Verify that a valid set of Sawtooth events can be converted to a `CommitEvent`.
    #[test]
    fn sawtooth_events_to_commit_event() {
        let block_id = "abcdef";
        let block_num = 1;

        let grid_state_changes = vec![
            create_state_change(format!("{}01", PIKE_NAMESPACE), Some(vec![0x01])),
            create_state_change(format!("{}02", GRID_NAMESPACE), Some(vec![0x02])),
            create_state_change(format!("{}03", TRACK_AND_TRACE_NAMESPACE), None),
        ];
        let non_grid_state_changes = vec![create_state_change("ef".into(), None)];

        let sawtooth_events = vec![
            create_block_event(block_id, block_num),
            create_state_change_event(&grid_state_changes[0..2]),
            create_state_change_event(&grid_state_changes[2..]),
            create_state_change_event(&non_grid_state_changes[..]),
        ];

        let commit_event = sawtooth_event_to_commit_event(sawtooth_events.as_slice())
            .expect("Failed to convert sawtooth events to a CommitEvent");

        assert!(&commit_event.service_id.is_none());
        assert_eq!(&commit_event.id, block_id);
        assert_eq!(commit_event.height, Some(block_num));
        assert_eq!(commit_event.state_changes.len(), grid_state_changes.len());
        for sawtooth_state_change in grid_state_changes {
            let expected_state_change = match sawtooth_state_change.get_field_type() {
                SawtoothStateChange_Type::SET => StateChange::Set {
                    key: sawtooth_state_change.get_address().into(),
                    value: sawtooth_state_change.get_value().into(),
                },
                SawtoothStateChange_Type::DELETE => StateChange::Delete {
                    key: sawtooth_state_change.get_address().into(),
                },
                _ => panic!("Sawtooth state change type unset"),
            };
            assert!(commit_event
                .state_changes
                .iter()
                .any(|state_change| state_change == &expected_state_change));
        }
    }

    fn create_block_event(block_id: &str, block_num: u64) -> SawtoothEvent {
        let mut event = SawtoothEvent::new();
        event.set_event_type(BLOCK_COMMIT_EVENT_TYPE.into());
        event.set_attributes(
            vec![
                create_attribute(BLOCK_ID_ATTR.into(), block_id.into()),
                create_attribute(BLOCK_NUM_ATTR.into(), block_num.to_string()),
            ]
            .into(),
        );
        event
    }

    fn create_attribute(key: String, value: String) -> Event_Attribute {
        let mut attribute = Event_Attribute::new();
        attribute.set_key(key);
        attribute.set_value(value);
        attribute
    }

    fn create_state_change(address: String, value: Option<Vec<u8>>) -> SawtoothStateChange {
        let mut state_change = SawtoothStateChange::new();
        state_change.set_address(address);
        match value {
            Some(value) => {
                state_change.set_field_type(SawtoothStateChange_Type::SET);
                state_change.set_value(value);
            }
            None => state_change.set_field_type(SawtoothStateChange_Type::DELETE),
        }
        state_change
    }

    fn create_state_change_event(state_changes: &[SawtoothStateChange]) -> SawtoothEvent {
        let mut event = SawtoothEvent::new();
        event.set_event_type(STATE_CHANGE_EVENT_TYPE.into());
        let mut state_change_list = StateChangeList::new();
        state_change_list.set_state_changes(state_changes.into());
        event.set_data(
            state_change_list
                .write_to_bytes()
                .expect("failed to serialize StateChangeList"),
        );
        event
    }
}
