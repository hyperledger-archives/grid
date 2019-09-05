/*
 * Copyright 2019 Cargill Incorporated
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

use std::error::Error;
use std::fmt;

use futures::future;
use libsplinter::events::ws;

use crate::application_metadata::ApplicationMetadataError;

#[derive(Debug)]
pub enum AppAuthHandlerError {
    IOError(std::io::Error),
    InvalidMessageError(String),
    DatabaseError(String),
    WebSocketError(ws::Error),
}

impl Error for AppAuthHandlerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            AppAuthHandlerError::IOError(err) => Some(err),
            AppAuthHandlerError::InvalidMessageError(_) => None,
            AppAuthHandlerError::DatabaseError(_) => None,
            AppAuthHandlerError::WebSocketError(err) => Some(err),
        }
    }
}

impl fmt::Display for AppAuthHandlerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppAuthHandlerError::IOError(msg) => write!(f, "An I/O error occurred: {}", msg),
            AppAuthHandlerError::InvalidMessageError(msg) => {
                write!(f, "The client received an invalid message: {}", msg)
            }
            AppAuthHandlerError::DatabaseError(msg) => {
                write!(f, "The database returned an error: {}", msg)
            }
            AppAuthHandlerError::WebSocketError(msg) => write!(f, "WebSocket Error: {}", msg),
        }
    }
}

impl From<std::io::Error> for AppAuthHandlerError {
    fn from(err: std::io::Error) -> AppAuthHandlerError {
        AppAuthHandlerError::IOError(err)
    }
}

impl From<serde_json::error::Error> for AppAuthHandlerError {
    fn from(err: serde_json::error::Error) -> AppAuthHandlerError {
        AppAuthHandlerError::InvalidMessageError(format!("{}", err))
    }
}

impl From<std::string::FromUtf8Error> for AppAuthHandlerError {
    fn from(err: std::string::FromUtf8Error) -> AppAuthHandlerError {
        AppAuthHandlerError::InvalidMessageError(format!("{}", err))
    }
}

impl From<ApplicationMetadataError> for AppAuthHandlerError {
    fn from(err: ApplicationMetadataError) -> AppAuthHandlerError {
        AppAuthHandlerError::InvalidMessageError(format!("{}", err))
    }
}

impl From<gameroom_database::DatabaseError> for AppAuthHandlerError {
    fn from(err: gameroom_database::DatabaseError) -> AppAuthHandlerError {
        AppAuthHandlerError::DatabaseError(format!("{}", err))
    }
}

impl From<diesel::result::Error> for AppAuthHandlerError {
    fn from(err: diesel::result::Error) -> Self {
        AppAuthHandlerError::DatabaseError(format!("Error performing query: {}", err))
    }
}

impl From<ws::Error> for AppAuthHandlerError {
    fn from(err: ws::Error) -> Self {
        AppAuthHandlerError::WebSocketError(err)
    }
}

impl<T> Into<future::FutureResult<T, AppAuthHandlerError>> for AppAuthHandlerError {
    fn into(self) -> future::FutureResult<T, AppAuthHandlerError> {
        future::err(self)
    }
}
