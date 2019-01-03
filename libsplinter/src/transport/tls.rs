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

use mio::{unix::EventedFd, Evented, Poll, PollOpt, Ready, Token};
use openssl::error::ErrorStack;
use openssl::ssl::{
    Error as OpensslError, HandshakeError, SslAcceptor, SslConnector, SslFiletype, SslMethod,
    SslStream,
};
use url::{ParseError, Url};

use std::io;
use std::net::{Ipv4Addr, Ipv6Addr, TcpListener, TcpStream};
use std::os::unix::io::{AsRawFd, RawFd};
use std::path::Path;

use crate::transport::{
    read, write, AcceptError, ConnectError, Connection, DisconnectError, ListenError, Listener,
    RecvError, SendError, Transport,
};

pub struct TlsTransport {
    connector: SslConnector,
    acceptor: SslAcceptor,
}

impl TlsTransport {
    pub fn new(
        ca_cert: String,
        client_key: String,
        client_cert: String,
        server_key: String,
        server_cert: String,
    ) -> Result<Self, TlsInitError> {
        let ca_cert_path = Path::new(&ca_cert);
        let client_cert_path = Path::new(&client_cert);
        let client_key_path = Path::new(&client_key);
        let server_cert_path = Path::new(&server_cert);
        let server_key_path = Path::new(&server_key);

        // Build TLS Connector
        let mut connector = SslConnector::builder(SslMethod::tls())?;
        connector.set_private_key_file(&client_key_path, SslFiletype::PEM)?;
        connector.set_certificate_chain_file(client_cert_path)?;
        connector.check_private_key()?;
        connector.set_ca_file(ca_cert_path)?;
        let connector = connector.build();

        // Build TLS Acceptor
        let mut acceptor = SslAcceptor::mozilla_modern(SslMethod::tls())?;
        acceptor.set_private_key_file(server_key_path, SslFiletype::PEM)?;
        acceptor.set_certificate_chain_file(&server_cert_path)?;
        acceptor.check_private_key()?;
        acceptor.set_ca_file(ca_cert_path)?;
        let acceptor = acceptor.build();

        Ok(TlsTransport {
            connector,
            acceptor,
        })
    }
}

fn endpoint_to_dns_name(endpoint: &str) -> Result<String, ParseError> {
    let mut address = String::from("tcp://");
    address.push_str(endpoint);
    let url = Url::parse(&address)?;
    let dns_name = match url.domain() {
        Some(d) if d.parse::<Ipv4Addr>().is_ok() => "localhost",
        Some(d) if d.parse::<Ipv6Addr>().is_ok() => "localhost",
        Some(d) => d,
        None => "localhost",
    };
    Ok(String::from(dns_name))
}

impl Transport for TlsTransport {
    fn connect(&mut self, endpoint: &str) -> Result<Box<dyn Connection>, ConnectError> {
        let dns_name = endpoint_to_dns_name(endpoint)?;

        let stream = TcpStream::connect(endpoint)?;
        let tls_stream = self.connector.connect(&dns_name, stream)?;

        tls_stream.get_ref().set_nonblocking(true)?;
        let connection = TlsConnection { stream: tls_stream };
        Ok(Box::new(connection))
    }

    fn listen(&mut self, bind: &str) -> Result<Box<dyn Listener>, ListenError> {
        Ok(Box::new(TlsListener {
            listener: TcpListener::bind(bind)?,
            acceptor: self.acceptor.clone(),
        }))
    }
}

pub struct TlsListener {
    listener: TcpListener,
    acceptor: SslAcceptor,
}

impl Listener for TlsListener {
    fn accept(&mut self) -> Result<Box<dyn Connection>, AcceptError> {
        let (stream, _) = self.listener.accept()?;
        let tls_stream = self.acceptor.accept(stream)?;
        tls_stream.get_ref().set_nonblocking(true)?;
        let connection = TlsConnection { stream: tls_stream };
        Ok(Box::new(connection))
    }

    fn endpoint(&self) -> String {
        self.listener.local_addr().unwrap().to_string()
    }
}

pub struct TlsConnection {
    stream: SslStream<TcpStream>,
}

