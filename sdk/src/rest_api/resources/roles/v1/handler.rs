// Copyright 2018-2021 Cargill Incorporated
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
use std::sync::Arc;

use crate::{
    pike::store::{PikeStore, PikeStoreError},
    rest_api::resources::{error::ErrorResponse, paging::v1::Paging},
};

use super::payloads::{RoleListSlice, RoleSlice};

pub async fn list_roles_for_organization(
    store: Arc<dyn PikeStore>,
    org_id: String,
    service_id: Option<&str>,
    offset: u64,
    limit: u16,
) -> Result<RoleListSlice, ErrorResponse> {
    let offset = i64::try_from(offset).unwrap_or(i64::MAX);

    let limit = i64::try_from(limit).unwrap_or(10);

    let role_list = store
        .list_roles_for_organization(&org_id, service_id, offset, limit)
        .map_err(|err| match err {
            PikeStoreError::InternalError(err) => ErrorResponse::internal_error(Box::new(err)),
            PikeStoreError::ConstraintViolationError(err) => {
                ErrorResponse::new(400, &format!("{}", err))
            }
            PikeStoreError::ResourceTemporarilyUnavailableError(_) => {
                ErrorResponse::new(503, "Service Unavailable")
            }
            PikeStoreError::NotFoundError(_) => ErrorResponse::new(404, "Resource not found"),
        })?;

    let data = role_list.data.into_iter().map(RoleSlice::from).collect();

    let paging = Paging::new("/role", role_list.paging, service_id);

    Ok(RoleListSlice { data, paging })
}

pub async fn fetch_role(
    store: Arc<dyn PikeStore>,
    org_id: String,
    name: String,
    service_id: Option<&str>,
) -> Result<RoleSlice, ErrorResponse> {
    let role = store
        .fetch_role(&name, &org_id, service_id)
        .map_err(|err| match err {
            PikeStoreError::InternalError(err) => ErrorResponse::internal_error(Box::new(err)),
            PikeStoreError::ConstraintViolationError(err) => {
                ErrorResponse::new(400, &format!("{}", err))
            }
            PikeStoreError::ResourceTemporarilyUnavailableError(_) => {
                ErrorResponse::new(503, "Service Unavailable")
            }
            PikeStoreError::NotFoundError(_) => {
                ErrorResponse::new(404, &format!("Role {} for {} not found", name, org_id))
            }
        })?;

    Ok(RoleSlice::from(role.ok_or_else(|| {
        ErrorResponse::new(404, &format!("Role {} for {} not found", name, org_id))
    })?))
}
