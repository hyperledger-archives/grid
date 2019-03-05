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

use ::log::{error, log, warn};
use crossbeam_channel::TrySendError;
use mio::{Event, Evented, Events, Poll, PollOpt, Ready, Token};
use mio_extras::channel as mio_channel;

use std::collections::HashMap;
use std::fmt;
use std::io;
use std::mem;
use std::sync::mpsc::TryRecvError;

use crate::mesh::Envelope;
use crate::transport::{Connection, RecvError, SendError};

/// A structure for holding onto many connections and receivers and assigning new connections
/// unique ids
pub struct Pool {
    entries: HashMap<usize, Entry>,
    tokens: HashMap<Token, usize>,
    next_id: usize,
    poll: Poll,
}

impl fmt::Debug for Pool {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut ids = self
            .entries
            .values()
            .map(|entry| entry.id())
            .collect::<Vec<usize>>();
        ids.sort();
        write!(f, "Pool {{ {:?} }}", ids)
    }
}

impl Pool {
    /// Create a new pool, reserving the given ids so that no connection is ever assigned them
    pub fn new() -> Self {
        let poll = Poll::new().expect("Failed to create mio::Poll");

        Pool {
            entries: HashMap::new(),
            tokens: HashMap::new(),
            next_id: 0,
            poll,
        }
    }

    /// Add a new connection to the reactor, returning unique ids for the actual connection and the
    /// outgoing queue
    pub fn add(
        &mut self,
        connection: Box<Connection>,
        outgoing: mio_channel::Receiver<Envelope>,
    ) -> Result<usize, io::Error> {
        let connection_token = self.next_token();
        let outgoing_token = self.next_token();
        let id = self.next_id();

        self.poll.register(
            connection.evented(),
            connection_token,
            Ready::readable(),
            PollOpt::level(),
        )?;

        self.poll.register(
            &outgoing,
            outgoing_token,
            Ready::readable(),
            PollOpt::level(),
        )?;

        self.tokens.insert(connection_token, id);
        self.tokens.insert(outgoing_token, id);
        self.entries.insert(
            id,
            Entry::new(id, connection, connection_token, outgoing, outgoing_token),
        );

        Ok(id)
    }

    /// Remove a connection from the reactor, returning it if it exists
    pub fn remove(&mut self, id: usize) -> Result<Option<Box<Connection>>, io::Error> {
        if let Some(entry) = self.entries.remove(&id) {
            let connection_token = entry.connection_token();
            let outgoing_token = entry.outgoing_token();

            self.tokens.remove(&connection_token);
            self.tokens.remove(&outgoing_token);

            let (connection, outgoing) = entry.into_evented();

            self.poll.deregister(connection.evented())?;
            self.poll.deregister(&outgoing)?;

            Ok(Some(connection))
        } else {
            Ok(None)
        }
    }

    pub fn register_external<E: Evented>(&mut self, evented: &E) -> Result<Token, io::Error> {
        let token = self.next_token();
        self.poll
            .register(evented, token, Ready::readable(), PollOpt::level())?;
        Ok(token)
    }

    /// Poll all connections, outgoings, and externally registered types
    pub fn poll(&self, events: &mut Events) -> Result<usize, io::Error> {
        self.poll.poll(events, None)
    }

    pub fn handle_event(
        &mut self,
        event: &Event,
        incoming_tx: &crossbeam_channel::Sender<Envelope>,
    ) {
        if let Err((id, err)) = self.try_handle_event(event, incoming_tx) {
            warn!(
                "Removing Connection {} due to error handling event: {:?}",
                id, err
            );
            if let Err(err) = self.remove(id) {
                error!("Error removing connection: {:?}", err);
            }
        }
    }

    fn try_handle_event(
        &mut self,
        event: &Event,
        incoming_tx: &crossbeam_channel::Sender<Envelope>,
    ) -> Result<(), (usize, TryEventError)> {
        if let Some(entry) = self.entry_by_token(event.token()) {
            entry
                .try_event(event, incoming_tx)
                .map_err(|err| (entry.id(), err))
        } else {
            Ok(())
        }
    }

    // Lookup an entry by either its connection's token or its outgoing queue's token
    fn entry_by_token(&mut self, token: Token) -> Option<&mut Entry> {
        match self.tokens.get(&token) {
            Some(id) => self.entries.get_mut(id),
            None => None,
        }
    }

    fn next_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn next_token(&mut self) -> Token {
        Token(self.next_id())
    }
}

