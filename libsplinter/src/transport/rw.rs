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

// Portions of read() and write() in this module are derived from the Rust
// std::io implementation which is licensed under the Apache 2.0 license. For
// specific copyright information on those functions, see additionally the
// following: https://github.com/rust-lang/rust/blob/master/COPYRIGHT

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Read, Write};
use std::mem;
use std::thread;
use std::time::Duration;

use crate::transport::{RecvError, SendError};

pub fn read<T: Read>(reader: &mut T) -> Result<Vec<u8>, RecvError> {
    let len = loop {
        match reader.read_u32::<BigEndian>() {
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(100));
                continue;
            }
            Err(e) => return Err(RecvError::IoError(e)),
            Ok(n) => break n,
        };
    };

    let mut buffer = vec![0; len as usize];
    let mut remaining = &mut buffer[..];

    while !remaining.is_empty() {
        match reader.read(remaining) {
            Ok(0) => break,
            Ok(n) => {
                let tmp = remaining;
                remaining = &mut tmp[n..];
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => return Err(RecvError::IoError(e)),
        }
    }
    if !remaining.is_empty() {
        Err(RecvError::Disconnected)
    } else {
        Ok(buffer)
    }
}

pub fn write<T: Write>(writer: &mut T, buffer: &[u8]) -> Result<(), SendError> {
    let mut packed = &pack(buffer)?[..];
    while !packed.is_empty() {
        match writer.write(packed) {
            Ok(0) => {
                return Err(SendError::IoError(std::io::Error::new(
                    std::io::ErrorKind::WriteZero,
                    "failed to write whole buffer",
                )))
            }
            Ok(n) => packed = &packed[n..],
            Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => return Err(SendError::IoError(e)),
        }
    }
    writer.flush()?;
    Ok(())
}

fn pack(buffer: &[u8]) -> Result<Vec<u8>, io::Error> {
    let capacity: usize = buffer.len() + mem::size_of::<u32>();
    let mut packed = Vec::with_capacity(capacity);

    packed.write_u32::<BigEndian>(buffer.len() as u32)?;
    packed.write_all(&buffer)?;

    Ok(packed)
}
