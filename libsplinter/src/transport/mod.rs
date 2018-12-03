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

pub mod raw;
pub mod tls;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use mio::Evented;

use std::io::{self, Read, Write};

pub enum Status {
    Connected,
    Disconnected,
}

/// A single, bi-directional connection between two nodes
pub trait Connection {
    fn send(&mut self, message: &[u8]) -> Result<(), SendError>;
    fn recv(&mut self) -> Result<Vec<u8>, RecvError>;

    fn remote_endpoint(&self) -> String;
    fn local_endpoint(&self) -> String;

    fn disconnect(&mut self) -> Result<(), DisconnectError>;

    fn evented(&self) -> &dyn Evented;
}

pub trait Listener: Send {
    fn accept(&mut self) -> Result<Box<dyn Connection>, AcceptError>;
    fn endpoint(&self) -> String;
}

pub trait Incoming {
    fn incoming<'a>(
        &'a mut self,
    ) -> Box<Iterator<Item = Result<Box<dyn Connection>, AcceptError>> + 'a>;
}

impl Incoming for Box<dyn Listener> {
    fn incoming<'a>(
        &'a mut self,
    ) -> Box<Iterator<Item = Result<Box<dyn Connection>, AcceptError>> + 'a> {
        Box::new(IncomingIter::new(self))
    }
}

/// Factory-pattern based type for creating connections
pub trait Transport {
    fn connect(&mut self, endpoint: &str) -> Result<Box<dyn Connection>, ConnectError>;
    fn listen(&mut self, bind: &str) -> Result<Box<dyn Listener>, ListenError>;
}

// Helper struct for extending Listener to Incoming

struct IncomingIter<'a> {
    listener: &'a mut Box<dyn Listener>,
}

impl<'a> IncomingIter<'a> {
    pub fn new(listener: &'a mut Box<dyn Listener>) -> Self {
        IncomingIter { listener }
    }
}

impl<'a> Iterator for IncomingIter<'a> {
    type Item = Result<Box<dyn Connection>, AcceptError>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.listener.accept())
    }
}

// -- Errors --

macro_rules! impl_from_io_error {
    ($err:ident) => {
        impl From<io::Error> for $err {
            fn from(io_error: io::Error) -> Self {
                $err::IoError(io_error)
            }
        }
    };
}

#[derive(Debug)]
pub enum SendError {
    IoError(io::Error),
    ProtocolError(String),
}

impl_from_io_error!(SendError);

#[derive(Debug)]
pub enum RecvError {
    IoError(io::Error),
    ProtocolError(String),
}

impl_from_io_error!(RecvError);

#[derive(Debug)]
pub enum StatusError {}

#[derive(Debug)]
pub enum DisconnectError {
    IoError(io::Error),
    ProtocolError(String),
}

impl_from_io_error!(DisconnectError);

#[derive(Debug)]
pub enum AcceptError {
    IoError(io::Error),
    ProtocolError(String),
}

impl_from_io_error!(AcceptError);

#[derive(Debug)]
pub enum ConnectError {
    IoError(io::Error),
    ParseError(String),
    ProtocolError(String),
}

impl_from_io_error!(ConnectError);

#[derive(Debug)]
pub enum ListenError {
    IoError(io::Error),
}

impl_from_io_error!(ListenError);

#[derive(Debug)]
pub enum PollError {}

pub fn read<T: Read>(reader: &mut T) -> Result<Vec<u8>, RecvError> {
    let len = reader.read_u32::<BigEndian>()?;
    let mut buffer = vec![0; len as usize];
    reader.read_exact(&mut buffer[..])?;
    Ok(buffer)
}

pub fn write<T: Write>(writer: &mut T, buffer: &[u8]) -> Result<(), SendError> {
    writer.write_u32::<BigEndian>(buffer.len() as u32)?;
    writer.write(&buffer)?;
    writer.flush()?;
    Ok(())
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::fmt::Debug;

    use std::io::ErrorKind;
    use std::sync::mpsc::channel;
    use std::thread;
    use std::time::Duration;

    use mio::{Events, Poll, PollOpt, Ready, Token};

    fn assert_ok<T, E: Debug>(result: Result<T, E>) -> T {
        match result {
            Ok(ok) => ok,
            Err(err) => panic!("Expected Ok(...), got Err({:?})", err),
        }
    }

    macro_rules! block {
        ($op:expr, $err:ident) => {
            loop {
                match $op {
                    Err($err::IoError(err)) => {
                        if err.kind() == ErrorKind::WouldBlock {
                            thread::sleep(Duration::from_millis(100));
                            continue;
                        }
                    }
                    Err(err) => break Err(err),
                    Ok(ok) => break Ok(ok),
                }
            }
        };
    }

    pub fn test_transport<T: Transport + Send + 'static>(mut transport: T, bind: &str) {
        let mut listener = assert_ok(transport.listen(bind));
        let endpoint = listener.endpoint();

        let handle = thread::spawn(move || {
            let mut client = assert_ok(transport.connect(&endpoint));
            assert_eq!(client.remote_endpoint(), endpoint);

            assert_ok(block!(client.send(&[0, 1, 2]), SendError));
            assert_eq!(vec![3, 4, 5], assert_ok(block!(client.recv(), RecvError)));
        });

        let mut server = assert_ok(listener.incoming().next().unwrap());

        assert_eq!(vec![0, 1, 2], assert_ok(block!(server.recv(), RecvError)));

        assert_ok(block!(server.send(&[3, 4, 5]), SendError));

        handle.join().unwrap();
    }

    fn assert_ready(events: &Events, token: Token, readiness: Ready) {
        assert_eq!(
            Some(readiness),
            events
                .iter()
                .filter(|event| event.token() == token)
                .map(|event| event.readiness())
                .next(),
        );
    }

    pub fn test_poll<T: Transport + Send + 'static>(mut transport: T, bind: &str) {
        // Create aconnections and register them with the poller
        const CONNECTIONS: usize = 16;

        let mut listener = assert_ok(transport.listen(bind));
        let endpoint = listener.endpoint();

        let (ready_tx, ready_rx) = channel();

        let handle = thread::spawn(move || {
            let mut connections = Vec::with_capacity(CONNECTIONS);
            for i in 0..CONNECTIONS {
                connections.push((assert_ok(transport.connect(&endpoint)), Token(i)));
            }

            // Register all connections with Poller
            let poll = Poll::new().unwrap();
            for (conn, token) in &connections {
                assert_ok(poll.register(
                    conn.evented(),
                    *token,
                    Ready::readable() | Ready::writable(),
                    PollOpt::level(),
                ));
            }

            // Block waiting for other thread to send everything
            ready_rx.recv().unwrap();

            let mut events = Events::with_capacity(CONNECTIONS * 2);
            assert_ok(poll.poll(&mut events, None));
            for (mut conn, token) in connections {
                assert_ready(&events, token, Ready::readable() | Ready::writable());
                assert_eq!(b"hello".to_vec(), assert_ok(conn.recv()));
                assert_ok(conn.send(b"world"));
            }
        });

        let mut connections = Vec::with_capacity(CONNECTIONS);
        for _ in 0..CONNECTIONS {
            let mut conn = assert_ok(listener.accept());
            assert_ok(block!(conn.send(b"hello"), SendError));
            connections.push(conn);
        }

        // Signal done sending to background thread
        ready_tx.send(()).unwrap();

        for mut conn in connections {
            assert_eq!(b"world".to_vec(), assert_ok(block!(conn.recv(), RecvError)));
        }

        handle.join().unwrap();
    }
}
