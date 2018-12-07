// Copyright 2018 Cargill Incorporated
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

use crossbeam_channel;

use std::time::Duration;

use mesh::Envelope;

/// Handle for receiving envelopes from the mesh
#[derive(Clone)]
pub struct Incoming {
    rx: crossbeam_channel::Receiver<Envelope>,
}

impl Incoming {
    pub(super) fn new(rx: crossbeam_channel::Receiver<Envelope>) -> Self {
        Incoming { rx }
    }

    pub fn recv(&self) -> Result<Envelope, RecvError> {
        self.rx.recv().map_err(|_| RecvError {})
    }

    pub fn recv_timeout(&self, timeout: Duration) -> Result<Envelope, RecvTimeoutError> {
        Ok(self.rx.recv_timeout(timeout)?)
    }
}

/// The background sender disconnected and the queue is empty
#[derive(Debug)]
pub struct RecvError;

#[derive(Debug)]
pub enum RecvTimeoutError {
    Disconnected,
    Timeout,
}

impl From<crossbeam_channel::RecvTimeoutError> for RecvTimeoutError {
    fn from(err: crossbeam_channel::RecvTimeoutError) -> Self {
        match err {
            crossbeam_channel::RecvTimeoutError::Disconnected => RecvTimeoutError::Disconnected,
            crossbeam_channel::RecvTimeoutError::Timeout => RecvTimeoutError::Timeout,
        }
    }
}
