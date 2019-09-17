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
use std::io::{self, Read, Write};
use std::mem;

use crate::transport::{RecvError, SendError};

pub fn read<T: Read>(reader: &mut T) -> Result<Vec<u8>, RecvError> {
    let len = reader.read_u32::<BigEndian>()?;
    let mut buffer = vec![0; len as usize];
    reader.read_exact(&mut buffer[..])?;
    Ok(buffer)
}

pub fn write<T: Write>(writer: &mut T, buffer: &[u8]) -> Result<(), SendError> {
    let packed = pack(buffer)?;
    writer.write_all(&packed)?;
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
