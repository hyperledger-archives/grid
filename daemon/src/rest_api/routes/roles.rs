// Copyright 2019-2021 Cargill Incorporated
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

use std::convert::TryFrom;

use crate::rest_api::{
    error::RestApiResponseError, routes::DbExecutor, AcceptServiceIdParam, AppState, QueryServiceId,
};

use actix::{Handler, Message, SyncContext};
use actix_web::{web, HttpResponse};
use grid_sdk::roles::store::Role;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RoleSlice {
    pub org_id: String,
    pub name: String,
    pub description: String,
    pub permissions: Vec<String>,
    pub allowed_organizations: Vec<String>,
    pub inherit_from: Vec<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_id: Option<String>,
}

impl TryFrom<Role> for RoleSlice {
    type Error = RestApiResponseError;

    fn try_from(role: Role) -> Result<Self, Self::Error> {
        let permissions = String::from_utf8(role.permissions)
            .map_err(|err| RestApiResponseError::DatabaseError(format!("{}", err)))?
            .split(",")
            .map(String::from)
            .collect();

        let allowed_organizations = String::from_utf8(role.allowed_orgs)
            .map_err(|err| RestApiResponseError::DatabaseError(format!("{}", err)))?
            .split(",")
            .map(String::from)
            .collect();

        let inherit_from = String::from_utf8(role.inherit_from)
            .map_err(|err| RestApiResponseError::DatabaseError(format!("{}", err)))?
            .split(",")
            .map(String::from)
            .collect();

        Ok(Self {
            org_id: role.org_id.clone(),
            name: role.name.clone(),
            description: role.description.clone(),
            permissions,
            allowed_organizations,
            inherit_from,
            service_id: role.service_id,
        })
    }
}

struct ListRoles {
    org_id: String,
    service_id: Option<String>,
}

impl Message for ListRoles {
    type Result = Result<Vec<RoleSlice>, RestApiResponseError>;
}

impl Handler<ListRoles> for DbExecutor {
    type Result = Result<Vec<RoleSlice>, RestApiResponseError>;

    fn handle(&mut self, msg: ListRoles, _: &mut SyncContext<Self>) -> Self::Result {
        self.role_store
            .list_roles_for_organization(&msg.org_id, msg.service_id.as_deref())
            .map_err(|err| RestApiResponseError::DatabaseError(format!("{}", err)))?
            .into_iter()
            .map(RoleSlice::try_from)
            .collect::<Result<Vec<RoleSlice>, RestApiResponseError>>()
    }
}

pub async fn list_roles_for_organization(
    state: web::Data<AppState>,
    org_id: web::Path<String>,
    query: web::Query<QueryServiceId>,
    _: AcceptServiceIdParam,
) -> Result<HttpResponse, RestApiResponseError> {
    state
        .database_connection
        .send(ListRoles {
            org_id: org_id.into_inner(),
            service_id: query.into_inner().service_id,
        })
        .await?
        .map(|roles| HttpResponse::Ok().json(roles))
}

struct FetchRole {
    name: String,
    service_id: Option<String>,
}

impl Message for FetchRole {
    type Result = Result<RoleSlice, RestApiResponseError>;
}

impl Handler<FetchRole> for DbExecutor {
    type Result = Result<RoleSlice, RestApiResponseError>;

    fn handle(&mut self, msg: FetchRole, _: &mut SyncContext<Self>) -> Self::Result {
        match self
            .role_store
            .fetch_role(&msg.name, msg.service_id.as_deref())
            .map_err(|err| RestApiResponseError::DatabaseError(format!("{}", err)))?
        {
            Some(role) => RoleSlice::try_from(role),
            None => Err(RestApiResponseError::NotFoundError(format!(
                "Could not find role: {}",
                msg.name,
            ))),
        }
    }
}

pub async fn fetch_role(
    state: web::Data<AppState>,
    name: web::Path<String>,
    query: web::Query<QueryServiceId>,
    _: AcceptServiceIdParam,
) -> Result<HttpResponse, RestApiResponseError> {
    state
        .database_connection
        .send(FetchRole {
            name: name.into_inner(),
            service_id: query.into_inner().service_id,
        })
        .await?
        .map(|role| HttpResponse::Ok().json(role))
}
