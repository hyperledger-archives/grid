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
use openssl::ssl::{SslConnector, SslAcceptor, SslStream, HandshakeError, Error as OpensslError};
use url::{ParseError, Url};

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, Ipv4Addr, Ipv6Addr};
use std::time::Duration;

use transport::*;

pub struct TlsTransport {
    connector: SslConnector,
    acceptor: SslAcceptor
}

impl TlsTransport {
  pub fn new(connector: SslConnector, acceptor: SslAcceptor) -> Self {
      TlsTransport {
          connector,
          acceptor
      }
  }
}


impl Transport for TlsTransport {
    fn connect(&mut self, endpoint: &str) -> Result<Box<dyn Connection>, ConnectError> {
        let mut address = String::from("tcp://");
        address.push_str(endpoint);
        let url = Url::parse(&address)?;
        let dns_name = match url.domain() {
            Some(d) if d.parse::<Ipv4Addr>().is_ok() => "localhost",
            Some(d) if d.parse::<Ipv6Addr>().is_ok() =>  "localhost",
            Some(d) => d,
            None => "localhost",
        };

        let stream = self.connector.connect(dns_name, TcpStream::connect(endpoint)?)?;
        let connection = TlsConnection {
            stream,
        };
        Ok(Box::new(connection))
    }

    fn listen(&mut self, bind: &str) -> Result<Box<dyn Listener>, ListenError> {
        Ok(Box::new(TlsListener{ listener: TcpListener::bind(bind)?,
            acceptor: self.acceptor.clone()
        }))
    }
}

pub struct TlsListener {
    listener: TcpListener,
    acceptor: SslAcceptor,
}

impl Listener for TlsListener {
    fn accept(&mut self) -> Result<Box<dyn Connection>, AcceptError> {
        let (tcp_stream, _) = self.listener.accept()?;
        let stream = self.acceptor.accept(tcp_stream)?;
        let connection = TlsConnection { stream };
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

    fn recv(&mut self, timeout: Option<Duration>) -> Result<Vec<u8>, RecvError> {
        self.stream.get_mut().set_read_timeout(timeout)?;
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
mod tests {
    use super::*;
    use openssl::ssl::{SslMethod, SslConnector, SslAcceptor};
    use openssl::asn1::Asn1Time;
    use openssl::bn::{BigNum, MsbOption};
    use openssl::hash::MessageDigest;
    use openssl::pkey::{PKey, PKeyRef, Private};
    use openssl::rsa::Rsa;
    use openssl::x509::{X509, X509NameBuilder, X509Ref};
    use openssl::x509::extension::{BasicConstraints, KeyUsage, ExtendedKeyUsage};
    use transport::tests;
    use std::path::Path;
    use std::env;
    use std::fs::File;

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

        cert_builder.append_extension(
            BasicConstraints::new().critical().ca().build().unwrap()
        ).unwrap();
        cert_builder.append_extension(KeyUsage::new()
            .key_cert_sign()
            .build().unwrap()).unwrap();

        cert_builder.sign(&privkey, MessageDigest::sha256()).unwrap();
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
        cert_builder.set_issuer_name(ca_cert.subject_name()).unwrap();
        cert_builder.set_pubkey(&privkey).unwrap();
        let not_before = Asn1Time::days_from_now(0).unwrap();
        cert_builder.set_not_before(&not_before).unwrap();
        let not_after = Asn1Time::days_from_now(365).unwrap();
        cert_builder.set_not_after(&not_after).unwrap();

        cert_builder.append_extension(ExtendedKeyUsage::new()
            .server_auth()
            .client_auth()
            .build().unwrap()).unwrap();

        cert_builder.sign(&ca_privkey, MessageDigest::sha256()).unwrap();
        let cert = cert_builder.build();

        (privkey, cert)
    }


    #[test]
    fn test_transport() {
        // Genearte Certificat Authority keys and certificate
        let (ca_key, ca_cert) = make_ca_cert();

        // create temp directory to store ca.cert
        let mut temp_dir = env::temp_dir();
        temp_dir.push("ca.cert");
        let ca_path = temp_dir.to_str().unwrap().to_string();

        let mut file = File::create(ca_path.to_string()).unwrap();
        let ca_pem = String::from_utf8(ca_cert.to_pem().unwrap()).unwrap();
        file.write_all(ca_pem.as_bytes()).unwrap();

        // Generate client and server keys and  certificates
        let (client_key, client_cert) = make_ca_signed_cert(&ca_cert, &ca_key);
        let (server_key, server_cert) = make_ca_signed_cert(&ca_cert, &ca_key);

        // Build TLS Connector
        let ca_cert_path = Path::new(&ca_path);
        let mut connector = SslConnector::builder(SslMethod::tls()).unwrap();
        connector.set_private_key(&client_key).unwrap();
        connector.set_certificate(&client_cert).unwrap();
        connector.check_private_key().unwrap();
        connector.set_ca_file(ca_cert_path).unwrap();
        let connector = connector.build();

        // Build TLS Acceptor
        let mut acceptor = SslAcceptor::mozilla_modern(SslMethod::tls()).unwrap();
        acceptor.set_private_key(&server_key).unwrap();
        acceptor.set_certificate(&server_cert).unwrap();
        acceptor.check_private_key().unwrap();
        acceptor.set_ca_file(ca_cert_path).unwrap();
        let acceptor = acceptor.build();

        // Create TLsTransport
        let transport = TlsTransport::new(
            connector,
            acceptor,
        );

        // Run transport test
        tests::test_transport(transport, "127.0.0.1:0");
    }
}
