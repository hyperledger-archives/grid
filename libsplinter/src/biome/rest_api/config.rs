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

use std::time::Duration;

use super::error::BiomeRestConfigBuilderError;

const DEFAULT_ISSUER: &str = "self-issued";
const DEFAULT_DURATION: u64 = 5400; // in seconds = 90 minutes

/// Configuration for Biome REST resources
#[derive(Deserialize, Debug)]
pub struct BiomeRestConfig {
    /// The issuer for JWT tokens issued by this service
    issuer: String,
    /// Duration of JWT tokens issued by this service
    access_token_duration: Duration,
}

impl BiomeRestConfig {
    pub fn issuer(&self) -> String {
        self.issuer.to_owned()
    }

    pub fn access_token_duration(&self) -> Duration {
        self.access_token_duration.to_owned()
    }
}

/// Builder for BiomeRestConfig
pub struct BiomeRestConfigBuilder {
    issuer: Option<String>,
    access_token_duration: Option<Duration>,
}

impl Default for BiomeRestConfigBuilder {
    fn default() -> BiomeRestConfigBuilder {
        BiomeRestConfigBuilder {
            issuer: Some(DEFAULT_ISSUER.to_string()),
            access_token_duration: Some(Duration::from_secs(DEFAULT_DURATION)),
        }
    }
}

impl BiomeRestConfigBuilder {
    pub fn new() -> Self {
        BiomeRestConfigBuilder {
            issuer: None,
            access_token_duration: None,
        }
    }

    pub fn with_issuer(mut self, issuer: &str) -> Self {
        self.issuer = Some(issuer.to_string());
        self
    }

    pub fn with_access_token_duration_in_secs(mut self, duration: u64) -> Self {
        self.access_token_duration = Some(Duration::from_secs(duration));
        self
    }

    pub fn build(self) -> Result<BiomeRestConfig, BiomeRestConfigBuilderError> {
        if self.issuer.is_none() {
            debug!("Using default value for issuer");
        }
        let issuer = self.issuer.unwrap_or_default();

        if self.access_token_duration.is_none() {
            debug!("Using default value for access_token_duration");
        }
        let access_token_duration = self.access_token_duration.unwrap_or_default();

        Ok(BiomeRestConfig {
            issuer,
            access_token_duration,
        })
    }
}
