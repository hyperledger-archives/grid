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

use std::fs::File;
use std::io;
use std::io::Read;
use toml;
use toml::de;

#[derive(Deserialize, Default, Debug)]
pub struct Config {
    storage: Option<String>,
    transport: Option<String>,
    ca_certs: Option<String>,
    client_cert: Option<String>,
    client_key: Option<String>,
    server_cert: Option<String>,
    server_key: Option<String>,
    service_endpoint: Option<String>,
    network_endpoint: Option<String>,
    peers: Option<Vec<String>>,
}

impl Config {
    pub fn from_file(mut f: File) -> Result<Config, ConfigError> {
        let mut toml = String::new();
        f.read_to_string(&mut toml)?;

        toml::from_str::<Config>(&toml).map_err(ConfigError::from)
    }

    pub fn storage(&self) -> Option<String> {
        self.storage.clone()
    }

    pub fn transport(&self) -> Option<String> {
        self.transport.clone()
    }

    pub fn ca_certs(&self) -> Option<String> {
        self.ca_certs.clone()
    }

    pub fn client_cert(&self) -> Option<String> {
        self.client_cert.clone()
    }

    pub fn client_key(&self) -> Option<String> {
        self.client_key.clone()
    }

    pub fn server_cert(&self) -> Option<String> {
        self.server_cert.clone()
    }

    pub fn server_key(&self) -> Option<String> {
        self.server_key.clone()
    }

    pub fn service_endpoint(&self) -> Option<String> {
        self.service_endpoint.clone()
    }

    pub fn network_endpoint(&self) -> Option<String> {
        self.network_endpoint.clone()
    }

    pub fn peers(&self) -> Option<Vec<String>> {
        self.peers.clone()
    }
}

#[derive(Debug)]
pub enum ConfigError {
    ReadError(io::Error),
    TomlParseError(de::Error),
}

impl From<io::Error> for ConfigError {
    fn from(e: io::Error) -> Self {
        ConfigError::ReadError(e)
    }
}

impl From<de::Error> for ConfigError {
    fn from(e: de::Error) -> Self {
        ConfigError::TomlParseError(e)
    }
}
