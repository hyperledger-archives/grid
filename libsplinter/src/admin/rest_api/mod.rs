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

//! This module defines the REST API endpoints for interacting with the Splinter admin service.

#[cfg(feature = "rest-api-actix")]
mod actix;
#[cfg(any(feature = "circuit-read", feature = "proposal-read"))]
mod error;
mod resources;

use crate::admin::service::AdminService;
use crate::circuit::store;
use crate::rest_api::{Resource, RestResourceProvider};

#[cfg(all(feature = "circuit-read", feature = "rest-api-actix"))]
use self::actix::circuits::make_list_circuits_resource;
#[cfg(all(feature = "circuit-read", feature = "rest-api-actix"))]
use self::actix::circuits_circuit_id::make_fetch_circuit_resource;
#[cfg(all(feature = "proposal-read", feature = "rest-api-actix"))]
use self::actix::proposals::make_list_proposals_resource;
#[cfg(all(feature = "proposal-read", feature = "rest-api-actix"))]
use self::actix::proposals_circuit_id::make_fetch_proposal_resource;
#[cfg(feature = "rest-api-actix")]
use self::actix::submit::make_submit_route;
#[cfg(feature = "rest-api-actix")]
use self::actix::ws_register_type::make_application_handler_registration_route;

impl RestResourceProvider for AdminService {
    fn resources(&self) -> Vec<Resource> {
        let mut resources = vec![];

        #[cfg(feature = "rest-api-actix")]
        resources.push(make_application_handler_registration_route(self.commands()));
        #[cfg(feature = "rest-api-actix")]
        resources.push(make_submit_route(self.commands()));

        #[cfg(all(feature = "proposal-read", feature = "rest-api-actix"))]
        resources.push(make_fetch_proposal_resource(self.proposals()));
        #[cfg(all(feature = "proposal-read", feature = "rest-api-actix"))]
        resources.push(make_list_proposals_resource(self.proposals()));

        resources
    }
}

/// Provides the REST API [Resource](splinter::rest_api::Resource) definitions for
/// listing and fetching the circuits in the splinter node's state.
///
/// The following endpoints are provided:
///
/// * `GET /admin/circuits` - List circuits in Splinter's state
/// * `GET /admin/circuits/{circuit_id}` - Fetch a specific circuit in Splinter's state by circuit
///   ID
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
        #[cfg(all(feature = "circuit-read", feature = "rest-api-actix"))]
        {
            resources.append(&mut vec![
                make_fetch_circuit_resource(self.store.clone()),
                make_list_circuits_resource(self.store.clone()),
            ])
        }
        resources
    }
}
