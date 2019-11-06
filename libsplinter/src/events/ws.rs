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
//! use splinter::events::{WsResponse, WebSocketClient, Reactor, ParseBytes};
//!
//! let reactor = Reactor::new();
//!
//! let mut ws = WebSocketClient::new(
//!    "http://echo.websocket.org", |ctx, msg: Vec<u8>| {
//!    if let Ok(s) = String::from_utf8(msg.clone()) {
//!         println!("Received {}", s);
//!    } else {
//!       println!("malformed message: {:?}", msg);
//!    };
//!    WsResponse::Text("welcome to earth!!!".to_string())
//! });
//!
//! // Optional callback for when connection is established
//! ws.on_open(|_| {
//!    println!("sending message");
//!    WsResponse::Text("hello, world".to_string())
//! });
//!
//! ws.on_error(move |err, ctx| {
//!     println!("Error!: {:?}", err);
//!     // ws instance can be used to restart websocket
//!     ctx.start_ws().unwrap();
//!     Ok(())
//! });
//!
//! reactor.igniter().start_ws(&ws).unwrap();
//!
//! sleep(time::Duration::from_secs(1));
//! println!("stopping");
//! reactor.shutdown().unwrap();
//! ```

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::{Duration, SystemTime};

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

use crate::events::{Igniter, ParseError, WebSocketError};

type OnErrorHandle<T> =
    dyn Fn(&WebSocketError, Context<T>) -> Result<(), WebSocketError> + Send + Sync + 'static;

const MAX_FRAME_SIZE: usize = 10_000_000;
const DEFAULT_RECONNECT: bool = false;
const DEFAULT_RECONNECT_LIMIT: u64 = 10;

/// Wrapper around future created by `WebSocketClient`. In order for
/// the future to run it must be passed to `Igniter::start_ws`
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

    pub fn running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

/// WebSocket client. Configures Websocket connection and produces `Listen` future.
pub struct WebSocketClient<T: ParseBytes<T> + 'static = Vec<u8>> {
    url: String,
    on_message: Arc<dyn Fn(Context<T>, T) -> WsResponse + Send + Sync + 'static>,
    on_open: Option<Arc<dyn Fn(Context<T>) -> WsResponse + Send + Sync + 'static>>,
    on_error: Option<Arc<OnErrorHandle<T>>>,
    reconnect: bool,
    reconnect_limit: u64,
}

impl<T: ParseBytes<T> + 'static> Clone for WebSocketClient<T> {
    fn clone(&self) -> Self {
        WebSocketClient {
            url: self.url.clone(),
            on_message: self.on_message.clone(),
            on_open: self.on_open.clone(),
            on_error: self.on_error.clone(),
            reconnect: self.reconnect,
            reconnect_limit: self.reconnect_limit,
        }
    }
}

