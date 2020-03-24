// Copyright 2018-2020 Cargill Incorporated
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

use std::error::Error;
use std::fmt;
use std::time::Duration;

use crate::transport::Connection;

/// Wrapper around payload to include connection id
#[derive(Debug, Default, PartialEq)]
pub struct Envelope {
    id: String,
    payload: Vec<u8>,
}

impl Envelope {
    pub fn new(id: String, payload: Vec<u8>) -> Self {
        Envelope { id, payload }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    pub fn take_payload(self) -> Vec<u8> {
        self.payload
    }
}

/// MatrixLifeCycle trait abstracts out adding and removing connections to a
/// connection handler without requiring knowledge about sending or receiving messges.
pub trait MatrixLifeCycle: Clone + Send {
    fn add(&self, connection: Box<dyn Connection>, id: String) -> Result<usize, MatrixAddError>;
    fn remove(&self, id: &str) -> Result<Box<dyn Connection>, MatrixRemoveError>;
}

pub trait MatrixSender: Clone + Send {
    fn send(&self, id: String, message: Vec<u8>) -> Result<(), MatrixSendError>;
}

pub trait MatrixReceiver: Clone + Send {
    fn recv(&self) -> Result<Envelope, MatrixRecvError>;
    fn recv_timeout(&self, timeout: Duration) -> Result<Envelope, MatrixRecvTimeoutError>;
}

#[derive(Debug)]
pub struct MatrixAddError {
    pub context: String,
    pub source: Option<Box<dyn Error + Send>>,
}

impl MatrixAddError {
    pub fn new(context: String, source: Option<Box<dyn Error + Send>>) -> Self {
        Self { context, source }
    }
}

impl Error for MatrixAddError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        if let Some(ref err) = self.source {
            Some(&**err)
        } else {
            None
        }
    }
}

impl fmt::Display for MatrixAddError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref err) = self.source {
            write!(f, "{}: {}", self.context, err)
        } else {
            f.write_str(&self.context)
        }
    }
}

#[derive(Debug)]
pub struct MatrixRemoveError {
    pub context: String,
    pub source: Option<Box<dyn Error + Send>>,
}

impl MatrixRemoveError {
    pub fn new(context: String, source: Option<Box<dyn Error + Send>>) -> Self {
        Self { context, source }
    }
}

impl Error for MatrixRemoveError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        if let Some(ref err) = self.source {
            Some(&**err)
        } else {
            None
        }
    }
}

impl fmt::Display for MatrixRemoveError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref err) = self.source {
            write!(f, "{}: {}", self.context, err)
        } else {
            f.write_str(&self.context)
        }
    }
}

#[derive(Debug)]
pub struct MatrixSendError {
    pub context: String,
    pub source: Option<Box<dyn Error + Send>>,
}

impl MatrixSendError {
    pub fn new(context: String, source: Option<Box<dyn Error + Send>>) -> Self {
        Self { context, source }
    }
}

impl Error for MatrixSendError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        if let Some(ref err) = self.source {
            Some(&**err)
        } else {
            None
        }
    }
}

impl fmt::Display for MatrixSendError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref err) = self.source {
            write!(f, "{}: {}", self.context, err)
        } else {
            f.write_str(&self.context)
        }
    }
}

#[derive(Debug)]
pub enum MatrixRecvError {
    Disconnected,
    InternalError {
        context: String,
        source: Option<Box<dyn Error + Send>>,
    },
}

impl MatrixRecvError {
    pub fn new_internal_error(context: String, source: Option<Box<dyn Error + Send>>) -> Self {
        MatrixRecvError::InternalError { context, source }
    }
}

impl Error for MatrixRecvError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            MatrixRecvError::Disconnected => None,
            MatrixRecvError::InternalError { source, .. } => {
                if let Some(ref err) = source {
                    Some(&**err)
                } else {
                    None
                }
            }
        }
    }
}

impl fmt::Display for MatrixRecvError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MatrixRecvError::Disconnected => {
                f.write_str("Unable to receive: channel has disconnected")
            }
            MatrixRecvError::InternalError { context, source } => {
                if let Some(ref err) = source {
                    write!(f, "{}: {}", context, err)
                } else {
                    f.write_str(&context)
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum MatrixRecvTimeoutError {
    Timeout,
    Disconnected,
    InternalError {
        context: String,
        source: Option<Box<dyn Error + Send>>,
    },
}

impl MatrixRecvTimeoutError {
    pub fn new_internal_error(context: String, source: Option<Box<dyn Error + Send>>) -> Self {
        MatrixRecvTimeoutError::InternalError { context, source }
    }
}

impl Error for MatrixRecvTimeoutError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            MatrixRecvTimeoutError::Timeout => None,
            MatrixRecvTimeoutError::Disconnected => None,
            MatrixRecvTimeoutError::InternalError { source, .. } => {
                if let Some(ref err) = source {
                    Some(&**err)
                } else {
                    None
                }
            }
        }
    }
}

impl std::fmt::Display for MatrixRecvTimeoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MatrixRecvTimeoutError::Timeout => f.write_str("Unable to receive: Timeout"),
            MatrixRecvTimeoutError::Disconnected => {
                f.write_str("Unable to receive: channel has disconnected")
            }
            MatrixRecvTimeoutError::InternalError { context, source } => {
                if let Some(ref err) = source {
                    write!(f, "{}: {}", context, err)
                } else {
                    f.write_str(&context)
                }
            }
        }
    }
}