impl Connection for TlsConnection {
    fn send(&mut self, message: &[u8]) -> Result<(), SendError> {
        write(&mut self.stream, message)
    }

    fn recv(&mut self) -> Result<Vec<u8>, RecvError> {
        read(&mut self.stream)
    }

    fn remote_endpoint(&self) -> String {
        self.stream.get_ref().peer_addr().unwrap().to_string()
    }

    fn local_endpoint(&self) -> String {
        self.stream.get_ref().local_addr().unwrap().to_string()
    }

    fn disconnect(&mut self) -> Result<(), DisconnectError> {
        // returns Shutdown state
        self.stream.shutdown()?;
        Ok(())
    }

    fn evented(&self) -> &dyn Evented {
        self
    }
}

impl AsRawFd for TlsConnection {
    fn as_raw_fd(&self) -> RawFd {
        self.stream.get_ref().as_raw_fd()
    }
}

impl Evented for TlsConnection {
    fn register(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        EventedFd(&self.as_raw_fd()).register(poll, token, interest, opts)
    }

    fn reregister(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        EventedFd(&self.as_raw_fd()).reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        EventedFd(&self.as_raw_fd()).deregister(poll)
    }
}

#[derive(Debug)]
pub enum TlsInitError {
    ProtocolError(String),
}

impl From<ErrorStack> for TlsInitError {
    fn from(error: ErrorStack) -> Self {
        TlsInitError::ProtocolError(format!("Openssl Error: {}", error))
    }
}

impl From<HandshakeError<TcpStream>> for AcceptError {
    fn from(handshake_error: HandshakeError<TcpStream>) -> Self {
        AcceptError::ProtocolError(format!("TLS Handshake Err: {}", handshake_error))
    }
}

impl From<HandshakeError<TcpStream>> for ConnectError {
    fn from(handshake_error: HandshakeError<TcpStream>) -> Self {
        ConnectError::ProtocolError(format!("TLS Handshake Err: {}", handshake_error))
    }
}

impl From<ParseError> for ConnectError {
    fn from(parse_error: ParseError) -> Self {
        ConnectError::ParseError(format!("Parse Error: {:?}", parse_error.to_string()))
    }
}

