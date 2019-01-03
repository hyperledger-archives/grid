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

use mio_extras::channel::{SyncSender, TrySendError};

use std::io;

use crate::mesh::Envelope;

/// Handle for sending to a specific connection in the mesh
#[derive(Clone)]
pub struct Outgoing {
    id: usize,
    tx: SyncSender<Envelope>,
}

impl Outgoing {
    pub(super) fn new(id: usize, tx: SyncSender<Envelope>) -> Self {
        Outgoing { id, tx }
    }

    pub fn send(&self, payload: Vec<u8>) -> Result<(), SendError> {
        Ok(self.tx.try_send(Envelope::new(self.id, payload))?)
    }

    pub fn id(&self) -> usize {
        self.id
    }
}

#[derive(Debug)]
pub enum SendError {
    IoError(io::Error),
    Full(Vec<u8>),
    Disconnected(Vec<u8>),
}

impl From<TrySendError<Envelope>> for SendError {
    fn from(err: TrySendError<Envelope>) -> Self {
        match err {
            TrySendError::Full(envelope) => SendError::Full(envelope.payload),
            TrySendError::Disconnected(envelope) => SendError::Disconnected(envelope.payload),
            TrySendError::Io(err) => SendError::IoError(err),
        }
    }
}
