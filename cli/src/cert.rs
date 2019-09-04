// Copyright 2019 Cargill Incorporated
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

use std::env;

use splinter_client::Certs;

pub fn get_certs() -> Certs {
    let ca_certs = if let Ok(s) = env::var("SPLINTER_CA_CERTS") {
        s.to_string()
    } else {
        "ca.crt".to_string()
    };

    let client_cert = if let Ok(s) = env::var("SPLINTER_CLIENT_CERTS") {
        s.to_string()
    } else {
        "client.crt".to_string()
    };

    let client_priv = if let Ok(s) = env::var("SPLINTER_CLIENT_SECRET") {
        s.to_string()
    } else {
        "client.key".to_string()
    };

    Certs::new(ca_certs, client_cert, client_priv)
}
