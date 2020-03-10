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

use std::io::{self, Cursor, Read, Write};
use std::thread;
use std::time::Duration;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

const HEADER_LENGTH: usize = 6;

/// An error that may be returned during frame-related operations
#[derive(Debug)]
pub enum FrameError {
    IoError(io::Error),
    InvalidChecksum,
    InvalidHeaderLength(usize),
    UnsupportedVersion,
}

impl std::fmt::Display for FrameError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FrameError::IoError(err) => f.write_str(&err.to_string()),
            FrameError::InvalidChecksum => f.write_str("Invalid checksum in frame header"),
            FrameError::InvalidHeaderLength(n) => write!(
                f,
                "Invalid header length expected {} but was {}",
                HEADER_LENGTH, n
            ),
            FrameError::UnsupportedVersion => f.write_str("Unsupported frame version"),
        }
    }
}

impl std::error::Error for FrameError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FrameError::IoError(err) => Some(&*err),
            FrameError::InvalidChecksum => None,
            FrameError::InvalidHeaderLength(_) => None,
            FrameError::UnsupportedVersion => None,
        }
    }
}

impl From<io::Error> for FrameError {
    fn from(err: io::Error) -> Self {
        FrameError::IoError(err)
    }
}

/// The Frame version
///
/// This specifies the version of the frame, based on what value is sent during frame transmission.
/// It indicates header style and data requirements.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum FrameVersion {
    V1 = 1,
}

/// A complete Frame of transmitted data.
///
/// This struct owns the data that has been transmitted.  It is essentially a receiving frame.
pub struct Frame {
    data: Vec<u8>,
}

impl Frame {
    /// Convert this frame into its inner byte vector.
    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    /// Read a frame from the given reader.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    ///
    /// - the header is malformed
    /// - the data length doesn't match the header length
    /// - an IO error occurs
    pub fn read<R: Read>(reader: &mut R) -> Result<Self, FrameError> {
        let frame_header = loop {
            match FrameHeader::read(reader) {
                Err(FrameError::IoError(ref e)) if e.kind() == io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(100));
                    continue;
                }
                Err(err) => return Err(err),
                Ok(header) => break header,
            };
        };

        match frame_header {
            FrameHeader::V1 { length } => {
                let mut buffer = vec![0; length as usize];
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
                        Err(e) => return Err(FrameError::IoError(e)),
                    }
                }
                if !remaining.is_empty() {
                    Err(FrameError::IoError(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "Could not receive complete frame",
                    )))
                } else {
                    Ok(Self { data: buffer })
                }
            }
        }
    }
}

/// A Frame of referenced data to be transmitted using a specified version.
///
/// This struct references the data that has been transmitted.  It is essentially a sending frame.
pub struct FrameRef<'a> {
    version: FrameVersion,
    data: &'a [u8],
}

impl<'a> FrameRef<'a> {
    /// Construct a FrameRef for the given byte slice, which will be transmitted using the given
    /// frame version.
    pub fn new<'b: 'a>(version: FrameVersion, data: &'b [u8]) -> FrameRef<'a> {
        Self { version, data }
    }

    /// Write the frame to the given writer.
    ///
    /// # Errors
    ///
    /// Returns a FrameError if an IO error occurs.
    pub fn write<W: Write>(self, writer: &mut W) -> Result<(), FrameError> {
        let frame_header = match self.version {
            FrameVersion::V1 => FrameHeader::v1(self.data.len() as u32),
        };
        loop {
            match frame_header.write(writer) {
                Err(FrameError::IoError(ref e)) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(100));
                    continue;
                }
                Err(err) => return Err(err),
                Ok(_) => break,
            }
        }

        let mut buffer = &self.data[..];
        while !buffer.is_empty() {
            match writer.write(buffer) {
                Ok(0) => {
                    return Err(FrameError::IoError(std::io::Error::new(
                        std::io::ErrorKind::WriteZero,
                        "failed to write whole buffer",
                    )))
                }
                Ok(n) => buffer = &buffer[n..],
                Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => {}
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(100));
                }
                Err(e) => return Err(FrameError::IoError(e)),
            }
        }
        writer.flush()?;
        Ok(())
    }
}

