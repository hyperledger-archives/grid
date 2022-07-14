// Copyright 2018-2022 Cargill Incorporated
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

//! `PartialGriddleConfig` builder using values from griddle command line arguments.

use std::convert::TryFrom;

use crate::config::{
    error::GriddleConfigError, GriddleConfigSource, PartialGriddleConfig,
    PartialGriddleConfigBuilder, Scope,
};
use clap::ArgMatches;

/// `PartialGriddleConfig` builder using command line arguments, represented as clap `ArgMatches`.
pub struct ClapPartialGriddleConfigBuilder<'a> {
    matches: ArgMatches<'a>,
}

impl<'a> ClapPartialGriddleConfigBuilder<'a> {
    pub fn new(matches: ArgMatches<'a>) -> Self {
        ClapPartialGriddleConfigBuilder { matches }
    }
}

impl PartialGriddleConfigBuilder for ClapPartialGriddleConfigBuilder<'_> {
    fn build(self) -> Result<PartialGriddleConfig, GriddleConfigError> {
        let mut partial_config = PartialGriddleConfig::new(GriddleConfigSource::CommandLine);

        partial_config = partial_config
            .with_rest_api_endpoint(self.matches.value_of("bind").map(String::from))
            .with_signing_key(self.matches.value_of("key").map(String::from))
            .with_verbosity(match self.matches.occurrences_of("verbose") {
                0 => None,
                1 => Some(log::Level::Info),
                2 => Some(log::Level::Debug),
                _ => Some(log::Level::Trace),
            });

        let scope: Option<Scope> = self
            .matches
            .value_of("scope")
            .map(Scope::try_from)
            .transpose()?;
        partial_config = partial_config.with_scope(scope);

        #[cfg(feature = "proxy")]
        {
            partial_config = partial_config
                .with_proxy_forward_url(self.matches.value_of("forward_url").map(String::from));
        }

        Ok(partial_config)
    }
}

impl TryFrom<&str> for Scope {
    type Error = GriddleConfigError;

    fn try_from(str: &str) -> Result<Self, Self::Error> {
        match str.to_lowercase().as_str() {
            "service" => Ok(Scope::Service),
            "global" => Ok(Scope::Global),
            _ => Err(GriddleConfigError::InvalidArgument(String::from(
                "Unable to parse `scope`",
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use clap::{clap_app, crate_version};

    /// Example configuration values
    static EXAMPLE_SIGNING_KEY: &str = "test-key";
    static EXAMPLE_REST_API_ENDPOINT: &str = "127.0.0.1:8000";
    static EXAMPLE_SCOPE: &str = "service";
    #[cfg(feature = "proxy")]
    static EXAMPLE_PROXY_FORWARD_URL: &str = "http://gridd-alpha:8080";

    static EXAMPLE_SCOPE_VARIANT: Scope = Scope::Service;

    // Create an `ArgMatches` object to construct a `ClapPartialGriddleConfigBuilder`
    fn create_arg_matches(args: Vec<&str>) -> ArgMatches<'static> {
        #[cfg(not(feature = "proxy"))]
        {
            clap_app!(griddleconfigtest =>
                (version: crate_version!())
                (about: "Griddle-Config-Test")
                (@arg key: -k --key +takes_value)
                (@arg bind: -b --bind +takes_value)
                (@arg scope: -s --scope +takes_value))
            .get_matches_from(args)
        }
        #[cfg(feature = "proxy")]
        {
            clap_app!(griddleconfigtest =>
                (version: crate_version!())
                (about: "Griddle-Config-Test")
                (@arg key: -k --key +takes_value)
                (@arg bind: -b --bind +takes_value)
                (@arg scope: -s --scope +takes_value)
                (@arg forward_url: --("forward-url") +takes_value))
            .get_matches_from(args)
        }
    }

    #[test]
    /// Validate a `PartialGriddleConfig` object can be constructed using command line arguments.
    /// The test follows these steps:
    ///
    /// 1. Create an `ArgMatches` object with example configuration values
    /// 2. Use the `ArgMatches` object to construct a new `ClapPartialGriddleConfigBuilder`
    /// 3. Generate a `PartialGriddleConfig` by calling `build` on the clap builder from step 2
    /// 4. Validate the expected values in the `PartialGriddleConfig`
    ///
    fn test_setup_command_line_config() {
        let args = vec![
            "Griddle-Config-Test",
            "-k",
            EXAMPLE_SIGNING_KEY,
            "-b",
            EXAMPLE_REST_API_ENDPOINT,
            "-s",
            EXAMPLE_SCOPE,
            #[cfg(feature = "proxy")]
            "--forward-url",
            #[cfg(feature = "proxy")]
            EXAMPLE_PROXY_FORWARD_URL,
        ];
        // Create an example `ArgMatches` object to create an `ClapPartialGriddleConfigBuilder`
        let matches = create_arg_matches(args);
        let clap_config_builder = ClapPartialGriddleConfigBuilder::new(matches);
        let generated_clap_config = clap_config_builder
            .build()
            .expect("Unable to build command line config");

        assert_eq!(
            generated_clap_config.source(),
            GriddleConfigSource::CommandLine
        );
        assert_eq!(
            generated_clap_config.signing_key(),
            Some(EXAMPLE_SIGNING_KEY.to_string())
        );
        assert_eq!(
            generated_clap_config.rest_api_endpoint(),
            Some(EXAMPLE_REST_API_ENDPOINT.to_string())
        );
        assert_eq!(
            generated_clap_config.scope(),
            Some(EXAMPLE_SCOPE_VARIANT.clone())
        );
        #[cfg(feature = "proxy")]
        {
            assert_eq!(
                generated_clap_config.proxy_forward_url(),
                Some(EXAMPLE_PROXY_FORWARD_URL.to_string())
            );
        }
    }
}
