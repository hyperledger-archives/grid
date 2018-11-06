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

use super::DaemonRequest;
use bytes::Bytes;
use protobuf;
use rustls::TLSError;
use std::io;
use std::net;
use std::sync::{mpsc::RecvError, mpsc::SendError};
use url;
use webpki;

use connection::ConnectionError;

#[derive(Debug)]
pub enum SplinterError {
    DnsError(String),
    IoError(io::Error),
    ProtobufError(protobuf::ProtobufError),
    CertUtf8Error(String),
    UrlParseError(url::ParseError),
    TlsError(TLSError),
    ChannelRecvError(RecvError),
    ChannelSendErrorBytes(SendError<Bytes>),
    ChannelSendErrorDaemonRequest(SendError<DaemonRequest>),
    WebpkiError(webpki::Error),
    AddrParseError(net::AddrParseError),
    AddCircuitError(AddCircuitError),
    RemoveCircuitError(RemoveCircuitError),
    CertificateCreationError,
    CouldNotResolveHostName,
    PrivateKeyNotFound,
    HostNameNotFound,
    PortNotIdentified,
    //
    ConnectionError(ConnectionError),
}

impl From<io::Error> for SplinterError {
    fn from(e: io::Error) -> Self {
        SplinterError::IoError(e)
    }
}

impl From<protobuf::ProtobufError> for SplinterError {
    fn from(e: protobuf::ProtobufError) -> Self {
        SplinterError::ProtobufError(e)
    }
}

impl From<url::ParseError> for SplinterError {
    fn from(e: url::ParseError) -> Self {
        SplinterError::UrlParseError(e)
    }
}

impl From<TLSError> for SplinterError {
    fn from(e: TLSError) -> Self {
        SplinterError::TlsError(e)
    }
}

impl From<RecvError> for SplinterError {
    fn from(e: RecvError) -> Self {
        SplinterError::ChannelRecvError(e)
    }
}

impl From<SendError<Bytes>> for SplinterError {
    fn from(e: SendError<Bytes>) -> Self {
        SplinterError::ChannelSendErrorBytes(e)
    }
}

impl From<SendError<DaemonRequest>> for SplinterError {
    fn from(e: SendError<DaemonRequest>) -> Self {
        SplinterError::ChannelSendErrorDaemonRequest(e)
    }
}

impl From<webpki::Error> for SplinterError {
    fn from(e: webpki::Error) -> Self {
        SplinterError::WebpkiError(e)
    }
}

impl From<net::AddrParseError> for SplinterError {
    fn from(e: net::AddrParseError) -> Self {
        SplinterError::AddrParseError(e)
    }
}

impl From<AddCircuitError> for SplinterError {
    fn from(e: AddCircuitError) -> Self {
        SplinterError::AddCircuitError(e)
    }
}

impl From<RemoveCircuitError> for SplinterError {
    fn from(e: RemoveCircuitError) -> Self {
        SplinterError::RemoveCircuitError(e)
    }
}

#[derive(Debug)]
pub enum AddCircuitError {
    ChannelSendError(SendError<DaemonRequest>),
    SendError(String),
    AddrParseError(net::AddrParseError),
}

impl From<net::AddrParseError> for AddCircuitError {
    fn from(e: net::AddrParseError) -> Self {
        AddCircuitError::AddrParseError(e)
    }
}

impl From<SendError<DaemonRequest>> for AddCircuitError {
    fn from(e: SendError<DaemonRequest>) -> Self {
        AddCircuitError::ChannelSendError(e)
    }
}

#[derive(Debug)]
pub enum RemoveCircuitError {
    SendError(String),
}