/// A FrameHeader.
///
/// Each variant corresponds to the implementation for a given version.
#[derive(Debug, PartialEq)]
enum FrameHeader {
    V1 { length: u32 },
}

impl FrameHeader {
    /// Construct a version 1 frame header.
    fn v1(length: u32) -> Self {
        FrameHeader::V1 { length }
    }

    /// Read a FrameHeader from the given reader.
    ///
    /// This function uses the first 2 bytes of the stream to read the version, and constructs the
    /// corresponding frame header variant accordingly.
    ///
    /// # Errors
    ///
    /// Returns a FrameError if:
    ///
    /// - the version received does not match any of the existing variants
    /// - the frame header is not the proper length
    /// - the frame version fails its checksum
    /// - an IO error occurs
    fn read<R: Read>(reader: &mut R) -> Result<Self, FrameError> {
        let version = reader.read_u16::<BigEndian>()?;
        match version {
            1 => {
                // Header length + checksum byte
                let mut buffer = [0u8; HEADER_LENGTH + 1];
                let mut cursor = Cursor::new(&mut buffer[..]);
                cursor.write_u16::<BigEndian>(1u16)?;

                let n = reader.read(&mut cursor.get_mut()[std::mem::size_of::<u16>()..])?;
                if n != HEADER_LENGTH + 1 - std::mem::size_of::<u16>() {
                    return Err(FrameError::InvalidHeaderLength(n));
                }

                let checksum = compute_checksum(&cursor.get_ref()[..HEADER_LENGTH]);
                if checksum != cursor.get_ref()[HEADER_LENGTH] {
                    return Err(FrameError::InvalidChecksum);
                }

                Ok(FrameHeader::V1 {
                    length: cursor.read_u32::<BigEndian>()?,
                })
            }
            _ => Err(FrameError::UnsupportedVersion),
        }
    }

    /// Write this FrameHeader to the given writer.
    ///
    /// # Errors
    ///
    /// Returns a FrameError if:
    ///
    /// - an IO error occurs
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), FrameError> {
        match *self {
            FrameHeader::V1 { length } => {
                let mut header_bytes = [0u8; HEADER_LENGTH + 1];
                let mut cursor = Cursor::new(&mut header_bytes[..]);

                cursor.write_u16::<BigEndian>(1)?;
                cursor.write_u32::<BigEndian>(length)?;

                // reserve the remaining bytes

                cursor.get_mut()[HEADER_LENGTH] =
                    compute_checksum(&cursor.get_ref()[..HEADER_LENGTH]);

                writer.write_all(&cursor.into_inner()[..])?;
            }
        }

        Ok(())
    }
}

/// Compute a longitudinal check-sum
fn compute_checksum(buffer: &[u8]) -> u8 {
    // International standard ISO 1155[7] states that a longitudinal redundancy check for a
    // sequence of bytes may be computed in software by the following algorithm
    // lrc := 0
    // for each byte b in the buffer do
    //     lrc := (lrc + b) and 0xFF
    // lrc := (((lrc XOR 0xFF) + 1) and 0xFF)

    let mut lrc = 0u16;
    for b in buffer {
        lrc = (lrc + (*b as u16)) & 0x00ff;
    }

    lrc = ((lrc ^ 0x00ff) + 1u16) & 0xff;

    lrc as u8
}

/// Negotiate the frame version for a given socket connection.
pub enum FrameNegotiation {
    /// The Outbound variant transmits the min and max supported version, and expects to receive
    /// either a version in that range, or `0` if the other end cannot support the a version in
    /// that range.
    Outbound {
        min: FrameVersion,
        max: FrameVersion,
    },
    /// The Inbound variant transmits receives the min and max and decides if it should send its
    /// version or `0`, depending on whether or not it falls in the range.
    Inbound { version: FrameVersion },
}

impl FrameNegotiation {
    /// Construct the outbound side of a negotiation with the given min,max.
    pub fn outbound(min: FrameVersion, max: FrameVersion) -> Self {
        FrameNegotiation::Outbound { min, max }
    }

