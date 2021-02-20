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

#[cfg(feature = "pike")]
use grid_sdk::pike::addressing::PIKE_NAMESPACE;

cfg_if! {
    if #[cfg(feature = "schema")] {
        use grid_sdk::schemas::addressing::GRID_NAMESPACE;
    } else if #[cfg(feature = "product")] {
        use grid_sdk::products::addressing::GRID_NAMESPACE;
    } else if #[cfg(feature = "location")] {
        use grid_sdk::locations::addressing::GRID_NAMESPACE;
    }
}

#[cfg(feature = "track-and-trace")]
use grid_sdk::track_and_trace::addressing::TRACK_AND_TRACE_NAMESPACE;

use grid_sdk::commits::store::{CommitEvent as DbCommitEvent, StateChange as DbStateChange};

pub use self::error::{EventError, EventIoError, EventProcessorError};

const ALL_GRID_NAMESPACES: &[&str] = &[
    #[cfg(feature = "pike")]
    PIKE_NAMESPACE,
    #[cfg(any(feature = "schema", feature = "product", feature = "location"))]
    GRID_NAMESPACE,
    #[cfg(feature = "track-and-trace")]
    TRACK_AND_TRACE_NAMESPACE,
];

const SABRE_NAMESPACE: &str = "00ec";

const IGNORED_NAMESPACES: &[&str] = &[SABRE_NAMESPACE];

/// A notification that some source has committed a set of changes to state
#[derive(Clone)]
pub struct CommitEvent {
    /// An identifier for specifying where the event came from
    pub service_id: Option<String>,
    /// An identifier that is unique among events from the source
    pub id: String,
    /// May be used to provide ordering of commits from the source. If `None`, ordering is not
    /// explicitly provided, so it must be inferred from the order in which events are received.
    pub height: Option<u64>,
    /// All state changes that are included in the commit
    pub state_changes: Vec<StateChange>,
}

impl std::fmt::Display for CommitEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("(")?;
        f.write_str(&self.id)?;
        f.write_str(", ")?;
        if self.service_id.is_some() {
            write!(f, "{}, ", self.service_id.as_ref().unwrap())?;
        }
        if self.height.is_some() {
            write!(f, "height: {}, ", self.height.as_ref().unwrap())?;
        }

        write!(f, "#changes: {})", self.state_changes.len())
    }
}

impl From<DbCommitEvent> for CommitEvent {
    fn from(event: DbCommitEvent) -> Self {
        Self {
            service_id: event.service_id,
            id: event.id,
            height: event.height,
            state_changes: event
                .state_changes
                .into_iter()
                .map(StateChange::from)
                .collect(),
        }
    }
}

impl From<&CommitEvent> for DbCommitEvent {
    fn from(event: &CommitEvent) -> Self {
        Self {
            service_id: event.service_id.clone(),
            id: event.id.to_string(),
            height: event.height,
            state_changes: event
                .state_changes
                .clone()
                .into_iter()
                .map(DbStateChange::from)
                .collect(),
        }
    }
}

/// A change that has been applied to state, represented in terms of a key/value pair
#[derive(Clone, Eq, PartialEq)]
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

impl From<DbStateChange> for StateChange {
    fn from(event: DbStateChange) -> StateChange {
        match event {
            DbStateChange::Set { key: k, value: v } => StateChange::Set { key: k, value: v },
            DbStateChange::Delete { key: k } => StateChange::Delete { key: k },
        }
    }
}

impl From<StateChange> for DbStateChange {
    fn from(event: StateChange) -> DbStateChange {
        match event {
            StateChange::Set { key: k, value: v } => DbStateChange::Set { key: k, value: v },
            StateChange::Delete { key: k } => DbStateChange::Delete { key: k },
        }
    }
}

pub trait EventHandler: Send {
    fn handle_event(&self, event: &CommitEvent) -> Result<(), EventError>;

    fn cloned_box(&self) -> Box<dyn EventHandler>;
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
        last_commit_id: Option<&str>,
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
        last_commit_id: Option<&str>,
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
        last_known_commit_id: Option<&str>,
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
                        Ok(commit_event) => handle_message(commit_event, &event_handlers),
                        Err(EventIoError::InvalidMessage(msg)) => {
                            warn!("{}; ignoring...", msg);
                        }
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

fn handle_message(event: CommitEvent, event_handlers: &[Box<dyn EventHandler>]) {
    for handler in event_handlers {
        if let Err(err) = handler.handle_event(&event) {
            error!("An error occurred while handling events: {}", err);
        }
    }
}
