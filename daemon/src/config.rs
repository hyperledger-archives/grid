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
pub struct GridConfig {
    endpoint: String,
    rest_api_endpoint: String,
    database_url: String,
    #[cfg(feature = "splinter-support")]
    admin_key_dir: String,
    #[cfg(feature = "integration")]
    key_file_name: String,
}

impl GridConfig {
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    // dead_code allowed because this isn't used when splinter and sawtooth
    // features are both off, as is the case with --no-default-features
    #[allow(dead_code)]
    pub fn rest_api_endpoint(&self) -> &str {
        &self.rest_api_endpoint
    }

    // dead_code allowed because this isn't used when splinter and sawtooth
    // features are both off, as is the case with --no-default-features
    #[allow(dead_code)]
    pub fn database_url(&self) -> &str {
        &self.database_url
    }

    #[cfg(feature = "splinter-support")]
    pub fn admin_key_dir(&self) -> &str {
        &self.admin_key_dir
    }

    #[cfg(feature = "integration")]
    pub fn key_file_name(&self) -> &str {
        &self.key_file_name
    }
}

pub struct GridConfigBuilder {
    endpoint: Option<String>,
    rest_api_endpoint: Option<String>,
    database_url: Option<String>,
    #[cfg(feature = "splinter-support")]
    admin_key_dir: Option<String>,
    #[cfg(feature = "integration")]
    key_file_name: Option<String>,
}

impl Default for GridConfigBuilder {
    fn default() -> Self {
        Self {
            endpoint: Some("tcp://127.0.0.1:4004".to_owned()),
            rest_api_endpoint: Some("127.0.0.1:8080".to_owned()),
            database_url: Some("postgres://grid:grid_example@localhost/grid".to_owned()),
            #[cfg(feature = "splinter-support")]
            admin_key_dir: Some("/etc/grid/keys".to_owned()),
            #[cfg(feature = "integration")]
            key_file_name: Some("root".to_string()),
        }
    }
}

impl GridConfigBuilder {
    pub fn with_cli_args(&mut self, matches: &clap::ArgMatches<'_>) -> Self {
        Self {
            endpoint: matches
                .value_of("connect")
                .map(ToOwned::to_owned)
                .map(|s| s.to_lowercase())
                .or_else(|| self.endpoint.take()),

            rest_api_endpoint: matches
                .value_of("bind")
                .map(ToOwned::to_owned)
                .or_else(|| self.rest_api_endpoint.take()),

            database_url: matches
                .value_of("database_url")
                .map(ToOwned::to_owned)
                .or_else(|| self.database_url.take()),

            #[cfg(feature = "splinter-support")]
            admin_key_dir: matches
                .value_of("admin_key_dir")
                .map(ToOwned::to_owned)
                .or_else(|| self.admin_key_dir.take()),

            #[cfg(feature = "integration")]
            key_file_name: matches
                .value_of("key")
                .map(ToOwned::to_owned)
                .or_else(|| self.key_file_name.take()),
        }
    }

    pub fn build(mut self) -> Result<GridConfig, ConfigurationError> {
        Ok(GridConfig {
            endpoint: self
                .endpoint
                .take()
                .ok_or_else(|| ConfigurationError::MissingValue("endpoint".to_owned()))?,
            rest_api_endpoint: self
                .rest_api_endpoint
                .take()
                .ok_or_else(|| ConfigurationError::MissingValue("rest_api_endpoint".to_owned()))?,
            database_url: self
                .database_url
                .take()
                .ok_or_else(|| ConfigurationError::MissingValue("database_url".to_owned()))?,
            #[cfg(feature = "splinter-support")]
            admin_key_dir: self
                .admin_key_dir
                .take()
                .ok_or_else(|| ConfigurationError::MissingValue("admin_key_dir".to_owned()))?,
            #[cfg(feature = "integration")]
            key_file_name: self
                .key_file_name
                .take()
                .ok_or_else(|| ConfigurationError::MissingValue("key_file_name".to_owned()))?,
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
            .arg(clap::Arg::with_name("bind").short("b").takes_value(true))
            .get_matches_from(vec![
                "testapp",
                "-C",
                "validator:4004",
                "-b",
                "rest_api:8080",
            ]);

        let config = GridConfigBuilder::default()
            .with_cli_args(&matches)
            .build()
            .expect("Unable to build configuration");

        assert_eq!("validator:4004", config.endpoint());
        assert_eq!("rest_api:8080", config.rest_api_endpoint());
    }

    #[test]
    fn build_with_missing_args() {
        let matches = clap::App::new("testapp")
            .arg(clap::Arg::with_name("connect").short("C").takes_value(true))
            .arg(clap::Arg::with_name("bind").short("b").takes_value(true))
            .get_matches_from(vec!["testapp"]);

        let config = GridConfigBuilder::default()
            .with_cli_args(&matches)
            .build()
            .expect("Unable to build configuration");

        assert_eq!("tcp://127.0.0.1:4004", config.endpoint());
        assert_eq!("127.0.0.1:8080", config.rest_api_endpoint());
    }
}
