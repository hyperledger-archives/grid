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

//! Configuration to provide the necessary values to start up the Griddle daemon.
//!
//! These values may be sourced from command line arguments, environment variables or pre-defined
//! defaults. This module allows for configuration values from each of these sources to be combined
//! into a final `GriddleConfig` object.

#[derive(Clone, Debug, PartialEq)]
/// Placeholder for indicating the scope of the requests, will be used to determine if requests
/// to Griddle should include a scope ID and what format ID to expect
pub enum Scope {
    Global,
    Service,
}
