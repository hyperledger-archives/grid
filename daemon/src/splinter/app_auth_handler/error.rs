/*
 * Copyright 2020 Cargill Incorporated
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * -----------------------------------------------------------------------------
 */

use splinter::events;
use std::error::Error;
use std::fmt;

use crate::event::EventIoError;
use crate::splinter::{app_auth_handler::node::GetNodeError, event::ScabbardEventConnectionError};

#[derive(Debug)]
pub enum AppAuthHandlerError {
    WebSocketError(events::WebSocketError),
    GetNodeError(GetNodeError),
    InvalidMessageError(String),
    ScabbardEventConnectionError(ScabbardEventConnectionError),
    EventIoError(EventIoError),
    EventProcessorError(String),
}

impl Error for AppAuthHandlerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            AppAuthHandlerError::WebSocketError(err) => Some(err),
            AppAuthHandlerError::GetNodeError(err) => Some(err),
            AppAuthHandlerError::InvalidMessageError(_) => None,
            AppAuthHandlerError::ScabbardEventConnectionError(err) => Some(err),
            AppAuthHandlerError::EventIoError(err) => Some(err),
            AppAuthHandlerError::EventProcessorError(_) => None,
        }
    }
}

impl fmt::Display for AppAuthHandlerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppAuthHandlerError::WebSocketError(msg) => write!(f, "WebsocketError {}", msg),
            AppAuthHandlerError::GetNodeError(msg) => write!(f, "GetNodeError {}", msg),
            AppAuthHandlerError::InvalidMessageError(msg) => {
                write!(f, "The client received an invalid message: {}", msg)
            }
            AppAuthHandlerError::ScabbardEventConnectionError(msg) => {
                write!(f, "ScabbardEventConnectionError {}", msg)
            }
            AppAuthHandlerError::EventIoError(msg) => write!(f, "EventIoError {}", msg),
            AppAuthHandlerError::EventProcessorError(msg) => {
                write!(f, "Event processor error: {}", msg)
            }
        }
    }
}

impl From<std::string::FromUtf8Error> for AppAuthHandlerError {
    fn from(err: std::string::FromUtf8Error) -> AppAuthHandlerError {
        AppAuthHandlerError::InvalidMessageError(format!("{}", err))
    }
}

impl From<events::WebSocketError> for AppAuthHandlerError {
    fn from(err: events::WebSocketError) -> Self {
        AppAuthHandlerError::WebSocketError(err)
    }
}

impl From<GetNodeError> for AppAuthHandlerError {
    fn from(err: GetNodeError) -> Self {
        AppAuthHandlerError::GetNodeError(err)
    }
}

impl From<ScabbardEventConnectionError> for AppAuthHandlerError {
    fn from(err: ScabbardEventConnectionError) -> Self {
        AppAuthHandlerError::ScabbardEventConnectionError(err)
    }
}

impl From<EventIoError> for AppAuthHandlerError {
    fn from(err: EventIoError) -> Self {
        AppAuthHandlerError::EventIoError(err)
    }
}
