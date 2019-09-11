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
use std::{error, fmt};
use std::{thread, time};

use actix_http::ws;
use awc::ws::{CloseCode, CloseReason, Codec, Frame, Message};
use crossbeam_channel::{bounded, Receiver, RecvError, TryRecvError};
use futures::{
    future::{self, Either},
    sink::Wait,
    Future,
};
use hyper::{self, header, upgrade::Upgraded, Body, Client, Request, StatusCode};
use tokio::codec::{Decoder, Framed};
use tokio::prelude::*;


/// Wrapper around future created by `WebSocketClient`. In order for
/// the future to run it must be passed to `WsRuntime::start`
pub struct Listen {
    future: Box<dyn Future<Item = (), Error = Error> + Send + 'static>,
    running: Arc<AtomicBool>,
    receiver: Receiver<Result<(), Error>>,
}

impl Listen {
    fn into_shutdown_handle(
        self,
    ) -> (
        Box<dyn Future<Item = (), Error = Error> + Send + 'static>,
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
    receiver: Receiver<Result<(), Error>>,
}

impl ShutdownHandle {
    /// Polls websocket for unexpected shutdowns
    /// and returns result.
    pub fn monitor(self) -> Result<(), Error> {
        loop {
            match self.receiver.try_recv() {
                Ok(res) => return res,
                Err(TryRecvError::Empty) => {
                    thread::sleep(time::Duration::from_secs(1));
                    continue;
                }
                Err(err) => return Err(Error::PollingError(err)),
            }
        }
    }

    /// Sends shutdown message to websocket
    pub fn shutdown(self) -> Result<(), Error> {
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
            .map_err(|err| Error::RequestBuilderError(format!("{:?}", err)))?;

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
                    Error::from(err)
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
                                Error::from(err)
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

                                        if result.is_err() {
                                            ConnectionStatus::Closed(result)
                                        } else {
                                            ConnectionStatus::Open(result)
                                        }
                                    }
                                    Frame::Ping(msg) => {
                                        debug!("Received Ping {} sending pong", msg);
                                        let result = handle_response(
                                            &mut blocking_sink,
                                            WsResponse::Text(msg.to_string()),
                                        );

                                        if result.is_err() {
                                            ConnectionStatus::Closed(result)
                                        } else {
                                            ConnectionStatus::Open(result)
                                        }
                                    }
                                    Frame::Pong(msg) => {
                                        debug!("Received Pong {}", msg);
                                        ConnectionStatus::Open(Ok(()))
                                    }
                                    Frame::Close(msg) => {
                                        debug!("Received close message {:?}", msg);
                                        let result =
                                            do_shutdown(&mut blocking_sink, CloseCode::Normal)
                                                .map_err(Error::from);
                                        ConnectionStatus::Closed(result)
                                    }
                                };

                                match (running_clone.load(Ordering::SeqCst), status) {
                                    (true, ConnectionStatus::Open(_)) => future::ok(true),
                                    (true, ConnectionStatus::Closed(res)) => {
                                        if let Err(err) = sender.send(res) {
                                            error!(
                                                "Failed to send response to shutdown handle: {}",
                                                err
                                            );
                                        }
                                        future::ok(false)
                                    }
                                    (false, ConnectionStatus::Open(_)) => {
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
                                    (false, ConnectionStatus::Closed(res)) => {
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
) -> Result<(), Error> {
    match res {
        WsResponse::Text(msg) => wait_sink
            .send(Message::Text(msg))
            .and_then(|_| wait_sink.flush())
            .or_else(|protocol_error| {
                error!("Error occurred while handling message {:?}", protocol_error);
                if let Err(shutdown_error) = do_shutdown(wait_sink, CloseCode::Protocol) {
                    Err(Error::AbnormalShutdownError {
                        protocol_error,
                        shutdown_error,
                    })
                } else {
                    Err(Error::from(protocol_error))
                }
            }),
        WsResponse::Bytes(bytes) => wait_sink
            .send(Message::Binary(bytes.as_slice().into()))
            .and_then(|_| wait_sink.flush())
            .or_else(|protocol_error| {
                error!("Error occurred while handling message {:?}", protocol_error);
                if let Err(shutdown_error) = do_shutdown(wait_sink, CloseCode::Protocol) {
                    Err(Error::AbnormalShutdownError {
                        protocol_error,
                        shutdown_error,
                    })
                } else {
                    Err(Error::from(protocol_error))
                }
            }),
        WsResponse::Close => do_shutdown(wait_sink, CloseCode::Normal).map_err(Error::from),
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
    Open(Result<(), Error>),
    Closed(Result<(), Error>),
}

/// Response object returned by `WebSocket` client callbacks.
#[derive(Debug)]
pub enum WsResponse {
    Empty,
    Close,
    Text(String),
    Bytes(Vec<u8>),
}

#[derive(Debug)]
pub enum Error {
    HyperError(hyper::error::Error),
    /// Error returned when the client is attempting to communicate to
    /// the server using an unrecognized protocol. An example of this
    /// would be sending bytes to a server expecting text responses.
    ///
    /// The client usually cannot not recover from these errors because
    /// they are usually caused by runtime error encountered in the
    /// listener or on open callbacks.
    ProtocolError(ws::ProtocolError),
    RequestBuilderError(String),
    IoError(io::Error),
    ShutdownHandleError(RecvError),
    PollingError(TryRecvError),
    RuntimeShutdownError,
    /// Error returned when Websocket fails to shutdown gracefully after
    /// encountering a protocol error.
    AbnormalShutdownError {
        protocol_error: ws::ProtocolError,
        shutdown_error: ws::ProtocolError,
    },
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::HyperError(err) => Some(err),
            Error::ProtocolError(_) => None,
            Error::RequestBuilderError(_) => None,
            Error::IoError(err) => Some(err),
            Error::ShutdownHandleError(err) => Some(err),
            Error::PollingError(err) => Some(err),
            Error::RuntimeShutdownError => None,
            Error::AbnormalShutdownError { .. } => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::HyperError(err) => write!(f, "Hyper Error: {}", err),
            Error::ProtocolError(err) => write!(f, "Protocol Error: {}", err),
            Error::RequestBuilderError(s) => write!(f, "Failed to build request: {}", s),
            Error::IoError(err) => write!(f, "IO Error: {}", err),
            Error::ShutdownHandleError(err) => write!(f, "Failed to retrieve listener shutdown: {}", err),
            Error::PollingError(err) => write!(f, "Polling Error: {}", err),
            Error::RuntimeShutdownError => write!(f, "Failed to gracefully shutdown Websocket runtime"),
            Error::AbnormalShutdownError { protocol_error, shutdown_error } => write!(f, "A shutdown error occurred while handling protocol error: protocol error {}, shutdown error {}", protocol_error, shutdown_error)
        }
    }
}

impl From<hyper::error::Error> for Error {
    fn from(err: hyper::error::Error) -> Self {
        Error::HyperError(err)
    }
}

impl From<ws::ProtocolError> for Error {
    fn from(err: ws::ProtocolError) -> Self {
        Error::ProtocolError(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IoError(err)
    }
}

impl From<RecvError> for Error {
    fn from(err: RecvError) -> Self {
        Error::ShutdownHandleError(err)
    }
}
