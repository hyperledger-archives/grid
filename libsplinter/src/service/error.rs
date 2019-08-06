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

//! Errors that can occur in a service and service processor
use std::error::Error;
use std::io::Error as IOError;

use protobuf::error::ProtobufError;

#[derive(Debug)]
pub struct ServiceSendError(pub Box<dyn Error + Send>);

impl Error for ServiceSendError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&*self.0)
    }
}

impl std::fmt::Display for ServiceSendError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "unable to send message: {}", self.0)
    }
}

#[derive(Debug)]
pub enum ServiceConnectionError {
    ConnectionError(Box<dyn Error + Send>),
    RejectedError(String),
    WrongResponse(String),
}

impl Error for ServiceConnectionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ServiceConnectionError::ConnectionError(err) => Some(&**err),
            ServiceConnectionError::RejectedError(_) => None,
            ServiceConnectionError::WrongResponse(_) => None,
        }
    }
}

impl std::fmt::Display for ServiceConnectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            ServiceConnectionError::ConnectionError(ref err) => {
                write!(f, "unable to connect service: {}", err)
            }
            ServiceConnectionError::RejectedError(ref err) => {
                write!(f, "connection request was rejected: {}", err)
            }
            ServiceConnectionError::WrongResponse(ref err) => {
                write!(f, "wrong response type was returned: {}", err)
            }
        }
    }
}

#[derive(Debug)]
pub enum ServiceDisconnectionError {
    DisconnectionError(Box<dyn Error + Send>),
    RejectedError(String),
    WrongResponse(String),
}

impl Error for ServiceDisconnectionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ServiceDisconnectionError::DisconnectionError(err) => Some(&**err),
            ServiceDisconnectionError::RejectedError(_) => None,
            ServiceDisconnectionError::WrongResponse(_) => None,
        }
    }
}

impl std::fmt::Display for ServiceDisconnectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            ServiceDisconnectionError::DisconnectionError(ref err) => {
                write!(f, "unable to disconnect service: {}", err)
            }
            ServiceDisconnectionError::RejectedError(ref err) => {
                write!(f, "disconnection request was rejected: {}", err)
            }
            ServiceDisconnectionError::WrongResponse(ref err) => {
                write!(f, "wrong response type was returned: {}", err)
            }
        }
    }
}

#[derive(Debug)]
pub enum ServiceStartError {
    AlreadyStarted,
    UnableToConnect(ServiceConnectionError),
    Internal(Box<dyn Error + Send>),
    PoisonedLock(String),
}

impl Error for ServiceStartError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ServiceStartError::AlreadyStarted => None,
            ServiceStartError::UnableToConnect(err) => Some(err),
            ServiceStartError::Internal(err) => Some(&**err),
            ServiceStartError::PoisonedLock(_) => None,
        }
    }
}

impl std::fmt::Display for ServiceStartError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ServiceStartError::AlreadyStarted => write!(f, "service already started"),
            ServiceStartError::UnableToConnect(err) => {
                write!(f, "unable to connect on start: {}", err)
            }
            ServiceStartError::Internal(err) => write!(f, "unable to start service: {}", err),
            ServiceStartError::PoisonedLock(msg) => write!(f, "a lock was poisoned: {}", msg),
        }
    }
}

impl From<ServiceConnectionError> for ServiceStartError {
    fn from(err: ServiceConnectionError) -> Self {
        ServiceStartError::UnableToConnect(err)
    }
}

#[derive(Debug)]
pub enum ServiceStopError {
    NotStarted,
    UnableToDisconnect(ServiceDisconnectionError),
    Internal(Box<dyn Error + Send>),
    PoisonedLock(String),
}

impl Error for ServiceStopError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ServiceStopError::NotStarted => None,
            ServiceStopError::UnableToDisconnect(err) => Some(err),
            ServiceStopError::Internal(err) => Some(&**err),
            ServiceStopError::PoisonedLock(_) => None,
        }
    }
}

