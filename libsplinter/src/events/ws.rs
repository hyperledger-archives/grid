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

//! WebSocket Module.
//!
//! Module for establishing WebSocket connections with Splinter Services.
//!
//!```
//! use std::{thread::sleep, time};
//! use libsplinter::events::ws::{WsResponse, WebSocketClient, WsRuntime};
//!
//! let mut ws = WebSocketClient::new("http://echo.websocket.org");
//!
//! ws.on_open(|| {
//!    println!("sending message");
//!    WsResponse::Text("hello, world".to_string())
//! });
//!
//! let listen = ws.listen(|msg| {
//!    if let Ok(s) = String::from_utf8(msg.clone()) {
//!         println!("Recieved {}", s);
//!    } else {
//!       println!("malformed message: {:?}", msg);
//!    };
//!
//!    WsResponse::Text("welcome to earth!!!".to_string())
//! }).unwrap();
//!
//! let mut runtime = WsRuntime::new().unwrap();
//!
//! let handle = runtime.start(listen);
//!
//! sleep(time::Duration::from_secs(1));
//! println!("stopping");
//! handle.shutdown().unwrap();
//! runtime.shutdown().unwrap();
//! ```

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use actix_http::ws;
use awc::ws::{CloseCode, CloseReason, Codec, Frame, Message};
use crossbeam_channel::{bounded, Receiver};
use futures::{
    future::{self, Either},
    sink::Wait,
    Future,
};
use hyper::{self, header, upgrade::Upgraded, Body, Client, Request, StatusCode};
use tokio::codec::{Decoder, Framed};
use tokio::prelude::*;

use crate::events::{ParseError, WebSocketError};

/// Wrapper around future created by `WebSocketClient`. In order for
/// the future to run it must be passed to `WsRuntime::start`
pub struct Listen {
    future: Box<dyn Future<Item = (), Error = WebSocketError> + Send + 'static>,
    running: Arc<AtomicBool>,
    receiver: Receiver<Result<(), WebSocketError>>,
}

impl Listen {
    pub fn into_shutdown_handle(
        self,
    ) -> (
        Box<dyn Future<Item = (), Error = WebSocketError> + Send + 'static>,
        ShutdownHandle,
    ) {
        (
            self.future,
            ShutdownHandle {
                running: self.running,
                receiver: self.receiver,
            },
        )
    }
}

#[derive(Clone)]
pub struct ShutdownHandle {
    running: Arc<AtomicBool>,
    receiver: Receiver<Result<(), WebSocketError>>,
}

impl ShutdownHandle {
    /// Sends shutdown message to websocket
    pub fn shutdown(self) -> Result<(), WebSocketError> {
        self.running.store(false, Ordering::SeqCst);
        self.receiver.recv()?
    }
}

/// WebSocket client. Configures Websocket connection and produces `Listen` future.
pub struct WebSocketClient {
    url: String,
    on_open: Option<Arc<dyn Fn() -> WsResponse + Send + Sync + 'static>>,
}

