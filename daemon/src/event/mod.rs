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

pub use self::error::EventProcessorError;

use grid_sdk::grid_db::commits::store::{
    CommitEvent, EventError, EventIoError, ALL_GRID_NAMESPACES,
};

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
                        Ok(commit_event) => handle_message(commit_event, &event_handlers)?,
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