struct Entry {
    id: usize,
    connection: Box<dyn Connection>,
    connection_token: Token,
    outgoing: mio_channel::Receiver<Envelope>,
    outgoing_token: Token,
    cached: Option<Vec<u8>>,
}

impl fmt::Debug for Entry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Entry {{ id: {:?}, connection: {:?}, outgoing: {:?}, cached: {:?} }}",
            self.id, self.connection_token, self.outgoing_token, self.cached,
        )
    }
}

impl Entry {
    fn new(
        id: usize,
        connection: Box<dyn Connection>,
        connection_token: Token,
        outgoing: mio_channel::Receiver<Envelope>,
        outgoing_token: Token,
    ) -> Self {
        Entry {
            id,
            connection,
            connection_token,
            outgoing,
            outgoing_token,
            cached: None,
        }
    }

    fn id(&self) -> usize {
        self.id
    }

    fn connection_token(&self) -> Token {
        self.connection_token
    }

    fn outgoing_token(&self) -> Token {
        self.outgoing_token
    }

    fn into_evented(self) -> (Box<dyn Connection>, mio_channel::Receiver<Envelope>) {
        (self.connection, self.outgoing)
    }

    fn try_event(
        &self,
        event: &Event,
        incoming_tx: &crossbeam_channel::Sender<Envelope>,
    ) -> Result<(), TryEventError> {
        if self.outgoing_wants_read(event) {
            self.try_read_outgoing()
        } else if self.connection_wants_write(event) {
            self.try_send_connection_from_cached()
        } else if self.connection_wants_read(event) {
            self.try_read_connection(incoming_tx)
        } else {
            Ok(())
        }
    }

    // -- Outgoing --

    fn outgoing_wants_read(&self, event: &Event) -> bool {
        self.outgoing_token == event.token()
            && event.readiness().is_readable()
            && self.cached.is_none()
    }

    fn try_read_outgoing(&mut self) -> Result<(), TryEventError> {
        let envelope = match self.outgoing.try_recv() {
            Ok(envelope) => envelope,
            Err(TryRecvError::Empty) => return Ok(()),
            Err(TryRecvError::Disconnected) => return Err(TryEventError::OutgoingDisconnected),
        };

        self.try_send_connection_or_cache(envelope.take_payload())
    }

    // -- Connection --

    fn connection_wants_write(&self, event: &Event) -> bool {
        self.connection_token == event.token()
            && event.readiness().is_writable()
            && self.cached.is_some()
    }

    fn connection_wants_read(&self, event: &Event) -> bool {
        self.connection_token == event.token() && event.readiness().is_readable()
    }

    fn try_send_connection_from_cached(&mut self) -> Result<(), TryEventError> {
        if let Some(cached) = mem::replace(&mut self.cached, None) {
            self.try_send_connection_or_cache(cached)
        } else {
            Ok(())
        }
    }

    fn try_send_connection_or_cache(&mut self, payload: Vec<u8>) -> Result<(), TryEventError> {
        match self.connection.send(&payload) {
            Ok(()) => Ok(()),
            Err(SendError::WouldBlock) => {
                self.cached = Some(payload);
                Ok(())
            }
            Err(SendError::Disconnected) => Err(TryEventError::ConnectionDisconnected),
            Err(SendError::ProtocolError(err)) => Err(TryEventError::ProtocolError(err)),
            Err(SendError::IoError(err)) => Err(TryEventError::IoError(err)),
        }
    }

    fn try_read_connection(
        &mut self,
        incoming_tx: &crossbeam_channel::Sender<Envelope>,
    ) -> Result<(), TryEventError> {
        if !incoming_tx.is_full() {
            match self.connection.recv() {
                Ok(payload) => match incoming_tx.try_send(Envelope::new(self.id, payload)) {
                    Err(TrySendError::Full(_)) => {
                        warn!("Dropped message due to full incoming queue");
                        Ok(())
                    }
                    Err(TrySendError::Disconnected(_)) => Err(TryEventError::IncomingDisconnected),
                    Ok(()) => Ok(()),
                },
                Err(RecvError::WouldBlock) => Ok(()),
                Err(RecvError::Disconnected) => Err(TryEventError::ConnectionDisconnected),
                Err(RecvError::ProtocolError(err)) => Err(TryEventError::ProtocolError(err)),
                Err(RecvError::IoError(err)) => Err(TryEventError::IoError(err)),
            }
        } else {
            Ok(())
        }
    }
}

#[derive(Debug)]
pub enum TryEventError {
    ConnectionDisconnected,
    IncomingDisconnected,
    OutgoingDisconnected,
    ProtocolError(String),
    IoError(io::Error),
}