    /// Construct the inbound side of a negotiation with the given version.
    pub fn inbound(version: FrameVersion) -> Self {
        FrameNegotiation::Inbound { version }
    }

    /// Negotiate frame version to use for future communications over the given stream.
    ///
    /// # Errors
    ///
    /// Returns a FrameError if:
    ///
    /// - either end cannot agree on a version
    /// - an IO error, if one occurs
    pub fn negotiate<S: Read + Write>(self, stream: &mut S) -> Result<FrameVersion, FrameError> {
        match self {
            FrameNegotiation::Outbound { min, max } => {
                stream.write_u16::<BigEndian>(min as u16)?;
                stream.write_u16::<BigEndian>(max as u16)?;

                let frame_version = stream.read_u16::<BigEndian>()?;

                match frame_version {
                    0 => Err(FrameError::UnsupportedVersion),
                    1 => Ok(FrameVersion::V1),
                    _ => Err(FrameError::UnsupportedVersion),
                }
            }
            FrameNegotiation::Inbound { version } => {
                let min = stream.read_u16::<BigEndian>()?;
                let max = stream.read_u16::<BigEndian>()?;
                if min > version as u16 || max < version as u16 {
                    stream.write_u16::<BigEndian>(0)?;
                    Err(FrameError::UnsupportedVersion)
                } else {
                    stream.write_u16::<BigEndian>(version as u16)?;
                    Ok(version)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::thread;

    /// Test that a v1 frame header with a data length of 0 and the correct checksum will be read
    /// correctly into a FrameHeader.
    #[test]
    fn read_version_1_zero_length() {
        let header_bytes = vec![0u8; HEADER_LENGTH + 1];
        let mut header_cursor = Cursor::new(header_bytes);

        header_cursor
            .write_u16::<BigEndian>(1)
            .expect("Could not write to cursor");
        // set the checksum for 1 xored + 1
        header_cursor.get_mut()[HEADER_LENGTH] = 0xff;

        header_cursor.set_position(0);

        let frame_header = FrameHeader::read(&mut header_cursor).expect("Unable to read_header");

        assert_eq!(FrameHeader::v1(0), frame_header);
    }

    /// Test that a v1 frame header with a data length of 2 and the correct checksum will be read
    /// correctly into a FrameHeader.
    #[test]
    fn read_version_and_length() {
        let header_bytes = vec![0u8; HEADER_LENGTH + 1];
        let mut header_cursor = Cursor::new(header_bytes);

        header_cursor
            .write_u16::<BigEndian>(1)
            .expect("Could not write version to cursor");
        header_cursor
            .write_u32::<BigEndian>(2)
            .expect("Could not write length to cursor");
        // set the checksum for 1 xored + 1
        header_cursor.get_mut()[HEADER_LENGTH] = 0xfd;

        header_cursor.set_position(0);

        let frame_header = FrameHeader::read(&mut header_cursor).expect("Unable to read_header");

        assert_eq!(FrameHeader::v1(2), frame_header);
    }

    /// Test that a v1 frame header with an invalid checksum will return an error when read.
    #[test]
    fn fail_checksum() {
        let header_bytes = vec![0u8; HEADER_LENGTH + 1];
        let mut header_cursor = Cursor::new(header_bytes);

        header_cursor
            .write_u16::<BigEndian>(1)
            .expect("Could not write version to cursor");
        header_cursor
            .write_u32::<BigEndian>(2)
            .expect("Could not write length to cursor");
        header_cursor.set_position(0);

        match FrameHeader::read(&mut header_cursor) {
            Ok(_) => panic!("Should not have produced a frame header"),
            Err(FrameError::InvalidChecksum) => (),
            Err(err) => panic!("Produced invalid error: {}", err),
        }
    }

    /// Test that a v1 FrameHeader with a given length will be correctly written to bytes,
    /// producing the correct checksum.
    #[test]
    fn standard_write() {
        let header_bytes = vec![0u8; HEADER_LENGTH + 1];
        let mut header_cursor = Cursor::new(header_bytes);

        let frame_header = FrameHeader::v1(3);

        frame_header
            .write(&mut header_cursor)
            .expect("Unable to write frame header");

        header_cursor.set_position(0);
        assert_eq!(
            1,
            header_cursor
                .read_u16::<BigEndian>()
                .expect("Unable to read version")
        );
        assert_eq!(
            3,
            header_cursor
                .read_u32::<BigEndian>()
                .expect("Unable to read length")
        );

        assert_eq!(0xFC, header_cursor.get_ref()[HEADER_LENGTH]);
    }

    /// Test a round-trip write and read of a FrameHeader.  Construct a valid FrameHeader, and
    /// write it to bytes.  Read a new FrameHeader from the bytes and verify that they are equal.
    #[test]
    fn round_trip() {
        let header_bytes = vec![0u8; HEADER_LENGTH + 1];
        let mut header_cursor = Cursor::new(header_bytes);

        let frame_header = FrameHeader::v1(100);

        frame_header
            .write(&mut header_cursor)
            .expect("Unable to write frame header");

        header_cursor.set_position(0);
        let FrameHeader::V1 { length } =
            FrameHeader::read(&mut header_cursor).expect("Unable to read header");

        assert_eq!(100, length);
    }

    /// Test that outbound frame version negotiation works:
    /// 1. Create a stream pair
    /// 2. Send one end to a thread, to act as the inbound receiver
    /// 3. Create an outbound negotiation and execute it on the stream
    /// 4. Verify that the agree on versions.
    #[test]
    fn basic_outbound_negotiation() {
        let (mut tx, mut rx) = stream::byte_stream_pair();

        let (done_tx, done_rx) = std::sync::mpsc::channel();
        let join_handle = thread::spawn(move || {
            let res = FrameNegotiation::inbound(FrameVersion::V1)
                .negotiate(&mut rx)
                .expect("Should have successfully negotiated");

            done_rx.recv().unwrap();

            res
        });

        let version = FrameNegotiation::outbound(FrameVersion::V1, FrameVersion::V1)
            .negotiate(&mut tx)
            .expect("Unable to negotiate a valid version");

        assert_eq!(FrameVersion::V1, version);

        done_tx.send(1u8).expect("unable to send stop signal");

        let remote_res = join_handle.join().expect("Unable to join thread");

        assert_eq!(FrameVersion::V1, remote_res);
    }

    /// Test that outbound frame version negotiation works:
    /// 1. Create a stream pair
    /// 2. Send one end to a thread, to act as the inbound receiver - this stream will return no
    ///    version supported.
    /// 3. Create an outbound negotiation and execute it on the stream.
    /// 4. Verify that negotiation returns an error.
    #[test]
    fn unsupported_range() {
        let (mut tx, mut rx) = stream::byte_stream_pair();

        let (done_tx, done_rx) = std::sync::mpsc::channel();
        let join_handle = thread::spawn(move || {
            rx.write_u16::<BigEndian>(0)
                .expect("Unable to write unsupported version");

            done_rx.recv().unwrap();
        });

        let res = FrameNegotiation::outbound(FrameVersion::V1, FrameVersion::V1).negotiate(&mut tx);

        done_tx.send(1u8).expect("Unable to send stop signal");

        join_handle.join().expect("Unable to join thread");

        match res {
            Err(FrameError::UnsupportedVersion) => (),
            res => {
                panic!("Unexpected result: {:?}", res);
            }
        }
    }

    /// Test that outbound frame version negotiation works:
    /// 1. Create a stream pair
    /// 2. Send one end to a thread, to act as the outbound end - this stream will send a range
    ///    that does not include V1.
    /// 3. Create an inbound negotiation and execute it on the stream.
    /// 4. Verify that negotiation returns an error.
    #[test]
    fn out_of_range() {
        let (mut tx, mut rx) = stream::byte_stream_pair();

        let (done_tx, done_rx) = std::sync::mpsc::channel();
        let join_handle = thread::spawn(move || {
            rx.write_u16::<BigEndian>(3)
                .expect("Unable to write min version");
            rx.write_u16::<BigEndian>(5)
                .expect("Unable to write min version");

            let res = rx
                .read_u16::<BigEndian>()
                .expect("Unable to read negotiated version");

            done_rx.recv().unwrap();

            res
        });

        let res = FrameNegotiation::inbound(FrameVersion::V1).negotiate(&mut tx);

        done_tx.send(1u8).expect("Unable to send stop signal");

        let remote_res = join_handle.join().expect("unable to join thread");

        match res {
            Err(FrameError::UnsupportedVersion) => (),
            res => panic!("Unexpected result: {:?}", res),
        }

        assert_eq!(0u16, remote_res);
    }

    /// Read a frame from a stream.  The stream will be constructed with a valid header and a short
    /// data payload. Frame::read should result in a valid frame, with the expected data.
    #[test]
    fn read_frame_v1() {
        let input = b"hello";
        let mut cursor = Cursor::new(vec![0; 128]);
        FrameHeader::v1(input.len() as u32)
            .write(&mut cursor)
            .expect("Unable to write header");

        cursor.write(&input[..]).expect("Unable to write data");

        cursor.set_position(0);

        let frame = Frame::read(&mut cursor).expect("Unable to read frame");

        assert_eq!(input.to_vec(), frame.data);
    }

    /// Write a frame to a stream and verify that an equivalent frame is read back from the stream.
    #[test]
    fn frame_round_trip() {
        let input = b"hello world";
        let frame_ref = FrameRef::new(FrameVersion::V1, input);

        let mut cursor = Cursor::new(vec![0, 128]);

        frame_ref.write(&mut cursor).expect("Unable to write data");

        cursor.set_position(0);

        let frame = Frame::read(&mut cursor).expect("Unable to read frame");

        assert_eq!(input.to_vec(), frame.data);
    }

    #[cfg(not(target_os = "unix"))]
    mod stream {
        use std::io::{Error as IoError, Read, Write};
        use std::sync::mpsc::{channel, Receiver, Sender};

        pub struct InProcByteStream {
            outbound: Sender<Vec<u8>>,
            inbound: Receiver<Vec<u8>>,
        }

        pub fn byte_stream_pair() -> (InProcByteStream, InProcByteStream) {
            let (left_tx, left_rx) = channel();
            let (right_tx, right_rx) = channel();

            (
                InProcByteStream {
                    outbound: right_tx,
                    inbound: left_rx,
                },
                InProcByteStream {
                    outbound: left_tx,
                    inbound: right_rx,
                },
            )
        }

        impl Read for InProcByteStream {
            fn read(&mut self, buf: &mut [u8]) -> Result<usize, IoError> {
                let bytes_received = self
                    .inbound
                    .recv()
                    .map_err(|e| IoError::new(std::io::ErrorKind::UnexpectedEof, e))?;

                buf.copy_from_slice(&bytes_received);

                Ok(bytes_received.len())
            }
        }

        impl Write for InProcByteStream {
            fn write(&mut self, buf: &[u8]) -> Result<usize, IoError> {
                let n = buf.len();

                self.outbound
                    .send(buf.to_vec())
                    .map_err(|e| IoError::new(std::io::ErrorKind::Interrupted, e))?;

                Ok(n)
            }

            fn flush(&mut self) -> Result<(), IoError> {
                Ok(())
            }
        }
    }

    #[cfg(target_os = "unix")]
    mod stream {
        use std::os::unix::net::UnixStream;

        pub struct InProcByteStream {
            inner: UnixStream,
        }

        pub fn byte_stream_pair() -> (InProcByteStream, InProcByteStream) {
            let (left, right) = UnixStream::pair().expect("Unable to create unix stream");

            (
                InProcByteStream { inner: left },
                InProcByteStream { inner: right },
            )
        }

        impl Read for InProcByteStream {
            fn read(&mut self, buf: &mut [u8]) -> Result<usize, IoError> {
                self.inner.read(buf)
            }
        }

        impl Write for InProcByteStream {
            fn write(&mut self, buf: &[u8]) -> Result<usize, IoError> {
                self.inner.write(buf)
            }
        }
    }
}
