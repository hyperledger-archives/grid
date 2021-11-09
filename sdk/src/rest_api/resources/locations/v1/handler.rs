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
use url::Url;

use crate::{
    location::store::{LocationStore, LocationStoreError},
    rest_api::resources::{error::ErrorResponse, paging::v1::Paging},
};

use super::payloads::{LocationListSlice, LocationSlice};

pub fn list_locations<'a>(
    url: Url,
    store: Box<dyn LocationStore + 'a>,
    service_id: Option<&str>,
    offset: u64,
    limit: u16,
) -> Result<LocationListSlice, ErrorResponse> {
    let offset = i64::try_from(offset).unwrap_or(i64::MAX);

    let limit = i64::try_from(limit).unwrap_or(10);

    let location_list =
        store
            .list_locations(service_id, offset, limit)
            .map_err(|err| match err {
                LocationStoreError::InternalError(err) => {
                    ErrorResponse::internal_error(Box::new(err))
                }
                LocationStoreError::ConstraintViolationError(err) => {
                    ErrorResponse::new(400, &format!("{}", err))
                }
                LocationStoreError::ResourceTemporarilyUnavailableError(_) => {
                    ErrorResponse::new(503, "Service Unavailable")
                }
                LocationStoreError::NotFoundError(_) => {
                    ErrorResponse::new(404, "Resource not found")
                }
            })?;

    let data = location_list
        .data
        .into_iter()
        .map(LocationSlice::from)
        .collect();

    let paging = Paging::new(url, location_list.paging, service_id);

    Ok(LocationListSlice { data, paging })
}

pub fn get_location<'a>(
    store: Box<dyn LocationStore + 'a>,
    location_id: String,
    service_id: Option<&str>,
) -> Result<LocationSlice, ErrorResponse> {
    let location = store
        .get_location(&location_id, service_id)
        .map_err(|err| match err {
            LocationStoreError::InternalError(err) => ErrorResponse::internal_error(Box::new(err)),
            LocationStoreError::ConstraintViolationError(err) => {
                ErrorResponse::new(400, &format!("{}", err))
            }
            LocationStoreError::ResourceTemporarilyUnavailableError(_) => {
                ErrorResponse::new(503, "Service Unavailable")
            }
            LocationStoreError::NotFoundError(_) => {
                ErrorResponse::new(404, &format!("Location {} not found", location_id))
            }
        })?;

    Ok(LocationSlice::from(location.ok_or_else(|| {
        ErrorResponse::new(404, &format!("Location {} not found", location_id))
    })?))
}