impl From<OpensslError> for DisconnectError {
    fn from(openssl_error: OpensslError) -> Self {
        DisconnectError::ProtocolError(format!("Openssl Err: {}", openssl_error))
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::transport::tests;
    use openssl::asn1::Asn1Time;
    use openssl::bn::{BigNum, MsbOption};
    use openssl::hash::MessageDigest;
    use openssl::pkey::{PKey, PKeyRef, Private};
    use openssl::rsa::Rsa;
    use openssl::x509::extension::{BasicConstraints, ExtendedKeyUsage, KeyUsage};
    use openssl::x509::{X509NameBuilder, X509Ref, X509};
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    use tempdir::TempDir;

    // Make a certificate and private key for the Certifcate Authority
    fn make_ca_cert() -> (PKey<Private>, X509) {
        let rsa = Rsa::generate(2048).unwrap();
        let privkey = PKey::from_rsa(rsa).unwrap();

        let mut x509_name = X509NameBuilder::new().unwrap();
        x509_name.append_entry_by_text("CN", "ca test").unwrap();
        let x509_name = x509_name.build();

        let mut cert_builder = X509::builder().unwrap();
        cert_builder.set_version(2).unwrap();
        cert_builder.set_subject_name(&x509_name).unwrap();
        cert_builder.set_issuer_name(&x509_name).unwrap();
        cert_builder.set_pubkey(&privkey).unwrap();

        let not_before = Asn1Time::days_from_now(0).unwrap();
        cert_builder.set_not_before(&not_before).unwrap();
        let not_after = Asn1Time::days_from_now(365).unwrap();
        cert_builder.set_not_after(&not_after).unwrap();

        cert_builder
            .append_extension(BasicConstraints::new().critical().ca().build().unwrap())
            .unwrap();
        cert_builder
            .append_extension(KeyUsage::new().key_cert_sign().build().unwrap())
            .unwrap();

        cert_builder
            .sign(&privkey, MessageDigest::sha256())
            .unwrap();
        let cert = cert_builder.build();

        (privkey, cert)
    }

    // Make a certificate and private key signed by the given CA cert and private key
    fn make_ca_signed_cert(
        ca_cert: &X509Ref,
        ca_privkey: &PKeyRef<Private>,
    ) -> (PKey<Private>, X509) {
        let rsa = Rsa::generate(2048).unwrap();
        let privkey = PKey::from_rsa(rsa).unwrap();

        let mut x509_name = X509NameBuilder::new().unwrap();
        x509_name.append_entry_by_text("CN", "localhost").unwrap();
        let x509_name = x509_name.build();

        let mut cert_builder = X509::builder().unwrap();
        cert_builder.set_version(2).unwrap();
        let serial_number = {
            let mut serial = BigNum::new().unwrap();
            serial.rand(159, MsbOption::MAYBE_ZERO, false).unwrap();
            serial.to_asn1_integer().unwrap()
        };
        cert_builder.set_serial_number(&serial_number).unwrap();
        cert_builder.set_subject_name(&x509_name).unwrap();
        cert_builder
            .set_issuer_name(ca_cert.subject_name())
            .unwrap();
        cert_builder.set_pubkey(&privkey).unwrap();
        let not_before = Asn1Time::days_from_now(0).unwrap();
        cert_builder.set_not_before(&not_before).unwrap();
        let not_after = Asn1Time::days_from_now(365).unwrap();
        cert_builder.set_not_after(&not_after).unwrap();

        cert_builder
            .append_extension(
                ExtendedKeyUsage::new()
                    .server_auth()
                    .client_auth()
                    .build()
                    .unwrap(),
            )
            .unwrap();

        cert_builder
            .sign(&ca_privkey, MessageDigest::sha256())
            .unwrap();
        let cert = cert_builder.build();

        (privkey, cert)
    }

    fn write_file(mut temp_dir: PathBuf, file_name: &str, bytes: &[u8]) -> String {
        temp_dir.push(file_name);
        let path = temp_dir.to_str().unwrap().to_string();
        let mut file = File::create(path.to_string()).unwrap();
        file.write_all(bytes).unwrap();

        path
    }

    pub fn create_test_tls_transport() -> TlsTransport {
        // Genearte Certificat Authority keys and certificate
        let (ca_key, ca_cert) = make_ca_cert();

        // create temp directory to store ca.cert
        let temp_dir = TempDir::new("tls-transport-test").unwrap();
        let temp_dir_path = temp_dir.path();
        let ca_path_file = write_file(
            temp_dir_path.to_path_buf(),
            "ca.cert",
            &ca_cert.to_pem().unwrap(),
        );

        // Generate client and server keys and certificates
        let (client_key, client_cert) = make_ca_signed_cert(&ca_cert, &ca_key);
        let (server_key, server_cert) = make_ca_signed_cert(&ca_cert, &ca_key);

        let client_cert_file = write_file(
            temp_dir_path.to_path_buf(),
            "client.cert",
            &client_cert.to_pem().unwrap(),
        );

        let client_key_file = write_file(
            temp_dir_path.to_path_buf(),
            "client.key",
            &client_key.private_key_to_pem_pkcs8().unwrap(),
        );

        let server_cert_file = write_file(
            temp_dir_path.to_path_buf(),
            "server.cert",
            &server_cert.to_pem().unwrap(),
        );

        let server_key_file = write_file(
            temp_dir_path.to_path_buf(),
            "server.key",
            &server_key.private_key_to_pem_pkcs8().unwrap(),
        );

        // Create TLsTransport
        TlsTransport::new(
            ca_path_file,
            client_key_file,
            client_cert_file,
            server_key_file,
            server_cert_file,
        )
        .unwrap()
    }

    #[test]
    fn test_transport() {
        let transport = create_test_tls_transport();
        tests::test_transport(transport, "127.0.0.1:0");
    }

    #[test]
    fn test_poll() {
        let transport = create_test_tls_transport();
        tests::test_poll(transport, "127.0.0.1:0");
    }
}
