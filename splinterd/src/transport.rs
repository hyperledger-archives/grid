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

use std::fs;
use std::path::Path;

use splinter::transport::raw::RawTransport;
use splinter::transport::tls::TlsTransport;
use splinter::transport::Transport;

use crate::config::Config;
use crate::error::GetTransportError;

pub fn get_transport(
    transport_type: &str,
    config: &Config,
) -> Result<Box<dyn Transport + Send>, GetTransportError> {
    match transport_type {
        "tls" => {
            let client_cert = config.client_cert();
            if !Path::new(&client_cert).is_file() {
                return Err(GetTransportError::CertError(format!(
                    "Must provide a valid client certificate: {}",
                    client_cert
                )));
            }

            let server_cert = config.server_cert();
            if !Path::new(&server_cert).is_file() {
                return Err(GetTransportError::CertError(format!(
                    "Must provide a valid server certificate: {}",
                    server_cert
                )));
            }

            let server_key_file = config.server_key();
            if !Path::new(&server_key_file).is_file() {
                return Err(GetTransportError::CertError(format!(
                    "Must provide a valid server key path: {}",
                    server_key_file
                )));
            }

            let client_key_file = config.client_key();
            if !Path::new(&client_key_file).is_file() {
                return Err(GetTransportError::CertError(format!(
                    "Must provide a valid client key path: {}",
                    client_key_file
                )));
            }

            let insecure = config.insecure();
            if insecure {
                warn!("Starting TlsTransport in insecure mode");
            }
            let ca_file = {
                if insecure {
                    None
                } else {
                    let ca_file = config.ca_certs();
                    if !Path::new(&ca_file).is_file() {
                        return Err(GetTransportError::CertError(format!(
                            "Must provide a valid file containing ca certs: {}",
                            ca_file
                        )));
                    }
                    match fs::canonicalize(&ca_file)?.to_str() {
                        Some(ca_path) => Some(ca_path.to_string()),
                        None => {
                            return Err(GetTransportError::CertError(
                                "CA path is not a valid path".to_string(),
                            ))
                        }
                    }
                }
            };

            let transport = TlsTransport::new(
                ca_file.map(String::from),
                String::from(client_key_file),
                String::from(client_cert),
                String::from(server_key_file),
                String::from(server_cert),
            )?;

            Ok(Box::new(transport))
        }
        "raw" => Ok(Box::new(RawTransport::default())),
        _ => Err(GetTransportError::NotSupportedError(format!(
            "Transport type {} is not supported",
            transport_type
        ))),
    }
}
