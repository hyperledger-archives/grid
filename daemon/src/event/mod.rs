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

pub mod db_handler;
mod error;

use std::cell::RefCell;
use std::thread;

pub use self::error::{EventError, EventIoError, EventProcessorError};

const PIKE_NAMESPACE: &str = "cad11d";
const PIKE_AGENT: &str = "cad11d00";
const PIKE_ORG: &str = "cad11d01";

const GRID_NAMESPACE: &str = "621dee";
const GRID_SCHEMA: &str = "621dee01";
const GRID_PRODUCT: &str = "621dee02";

const TRACK_AND_TRACE_NAMESPACE: &str = "a43b46";
const TRACK_AND_TRACE_PROPERTY: &str = "a43b46ea";
const TRACK_AND_TRACE_PROPOSAL: &str = "a43b46aa";
const TRACK_AND_TRACE_RECORD: &str = "a43b46ec";

const ALL_GRID_NAMESPACES: &[&str] = &[PIKE_NAMESPACE, GRID_NAMESPACE, TRACK_AND_TRACE_NAMESPACE];

/// A notification that some source has committed a set of changes to state
pub struct CommitEvent {
    /// An identifier for specifying where the event came from
    pub source: String,
    /// An identifier that is unique among events from the source
    pub id: String,
    /// May be used to provide ordering of commits from the source. If `None`, ordering is not
    /// explicitly provided, so it must be inferred from the order in which events are received.
    pub height: Option<u64>,
    /// All state changes that are included in the commit
    pub state_changes: Vec<StateChange>,
}

/// A change that has been applied to state, represented in terms of a key/value pair
#[derive(Eq, PartialEq)]
pub enum StateChange {
    Set { key: String, value: Vec<u8> },
    Delete { key: String },
}

impl StateChange {
    pub fn key_has_prefix(&self, prefix: &str) -> bool {
        let key = match self {
            Self::Set { key, .. } => key,
            Self::Delete { key, .. } => key,
        };
        key.get(0..prefix.len())
            .map(|key_prefix| key_prefix == prefix)
            .unwrap_or(false)
    }

    pub fn is_grid_state_change(&self) -> bool {
        ALL_GRID_NAMESPACES
            .iter()
            .any(|namespace| self.key_has_prefix(namespace))
    }
}

pub trait EventHandler: Send {
    fn handle_event(&self, event: &CommitEvent) -> Result<(), EventError>;
}

#[macro_export]
macro_rules! event_handlers {
    [$($handler:expr),*] => {
        vec![$(Box::new($handler),)*]
    };
}

pub trait EventConnectionUnsubscriber: Send {
    fn unsubscribe(self) -> Result<(), EventIoError>;
}

pub trait EventConnection: Send {
    type Unsubscriber: EventConnectionUnsubscriber;

    fn name(&self) -> &str;

    fn recv(&self) -> Result<CommitEvent, EventIoError>;

    fn subscribe(
        &mut self,
        namespaces: &[&str],
        last_commit_id: &str,
    ) -> Result<Self::Unsubscriber, EventIoError>;

    fn close(self) -> Result<(), EventIoError>;
}

impl<EC: EventConnection> EventConnection for Box<EC> {
    type Unsubscriber = EC::Unsubscriber;

    fn name(&self) -> &str {
        (**self).name()
    }

    fn recv(&self) -> Result<CommitEvent, EventIoError> {
        (**self).recv()
    }

    fn subscribe(
        &mut self,
        namespaces: &[&str],
        last_commit_id: &str,
    ) -> Result<Self::Unsubscriber, EventIoError> {
        (**self).subscribe(namespaces, last_commit_id)
    }

    fn close(self) -> Result<(), EventIoError> {
        (*self).close()
    }
}

pub struct EventProcessorShutdownHandle<Unsubscriber: EventConnectionUnsubscriber> {
    unsubscriber: RefCell<Option<Unsubscriber>>,
}

impl<Unsubscriber: EventConnectionUnsubscriber> EventProcessorShutdownHandle<Unsubscriber> {
    pub fn shutdown(&self) -> Result<(), EventProcessorError> {
        if let Some(unsubscriber) = self.unsubscriber.borrow_mut().take() {
            unsubscriber
                .unsubscribe()
                .map_err(|err| EventProcessorError(format!("Unable to unsubscribe: {}", err)))?;
        }

        Ok(())
    }
}

pub struct EventProcessor<Conn: EventConnection> {
    join_handle: thread::JoinHandle<Result<(), EventProcessorError>>,
    unsubscriber: Option<Conn::Unsubscriber>,
}

impl<Conn: EventConnection + 'static> EventProcessor<Conn> {
    pub fn start(
        mut connection: Conn,
        last_known_commit_id: &str,
        event_handlers: Vec<Box<dyn EventHandler>>,
    ) -> Result<Self, EventProcessorError> {
        let unsubscriber = connection
            .subscribe(ALL_GRID_NAMESPACES, last_known_commit_id)
            .map_err(|err| EventProcessorError(format!("Unable to unsubscribe: {}", err)))?;

        let join_handle = thread::Builder::new()
            .name(format!("EventProcessor[{}]", connection.name()))
            .spawn(move || {
                loop {
                    match connection.recv() {
                        Ok(commit_event) => handle_message(commit_event, &event_handlers)?,
                        Err(err) => {
                            error!("Failed to receive events; aborting: {}", err);
                            break;
                        }
                    }
                }

                info!(
                    "Disconnecting from {}; terminating Event Processor",
                    connection.name()
                );

                if let Err(err) = connection.close() {
                    error!("Unable to close connection: {}", err);
                }

                Ok(())
            })
            .map_err(|err| {
                EventProcessorError(format!("Unable to start EventProcessor thread: {}", err))
            })?;

        Ok(Self {
            join_handle,
            unsubscriber: Some(unsubscriber),
        })
    }

    pub fn take_shutdown_controls(
        self,
    ) -> (
        EventProcessorShutdownHandle<Conn::Unsubscriber>,
        thread::JoinHandle<Result<(), EventProcessorError>>,
    ) {
        (
            EventProcessorShutdownHandle {
                unsubscriber: RefCell::new(self.unsubscriber),
            },
            self.join_handle,
        )
    }
}

fn handle_message(
    event: CommitEvent,
    event_handlers: &[Box<dyn EventHandler>],
) -> Result<(), EventProcessorError> {
    for handler in event_handlers {
        if let Err(err) = handler.handle_event(&event) {
            error!("An error occurred while handling events: {}", err);
        }
    }

    Ok(())
}
