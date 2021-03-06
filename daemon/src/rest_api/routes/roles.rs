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
    error::RestApiResponseError, routes::DbExecutor, AcceptServiceIdParam, AppState, QueryPaging,
    QueryServiceId,
};

use actix::{Handler, Message, SyncContext};
use actix_web::{web, HttpResponse};
use grid_sdk::pike::store::Role;
use grid_sdk::rest_api::resources::paging::v1::Paging;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RoleSlice {
    pub org_id: String,
    pub name: String,
    pub description: String,
    pub active: bool,
    pub permissions: Vec<String>,
    pub allowed_organizations: Vec<String>,
    pub inherit_from: Vec<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoleListSlice {
    pub data: Vec<RoleSlice>,
    pub paging: Paging,
}

impl TryFrom<Role> for RoleSlice {
    type Error = RestApiResponseError;

    fn try_from(role: Role) -> Result<Self, Self::Error> {
        let permissions = role.permissions.into_iter().map(String::from).collect();

        let allowed_organizations = role
            .allowed_organizations
            .into_iter()
            .map(String::from)
            .collect();

        let inherit_from = role.inherit_from.into_iter().map(String::from).collect();

        Ok(Self {
            org_id: role.org_id.clone(),
            name: role.name.clone(),
            description: role.description.clone(),
            active: role.active,
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
    offset: u64,
    limit: u16,
}

impl Message for ListRoles {
    type Result = Result<RoleListSlice, RestApiResponseError>;
}

impl Handler<ListRoles> for DbExecutor {
    type Result = Result<RoleListSlice, RestApiResponseError>;

    fn handle(&mut self, msg: ListRoles, _: &mut SyncContext<Self>) -> Self::Result {
        let offset = i64::try_from(msg.offset).unwrap_or(i64::MAX);

        let limit = i64::try_from(msg.limit).unwrap_or(10);

        let roles_list = self.pike_store.list_roles_for_organization(
            &msg.org_id,
            msg.service_id.as_deref(),
            offset,
            limit,
        )?;

        let data = roles_list
            .data
            .into_iter()
            .map(RoleSlice::try_from)
            .collect::<Result<Vec<RoleSlice>, RestApiResponseError>>()?;

        let paging = Paging::new(
            &format!("/role/{}", &msg.org_id),
            roles_list.paging,
            msg.service_id.as_deref(),
        );

        Ok(RoleListSlice { data, paging })
    }
}

pub async fn list_roles_for_organization(
    state: web::Data<AppState>,
    org_id: web::Path<String>,
    query: web::Query<QueryServiceId>,
    query_paging: web::Query<QueryPaging>,
    _: AcceptServiceIdParam,
) -> Result<HttpResponse, RestApiResponseError> {
    let paging = query_paging.into_inner();
    state
        .database_connection
        .send(ListRoles {
            org_id: org_id.into_inner(),
            service_id: query.into_inner().service_id,
            offset: paging.offset.unwrap_or(0),
            limit: paging.limit.unwrap_or(10),
        })
        .await?
        .map(|roles| HttpResponse::Ok().json(roles))
}

struct FetchRole {
    name: String,
    org_id: String,
    service_id: Option<String>,
}

impl Message for FetchRole {
    type Result = Result<RoleSlice, RestApiResponseError>;
}

impl Handler<FetchRole> for DbExecutor {
    type Result = Result<RoleSlice, RestApiResponseError>;

    fn handle(&mut self, msg: FetchRole, _: &mut SyncContext<Self>) -> Self::Result {
        match self
            .pike_store
            .fetch_role(&msg.name, &msg.org_id, msg.service_id.as_deref())
            .map_err(|err| RestApiResponseError::DatabaseError(format!("{}", err)))?
        {
            Some(role) => RoleSlice::try_from(role),
            None => Err(RestApiResponseError::NotFoundError(format!(
                "Could not find role: {} for organization: {}",
                msg.name, msg.org_id,
            ))),
        }
    }
}

pub async fn fetch_role(
    state: web::Data<AppState>,
    params: web::Path<(String, String)>,
    query: web::Query<QueryServiceId>,
    _: AcceptServiceIdParam,
) -> Result<HttpResponse, RestApiResponseError> {
    let (org_id, name) = params.into_inner();
    state
        .database_connection
        .send(FetchRole {
            name,
            org_id,
            service_id: query.into_inner().service_id,
        })
        .await?
        .map(|role| HttpResponse::Ok().json(role))
}
