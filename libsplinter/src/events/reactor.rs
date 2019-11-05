// Copyright 2019 Cargill Incorporated
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

use std::thread;

use crossbeam_channel::{bounded, Sender};
use futures::Future;
use tokio::runtime::Runtime;

use crate::events::ws::{Listen, ParseBytes, WebSocketClient};
use crate::events::{ReactorError, WebSocketError};

/// Reactor
///
/// Reactor creates a runtime environment for http related futures
/// on start up. Reactors create `Igniter` object that are used to
/// send futures to the runtime.
pub struct Reactor {
    sender: Sender<ReactorMessage>,
    thread_handle: thread::JoinHandle<()>,
}

impl Reactor {
    pub fn new() -> Self {
        let (sender, receiver) = bounded::<ReactorMessage>(10);
        let thread_handle = thread::spawn(move || {
            let mut runtime = match Runtime::new() {
                Ok(runtime) => runtime,
                Err(err) => {
                    error!("Unable to create event reactor runtime: {}", err);
                    return;
                }
            };

            let mut connections = Vec::new();
            loop {
                match receiver.recv() {
                    Ok(ReactorMessage::StartWs(listen)) => {
                        let (future, handle) = listen.into_shutdown_handle();
                        runtime.spawn(futures::lazy(|| future.map_err(|_| ())));
                        connections.push(handle);
                    }
                    Ok(ReactorMessage::HttpRequest(req)) => {
                        runtime.spawn(req);
                    }
                    Ok(ReactorMessage::Stop) => break,
                    Err(err) => {
                        error!("Failed to receive message {}", err);
                        break;
                    }
                }
            }

            let shutdown_errors = connections
                .into_iter()
                .map(|connection| connection.shutdown())
                .filter_map(|res| if let Err(err) = res { Some(err) } else { None })
                .collect::<Vec<WebSocketError>>();

            if let Err(err) = runtime
                .shutdown_on_idle()
                .wait()
                .map_err(|_| {
                    ReactorError::ReactorShutdownError(
                        "An Error occured while shutting down Reactor".to_string(),
                    )
                })
                .and_then(|_| {
                    if shutdown_errors.is_empty() {
                        Ok(())
                    } else {
                        Err(ReactorError::ShutdownHandleErrors(shutdown_errors))
                    }
                })
            {
                error!("Unable to cleanly shutdown event reactor: {}", err);
            }
        });

        Self {
            thread_handle,
            sender,
        }
    }

    pub fn igniter(&self) -> Igniter {
        Igniter {
            sender: self.sender.clone(),
        }
    }

    pub fn shutdown(self) -> Result<(), ReactorError> {
        self.sender.send(ReactorMessage::Stop).map_err(|_| {
            ReactorError::ReactorShutdownError("Failed to send shutdown message".to_string())
        })?;

        self.thread_handle
            .join()
            .map_err(|_| ReactorError::ReactorShutdownError("Failed to join thread".to_string()))
    }
}

impl std::default::Default for Reactor {
    fn default() -> Self {
        Self::new()
    }
}

/// The Igniter is a channel that allows for communication with a Reactor runtime
#[derive(Clone)]
pub struct Igniter {
    sender: Sender<ReactorMessage>,
}

impl Igniter {
    pub fn start_ws<T: ParseBytes<T>>(
        &self,
        ws: &WebSocketClient<T>,
    ) -> Result<(), WebSocketError> {
        self.sender
            .send(ReactorMessage::StartWs(ws.listen(self.clone())?))
            .map_err(|err| {
                WebSocketError::ListenError(format!("Failed to start ws {}: {}", ws.url(), err))
            })
    }

    pub fn send(
        &self,
        req: Box<dyn Future<Item = (), Error = ()> + Send + 'static>,
    ) -> Result<(), ReactorError> {
        self.sender
            .send(ReactorMessage::HttpRequest(req))
            .map_err(|err| {
                ReactorError::RequestSendError(format!("Failed to send request to reactor {}", err))
            })
    }
}

enum ReactorMessage {
    Stop,
    StartWs(Listen),
    HttpRequest(Box<dyn Future<Item = (), Error = ()> + Send + 'static>),
}
