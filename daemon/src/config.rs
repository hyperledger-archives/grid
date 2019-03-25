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

use log::Level;

use crate::error::ConfigurationError;

pub struct GridConfig {
    validator_endpoint: String,
    log_level: Level,
}

impl GridConfig {
    pub fn validator_endpoint(&self) -> &str {
        &self.validator_endpoint
    }

    pub fn log_level(&self) -> Level {
        self.log_level
    }
}

pub struct GridConfigBuilder {
    validator_endpoint: Option<String>,
    log_level: Option<Level>,
}

impl Default for GridConfigBuilder {
    fn default() -> Self {
        Self {
            validator_endpoint: Some("localhost:4004".to_owned()),
            log_level: Some(Level::Warn),
        }
    }
}

impl GridConfigBuilder {
    pub fn with_cli_args(&mut self, matches: &clap::ArgMatches<'_>) -> Self {
        Self {
            validator_endpoint: matches
                .value_of("connect")
                .map(ToOwned::to_owned)
                .or_else(|| self.validator_endpoint.take()),
            log_level: (match matches.occurrences_of("verbose") {
                0 => Some(Level::Warn),
                1 => Some(Level::Info),
                _ => Some(Level::Debug),
            })
            .or_else(|| self.log_level.take()),
        }
    }

    pub fn build(mut self) -> Result<GridConfig, ConfigurationError> {
        Ok(GridConfig {
            validator_endpoint: self
                .validator_endpoint
                .take()
                .ok_or_else(|| ConfigurationError::MissingValue("validator_endpoint".to_owned()))?,
            log_level: self
                .log_level
                .take()
                .ok_or_else(|| ConfigurationError::MissingValue("log_level".to_owned()))?,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn build_with_args() {
        let matches = clap::App::new("testapp")
            .arg(clap::Arg::with_name("connect").short("C").takes_value(true))
            .get_matches_from(vec!["testapp", "-C", "validator:4004"]);

        let config = GridConfigBuilder::default()
            .with_cli_args(&matches)
            .build()
            .expect("Unable to build configuration");

        assert_eq!("validator:4004", config.validator_endpoint());
    }

    #[test]
    fn build_with_missing_args() {
        let matches = clap::App::new("testapp")
            .arg(clap::Arg::with_name("connect").short("C").takes_value(true))
            .get_matches_from(vec!["testapp"]);

        let config = GridConfigBuilder::default()
            .with_cli_args(&matches)
            .build()
            .expect("Unable to build configuration");

        assert_eq!("localhost:4004", config.validator_endpoint());
    }
}
