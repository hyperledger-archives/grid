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

use crate::splinter::app_auth_handler::node::GetNodeError;

#[derive(Debug)]
pub enum AppAuthHandlerError {
    WebSocketError(events::WebSocketError),
    GetNodeError(GetNodeError),
}

impl Error for AppAuthHandlerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            AppAuthHandlerError::WebSocketError(err) => Some(err),
            AppAuthHandlerError::GetNodeError(err) => Some(err),
        }
    }
}

impl fmt::Display for AppAuthHandlerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppAuthHandlerError::WebSocketError(msg) => write!(f, "WebsocketError {}", msg),
            AppAuthHandlerError::GetNodeError(msg) => write!(f, "GetNodeError {}", msg),
        }
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
