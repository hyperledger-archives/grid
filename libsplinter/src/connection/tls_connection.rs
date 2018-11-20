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
use protobuf::{self, ProtobufError};
use rustls::{Session, TLSError};

use std::io::{ErrorKind, Write};
use std::mem;
use std::net::TcpStream;

use async::NoBlock;
use connection::*;

pub struct TlsConnection<T: Session> {
    socket: TcpStream,
    session: T,
}

impl<T: Session> TlsConnection<T> {
    pub fn new(socket: TcpStream, session: T) -> Self {
        TlsConnection { socket, session }
    }
}

impl<T: Session> Connection for TlsConnection<T> {
    fn handshake(&mut self) -> Result<NoBlock<()>, HandshakeError> {
        if self.session.is_handshaking() {
            match self.session.complete_io(&mut self.socket) {
                Ok(_) => Ok(NoBlock::Ready(())),
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => Ok(NoBlock::WouldBlock),
                Err(err) => Err(HandshakeError::ProtocolError(format!(
                    "Error completing handshake: {:?}",
                    err,
                ))),
            }
        } else {
            Ok(NoBlock::Ready(()))
        }
    }

    fn read(&mut self) -> Result<NoBlock<Message>, ReadError> {
        if self.session.wants_read() {
            if let Err(err) = self.session.read_tls(&mut self.socket) {
                if err.kind() == ErrorKind::WouldBlock {
                    return Ok(NoBlock::WouldBlock);
                } else {
                    return Err(ReadError::IoError(err));
                }
            };

            self.session.process_new_packets()?;

            Ok(NoBlock::Ready(read_msg(&mut self.session)?))
        } else {
            Ok(NoBlock::WouldBlock)
        }
    }

    fn write(&mut self, msg: &Message) -> Result<NoBlock<()>, WriteError> {
        if let Err(err) = self.session.write_tls(&mut self.socket) {
            if err.kind() == ErrorKind::WouldBlock {
                return Ok(NoBlock::WouldBlock);
            } else {
                return Err(WriteError::IoError(err));
            }
        };

        let packed = pack_msg(msg)?;

        match self.session.write(&packed) {
            Ok(n) => {
                debug!("Wrote {} bytes", n);
                Ok(NoBlock::Ready(()))
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => Ok(NoBlock::WouldBlock),
            Err(err) => Err(WriteError::from(err)),
        }
    }
}

fn read_msg(session: &mut Session) -> Result<Message, ReadError> {
    let mut msg_len_buff = vec![0; mem::size_of::<u32>()];
    session.read_exact(&mut msg_len_buff)?;
    let msg_size = msg_len_buff.as_slice().read_u32::<BigEndian>()? as usize;

    // Read Message
    let mut msg_buff = vec![0; msg_size];
    session.read_exact(&mut msg_buff)?;

    Ok(protobuf::parse_from_bytes::<Message>(&msg_buff)?)
}

impl From<TLSError> for ReadError {
    fn from(tls_error: TLSError) -> Self {
        ReadError::ProtocolError(format!("TLS protocol error: {:?}", tls_error))
    }
}

impl From<ProtobufError> for ReadError {
    fn from(pb_error: ProtobufError) -> Self {
        ReadError::ParseError(format!("Protobuf parse error: {:?}", pb_error))
    }
}

fn pack_msg(msg: &Message) -> Result<Vec<u8>, WriteError> {
    let raw_msg = protobuf::Message::write_to_bytes(msg)?;
    let mut buff = Vec::new();

    buff.write_u32::<BigEndian>(raw_msg.len() as u32)?;
    buff.write(&raw_msg)?;

    Ok(buff)
}

impl From<TLSError> for WriteError {
    fn from(tls_error: TLSError) -> Self {
        WriteError::ProtocolError(format!("TLS protocol error: {:?}", tls_error))
    }
}

impl From<ProtobufError> for WriteError {
    fn from(pb_error: ProtobufError) -> Self {
        WriteError::ParseError(format!("Protobuf pack error: {:?}", pb_error))
    }
}
