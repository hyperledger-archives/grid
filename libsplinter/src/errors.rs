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

use protobuf;
use std::io;
use url;
use webpki;
use rustls::TLSError;
use std::sync::{
    PoisonError,
    MutexGuard,
    mpsc::RecvError,
    mpsc::SendError
};
use bytes::Bytes;

#[derive(Debug)]
pub enum SplinterError {
    DnsError(String),
    IoError(io::Error),
    ProtobufError(protobuf::ProtobufError),
    CertUtf8Error(String),
    UrlParseError(url::ParseError),
    TlsError(TLSError),
    ChannelRecvError(RecvError),
    ChannelSendError(SendError<Bytes>),
    WebpkiError(webpki::Error),
    CertificateCreationError,
    CouldNotResolveHostName,
    PrivateKeyNotFound,
    HostNameNotFound
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
        SplinterError::ChannelSendError(e)
    }
}

impl From<webpki::Error> for SplinterError {
    fn from(e: webpki::Error) -> Self {
        SplinterError::WebpkiError(e)
    }
}