impl std::fmt::Display for ServiceStopError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ServiceStopError::NotStarted => write!(f, "service not started"),
            ServiceStopError::UnableToDisconnect(err) => {
                write!(f, "unable to disconnect on stop: {}", err)
            }
            ServiceStopError::Internal(err) => write!(f, "unable to stop service: {}", err),
            ServiceStopError::PoisonedLock(msg) => write!(f, "a lock was poisoned: {}", msg),
        }
    }
}

impl From<ServiceDisconnectionError> for ServiceStopError {
    fn from(err: ServiceDisconnectionError) -> Self {
        ServiceStopError::UnableToDisconnect(err)
    }
}

#[derive(Debug)]
pub enum ServiceDestroyError {
    NotStopped,
    Internal(Box<dyn Error + Send>),
}

impl Error for ServiceDestroyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ServiceDestroyError::NotStopped => None,
            ServiceDestroyError::Internal(err) => Some(&**err),
        }
    }
}

impl std::fmt::Display for ServiceDestroyError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ServiceDestroyError::NotStopped => write!(f, "service not stopped"),
            ServiceDestroyError::Internal(err) => write!(f, "unable to destroy service: {}", err),
        }
    }
}

#[derive(Debug)]
pub enum ServiceError {
    /// Returned if an error is detected when parsing a message
    InvalidMessageFormat(Box<dyn Error + Send>),
    /// Returned if an error is detected during the handling of a message
    UnableToHandleMessage(Box<dyn Error + Send>),
    /// Returned if an error occurs during the sending of an outbound message
    UnableToSendMessage(Box<ServiceSendError>),

    /// Returned if a service encounters a poisoned lock and is unable to recover
    PoisonedLock(String),

    /// Returned if handle_message is called when not yet registered.
    NotStarted,
}

impl Error for ServiceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ServiceError::InvalidMessageFormat(err) => Some(&**err),
            ServiceError::UnableToHandleMessage(err) => Some(&**err),
            ServiceError::UnableToSendMessage(err) => Some(err),
            ServiceError::PoisonedLock(_) => None,
            ServiceError::NotStarted => None,
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
            ServiceError::PoisonedLock(ref msg) => write!(f, "a lock was poisoned: {}", msg),
            ServiceError::NotStarted => f.write_str("service not started"),
        }
    }
}

impl From<ProtobufError> for ServiceError {
    fn from(err: ProtobufError) -> Self {
        ServiceError::InvalidMessageFormat(Box::new(err))
    }
}

impl From<ServiceSendError> for ServiceError {
    fn from(err: ServiceSendError) -> Self {
        ServiceError::UnableToSendMessage(Box::new(err))
    }
}

#[derive(Debug)]
pub enum ServiceProcessorError {
    /// Returned if an error is detected adding a new service
    AddServiceError(String),
    /// Returned if an error is detected while processing requests
    ProcessError(Box<dyn Error + Send>),
    /// Returned if an IO error is detected while processing requests
    IOError(IOError),
    /// Returned if an error is detected when trying to shutdown
    ShutdownError(String),
}

impl Error for ServiceProcessorError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ServiceProcessorError::AddServiceError(_) => None,
            ServiceProcessorError::ProcessError(err) => Some(&**err),
            ServiceProcessorError::IOError(err) => Some(err),
            ServiceProcessorError::ShutdownError(_) => None,
        }
    }
}

impl std::fmt::Display for ServiceProcessorError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            ServiceProcessorError::AddServiceError(ref err) => {
                write!(f, "service cannot be added: {}", err)
            }
            ServiceProcessorError::ProcessError(ref err) => {
                write!(f, "error processing message {}", err)
            }
            ServiceProcessorError::IOError(ref err) => {
                write!(f, "io error processing message {}", err)
            }
            ServiceProcessorError::ShutdownError(ref err) => {
                write!(f, "error shutting down: {}", err)
            }
        }
    }
}

impl From<IOError> for ServiceProcessorError {
    fn from(error: IOError) -> Self {
        ServiceProcessorError::IOError(error)
    }
}
