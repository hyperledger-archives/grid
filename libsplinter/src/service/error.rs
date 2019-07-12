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

//! Errors that can occur in a service
use std::borrow::Borrow;
use std::error::Error;

#[derive(Debug)]
pub struct ServiceSendError(pub Box<dyn Error>);

impl Error for ServiceSendError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.0.borrow())
    }
}

impl std::fmt::Display for ServiceSendError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "unable to send message: {}", self.0)
    }
}

#[derive(Debug)]
pub struct ServiceConnectionError(pub Box<dyn Error>);

impl Error for ServiceConnectionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.0.borrow())
    }
}

impl std::fmt::Display for ServiceConnectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "unable to connect service: {}", self.0)
    }
}

#[derive(Debug)]
pub struct ServiceDisconnectionError(pub Box<dyn Error>);

impl Error for ServiceDisconnectionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.0.borrow())
    }
}

impl std::fmt::Display for ServiceDisconnectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "unable to disconnect service: {}", self.0)
    }
}
#[derive(Debug)]
pub struct ServiceStartError(pub Box<dyn Error>);

impl Error for ServiceStartError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.0.borrow())
    }
}

impl std::fmt::Display for ServiceStartError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "unable to start service: {}", self.0)
    }
}

#[derive(Debug)]
pub struct ServiceStopError(pub Box<dyn Error>);

impl Error for ServiceStopError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.0.borrow())
    }
}

impl std::fmt::Display for ServiceStopError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "unable to stop service : {}", self.0)
    }
}

#[derive(Debug)]
pub struct ServiceDestroyError(pub Box<dyn Error>);

impl Error for ServiceDestroyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.0.borrow())
    }
}

impl std::fmt::Display for ServiceDestroyError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "unable to destroy service: {}", self.0)
    }
}

#[derive(Debug)]
pub enum ServiceError {
    /// Returned if an error is detected when parsing a message
    InvalidMessageFormat(Box<dyn Error>),
    /// Returned if an error is detected during the handling of a message
    UnableToHandleMessage(Box<dyn Error>),
    /// Returned if an error occurs during the sending of an outbound message
    UnableToSendMessage(Box<ServiceSendError>),
}

impl Error for ServiceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ServiceError::InvalidMessageFormat(ref err) => Some(err.borrow()),
            ServiceError::UnableToHandleMessage(ref err) => Some(err.borrow()),
            ServiceError::UnableToSendMessage(err) => Some(err),
        }
    }
}

impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            ServiceError::InvalidMessageFormat(ref err) => {
                write!(f, "message is in an invalid format: {}", err)
            }
            ServiceError::UnableToHandleMessage(ref err) => {
                write!(f, "cannot handle message {}", err)
            }
            ServiceError::UnableToSendMessage(ref err) => {
                write!(f, "unable to send message: {}", err)
            }
        }
    }
}
