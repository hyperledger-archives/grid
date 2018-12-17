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
use mio::{Event, Events, Token};
use mio_extras::channel as mio_channel;

use std::sync::mpsc::TryRecvError;
use std::thread;

use crate::mesh::{
    control::{
        AddError, AddRequest, AddResponse, Control, ControlRequest, RemoveError, RemoveRequest,
        RemoveResponse,
    },
    incoming::Incoming,
    outgoing::Outgoing,
    pool::Pool,
    Envelope,
};
use crate::transport::Connection;

// Maximum number of events to receive and handle per turn of the reactor
const MAX_EVENTS_PER_TURN: usize = 1024;

pub struct Reactor {
    pool: Pool,
    ctrl_rx: mio_channel::Receiver<ControlRequest>,
    ctrl_token: Token,
    incoming_tx: crossbeam_channel::Sender<Envelope>,
    outgoing_capacity: usize,
}

enum Turn {
    Shutdown,
    Continue,
}

impl Reactor {
    fn new(
        ctrl_rx: mio_channel::Receiver<ControlRequest>,
        incoming_tx: crossbeam_channel::Sender<Envelope>,
        outgoing_capacity: usize,
    ) -> Self {
        let mut pool = Pool::new();

        let ctrl_token = pool
            .register_external(&ctrl_rx)
            .expect("Failed to register Control");

        Reactor {
            pool,
            ctrl_rx,
            ctrl_token,
            incoming_tx,
            outgoing_capacity,
        }
    }

    pub(super) fn spawn(incoming_capacity: usize, outgoing_capacity: usize) -> (Control, Incoming) {
        let (ctrl_tx, ctrl_rx) = mio_channel::channel();
        let (incoming_tx, incoming_rx) = crossbeam_channel::bounded(incoming_capacity);

        thread::Builder::new()
            .name(String::from("mesh::Reactor"))
            .spawn(move || {
                let mut reactor = Reactor::new(ctrl_rx, incoming_tx, outgoing_capacity);
                reactor.run();
            })
            .expect("Failed to spawn mesh::Reactor thread");

        (Control::new(ctrl_tx), Incoming::new(incoming_rx))
    }

    fn run(&mut self) {
        let mut events = Events::with_capacity(MAX_EVENTS_PER_TURN);
        loop {
            match self.turn(&mut events) {
                Turn::Shutdown => break,
                Turn::Continue => (),
            }
        }
    }

    fn turn(&mut self, events: &mut Events) -> Turn {
        if let Err(err) = self.pool.poll(events) {
            error!("Error polling: {:?}", err);
            return Turn::Shutdown;
        }

        for event in events.iter() {
            if let Turn::Shutdown = self.handle_event(&event) {
                return Turn::Shutdown;
            }
        }

        Turn::Continue
    }

    fn handle_event(&mut self, event: &Event) -> Turn {
        if event.token() == self.ctrl_token {
            self.handle_control_ready()
        } else {
            self.pool.handle_event(event, &self.incoming_tx);
            Turn::Continue
        }
    }

    fn handle_control_ready(&mut self) -> Turn {
        match self.ctrl_rx.try_recv() {
            Ok(ControlRequest::Add(AddRequest {
                connection,
                response_tx,
                ..
            })) => {
                if let Err(err) = response_tx.send(self.add_connection(connection)) {
                    error!("Failed to send back AddResponse: {:?}", err);
                }
                Turn::Continue
            }
            Ok(ControlRequest::Remove(RemoveRequest {
                id, response_tx, ..
            })) => {
                if let Err(err) = response_tx.send(self.remove_connection(id)) {
                    error!("Failed to send back RemoveResponse: {:?}", err);
                }
                Turn::Continue
            }
            Err(TryRecvError::Empty) => Turn::Continue,
            Err(TryRecvError::Disconnected) => Turn::Shutdown,
        }
    }

    fn add_connection(&mut self, connection: Box<dyn Connection>) -> AddResponse {
        let (tx, rx) = mio_channel::sync_channel(self.outgoing_capacity);

        match self.pool.add(connection, rx) {
            Ok(id) => Ok(Outgoing::new(id, tx)),
            Err(err) => Err(AddError::Io(err)),
        }
    }

    fn remove_connection(&mut self, id: usize) -> RemoveResponse {
        match self.pool.remove(id) {
            Ok(Some(connection)) => Ok(connection),
            Ok(None) => Err(RemoveError::NotFound),
            Err(err) => Err(RemoveError::Io(err)),
        }
    }
}