impl<T: ParseBytes<T> + 'static> WebSocketClient<T> {
    pub fn new<F>(url: &str, on_message: F) -> Self
    where
        F: Fn(Context<T>, T) -> WsResponse + Send + Sync + 'static,
    {
        Self {
            url: url.to_string(),
            on_message: Arc::new(on_message),
            on_open: None,
            on_error: None,
            reconnect: DEFAULT_RECONNECT,
            reconnect_limit: DEFAULT_RECONNECT_LIMIT,
        }
    }

    pub fn url(&self) -> String {
        self.url.clone()
    }

    pub fn set_reconnect(&mut self, reconnect: bool) {
        self.reconnect = reconnect
    }

    pub fn set_reconnect_limit(&mut self, reconnect_limit: u64) {
        self.reconnect_limit = reconnect_limit
    }

    pub fn reconnect(&self) -> bool {
        self.reconnect
    }

    pub fn reconnect_limit(&self) -> u64 {
        self.reconnect_limit
    }
    /// Adds optional `on_open` closure. This closer is called after a connection is initially
    /// established with the server, and is used for printing debug information and sending initial
    /// messages to server if necessary.
    pub fn on_open<F>(&mut self, on_open: F)
    where
        F: Fn(Context<T>) -> WsResponse + Send + Sync + 'static,
    {
        self.on_open = Some(Arc::new(on_open));
    }

    /// Adds optional `on_error` closure. This closure would be called when the Websocket has closed due to
    /// an unexpected error. This callback should be used to shutdown any IO resources being used by the
    /// Websocket or to reestablish the connection if appropriate.
    pub fn on_error<F>(&mut self, on_error: F)
    where
        F: Fn(&WebSocketError, Context<T>) -> Result<(), WebSocketError> + Send + Sync + 'static,
    {
        self.on_error = Some(Arc::new(on_error));
    }

    /// Returns `Listen` for WebSocket.
    pub fn listen(&self, mut context: Context<T>) -> Result<Listen, WebSocketError> {
        let url = self.url.clone();
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();
        let on_open = self
            .on_open
            .clone()
            .unwrap_or_else(|| Arc::new(|_| WsResponse::Empty));
        let on_message = self.on_message.clone();
        let on_error = self
            .on_error
            .clone()
            .unwrap_or_else(|| Arc::new(|_, _| Ok(())));

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
                    let codec = Codec::new().max_size(MAX_FRAME_SIZE).client_mode();
                    let framed = codec.framed(upgraded);
                    let (sink, stream) = framed.split();
                    let mut blocking_sink = sink.wait();

                    if let Err(err) = handle_response(
                        &mut blocking_sink,
                        on_open(context.clone()),
                        running_clone.clone(),
                    ) {
                        if let Err(err) = sender.send(Err(err)) {
                            error!("Failed to send response to shutdown handle: {}", err);
                        }
                        return Either::A(future::ok(()));
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
                                        let result = T::from_bytes(&bytes)
                                            .map_err(|parse_error| {
                                                error!(
                                                    "Failed to parse server message {}",
                                                    parse_error
                                                );
                                                if let Err(protocol_error) = do_shutdown(
                                                    &mut blocking_sink,
                                                    CloseCode::Protocol,
                                                    running_clone.clone(),
                                                ) {
                                                    WebSocketError::ParserError {
                                                        parse_error,
                                                        shutdown_error: Some(protocol_error),
                                                    }
                                                } else {
                                                    WebSocketError::ParserError {
                                                        parse_error,
                                                        shutdown_error: None,
                                                    }
                                                }
                                            })
                                            .and_then(|message| {
                                                handle_response(
                                                    &mut blocking_sink,
                                                    on_message(context.clone(), message),
                                                    running_clone.clone(),
                                                )
                                            });

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
                                            running_clone.clone(),
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
                                        let result = do_shutdown(
                                            &mut blocking_sink,
                                            CloseCode::Normal,
                                            running_clone.clone(),
                                        )
                                        .map_err(WebSocketError::from);
                                        ConnectionStatus::Close(result)
                                    }
                                };

                                match (running_clone.load(Ordering::SeqCst), status) {
                                    (true, ConnectionStatus::Open) => future::ok(true),
                                    (false, ConnectionStatus::Open) => {
                                        let shutdown_result = do_shutdown(
                                            &mut blocking_sink,
                                            CloseCode::Normal,
                                            running_clone.clone(),
                                        )
                                        .map_err(WebSocketError::from);
                                        if let Err(err) = sender.send(shutdown_result) {
                                            error!(
                                                "Failed to send response to shutdown handle: {}",
                                                err
                                            );
                                        }
                                        future::ok(false)
                                    }
                                    (_, ConnectionStatus::UnexpectedClose(original_error)) => {
                                        let result = on_error(&original_error, context.clone())
                                            .map_err(|on_fail_error| WebSocketError::OnFailError {
                                                original_error: Box::new(original_error),
                                                on_fail_error: Box::new(on_fail_error),
                                            });
                                        if let Err(err) = sender.send(result) {
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
    running: Arc<AtomicBool>,
) -> Result<(), WebSocketError> {
    match res {
        WsResponse::Text(msg) => wait_sink
            .send(Message::Text(msg))
            .and_then(|_| wait_sink.flush())
            .or_else(|protocol_error| {
                error!("Error occurred while handling message {:?}", protocol_error);
                if let Err(shutdown_error) = do_shutdown(wait_sink, CloseCode::Protocol, running) {
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
                if let Err(shutdown_error) = do_shutdown(wait_sink, CloseCode::Protocol, running) {
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
                if let Err(shutdown_error) = do_shutdown(wait_sink, CloseCode::Protocol, running) {
                    Err(WebSocketError::AbnormalShutdownError {
                        protocol_error,
                        shutdown_error,
                    })
                } else {
                    Err(WebSocketError::from(protocol_error))
                }
            }),
        WsResponse::Close => {
            do_shutdown(wait_sink, CloseCode::Normal, running).map_err(WebSocketError::from)
        }
        WsResponse::Empty => Ok(()),
    }
}

fn do_shutdown(
    blocking_sink: &mut Wait<stream::SplitSink<Framed<Upgraded, Codec>>>,
    close_code: CloseCode,
    running: Arc<AtomicBool>,
) -> Result<(), ws::ProtocolError> {
    debug!("Sending close to server");

    running.store(false, Ordering::SeqCst);
    blocking_sink
        .send(Message::Close(Some(CloseReason::from(close_code))))
        .and_then(|_| blocking_sink.flush())
        .and_then(|_| {
            debug!("Socket connection closed successfully");
            blocking_sink.close()
        })
        .or_else(|_| blocking_sink.close())
}

/// Websocket context object. It contains an Igniter pointing
/// to the Reactor on which the websocket future is running and
/// a copy of the WebSocketClient object.
#[derive(Clone)]
pub struct Context<T: ParseBytes<T> + 'static> {
    igniter: Igniter,
    ws: WebSocketClient<T>,
    reconnect_count: u64,
    last_reconnect: SystemTime,
    wait: Duration,
}

impl<T: ParseBytes<T> + 'static> Context<T> {
    pub fn new(igniter: Igniter, ws: WebSocketClient<T>) -> Self {
        Self {
            igniter,
            ws,
            reconnect_count: 0,
            last_reconnect: SystemTime::now(),
            wait: Duration::from_secs(1),
        }
    }

    /// Starts an instance of the Context's websocket.
    pub fn start_ws(&self) -> Result<(), WebSocketError> {
        let listen = self.ws.listen(self.clone())?;
        self.igniter.start_ws_with_listen(listen)
    }

    /// Returns a copy of the igniter used to start the websocket.
    pub fn igniter(&self) -> Igniter {
        self.igniter.clone()
    }

    /// Should called by the ws to inform that the connection was established successfully
    /// the Context resets the wait and reconnect cound to its intial values.
    pub fn ws_connected(&mut self) {
        self.reset_wait();
        self.reset_reconnect_count();
    }

    /// Checks that ws client can reconnect. If it can it attempts to reconnect if it cannot it
    /// calls the on_error function provided by the user and exits.
    pub fn try_reconnect(&mut self) -> Result<(), WebSocketError> {
        // Check that the ws is configure for automatic reconnect attempts and that the number
        // of reconnect attempts hasn't exceeded the maximum configure
        if self.ws.reconnect && self.reconnect_count < self.ws.reconnect_limit {
            self.reconnect()
        } else {
            let error_message = if self.ws.reconnect {
                WebSocketError::ReconnectError(
                    "Cannot connect to ws server. Reached maximum limit of reconnection attempts"
                        .to_string(),
                )
            } else {
                WebSocketError::ConnectError("Cannot connect to ws server".to_string())
            };
            let on_error = self
                .ws
                .on_error
                .clone()
                .unwrap_or_else(|| Arc::new(|_, _| Ok(())));

            self.reset_wait();
            self.reset_reconnect_count();
            on_error(&error_message, self.clone())
        }
    }

    fn reconnect(&mut self) -> Result<(), WebSocketError> {
        // loop until wait time has passed or reactor received shutdown signal
        debug!("Reconnecting in {:?}", self.wait);
        loop {
            // time elapsed since last reconnect attempt
            let elapsed = SystemTime::now()
                .duration_since(self.last_reconnect)
                .unwrap_or(Duration::from_secs(0));

            if elapsed >= self.wait {
                break;
            }

            if !self.igniter.is_reactor_running() {
                return Ok(());
            }
        }

        self.reconnect_count += 1;
        self.last_reconnect = SystemTime::now();

        let new_wait = self.wait.as_secs_f64() * 2.0;

        self.wait = Duration::from_secs_f64(new_wait);

        debug!(
            "Attempting to reconnect. Attempt number {} out of {}",
            self.reconnect_count, self.ws.reconnect_limit
        );

        self.start_ws()
    }

    fn reset_reconnect_count(&mut self) {
        self.reconnect_count = 0
    }

    fn reset_wait(&mut self) {
        self.wait = Duration::from_secs(1)
    }
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

pub trait ParseBytes<T: 'static>: Send + Sync + Clone {
    fn from_bytes(bytes: &[u8]) -> Result<T, ParseError>;
}

impl ParseBytes<Vec<u8>> for Vec<u8> {
    fn from_bytes(bytes: &[u8]) -> Result<Vec<u8>, ParseError> {
        Ok(bytes.to_vec())
    }
}
