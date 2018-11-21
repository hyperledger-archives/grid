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

use mio::Evented;

use std::io::Error as IoError;

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

pub trait Listener {
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
        impl From<IoError> for $err {
            fn from(io_error: IoError) -> Self {
                $err::IoError(io_error)
            }
        }
    };
}

#[derive(Debug)]
pub enum SendError {
    IoError(IoError),
    ProtocolError(String),
}

impl_from_io_error!(SendError);

#[derive(Debug)]
pub enum RecvError {
    IoError(IoError),
    ProtocolError(String),
}

impl_from_io_error!(RecvError);

#[derive(Debug)]
pub enum StatusError {}

#[derive(Debug)]
pub enum DisconnectError {
    IoError(IoError),
    ProtocolError(String),
}

impl_from_io_error!(DisconnectError);

#[derive(Debug)]
pub enum AcceptError {
    IoError(IoError),
    ProtocolError(String),
}

impl_from_io_error!(AcceptError);

#[derive(Debug)]
pub enum ConnectError {
    IoError(IoError),
    ParseError(String),
    ProtocolError(String),
}

impl_from_io_error!(ConnectError);

#[derive(Debug)]
pub enum ListenError {
    IoError(IoError),
}

impl_from_io_error!(ListenError);

#[derive(Debug)]
pub enum PollError {}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::fmt::Debug;

    use std::io::ErrorKind;
    use std::time::Duration;
    use std::thread;

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
                    },
                    Err(err) => break Err(err),
                    Ok(ok) => break Ok(ok),
                }
            }
        }
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
}
