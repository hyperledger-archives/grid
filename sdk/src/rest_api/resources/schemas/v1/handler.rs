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

use crate::{
    rest_api::resources::{error::ErrorResponse, paging::v1::Paging},
    schema::store::{SchemaStore, SchemaStoreError},
};

use super::payloads::{SchemaListSlice, SchemaSlice};

pub fn list_schemas<'a>(
    store: Box<dyn SchemaStore + 'a>,
    service_id: Option<&str>,
    offset: u64,
    limit: u16,
) -> Result<SchemaListSlice, ErrorResponse> {
    let offset = i64::try_from(offset).unwrap_or(i64::MAX);

    let limit = i64::try_from(limit).unwrap_or(10);

    let schema_list = store
        .list_schemas(service_id, offset, limit)
        .map_err(|err| match err {
            SchemaStoreError::InternalError(err) => ErrorResponse::internal_error(Box::new(err)),
            SchemaStoreError::ConstraintViolationError(err) => {
                ErrorResponse::new(400, &format!("{}", err))
            }
            SchemaStoreError::ResourceTemporarilyUnavailableError(_) => {
                ErrorResponse::new(503, "Service Unavailable")
            }
            SchemaStoreError::NotFoundError(_) => ErrorResponse::new(404, "Resource not found"),
        })?;

    let data = schema_list
        .data
        .into_iter()
        .map(SchemaSlice::from)
        .collect();

    let paging = Paging::new("/schema", schema_list.paging, service_id);

    Ok(SchemaListSlice { data, paging })
}

pub fn get_schema<'a>(
    store: Box<dyn SchemaStore + 'a>,
    name: String,
    service_id: Option<&str>,
) -> Result<SchemaSlice, ErrorResponse> {
    let schema = store
        .get_schema(&name, service_id)
        .map_err(|err| match err {
            SchemaStoreError::InternalError(err) => ErrorResponse::internal_error(Box::new(err)),
            SchemaStoreError::ConstraintViolationError(err) => {
                ErrorResponse::new(400, &format!("{}", err))
            }
            SchemaStoreError::ResourceTemporarilyUnavailableError(_) => {
                ErrorResponse::new(503, "Service Unavailable")
            }
            SchemaStoreError::NotFoundError(_) => {
                ErrorResponse::new(404, &format!("Schema {} not found", name))
            }
        })?;

    Ok(SchemaSlice::from(schema.ok_or_else(|| {
        ErrorResponse::new(404, &format!("Schema {} not found", name))
    })?))
}
