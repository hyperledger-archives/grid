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
use std::io::Error as IoError;
use protobuf::ProtobufError;
use rustls::TLSError;
use url::ParseError;

#[derive(Debug)]
pub enum SplinterError {
    CertUtf8Error(String),
    CertificateCreationError,
    CouldNotResolveHostName,
    HostNameNotFound,
    PrivateKeyNotFound,
    ProtobufError(ProtobufError),
    IoError(IoError),
    TLSError(TLSError),
    ParseError(ParseError),
}

impl From<IoError> for SplinterError {
    fn from(e: IoError) -> Self {
        SplinterError::IoError(e)
    }
}

impl From<ProtobufError> for SplinterError {
    fn from(e: ProtobufError) -> Self {
        SplinterError::ProtobufError(e)
    }
}

impl From<TLSError> for SplinterError {
    fn from(e: TLSError) -> Self {
        SplinterError::TLSError(e)
    }
}

impl From<ParseError> for SplinterError {
    fn from(e: ParseError) -> Self {
        SplinterError::ParseError(e)
    }
}
