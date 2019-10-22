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

use openssl::ssl::{SslConnector, SslFiletype, SslMethod};
use protobuf;
use splinter::protos::network::NetworkMessage;
use splinter::transport::tls::TlsConnection;
use splinter::transport::Connection;
use url;

use std::net::{SocketAddr, TcpStream, ToSocketAddrs};

use crate::error::SplinterError;

pub struct Certs {
    ca_cert: String,
    client_cert: String,
    client_priv: String,
}

impl Certs {
    pub fn new(ca_cert: String, client_cert: String, client_priv: String) -> Certs {
        Certs {
            ca_cert,
            client_cert,
            client_priv,
        }
    }

    pub fn get_ca_cert(&self) -> &str {
        &self.ca_cert
    }

    pub fn get_client_cert(&self) -> &str {
        &self.client_cert
    }

    pub fn get_client_priv(&self) -> &str {
        &self.client_priv
    }
}

pub struct SplinterClient {
    socket: TlsConnection,
}

impl SplinterClient {
    pub fn connect(url: &str, certs: Certs) -> Result<SplinterClient, SplinterError> {
        let (hostname, port) = {
            let url = url::Url::parse(url)?;
            let hs = if let Some(hs) = url.host_str() {
                hs.to_string()
            } else {
                return Err(SplinterError::HostNameNotFound);
            };

            let p = if let Some(p) = url.port() {
                p
            } else {
                return Err(SplinterError::HostNameNotFound);
            };

            (hs, p)
        };

        let addr = resolve_hostname(&format!("{}:{}", hostname, port))?;

        // Build TLS Connector
        let mut connector = SslConnector::builder(SslMethod::tls())?;
        connector.set_private_key_file(certs.get_client_priv(), SslFiletype::PEM)?;
        connector.set_certificate_chain_file(certs.get_client_cert())?;
        connector.check_private_key()?;
        connector.set_ca_file(certs.get_ca_cert())?;
        let connector = connector.build();

        let endpoint = &format!("{}:{}", addr.ip(), addr.port());
        let stream = TcpStream::connect(endpoint)?;
        let tls_stream = connector.connect("localhost", stream)?;
        let connection = TlsConnection::new(tls_stream);

        Ok(SplinterClient { socket: connection })
    }

    pub fn send(&mut self, req: &NetworkMessage) -> Result<(), SplinterError> {
        let raw_msg = protobuf::Message::write_to_bytes(req)?;
        self.socket.send(&raw_msg)?;
        Ok(())
    }
}

fn resolve_hostname(hostname: &str) -> Result<SocketAddr, SplinterError> {
    hostname
        .to_socket_addrs()?
        .find(std::net::SocketAddr::is_ipv4)
        .ok_or(SplinterError::CouldNotResolveHostName)
}
