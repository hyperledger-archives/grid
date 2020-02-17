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

use mio::{net::TcpStream as MioTcpStream, Evented};

use std::net::{Shutdown, TcpListener, TcpStream};

use crate::transport::rw::{read, write};
use crate::transport::{
    AcceptError, ConnectError, Connection, DisconnectError, ListenError, Listener, RecvError,
    SendError, Transport,
};

const PROTOCOL_PREFIX: &str = "tcp://";

#[derive(Default)]
pub struct RawTransport {}

impl Transport for RawTransport {
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
        // Connect a std::net::TcpStream to make sure connect() block
        let stream = TcpStream::connect(address)?;
        let mio_stream = MioTcpStream::from_stream(stream)?;
        Ok(Box::new(RawConnection { stream: mio_stream }))
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

        Ok(Box::new(RawListener {
            listener: TcpListener::bind(address)?,
        }))
    }
}

pub struct RawListener {
    listener: TcpListener,
}

impl Listener for RawListener {
    fn accept(&mut self) -> Result<Box<dyn Connection>, AcceptError> {
        let (stream, _) = self.listener.accept()?;
        let connection = RawConnection {
            stream: MioTcpStream::from_stream(stream)?,
        };
        Ok(Box::new(connection))
    }

    fn endpoint(&self) -> String {
        format!("tcp://{}", self.listener.local_addr().unwrap())
    }
}

pub struct RawConnection {
    stream: MioTcpStream,
}

impl Connection for RawConnection {
    fn send(&mut self, message: &[u8]) -> Result<(), SendError> {
        write(&mut self.stream, message)
    }

    fn recv(&mut self) -> Result<Vec<u8>, RecvError> {
        read(&mut self.stream)
    }

    fn remote_endpoint(&self) -> String {
        format!("tcp://{}", self.stream.peer_addr().unwrap())
    }

    fn local_endpoint(&self) -> String {
        format!("tcp://{}", self.stream.local_addr().unwrap())
    }

    fn disconnect(&mut self) -> Result<(), DisconnectError> {
        self.stream
            .shutdown(Shutdown::Both)
            .map_err(DisconnectError::from)
    }

    fn evented(&self) -> &dyn Evented {
        &self.stream
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::tests;
    use mio::Ready;

    #[test]
    fn test_accepts() {
        let transport = RawTransport::default();
        assert!(transport.accepts("127.0.0.1:0"));
        assert!(transport.accepts("tcp://127.0.0.1:0"));
        assert!(transport.accepts("tcp://somewhere.example.com:4000"));

        assert!(!transport.accepts("tls://somewhere.example.com:4000"));
    }

    #[test]
    fn test_transport() {
        let transport = RawTransport::default();

        tests::test_transport(transport, "127.0.0.1:0");
    }

    #[test]
    fn test_transport_explicit_protocol() {
        let transport = RawTransport::default();

        tests::test_transport(transport, "tcp://127.0.0.1:0");
    }

    #[test]
    fn test_poll() {
        let transport = RawTransport::default();
        tests::test_poll(
            transport,
            "127.0.0.1:0",
            Ready::readable() | Ready::writable(),
        );
    }
}
