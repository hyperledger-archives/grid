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

use std::sync::mpsc::SyncSender;

use crate::network::connection_manager::error::ConnectionManagerError;

pub enum CmMessage {
    Shutdown,
    Subscribe(String, SyncSender<Vec<CmNotification>>),
    UnSubscribe(String),
    Request(CmRequest),
    SendHeartbeats,
}

pub struct CmRequest {
    pub sender: SyncSender<CmResponse>,
    pub payload: CmPayload,
}

#[derive(Debug, PartialEq)]
pub enum CmPayload {
    AddConnection { endpoint: String },
    RemoveConnection { endpoint: String },
}

#[derive(Debug, PartialEq)]
pub enum CmResponse {
    AddConnection {
        status: CmResponseStatus,
        error_message: Option<String>,
    },
    RemoveConnection {
        status: CmResponseStatus,
        error_message: Option<String>,
    },
}

#[derive(Debug, PartialEq)]
pub enum CmResponseStatus {
    OK,
    Error,
    ConnectionNotFound,
}

/// Messages that will be dispatched to all
/// subscription handlers
#[derive(Debug, PartialEq, Clone)]
pub enum CmNotification {
    FatalError {
        error: ConnectionManagerError,
        message: String,
    },
    HeartbeatSent {
        endpoint: String,
    },
    HeartbeatSendFail {
        endpoint: String,
        message: String,
    },
    ReconnectAttemptSuccess {
        endpoint: String,
    },
    ReconnectAttemptFailed {
        endpoint: String,
        message: String,
    },
}
