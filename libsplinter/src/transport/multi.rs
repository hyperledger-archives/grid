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
use super::{ConnectError, Connection, ListenError, Listener, Transport};

type SendableTransport = Box<dyn Transport + Send>;

/// A MultiTransport holds a collection of transports, referenced by protocol.
///
/// Endpoints and bind strings are specified using standard url-style strings.  For example,
/// connecting over TLS would be handled with the connect string `"tls://<some-address>:<port>"`
///
/// Endpoints and bind strings provided without a protocol will use the provided default transport
/// protocol type.
pub struct MultiTransport {
    transports: Vec<SendableTransport>,
}

impl MultiTransport {
    /// Construct a new MultiTransport
    pub fn new(transports: Vec<SendableTransport>) -> Self {
        Self { transports }
    }
}

impl Transport for MultiTransport {
    fn accepts(&self, address: &str) -> bool {
        self.transports
            .iter()
            .any(|transport| transport.accepts(address))
    }

    fn connect(&mut self, endpoint: &str) -> Result<Box<dyn Connection>, ConnectError> {
        self.transports
            .iter_mut()
            .find(|transport| transport.accepts(endpoint))
            .ok_or_else(|| {
                ConnectError::ProtocolError(format!("Unknown protocol \"{}\"", endpoint))
            })
            .and_then(|transport| transport.connect(endpoint))
    }

    fn listen(&mut self, bind: &str) -> Result<Box<dyn Listener>, ListenError> {
        self.transports
            .iter_mut()
            .find(|transport| transport.accepts(bind))
            .ok_or_else(|| ListenError::ProtocolError(format!("Unknown protocol \"{}\"", bind)))
            .and_then(|transport| transport.listen(bind))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::channel;
    use std::thread;
    use std::time::Duration;

    use super::*;
    use crate::transport::{
        raw, tests, tls::tests::create_test_tls_transport, RecvError, SendError,
    };

    /// Test that the MultiTransport will accept all possible connect/bind strings for the
    /// underlying transports.
    #[test]
    fn test_accepts() {
        let raw_transport = Box::new(raw::RawTransport::default());
        let tls_transport = Box::new(create_test_tls_transport(true));

        let transport = MultiTransport::new(vec![raw_transport, tls_transport]);
        assert!(transport.accepts("127.0.0.1:0"));
        assert!(transport.accepts("tcp://127.0.0.1:0"));
        assert!(transport.accepts("tls://127.0.0.1:0"));
        assert!(!transport.accepts("foo://127.0.0.1:0"));
    }

    /// Test MultiTransport using a raw transport for the listening endpoint, with the standard
    /// transport tests.
    #[test]
    fn test_transport_raw_default_listener() {
        let raw_transport = Box::new(raw::RawTransport::default());
        let tls_transport = Box::new(create_test_tls_transport(true));

        let transport = MultiTransport::new(vec![raw_transport, tls_transport]);
        tests::test_transport(transport, "127.0.0.1:0");
    }

    /// Test MultiTransport using a TLS transport for the listening endpoint, with the standard
    /// transport tests.
    #[test]
    fn test_transport_tls_default_listener() {
        let raw_transport = Box::new(raw::RawTransport::default());
        let tls_transport = Box::new(create_test_tls_transport(true));

        let transport = MultiTransport::new(vec![tls_transport, raw_transport]);
        tests::test_transport(transport, "127.0.0.1:0");
    }

    /// Create a transport with tcp and tls transports and attempt to create an unknown protocol.
    /// Expect that a protocol error should be returned.
    #[test]
    fn test_invalid_protocol() {
        let raw_transport = Box::new(raw::RawTransport::default());
        let tls_transport = Box::new(create_test_tls_transport(true));

        let mut transport = MultiTransport::new(vec![raw_transport, tls_transport]);

        match transport.connect("foo://someplace:8000") {
            Ok(_) => panic!("Unexpected successful result"),
            Err(ConnectError::ProtocolError(msg)) => {
                assert_eq!("Unknown protocol \"foo://someplace:8000\"", msg)
            }
            Err(err) => panic!("Unexpected error {:?}", err),
        }
    }

    macro_rules! block {
        ($op:expr, $err:ident) => {
            loop {
                match $op {
                    Err($err::WouldBlock) => {
                        thread::sleep(Duration::from_millis(100));
                        continue;
                    }
                    Err(err) => break Err(err),
                    Ok(ok) => break Ok(ok),
                }
            }
        };
    }

    macro_rules! assert_ok {
        ($op:expr) => {
            match $op {
                Ok(ok) => ok,
                Err(err) => panic!("Expected Ok(...), got Err({:?})", err),
            }
        };
    }

    /// Test that an outbound connection is properly made when using a multi-transport with tls as
    /// an outbound-only transport.
    #[cfg(not(unix))]
    #[test]
    fn test_outbound_tls_only() {
        test_outgoing_connections(create_test_tls_transport(true), "127.0.0.1:0", {
            let raw_transport = Box::new(raw::RawTransport::default());
            let tls_transport = Box::new(create_test_tls_transport(true));

            MultiTransport::new(vec![raw_transport, tls_transport])
        });
    }

    /// Test that an outbound connection is properly made when using a multi-transport with tls as
    /// the default connection
    #[cfg(not(unix))]
    #[test]
    fn test_outbound_tls_listener() {
        test_outgoing_connections(create_test_tls_transport(true), "127.0.0.1:0", {
            let raw_transport = Box::new(raw::RawTransport::default());
            let tls_transport = Box::new(create_test_tls_transport(true));

            MultiTransport::new(vec![tls_transport, raw_transport])
        });
    }

    /// Test that an outbound connection is properly made when using a multi-transport with raw as
    /// an outbound-only transport.
    #[test]
    fn test_outbound_raw_only() {
        test_outgoing_connections(raw::RawTransport::default(), "127.0.0.1:0", {
            let raw_transport = Box::new(raw::RawTransport::default());
            let tls_transport = Box::new(create_test_tls_transport(true));

            MultiTransport::new(vec![tls_transport, raw_transport])
        });
    }

    /// Test that an outbound connection is properly made when using a multi-transport with raw as
    /// the listenting connection
    #[test]
    fn test_outbound_raw_listener() {
        test_outgoing_connections(raw::RawTransport::default(), "127.0.0.1:0", {
            let raw_transport = Box::new(raw::RawTransport::default());
            let tls_transport = Box::new(create_test_tls_transport(true));

            MultiTransport::new(vec![raw_transport, tls_transport])
        });
    }

    fn test_outgoing_connections<T>(
        listening_transport: T,
        bind: &str,
        mult_transport: MultiTransport,
    ) where
        T: Transport,
    {
        let mut listening_transport = listening_transport;
        let mut listener = assert_ok!(listening_transport.listen(bind));
        let endpoint = listener.endpoint();

        let (ready_tx, ready_rx) = channel();

        let handle = thread::spawn(move || {
            let mut transport = mult_transport;

            let mut conn = assert_ok!(transport.connect(&endpoint));

            // Block waiting for other thread to send everything
            ready_rx.recv().unwrap();

            assert_eq!(b"hello".to_vec(), assert_ok!(Connection::recv(&mut *conn)));
            assert_ok!(conn.send(b"world"));
        });

        let mut conn = assert_ok!(listener.accept());
        assert_ok!(block!(conn.send(b"hello"), SendError));

        // Signal done sending to background thread
        ready_tx.send(()).unwrap();

        assert_eq!(
            b"world".to_vec(),
            assert_ok!(block!(conn.recv(), RecvError))
        );

        handle.join().unwrap();
    }
}
