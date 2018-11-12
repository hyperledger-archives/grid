// Copyright 2018 Cargill Incorporated
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

pub mod tls_connection;

use async::NoBlock;
use errors::SplinterError;

use std::io::Error as IoError;

use messaging::protocol::Message;

/// A single, bi-directional connection between two nodes, possibly secured
pub trait Connection {
    fn handshake(&mut self) -> Result<NoBlock<()>, HandshakeError>;
    fn read(&mut self) -> Result<NoBlock<Message>, ReadError>;
    fn write(&mut self, msg: &Message) -> Result<NoBlock<()>, WriteError>;
}

// -- Errors --

#[derive(Debug)]
pub enum HandshakeError {
    ProtocolError(String),
}

impl From<HandshakeError> for ConnectionError {
    fn from(handshake_error: HandshakeError) -> Self {
        ConnectionError::HandshakeError(handshake_error)
    }
}

#[derive(Debug)]
pub enum ReadError {
    IoError(IoError),
    ProtocolError(String),
    ParseError(String),
}

impl From<IoError> for ReadError {
    fn from(io_error: IoError) -> Self {
        ReadError::IoError(io_error)
    }
}

impl From<ReadError> for ConnectionError {
    fn from(read_error: ReadError) -> Self {
        ConnectionError::ReadError(read_error)
    }
}

#[derive(Debug)]
pub enum WriteError {
    IoError(IoError),
    ProtocolError(String),
    ParseError(String),
}

impl From<IoError> for WriteError {
    fn from(io_error: IoError) -> Self {
        WriteError::IoError(io_error)
    }
}

impl From<WriteError> for ConnectionError {
    fn from(write_error: WriteError) -> Self {
        ConnectionError::WriteError(write_error)
    }
}

#[derive(Debug)]
pub enum ConnectionError {
    HandshakeError(HandshakeError),
    ReadError(ReadError),
    WriteError(WriteError),
}

impl<T: Into<ConnectionError>> From<T> for SplinterError {
    fn from(err: T) -> Self {
        SplinterError::ConnectionError(err.into())
    }
}
