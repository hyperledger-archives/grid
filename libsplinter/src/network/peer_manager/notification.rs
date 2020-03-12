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

use super::error::PeerManagerError;
use std::sync::mpsc::{Receiver, TryRecvError};

/// Messages that will be dispatched to all
/// subscription handlers
#[derive(Debug, PartialEq, Clone)]
pub enum PeerManagerNotification {
    Connected { peer: String },
    Disconnected { peer: String },
}

/// PeerNotificationIter is used to receive notfications from the PeerManager. The notifications
/// include:
/// - PeerManagerNotification::Disconnected: peer disconnected and reconnection
///     is being attempted.
/// - PeerManagerNotification::Connected: reconnection to peer was successful
pub struct PeerNotificationIter {
    pub(super) recv: Receiver<PeerManagerNotification>,
}

impl PeerNotificationIter {
    pub fn try_next(&self) -> Result<Option<PeerManagerNotification>, PeerManagerError> {
        match self.recv.try_recv() {
            Ok(notifications) => Ok(Some(notifications)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => Err(PeerManagerError::SendMessageError(
                "The peer manager is no longer running".into(),
            )),
        }
    }
}

impl Iterator for PeerNotificationIter {
    type Item = PeerManagerNotification;

    fn next(&mut self) -> Option<Self::Item> {
        match self.recv.recv() {
            Ok(notification) => Some(notification),
            Err(_) => {
                // This is expected if the peer manager shuts down before
                // this end
                None
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::sync::mpsc::channel;
    use std::thread;

    /// Tests that notifier iterator correctly exists when sender
    /// is dropped.
    ///
    /// Procedure:
    ///
    /// The test creates a channel and a notifier, then it
    /// creates a thread that sends Connected notifications to
    /// the notifier.
    ///
    /// Asserts:
    ///
    /// The notifications sent are received by the NotificationIter
    /// correctly
    ///
    /// That the total number of notifications sent equals 5
    #[test]
    fn test_peer_manager_notifications() {
        let (send, recv) = channel();

        let notifcation_iter = PeerNotificationIter { recv };

        let join_handle = thread::spawn(move || {
            for i in 0..5 {
                send.send(PeerManagerNotification::Connected {
                    peer: format!("test_peer{}", i),
                })
                .unwrap();
            }
        });

        let mut notifications_sent = 0;
        for notifcation in notifcation_iter {
            assert_eq!(
                notifcation,
                PeerManagerNotification::Connected {
                    peer: format!("test_peer{}", notifications_sent),
                }
            );
            notifications_sent += 1;
        }

        assert_eq!(notifications_sent, 5);

        join_handle.join().unwrap();
    }
}
