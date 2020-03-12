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

use protobuf;
use splinter::protos::network::NetworkMessage;
use splinter::transport::socket::TlsTransport;
use splinter::transport::{Connection, Transport};
use url;

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
    socket: Box<dyn Connection>,
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

        let mut transport = match TlsTransport::new(
            Some(certs.get_ca_cert().to_string()),
            certs.get_client_priv().to_string(),
            certs.get_client_cert().to_string(),
            certs.get_client_priv().to_string(),
            certs.get_client_cert().to_string(),
        ) {
            Ok(transport) => transport,
            Err(err) => {
                return Err(SplinterError::TLSError(format!(
                    "An error occurred while creating TLS transport: {}",
                    err
                )))
            }
        };

        Ok(SplinterClient {
            socket: transport
                .connect(&format!("tls://{}:{}", hostname, port))
                .map_err(|err| {
                    SplinterError::TLSError(format!(
                        "Unable to connect to \"{}:{}\":h  {}",
                        hostname, port, err
                    ))
                })?,
        })
    }

    pub fn send(&mut self, req: &NetworkMessage) -> Result<(), SplinterError> {
        let raw_msg = protobuf::Message::write_to_bytes(req)?;
        self.socket.send(&raw_msg)?;
        Ok(())
    }
}
