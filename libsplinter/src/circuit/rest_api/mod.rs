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

use crate::rest_api::{Resource, RestResourceProvider};

use super::store;

#[derive(Debug)]
pub enum CircuitRouteError {
    NotFound(String),
    CircuitStoreError(store::CircuitStoreError),
    PoisonedLock,
}

impl StdError for CircuitRouteError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            CircuitRouteError::NotFound(_) => None,
            CircuitRouteError::CircuitStoreError(err) => Some(err),
            CircuitRouteError::PoisonedLock => None,
        }
    }
}

impl std::fmt::Display for CircuitRouteError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CircuitRouteError::NotFound(msg) => write!(f, "Circuit not found: {}", msg),
            CircuitRouteError::CircuitStoreError(err) => write!(f, "{}", err),
            CircuitRouteError::PoisonedLock => write!(f, "Splinter State lock was poisoned"),
        }
    }
}

impl From<store::CircuitStoreError> for CircuitRouteError {
    fn from(err: store::CircuitStoreError) -> Self {
        CircuitRouteError::CircuitStoreError(err)
    }
}

#[derive(Clone)]
pub struct CircuitResourceProvider<T: store::CircuitStore> {
    node_id: String,
    store: T,
}

impl<T: store::CircuitStore + 'static> CircuitResourceProvider<T> {
    pub fn new(node_id: String, store: T) -> Self {
        Self { node_id, store }
    }
}

impl<T: store::CircuitStore + 'static> RestResourceProvider for CircuitResourceProvider<T> {
    fn resources(&self) -> Vec<Resource> {
        // Allowing unused_mut because resources must be mutable if feature circuit-read is enabled
        #[allow(unused_mut)]
        let mut resources = Vec::new();
        #[cfg(feature = "circuit-read")]
        {
            resources.append(&mut vec![
                circuit_read::make_fetch_circuit_resource(self.store.clone()),
                circuit_read::make_list_circuits_resource(self.store.clone()),
            ])
        }
        resources
    }
}
