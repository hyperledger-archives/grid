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

use std::{error, fmt};

use crate::events::ws;

#[derive(Debug)]
pub enum EventError {
    WebSocketError(ws::Error),
    ShutdownHandleError(String),
}

impl From<ws::Error> for EventError {
    fn from(err: ws::Error) -> Self {
        EventError::WebSocketError(err)
    }
}

impl error::Error for EventError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            EventError::WebSocketError(err) => Some(err),
            EventError::ShutdownHandleError(_) => None,
        }
    }
}

impl fmt::Display for EventError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EventError::WebSocketError(err) => write!(f, "Websocket Error: {}", err),
            EventError::ShutdownHandleError(s) => write!(f, "Shutdown Handle Error: {}", s),
        }
    }
}
