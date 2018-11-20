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

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::time::Duration;

use transport::{
    AcceptError, ConnectError, Connection, DisconnectError, ListenError, Listener, RecvError,
    SendError, Transport,
};

#[derive(Default)]
pub struct RawTransport {}

impl Transport for RawTransport {
    fn connect(&mut self, endpoint: &str) -> Result<Box<dyn Connection>, ConnectError> {
        Ok(Box::new(RawConnection {
            stream: TcpStream::connect(endpoint)?,
        }))
    }

    fn listen(&mut self, bind: &str) -> Result<Box<dyn Listener>, ListenError> {
        Ok(Box::new(RawListener {
            listener: TcpListener::bind(bind)?,
        }))
    }
}

pub struct RawListener {
    listener: TcpListener,
}

impl Listener for RawListener {
    fn accept(&mut self) -> Result<Box<dyn Connection>, AcceptError> {
        let (stream, _) = self.listener.accept()?;
        let connection = RawConnection { stream };
        Ok(Box::new(connection))
    }

    fn endpoint(&self) -> String {
        self.listener.local_addr().unwrap().to_string()
    }
}

pub struct RawConnection {
    stream: TcpStream,
}

impl Connection for RawConnection {
    fn send(&mut self, message: &[u8]) -> Result<(), SendError> {
        write(&mut self.stream, message)
    }

    fn recv(&mut self, timeout: Option<Duration>) -> Result<Vec<u8>, RecvError> {
        self.stream.set_read_timeout(timeout)?;
        read(&mut self.stream)
    }

    fn remote_endpoint(&self) -> String {
        self.stream.peer_addr().unwrap().to_string()
    }

    fn local_endpoint(&self) -> String {
        self.stream.local_addr().unwrap().to_string()
    }

    fn disconnect(&mut self) -> Result<(), DisconnectError> {
        Ok(self.stream.shutdown(Shutdown::Both)?)
    }
}

fn read<T: Read>(reader: &mut T) -> Result<Vec<u8>, RecvError> {
    let len = reader.read_u32::<BigEndian>()?;
    let mut buffer = vec![0; len as usize];
    reader.read_exact(&mut buffer[..])?;
    Ok(buffer)
}

fn write<T: Write>(writer: &mut T, buffer: &[u8]) -> Result<(), SendError> {
    writer.write_u32::<BigEndian>(buffer.len() as u32)?;
    writer.write(&buffer)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use transport::tests;

    #[test]
    fn test_transport() {
        let transport = RawTransport::default();

        tests::test_transport(transport, "127.0.0.1:0");
    }
}
