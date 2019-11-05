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

use actix_http::ws;
use crossbeam_channel::RecvError;
use std::{error, fmt};
use tokio::io;

#[derive(Debug)]
pub enum ParseError {
    MalformedMessage(Box<dyn error::Error + Send + Sync + 'static>),
}

impl error::Error for ParseError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            ParseError::MalformedMessage(_) => None,
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::MalformedMessage(err) => write!(f, "Malformed message {}", err),
        }
    }
}

#[derive(Debug)]
pub enum ReactorError {
    WsStartError(String),
    ListenError(WebSocketError),
    RequestSendError(String),
    ReactorShutdownError(String),
    ShutdownHandleErrors(Vec<WebSocketError>),
    IoError(io::Error),
}

impl error::Error for ReactorError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            ReactorError::ListenError(err) => Some(err),
            ReactorError::WsStartError(_) => None,
            ReactorError::RequestSendError(_) => None,
            ReactorError::ReactorShutdownError(_) => None,
            ReactorError::ShutdownHandleErrors(_) => None,
            ReactorError::IoError(err) => Some(err),
        }
    }
}

impl fmt::Display for ReactorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ReactorError::ListenError(err) => write!(f, "{}", err),
            ReactorError::WsStartError(err) => write!(f, "{}", err),
            ReactorError::RequestSendError(err) => write!(f, "{}", err),
            ReactorError::ReactorShutdownError(err) => write!(f, "{}", err),
            ReactorError::ShutdownHandleErrors(err) => {
                let err_message = err
                    .iter()
                    .map(|err| format!("{}", err))
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "Websockets did not shut down correctly: {}", err_message)
            }
            ReactorError::IoError(err) => write!(f, "IO Error: {}", err),
        }
    }
}

impl From<io::Error> for ReactorError {
    fn from(err: io::Error) -> Self {
        ReactorError::IoError(err)
    }
}

impl From<WebSocketError> for ReactorError {
    fn from(err: WebSocketError) -> Self {
        ReactorError::ListenError(err)
    }
}

#[derive(Debug)]
pub enum WebSocketError {
    HyperError(hyper::error::Error),
    /// Error returned when the client is attempting to communicate to
    /// the server using an unrecognized protocol. An example of this
    /// would be sending bytes to a server expecting text responses.
    ///
    /// The client usually cannot recover from these errors because
    /// they are usually caused by runtime error encountered in the
    /// listener or on open callbacks.
    ProtocolError(ws::ProtocolError),
    ShutdownHandleError(RecvError),
    RequestBuilderError(String),
    /// Error returned when Websocket fails to shutdown gracefully after
    /// encountering a protocol error.
    AbnormalShutdownError {
        protocol_error: ws::ProtocolError,
        shutdown_error: ws::ProtocolError,
    },
    OnFailError {
        original_error: Box<dyn error::Error + Send + Sync + 'static>,
        on_fail_error: Box<dyn error::Error + Send + Sync + 'static>,
    },
    NoParserDefinedError,
    ParserError {
        parse_error: ParseError,
        shutdown_error: Option<ws::ProtocolError>,
    },
    ListenError(String),
    /// Error returned when the client cannot establish a connection to the server
    ConnectError(String),
    /// Error returned when the client, after the connection with the server closed unexpectedly,
    /// tries to reestablish the connection but fails.
    ReconnectError(String),
    OnFailCallbackError(String),
}

impl error::Error for WebSocketError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            WebSocketError::HyperError(err) => Some(err),
            WebSocketError::ProtocolError(_) => None,
            WebSocketError::ShutdownHandleError(err) => Some(err),
            WebSocketError::RequestBuilderError(_) => None,
            WebSocketError::AbnormalShutdownError { .. } => None,
            WebSocketError::OnFailError { .. } => None,
            WebSocketError::NoParserDefinedError => None,
            WebSocketError::ParserError { .. } => None,
            WebSocketError::ListenError(_) => None,
            WebSocketError::ConnectError(_) => None,
            WebSocketError::ReconnectError(_) => None,
            WebSocketError::OnFailCallbackError(_) => None,
        }
    }
}

impl fmt::Display for WebSocketError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WebSocketError::HyperError(err) => write!(f, "Hyper Error: {}", err),
            WebSocketError::ProtocolError(err) => write!(f, "Protocol Error: {}", err),
            WebSocketError::ShutdownHandleError(err) => {
                write!(f, "Shutdown handle failed unexpectedly: {}", err)
            }
            WebSocketError::RequestBuilderError(err) => {
                write!(f, "Failed to build request: {}", err)
            }
            WebSocketError::AbnormalShutdownError {
                protocol_error,
                shutdown_error,
            } => write!(
                f,
                "A shutdown error \
                 occurred while handling protocol error: protocol error {}, shutdown error {}",
                protocol_error, shutdown_error
            ),
            WebSocketError::OnFailError {
                on_fail_error,
                original_error,
            } => write!(
                f,
                "A failure occured while executing \
                 the on fail callback: original error: {}, on fail error: {}",
                on_fail_error, original_error
            ),
            WebSocketError::NoParserDefinedError => write!(f, "Parsing function required"),
            WebSocketError::ParserError {
                parse_error,
                shutdown_error: Some(shutdown_error),
            } => write!(
                f,
                "Failed to parse message from server: parse error: {} shutdown error: {}",
                parse_error, shutdown_error
            ),
            WebSocketError::ParserError {
                parse_error,
                shutdown_error: None,
            } => write!(
                f,
                "Failed to parse message from server: parse error: {}",
                parse_error
            ),
            WebSocketError::ListenError(err) => write!(f, "{}", err),
            WebSocketError::ConnectError(err) => write!(f, "{}", err),
            WebSocketError::ReconnectError(err) => write!(f, "{}", err),
            WebSocketError::OnFailCallbackError(err) => write!(f, "{}", err),
        }
    }
}

impl From<hyper::error::Error> for WebSocketError {
    fn from(err: hyper::error::Error) -> Self {
        WebSocketError::HyperError(err)
    }
}

impl From<ws::ProtocolError> for WebSocketError {
    fn from(err: ws::ProtocolError) -> Self {
        WebSocketError::ProtocolError(err)
    }
}

impl From<RecvError> for WebSocketError {
    fn from(err: RecvError) -> Self {
        WebSocketError::ShutdownHandleError(err)
    }
}
