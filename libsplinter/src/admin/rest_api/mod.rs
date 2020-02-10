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

#[cfg(feature = "rest-api-actix")]
mod actix;
#[cfg(feature = "proposal-read")]
mod error;
mod resources;

use crate::admin::service::AdminService;
use crate::rest_api::{Resource, RestResourceProvider};

#[cfg(all(feature = "proposal-read", feature = "rest-api-actix"))]
use self::actix::proposals_read::make_list_proposals_resource;
#[cfg(all(feature = "proposal-read", feature = "rest-api-actix"))]
use self::actix::proposals_read_circuit_id::make_fetch_proposal_resource;
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
        resources.push(make_fetch_proposal_resource(self.commands()));
        #[cfg(all(feature = "proposal-read", feature = "rest-api-actix"))]
        resources.push(make_list_proposals_resource(self.commands()));

        resources
    }
}
