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

use std::{convert::TryFrom, str::FromStr};

use crate::rest_api::{
    error::RestApiResponseError, routes::DbExecutor, AcceptServiceIdParam, AppState, QueryServiceId,
};

use actix::{Handler, Message, SyncContext};
use actix_web::{web, HttpResponse};
use grid_sdk::organizations::store::Organization;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Debug, Serialize, Deserialize)]
pub struct OrganizationSlice {
    pub org_id: String,
    pub name: String,
    pub address: String,
    pub metadata: JsonValue,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_id: Option<String>,
}

impl TryFrom<Organization> for OrganizationSlice {
    type Error = RestApiResponseError;

    fn try_from(organization: Organization) -> Result<Self, Self::Error> {
        let metadata = if !organization.metadata.is_empty() {
            JsonValue::from_str(
                &String::from_utf8(organization.metadata.clone())
                    .map_err(|err| RestApiResponseError::DatabaseError(format!("{}", err)))?,
            )
            .map_err(|err| RestApiResponseError::DatabaseError(format!("{}", err)))?
        } else {
            json!([])
        };

        Ok(Self {
            org_id: organization.org_id.clone(),
            name: organization.name.clone(),
            address: organization.address.clone(),
            metadata,
            service_id: organization.service_id,
        })
    }
}

struct ListOrganizations {
    service_id: Option<String>,
}

impl Message for ListOrganizations {
    type Result = Result<Vec<OrganizationSlice>, RestApiResponseError>;
}

impl Handler<ListOrganizations> for DbExecutor {
    type Result = Result<Vec<OrganizationSlice>, RestApiResponseError>;

    fn handle(&mut self, msg: ListOrganizations, _: &mut SyncContext<Self>) -> Self::Result {
        self.organization_store
            .list_organizations(msg.service_id.as_deref())?
            .into_iter()
            .map(OrganizationSlice::try_from)
            .collect::<Result<Vec<OrganizationSlice>, RestApiResponseError>>()
    }
}

pub async fn list_organizations(
    state: web::Data<AppState>,
    query: web::Query<QueryServiceId>,
    _: AcceptServiceIdParam,
) -> Result<HttpResponse, RestApiResponseError> {
    state
        .database_connection
        .send(ListOrganizations {
            service_id: query.into_inner().service_id,
        })
        .await?
        .map(|organizations| HttpResponse::Ok().json(organizations))
}

struct FetchOrganization {
    organization_id: String,
    service_id: Option<String>,
}

impl Message for FetchOrganization {
    type Result = Result<OrganizationSlice, RestApiResponseError>;
}

impl Handler<FetchOrganization> for DbExecutor {
    type Result = Result<OrganizationSlice, RestApiResponseError>;

    fn handle(&mut self, msg: FetchOrganization, _: &mut SyncContext<Self>) -> Self::Result {
        match self
            .organization_store
            .fetch_organization(&msg.organization_id, msg.service_id.as_deref())?
        {
            Some(organization) => OrganizationSlice::try_from(organization),
            None => Err(RestApiResponseError::NotFoundError(format!(
                "Could not find organization with id: {}",
                msg.organization_id
            ))),
        }
    }
}

pub async fn fetch_organization(
    state: web::Data<AppState>,
    organization_id: web::Path<String>,
    query: web::Query<QueryServiceId>,
    _: AcceptServiceIdParam,
) -> Result<HttpResponse, RestApiResponseError> {
    state
        .database_connection
        .send(FetchOrganization {
            organization_id: organization_id.into_inner(),
            service_id: query.into_inner().service_id,
        })
        .await?
        .map(|organization| HttpResponse::Ok().json(organization))
}
