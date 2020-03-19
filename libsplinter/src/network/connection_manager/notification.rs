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

use std::sync::mpsc::Receiver;
#[cfg(feature = "connection-manager-notification-iter-try-next")]
use std::sync::mpsc::TryRecvError;

#[cfg(feature = "connection-manager-notification-iter-try-next")]
use super::error::ConnectionManagerError;

/// Messages that will be dispatched to all subscription handlers
#[derive(Debug, PartialEq, Clone)]
pub enum ConnectionManagerNotification {
    Connected {
        endpoint: String,
    },
    InboundConnection {
        endpoint: String,
        connection_id: String,
    },
    Disconnected {
        endpoint: String,
    },
    ReconnectionFailed {
        endpoint: String,
        attempts: u64,
    },
}

/// An iterator over ConnectionManagerNotification values
pub struct NotificationIter {
    pub(super) recv: Receiver<ConnectionManagerNotification>,
}

#[cfg(feature = "connection-manager-notification-iter-try-next")]
impl NotificationIter {
    /// Try to get the next notificaion, if it is available.
    pub fn try_next(
        &self,
    ) -> Result<Option<ConnectionManagerNotification>, ConnectionManagerError> {
        match self.recv.try_recv() {
            Ok(notifications) => Ok(Some(notifications)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => Err(ConnectionManagerError::SendMessageError(
                "The connection manager is no longer running".into(),
            )),
        }
    }
}

impl Iterator for NotificationIter {
    type Item = ConnectionManagerNotification;

    fn next(&mut self) -> Option<Self::Item> {
        match self.recv.recv() {
            Ok(notification) => Some(notification),
            Err(_) => {
                // This is expected if the connection manager shuts down before
                // this end
                None
            }
        }
    }
}
