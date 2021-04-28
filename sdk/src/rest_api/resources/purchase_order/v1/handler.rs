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
    purchase_order::store::{PurchaseOrderStore, PurchaseOrderStoreError},
    rest_api::resources::{error::ErrorResponse, paging::v1::Paging},
};

use super::payloads::{PurchaseOrderListSlice, PurchaseOrderSlice};

pub async fn list_purchase_orders(
    store: Arc<dyn PurchaseOrderStore>,
    org_id: Option<String>,
    service_id: Option<&str>,
    offset: u64,
    limit: u16,
) -> Result<PurchaseOrderListSlice, ErrorResponse> {
    let offset = i64::try_from(offset).unwrap_or(i64::MAX);

    let limit = i64::try_from(limit).unwrap_or(10);

    let purchase_order_list = store
        .list_purchase_orders(org_id, service_id, offset, limit)
        .map_err(|err| match err {
            PurchaseOrderStoreError::InternalError(err) => {
                ErrorResponse::internal_error(Box::new(err))
            }
            PurchaseOrderStoreError::ConstraintViolationError(err) => {
                ErrorResponse::new(400, &format!("{}", err))
            }
            PurchaseOrderStoreError::ResourceTemporarilyUnavailableError(_) => {
                ErrorResponse::new(503, "Service Unavailable")
            }
            PurchaseOrderStoreError::NotFoundError(_) => {
                ErrorResponse::new(404, "Resource not found")
            }
        })?;

    let data = purchase_order_list
        .data
        .into_iter()
        .map(PurchaseOrderSlice::from)
        .collect();

    let paging = Paging::new("/purchase-order", purchase_order_list.paging, service_id);

    Ok(PurchaseOrderListSlice { data, paging })
}

pub async fn get_purchase_order(
    store: Arc<dyn PurchaseOrderStore>,
    uuid: String,
    service_id: Option<&str>,
) -> Result<PurchaseOrderSlice, ErrorResponse> {
    let purchase_order = store
        .get_purchase_order(&uuid, service_id)
        .map_err(|err| match err {
            PurchaseOrderStoreError::InternalError(err) => {
                ErrorResponse::internal_error(Box::new(err))
            }
            PurchaseOrderStoreError::ConstraintViolationError(err) => {
                ErrorResponse::new(400, &format!("{}", err))
            }
            PurchaseOrderStoreError::ResourceTemporarilyUnavailableError(_) => {
                ErrorResponse::new(503, "Service Unavailable")
            }
            PurchaseOrderStoreError::NotFoundError(_) => {
                ErrorResponse::new(404, &format!("Purchase order {} not found", uuid))
            }
        })?;

    Ok(PurchaseOrderSlice::from(purchase_order.ok_or_else(
        || ErrorResponse::new(404, &format!("PurchaseOrder {} not found", uuid)),
    )?))
}