impl WebSocketClient {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            on_open: None,
        }
    }

    /// Adds optional `on_open` closure. This closer is called after a connection is initially
    /// established with the server, and is used for printing debug information and sending initial
    /// messages to server if necessary.
    pub fn on_open<F>(&mut self, on_open: F)
    where
        F: Fn() -> WsResponse + Send + Sync + 'static,
    {
        self.on_open = Some(Arc::new(on_open));
    }

    /// Starts listener as a separate thread. Whenever a message is received from the server
    /// callback `f` is called.
    pub fn listen<F>(self, f: F) -> Result<Listen, Error>
    where
        F: Fn(Vec<u8>) -> WsResponse + Send + Sync + 'static,
    {
        let url = self.url.clone();
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();
        let on_open = self.on_open.clone();
        let (sender, receiver) = bounded(1);

        debug!("starting: {}", url);

        let request = Request::builder()
            .uri(url)
            .header(header::UPGRADE, "websocket")
            .header(header::CONNECTION, "Upgrade")
            .header(header::SEC_WEBSOCKET_VERSION, "13")
            .header(header::SEC_WEBSOCKET_KEY, "13")
            .body(Body::empty())
            .map_err(|err| WebSocketError::RequestBuilderError(format!("{:?}", err)))?;

        let future = Box::new(
            Client::new()
                .request(request)
                .and_then(|res| {
                    if res.status() != StatusCode::SWITCHING_PROTOCOLS {
                        error!("The server didn't upgrade: {}", res.status());
                    }
                    debug!("response: {:?}", res);

                    res.into_body().on_upgrade()
                })
                .map_err(|err| {
                    error!("Client Error: {:?}", err);
                    WebSocketError::from(err)
                })
                .and_then(move |upgraded| {
                    let codec = Codec::new().client_mode();
                    let framed = codec.framed(upgraded);
                    let (sink, stream) = framed.split();

                    let mut blocking_sink = sink.wait();

                    if let Some(f) = on_open {
                        if let Err(err) = handle_response(&mut blocking_sink, f()) {
                            if let Err(err) = sender.send(Err(err)) {
                                error!("Failed to send response to shutdown handle: {}", err);
                            }
                            return Either::A(future::ok(()));
                        }
                    }

                    Either::B(
                        stream
                            .map_err(|err| {
                                error!("Protocol Error: {:?}", err);
                                WebSocketError::from(err)
                            })
                            .take_while(move |message| {
                                let status = match message {
                                    Frame::Text(msg) | Frame::Binary(msg) => {
                                        let bytes = if let Some(bytes) = msg {
                                            bytes.to_vec()
                                        } else {
                                            Vec::new()
                                        };
                                        let result = handle_response(&mut blocking_sink, f(bytes));

                                        if let Err(err) = result {
                                            ConnectionStatus::UnexpectedClose(err)
                                        } else {
                                            ConnectionStatus::Open
                                        }
                                    }
                                    Frame::Ping(msg) => {
                                        debug!("Received Ping {} sending pong", msg);
                                        if let Err(err) = handle_response(
                                            &mut blocking_sink,
                                            WsResponse::Pong(msg.to_string()),
                                        ) {
                                            ConnectionStatus::UnexpectedClose(err)
                                        } else {
                                            ConnectionStatus::Open
                                        }
                                    }
                                    Frame::Pong(msg) => {
                                        debug!("Received Pong {}", msg);
                                        ConnectionStatus::Open
                                    }
                                    Frame::Close(msg) => {
                                        debug!("Received close message {:?}", msg);
                                        let result =
                                            do_shutdown(&mut blocking_sink, CloseCode::Normal)
                                                .map_err(WebSocketError::from);
                                        ConnectionStatus::Close(result)
                                    }
                                };

                                match (running_clone.load(Ordering::SeqCst), status) {
                                    (true, ConnectionStatus::Open) => future::ok(true),
                                    (false, ConnectionStatus::Open) => {
                                        let shutdown_result =
                                            do_shutdown(&mut blocking_sink, CloseCode::Normal)
                                                .map_err(WebSocketError::from);
                                        if let Err(err) = sender.send(shutdown_result) {
                                            error!(
                                                "Failed to send response to shutdown handle: {}",
                                                err
                                            );
                                        }
                                        future::ok(false)
                                    }
                                    (false, ConnectionStatus::Open) => {
                                        let shutdown_result =
                                            do_shutdown(&mut blocking_sink, CloseCode::Normal)
                                                .map_err(Error::from);
                                        if let Err(err) = sender.send(shutdown_result) {
                                            error!(
                                                "Failed to send response to shutdown handle: {}",
                                                err
                                            );
                                        }
                                        future::ok(false)
                                    }
                                    (_, ConnectionStatus::Close(res)) => {
                                        if let Err(err) = sender.send(res) {
                                            error!(
                                                "Failed to send response to shutdown handle: {}",
                                                err
                                            );
                                        }
                                        future::ok(false)
                                    }
                                }
                            })
                            .for_each(|_| future::ok(())),
                    )
                }),
        );

        Ok(Listen {
            future,
            running,
            receiver,
        })
    }
}

fn handle_response(
    wait_sink: &mut Wait<stream::SplitSink<Framed<Upgraded, Codec>>>,
    res: WsResponse,
) -> Result<(), WebSocketError> {
    match res {
        WsResponse::Text(msg) => wait_sink
            .send(Message::Text(msg))
            .and_then(|_| wait_sink.flush())
            .or_else(|protocol_error| {
                error!("Error occurred while handling message {:?}", protocol_error);
                if let Err(shutdown_error) = do_shutdown(wait_sink, CloseCode::Protocol) {
                    Err(WebSocketError::AbnormalShutdownError {
                        protocol_error,
                        shutdown_error,
                    })
                } else {
                    Err(WebSocketError::from(protocol_error))
                }
            }),
        WsResponse::Bytes(bytes) => wait_sink
            .send(Message::Binary(bytes.as_slice().into()))
            .and_then(|_| wait_sink.flush())
            .or_else(|protocol_error| {
                error!("Error occurred while handling message {:?}", protocol_error);
                if let Err(shutdown_error) = do_shutdown(wait_sink, CloseCode::Protocol) {
                    Err(WebSocketError::AbnormalShutdownError {
                        protocol_error,
                        shutdown_error,
                    })
                } else {
                    Err(WebSocketError::from(protocol_error))
                }
            }),
        WsResponse::Pong(msg) => wait_sink
            .send(Message::Pong(msg))
            .or_else(|protocol_error| {
                error!("Error occurred while handling message {:?}", protocol_error);
                if let Err(shutdown_error) = do_shutdown(wait_sink, CloseCode::Protocol) {
                    Err(WebSocketError::AbnormalShutdownError {
                        protocol_error,
                        shutdown_error,
                    })
                } else {
                    Err(WebSocketError::from(protocol_error))
                }
            }),
        WsResponse::Close => {
            do_shutdown(wait_sink, CloseCode::Normal).map_err(WebSocketError::from)
        }
        WsResponse::Empty => Ok(()),
    }
}

fn do_shutdown(
    blocking_sink: &mut Wait<stream::SplitSink<Framed<Upgraded, Codec>>>,
    close_code: CloseCode,
) -> Result<(), ws::ProtocolError> {
    blocking_sink
        .send(Message::Close(Some(CloseReason::from(close_code))))
        .and_then(|_| blocking_sink.flush())
        .and_then(|_| {
            debug!("Socket connection closed successfully");
            blocking_sink.close()
        })
        .or_else(|_| blocking_sink.close())
}

enum ConnectionStatus {
    Open,
    UnexpectedClose(WebSocketError),
    Close(Result<(), WebSocketError>),
}

/// Response object returned by `WebSocket` client callbacks.
#[derive(Debug)]
pub enum WsResponse {
    Empty,
    Close,
    Pong(String),
    Text(String),
    Bytes(Vec<u8>),
}
