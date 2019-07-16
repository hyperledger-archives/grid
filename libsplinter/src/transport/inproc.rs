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

use mio::{Evented, Registration, SetReadiness};
use mio::{Poll, PollOpt, Ready, Token};

use std::collections::HashMap;
use std::io::{self, ErrorKind};
use std::sync::{
    mpsc::{channel, Receiver, Sender},
    Arc, Mutex,
};

use crate::transport::{
    AcceptError, ConnectError, Connection, DisconnectError, ListenError, Listener, RecvError,
    SendError, Transport,
};

type Incoming = Arc<Mutex<HashMap<String, Sender<Pair<Vec<u8>>>>>>;

const PROTOCOL_PREFIX: &str = "inproc://";

#[derive(Clone, Default)]
pub struct InprocTransport {
    incoming: Incoming,
}

impl Transport for InprocTransport {
    fn accepts(&self, address: &str) -> bool {
        address.starts_with(PROTOCOL_PREFIX) || !address.contains("://")
    }

    fn connect(&mut self, endpoint: &str) -> Result<Box<dyn Connection>, ConnectError> {
        if !self.accepts(endpoint) {
            return Err(ConnectError::ProtocolError(format!(
                "Invalid protocol \"{}\"",
                endpoint
            )));
        }
        let address = if endpoint.starts_with(PROTOCOL_PREFIX) {
            &endpoint[PROTOCOL_PREFIX.len()..]
        } else {
            endpoint
        };

        match self.incoming.lock().unwrap().get(address) {
            Some(sender) => {
                let (p0, p1) = Pair::new();
                sender.send(p0).unwrap();
                Ok(Box::new(InprocConnection::new(address.into(), p1)))
            }
            None => Err(ConnectError::IoError(io::Error::new(
                ErrorKind::ConnectionRefused,
                "No Listener",
            ))),
        }
    }

    fn listen(&mut self, bind: &str) -> Result<Box<dyn Listener>, ListenError> {
        if !self.accepts(bind) {
            return Err(ListenError::ProtocolError(format!(
                "Invalid protocol \"{}\"",
                bind
            )));
        }
        let address = if bind.starts_with(PROTOCOL_PREFIX) {
            &bind[PROTOCOL_PREFIX.len()..]
        } else {
            bind
        };

        let (tx, rx) = channel();
        self.incoming.lock().unwrap().insert(address.into(), tx);
        Ok(Box::new(InprocListener::new(address.into(), rx)))
    }
}

pub struct InprocListener {
    endpoint: String,
    rx: Receiver<Pair<Vec<u8>>>,
}

impl InprocListener {
    fn new(endpoint: String, rx: Receiver<Pair<Vec<u8>>>) -> Self {
        InprocListener { endpoint, rx }
    }
}

impl Listener for InprocListener {
    fn accept(&mut self) -> Result<Box<dyn Connection>, AcceptError> {
        Ok(Box::new(InprocConnection::new(
            self.endpoint.clone(),
            self.rx.recv().unwrap(),
        )))
    }

    fn endpoint(&self) -> String {
        let mut buf = String::from(PROTOCOL_PREFIX);
        buf.push_str(&self.endpoint);
        buf
    }
}

pub struct InprocConnection {
    endpoint: String,
    pair: Pair<Vec<u8>>,
}

impl InprocConnection {
    fn new(endpoint: String, pair: Pair<Vec<u8>>) -> Self {
        InprocConnection { endpoint, pair }
    }
}

impl Connection for InprocConnection {
    fn send(&mut self, message: &[u8]) -> Result<(), SendError> {
        self.pair.send(message.to_vec());
        Ok(())
    }

    fn recv(&mut self) -> Result<Vec<u8>, RecvError> {
        match self.pair.recv() {
            Some(message) => Ok(message),
            None => Err(RecvError::WouldBlock),
        }
    }

    fn remote_endpoint(&self) -> String {
        let mut buf = String::from(PROTOCOL_PREFIX);
        buf.push_str(&self.endpoint);
        buf
    }

    fn local_endpoint(&self) -> String {
        let mut buf = String::from(PROTOCOL_PREFIX);
        buf.push_str(&self.endpoint);
        buf
    }

    fn disconnect(&mut self) -> Result<(), DisconnectError> {
        Ok(())
    }

    fn evented(&self) -> &dyn Evented {
        &self.pair
    }
}

struct Pair<T> {
    outgoing: Arc<Mutex<Vec<T>>>,
    incoming: Arc<Mutex<Vec<T>>>,
    set: Arc<Mutex<SetReadiness>>,
    other_set: Arc<Mutex<SetReadiness>>,
    registration: Registration,
}

impl<T> Pair<T> {
    fn new() -> (Self, Self) {
        let queue1 = Arc::new(Mutex::new(Vec::new()));
        let queue2 = Arc::new(Mutex::new(Vec::new()));

        let (registration1, set_readiness1) = Registration::new2();
        let (registration2, set_readiness2) = Registration::new2();

        let set_readiness1 = Arc::new(Mutex::new(set_readiness1));
        let set_readiness2 = Arc::new(Mutex::new(set_readiness2));

        (
            Pair {
                outgoing: Arc::clone(&queue1),
                incoming: Arc::clone(&queue2),
                set: Arc::clone(&set_readiness1),
                other_set: Arc::clone(&set_readiness2),
                registration: registration1,
            },
            Pair {
                outgoing: queue2,
                incoming: queue1,
                set: set_readiness2,
                other_set: set_readiness1,
                registration: registration2,
            },
        )
    }

    fn send(&self, t: T) {
        let mut outgoing = self.outgoing.lock().unwrap();
        let set = self.set.lock().unwrap();
        let other_set = self.other_set.lock().unwrap();
        outgoing.insert(0, t);
        other_set
            .set_readiness(other_set.readiness() | Ready::readable())
            .unwrap();
        set.set_readiness(set.readiness() | Ready::writable())
            .unwrap();
    }

    fn recv(&self) -> Option<T> {
        let mut incoming = self.incoming.lock().unwrap();
        let set = self.set.lock().unwrap();
        if incoming.len() < 1 {
            set.set_readiness(set.readiness() - Ready::readable())
                .unwrap();
        } else {
            set.set_readiness(set.readiness() | Ready::readable())
                .unwrap();
        }
        incoming.pop()
    }
}

impl<T> Evented for Pair<T> {
    fn register(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        match self.registration.register(poll, token, interest, opts) {
            Ok(()) => {
                let set = self.set.lock().unwrap();
                set.set_readiness(set.readiness() | Ready::writable())
            }
            Err(err) => Err(err),
        }
    }

    fn reregister(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        match self.registration.reregister(poll, token, interest, opts) {
            Ok(()) => {
                let set = self.set.lock().unwrap();
                set.set_readiness(set.readiness() | Ready::writable())
            }
            Err(err) => Err(err),
        }
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        poll.deregister(&self.registration)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::transport::tests;

    #[test]
    fn test_transport() {
        let transport = InprocTransport::default();
        tests::test_transport(transport, "test");
    }

    #[cfg(not(unix))]
    #[test]
    fn test_poll() {
        let transport = InprocTransport::default();
        tests::test_poll(transport, "test", Ready::readable() | Ready::writable());
    }
}
