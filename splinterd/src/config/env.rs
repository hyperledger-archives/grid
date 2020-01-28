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

use std::env;

use crate::config::{PartialConfig, PartialConfigBuilder};

const STATE_DIR_ENV: &str = "SPLINTER_STATE_DIR";
const CERT_DIR_ENV: &str = "SPLINTER_CERT_DIR";

pub struct EnvVarConfig {
    state_dir: Option<String>,
    cert_dir: Option<String>,
}

impl EnvVarConfig {
    #[allow(dead_code)]
    pub fn new() -> Self {
        EnvVarConfig {
            state_dir: env::var(STATE_DIR_ENV).ok(),
            cert_dir: env::var(CERT_DIR_ENV).ok(),
        }
    }
}

impl PartialConfigBuilder for EnvVarConfig {
    fn build(self) -> PartialConfig {
        PartialConfig::default()
            .with_cert_dir(self.cert_dir)
            .with_state_dir(self.state_dir)
    }
}
