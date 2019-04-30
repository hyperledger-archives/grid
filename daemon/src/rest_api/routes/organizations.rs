// Copyright 2019 Cargill Incorporated
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

use crate::database::{helpers as db, models::Organization};
use crate::rest_api::{error::RestApiResponseError, routes::DbExecutor, AppState};

use actix::{Handler, Message, SyncContext};
use actix_web::{HttpRequest, HttpResponse, Path};
use futures::Future;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Debug, Serialize, Deserialize)]
pub struct OrganizationSlice {
    pub org_id: String,
    pub name: String,
    pub address: String,
    pub metadata: Vec<JsonValue>,
}

impl OrganizationSlice {
    pub fn from_organization(organization: &Organization) -> Self {
        Self {
            org_id: organization.org_id.clone(),
            name: organization.name.clone(),
            address: organization.address.clone(),
            metadata: organization.metadata.clone(),
        }
    }
}

struct ListOrganizations;

impl Message for ListOrganizations {
    type Result = Result<Vec<OrganizationSlice>, RestApiResponseError>;
}

impl Handler<ListOrganizations> for DbExecutor {
    type Result = Result<Vec<OrganizationSlice>, RestApiResponseError>;

    fn handle(&mut self, _msg: ListOrganizations, _: &mut SyncContext<Self>) -> Self::Result {
        let fetched_organizations = db::list_organizations(&*self.connection_pool.get()?)?
            .iter()
            .map(|organization| OrganizationSlice::from_organization(organization))
            .collect();
        Ok(fetched_organizations)
    }
}

pub fn list_organizations(
    req: HttpRequest<AppState>,
) -> impl Future<Item = HttpResponse, Error = RestApiResponseError> {
    req.state()
        .database_connection
        .send(ListOrganizations)
        .from_err()
        .and_then(move |res| match res {
            Ok(organizations) => Ok(HttpResponse::Ok().json(organizations)),
            Err(err) => Err(err),
        })
}
