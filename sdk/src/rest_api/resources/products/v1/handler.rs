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
    products::store::{ProductStore, ProductStoreError},
    rest_api::resources::{error::ErrorResponse, paging::v1::Paging},
};

use super::payloads::{ProductListSlice, ProductSlice};

pub async fn list_products(
    store: Arc<dyn ProductStore>,
    service_id: Option<&str>,
    offset: u64,
    limit: u16,
) -> Result<ProductListSlice, ErrorResponse> {
    let offset = i64::try_from(offset).unwrap_or(i64::MAX);

    let limit = i64::try_from(limit).unwrap_or(10);

    let product_list = store
        .list_products(service_id, offset, limit)
        .map_err(|err| match err {
            ProductStoreError::InternalError(err) => ErrorResponse::internal_error(Box::new(err)),
            ProductStoreError::ConstraintViolationError(err) => {
                ErrorResponse::new(400, &format!("{}", err))
            }
            ProductStoreError::ResourceTemporarilyUnavailableError(_) => {
                ErrorResponse::new(503, "Service Unavailable")
            }
            ProductStoreError::NotFoundError(_) => ErrorResponse::new(404, "Resource not found"),
        })?;

    let data = product_list
        .data
        .into_iter()
        .map(ProductSlice::from)
        .collect();

    let paging = Paging::new("/product", product_list.paging, service_id);

    Ok(ProductListSlice { data, paging })
}

pub async fn fetch_product(
    store: Arc<dyn ProductStore>,
    product_id: String,
    service_id: Option<&str>,
) -> Result<ProductSlice, ErrorResponse> {
    let product = store
        .fetch_product(&product_id, service_id)
        .map_err(|err| match err {
            ProductStoreError::InternalError(err) => ErrorResponse::internal_error(Box::new(err)),
            ProductStoreError::ConstraintViolationError(err) => {
                ErrorResponse::new(400, &format!("{}", err))
            }
            ProductStoreError::ResourceTemporarilyUnavailableError(_) => {
                ErrorResponse::new(503, "Service Unavailable")
            }
            ProductStoreError::NotFoundError(_) => {
                ErrorResponse::new(404, &format!("Product {} not found", product_id))
            }
        })?;

    Ok(ProductSlice::from(product.ok_or_else(|| {
        ErrorResponse::new(404, &format!("Product {} not found", product_id))
    })?))
}
