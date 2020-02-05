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

mod builder;
#[cfg(feature = "config-command-line")]
mod command_line;
#[cfg(feature = "config-default")]
mod default;
#[cfg(feature = "config-env-var")]
mod env;
mod error;
mod partial;
mod toml;

#[cfg(feature = "config-command-line")]
pub use crate::config::command_line::CommandLineConfig;
#[cfg(feature = "config-default")]
pub use crate::config::default::DefaultConfig;
#[cfg(feature = "config-env-var")]
pub use crate::config::env::EnvVarConfig;
#[cfg(not(feature = "config-toml"))]
pub use crate::config::toml::from_file;
#[cfg(feature = "config-toml")]
pub use crate::config::toml::TomlConfig;
pub use builder::PartialConfigBuilder;
pub use error::ConfigError;
pub use partial::PartialConfig;
