// Copyright 2021 Cargill Incorporated
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

use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::Sender,
    Arc,
};
use std::thread;
use std::time::{Duration, Instant};

use crate::error::InternalError;

/// Build new Pacemakers
///
/// This builder is used to build new pacemakers, where a pacemaker is defined by a firing
/// interval, a message factory used to construct the messages fired, and a sender that will accept
/// the messages.
pub struct PacemakerBuilder<M, F>
where
    M: Send + 'static,
    F: Fn() -> M + Send + 'static,
{
    interval: Option<u64>,
    sender: Option<Sender<M>>,
    message_factory: Option<F>,
}

impl<M, F> PacemakerBuilder<M, F>
where
    M: Send + 'static,
    F: Fn() -> M + Send + 'static,
{
    /// Construct a new builder.
    pub fn new() -> Self {
        Self {
            interval: None,
            sender: None,
            message_factory: None,
        }
    }

    /// Set the firing interval in seconds.
    pub fn with_interval(mut self, interval: u64) -> Self {
        self.interval = Some(interval);
        self
    }

    /// Set the sender that will accept the messages on the interval.
    pub fn with_sender(mut self, sender: Sender<M>) -> Self {
        self.sender = Some(sender);
        self
    }

    /// Set the message factory function that will generate messages on the interval.
    pub fn with_message_factory(mut self, message_factory: F) -> Self {
        self.message_factory = Some(message_factory);
        self
    }

    /// Construct and start the Pacemaker.
    ///
    /// # Errors
    ///
    /// A `PacemakerStartError` will be returned if none of the fields are configured, or if the
    /// pacemaker thread fails to start.
    pub fn start(mut self) -> Result<Pacemaker, InternalError> {
        let running = Arc::new(AtomicBool::new(true));

        let running_clone = running.clone();
        let interval = self
            .interval
            .take()
            .ok_or_else(|| InternalError::with_message("No interval provided".into()))?;
        let sender = self
            .sender
            .take()
            .ok_or_else(|| InternalError::with_message("No sender provided".into()))?;
        let new_message = self.message_factory.take().ok_or_else(|| {
            InternalError::with_message("No message factory function provided".into())
        })?;

        let join_handle = thread::Builder::new()
            .name("Pacemaker".into())
            .spawn(move || {
                let mut start = Instant::now();
                let loop_duration = Duration::from_secs(1);
                let pace_duration = Duration::from_secs(interval);

                while running_clone.load(Ordering::SeqCst) {
                    if start.elapsed() >= pace_duration {
                        start = Instant::now();
                        if let Err(err) = sender.send(new_message()) {
                            warn!(
                                "Sender has disconnected before \
                                shutting down pacemaker {:?}",
                                err
                            );
                            break;
                        }
                    }
                    thread::sleep(loop_duration);
                }
            })
            .map_err(|err| InternalError::from_source(Box::new(err)))?;
        Ok(Pacemaker {
            join_handle,
            shutdown_signaler: ShutdownSignaler { running },
        })
    }
}

/// Pacemaker is responsible for periodically sending a message to
/// another component over a channel. The message is meant to be used as
/// a notfication that some action should take place.
pub struct Pacemaker {
    join_handle: thread::JoinHandle<()>,
    shutdown_signaler: ShutdownSignaler,
}

impl Pacemaker {
    /// Construct a new `PacemakerBuilder` for creating a `Pacemaker` instance.
    pub fn builder<M, F>() -> PacemakerBuilder<M, F>
    where
        M: Send + 'static,
        F: Fn() -> M + Send + 'static,
    {
        PacemakerBuilder::new()
    }

    pub fn shutdown_signaler(&self) -> ShutdownSignaler {
        self.shutdown_signaler.clone()
    }

    pub fn await_shutdown(self) {
        if let Err(err) = self.join_handle.join() {
            error!("Failed to shutdown heartbeat monitor gracefully: {:?}", err);
        }
    }
}

#[derive(Clone)]
pub struct ShutdownSignaler {
    running: Arc<AtomicBool>,
}

impl ShutdownSignaler {
    pub fn shutdown(&self) {
        self.running.store(false, Ordering::SeqCst)
    }
}
