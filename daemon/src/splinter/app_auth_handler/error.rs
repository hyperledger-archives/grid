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

use sabre_sdk::protocol::payload::{ActionBuildError, SabrePayloadBuildError};
use sawtooth_sdk::signing::Error as SigningError;
use scabbard::client::ScabbardClientError;
use splinter::events;
use std::error::Error;
use std::fmt;

use crate::event::EventIoError;
use crate::splinter::{app_auth_handler::node::GetNodeError, event::ScabbardEventConnectionError};
use transact::{
    contract::archive::Error as ContractArchiveError,
    protocol::{batch::BatchBuildError, transaction::TransactionBuildError},
};

#[derive(Debug)]
pub struct AppAuthHandlerError {
    message: Option<String>,
    source: Option<Box<dyn Error>>,
}

impl AppAuthHandlerError {
    pub fn with_message(message: &str) -> Self {
        Self {
            message: Some(message.to_string()),
            source: None,
        }
    }

    pub fn from_source(source: Box<dyn Error>) -> Self {
        Self {
            message: None,
            source: Some(source),
        }
    }
}

impl fmt::Display for AppAuthHandlerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match (&self.message, &self.source) {
            (Some(m), Some(s)) => write!(f, "{}: {}", m, s),
            (Some(m), _) => write!(f, "{}", m),
            (_, Some(s)) => write!(f, "{:?}", s),
            (None, None) => write!(f, "An internal error occured"),
        }
    }
}

impl Error for AppAuthHandlerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_ref().map(|s| s.as_ref())
    }
}

impl From<ActionBuildError> for AppAuthHandlerError {
    fn from(err: ActionBuildError) -> AppAuthHandlerError {
        AppAuthHandlerError::from_source(Box::new(err))
    }
}

impl From<SabrePayloadBuildError> for AppAuthHandlerError {
    fn from(err: SabrePayloadBuildError) -> AppAuthHandlerError {
        AppAuthHandlerError::from_source(Box::new(err))
    }
}

impl From<std::string::FromUtf8Error> for AppAuthHandlerError {
    fn from(err: std::string::FromUtf8Error) -> AppAuthHandlerError {
        AppAuthHandlerError::from_source(Box::new(err))
    }
}

impl From<events::WebSocketError> for AppAuthHandlerError {
    fn from(err: events::WebSocketError) -> Self {
        AppAuthHandlerError::from_source(Box::new(err))
    }
}

impl From<GetNodeError> for AppAuthHandlerError {
    fn from(err: GetNodeError) -> Self {
        AppAuthHandlerError::from_source(Box::new(err))
    }
}

impl From<ScabbardEventConnectionError> for AppAuthHandlerError {
    fn from(err: ScabbardEventConnectionError) -> Self {
        AppAuthHandlerError::from_source(Box::new(err))
    }
}

impl From<EventIoError> for AppAuthHandlerError {
    fn from(err: EventIoError) -> Self {
        AppAuthHandlerError::from_source(Box::new(err))
    }
}

impl From<SigningError> for AppAuthHandlerError {
    fn from(err: SigningError) -> Self {
        AppAuthHandlerError::from_source(Box::new(err))
    }
}

impl From<ScabbardClientError> for AppAuthHandlerError {
    fn from(err: ScabbardClientError) -> Self {
        AppAuthHandlerError::from_source(Box::new(err))
    }
}

impl From<ContractArchiveError> for AppAuthHandlerError {
    fn from(err: ContractArchiveError) -> Self {
        AppAuthHandlerError::from_source(Box::new(err))
    }
}

impl From<BatchBuildError> for AppAuthHandlerError {
    fn from(err: BatchBuildError) -> Self {
        AppAuthHandlerError::from_source(Box::new(err))
    }
}

impl From<TransactionBuildError> for AppAuthHandlerError {
    fn from(err: TransactionBuildError) -> Self {
        AppAuthHandlerError::from_source(Box::new(err))
    }
}
