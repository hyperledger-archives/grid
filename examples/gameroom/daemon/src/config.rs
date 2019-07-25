/*
 * Copyright 2019 Cargill Incorporated
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * -----------------------------------------------------------------------------
 */

use crate::error::ConfigurationError;

#[derive(Debug)]
pub struct GameroomConfig {
    rest_api_endpoint: String,
    database_url: String,
}

impl GameroomConfig {
    pub fn rest_api_endpoint(&self) -> &str {
        &self.rest_api_endpoint
    }
    pub fn database_url(&self) -> &str {
        &self.database_url
    }
}

pub struct GameroomConfigBuilder {
    rest_api_endpoint: Option<String>,
    database_url: Option<String>,
}

impl Default for GameroomConfigBuilder {
    fn default() -> Self {
        Self {
            rest_api_endpoint: Some("127.0.0.1:8000".to_owned()),
            database_url: Some(
                "postgres://gameroom:gameroom_example@postgres:5432/gameroom".to_owned(),
            ),
        }
    }
}

impl GameroomConfigBuilder {
    pub fn with_cli_args(&mut self, matches: &clap::ArgMatches<'_>) -> Self {
        Self {
            rest_api_endpoint: matches
                .value_of("bind")
                .map(ToOwned::to_owned)
                .or_else(|| self.rest_api_endpoint.take()),

            database_url: matches
                .value_of("database_url")
                .map(ToOwned::to_owned)
                .or_else(|| self.database_url.take()),
        }
    }

    pub fn build(mut self) -> Result<GameroomConfig, ConfigurationError> {
        Ok(GameroomConfig {
            rest_api_endpoint: self
                .rest_api_endpoint
                .take()
                .ok_or_else(|| ConfigurationError::MissingValue("rest_api_endpoint".to_owned()))?,
            database_url: self
                .database_url
                .take()
                .ok_or_else(|| ConfigurationError::MissingValue("database_url".to_owned()))?,
        })
    }
}
