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

#[cfg(feature = "circuit-read")]
mod circuit_read;

use std::error::Error as StdError;
use std::sync::{Arc, RwLock};

use crate::rest_api::{Resource, RestResourceProvider};

use super::SplinterState;

#[derive(Debug)]
pub enum CircuitRouteError {
    NotFound(String),
    PoisonedLock,
}

impl StdError for CircuitRouteError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            CircuitRouteError::NotFound(_) => None,
            CircuitRouteError::PoisonedLock => None,
        }
    }
}

impl std::fmt::Display for CircuitRouteError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CircuitRouteError::NotFound(msg) => write!(f, "Circuit not found: {}", msg),
            CircuitRouteError::PoisonedLock => write!(f, "Splinter State lock was poisoned"),
        }
    }
}

#[derive(Clone)]
pub struct CircuitResourceProvider {
    node_id: String,
    state: Arc<RwLock<SplinterState>>,
}

impl CircuitResourceProvider {
    pub fn new(node_id: String, state: Arc<RwLock<SplinterState>>) -> Self {
        Self { node_id, state }
    }
}

impl RestResourceProvider for CircuitResourceProvider {
    fn resources(&self) -> Vec<Resource> {
        // Allowing unused_mut because resources must be mutable if feature circuit-read is enabled
        #[allow(unused_mut)]
        let mut resources = Vec::new();
        #[cfg(feature = "circuit-read")]
        {
            resources.append(&mut vec![
                circuit_read::make_fetch_circuit_resource(self.state.clone()),
                circuit_read::make_list_circuits_resource(self.state.clone()),
            ])
        }
        resources
    }
}
