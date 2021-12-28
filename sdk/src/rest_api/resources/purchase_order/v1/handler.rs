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
    purchase_order::store::{
        ListPOFilters, ListVersionFilters, PurchaseOrderStore, PurchaseOrderStoreError,
    },
    rest_api::resources::{error::ErrorResponse, paging::v1::Paging},
};

use super::payloads::{
    PurchaseOrderListSlice, PurchaseOrderRevisionListSlice, PurchaseOrderRevisionSlice,
    PurchaseOrderSlice, PurchaseOrderVersionListSlice, PurchaseOrderVersionSlice,
};

/// Makes a request to return the list of purchase orders from the store
///
/// # Arguments
///
///  * `url` - The base url for the request
///  * `store` - The store to fetch the orders from
///  * `filters` - The list of filters to filter the results
///  * `service_id` - optional - The service ID when running on Splinter
///  * `offset` - The index of the first purchase order in storage to retrieve
///  * `limit` - The number of items to retrieve from the offset
pub fn list_purchase_orders<'a>(
    url: Url,
    store: Box<dyn PurchaseOrderStore + 'a>,
    filters: ListPOFilters,
    service_id: Option<&str>,
    offset: u64,
    limit: u16,
) -> Result<PurchaseOrderListSlice, ErrorResponse> {
    let offset = i64::try_from(offset).unwrap_or(i64::MAX);

    let limit = i64::try_from(limit).unwrap_or(10);

    let purchase_order_list = store
        .list_purchase_orders(filters, service_id, offset, limit)
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

    let paging = Paging::new(url, purchase_order_list.paging, service_id);

    Ok(PurchaseOrderListSlice { data, paging })
}

/// Makes a request to return a purchase order
///
/// # Arguments
///
///  * `store` - The store to fetch the order from
///  * `purchase_order_uid` - The UID of the purchase order
///  * `version_id` - optional - filter for version
///  * `revision_number` - optional - filter for version revision
///  * `service_id` - optional - The service ID when running on Splinter
pub fn get_purchase_order<'a>(
    store: Box<dyn PurchaseOrderStore + 'a>,
    purchase_order_uid: String,
    version_id: Option<&str>,
    revision_number: Option<i64>,
    service_id: Option<&str>,
) -> Result<PurchaseOrderSlice, ErrorResponse> {
    let purchase_order = store
        .get_purchase_order(&purchase_order_uid, version_id, revision_number, service_id)
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
            PurchaseOrderStoreError::NotFoundError(_) => ErrorResponse::new(
                404,
                &format!("Purchase order {} not found", purchase_order_uid),
            ),
        })?;

    Ok(PurchaseOrderSlice::from(purchase_order.ok_or_else(
        || {
            ErrorResponse::new(
                404,
                &format!("Purchase order {} not found", purchase_order_uid),
            )
        },
    )?))
}

/// Makes a request to return a purchase order version
///
/// # Arguments
///
///  * `store` - The store to fetch the order from
///  * `purchase_order_uid` - The UID of the purchase order
///  * `version_id` - ID for the version
///  * `service_id` - optional - The service ID when running on Splinter
pub fn get_purchase_order_version<'a>(
    store: Box<dyn PurchaseOrderStore + 'a>,
    purchase_order_uid: String,
    version_id: &str,
    service_id: Option<&str>,
) -> Result<PurchaseOrderVersionSlice, ErrorResponse> {
    let purchase_order_version = store
        .get_purchase_order_version(&purchase_order_uid, version_id, service_id)
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
            PurchaseOrderStoreError::NotFoundError(_) => ErrorResponse::new(
                404,
                &format!("Purchase order {} not found", purchase_order_uid),
            ),
        })?;

    Ok(PurchaseOrderVersionSlice::from(
        purchase_order_version.ok_or_else(|| {
            ErrorResponse::new(
                404,
                &format!("Purchase order {} not found", purchase_order_uid),
            )
        })?,
    ))
}

/// Makes a request to return the versions for a purchase order
///
/// # Arguments
///
///  * `url` - The base url for the request
///  * `store` - The store to fetch the order from
///  * `purchase_order_uid` - The UID of the purchase order
///  * `filters` - The list of filters to filter the results
///  * `service_id` - optional - The service ID when running on Splinter
///  * `offset` - The index of the first version in storage to retrieve
///  * `limit` - The number of items to retrieve from the offset
pub fn list_purchase_order_versions<'a>(
    url: Url,
    store: Box<dyn PurchaseOrderStore + 'a>,
    purchase_order_uid: String,
    filters: ListVersionFilters,
    service_id: Option<&str>,
    offset: u64,
    limit: u16,
) -> Result<PurchaseOrderVersionListSlice, ErrorResponse> {
    let offset = i64::try_from(offset).unwrap_or(i64::MAX);

    let limit = i64::try_from(limit).unwrap_or(10);

    let purchase_order_version_list = store
        .list_purchase_order_versions(&purchase_order_uid, filters, service_id, offset, limit)
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
            PurchaseOrderStoreError::NotFoundError(_) => ErrorResponse::new(
                404,
                &format!("Purchase order {} not found", purchase_order_uid),
            ),
        })?;

    let data = purchase_order_version_list
        .data
        .into_iter()
        .map(PurchaseOrderVersionSlice::from)
        .collect();

    let paging = Paging::new(url, purchase_order_version_list.paging, service_id);

    Ok(PurchaseOrderVersionListSlice { data, paging })
}

/// Makes a request to return the revisions for a purchase order version
///
/// # Arguments
///
///  * `url` - The base url for the request
///  * `store` - TThe store to fetch the order from
///  * `purchase_order_uid` - The UID of the purchase order
///  * `version_id` - The ID of the version to fetch revisions for
///  * `service_id` - optional - The service ID when running on Splinter
///  * `offset` - The index of the first revision in storage to retrieve
///  * `limit` - The number of items to retrieve from the offset
pub fn list_purchase_order_revisions<'a>(
    url: Url,
    store: Box<dyn PurchaseOrderStore + 'a>,
    purchase_order_uid: String,
    version_id: String,
    service_id: Option<&str>,
    offset: u64,
    limit: u16,
) -> Result<PurchaseOrderRevisionListSlice, ErrorResponse> {
    let offset = i64::try_from(offset).unwrap_or(i64::MAX);

    let limit = i64::try_from(limit).unwrap_or(10);

    let purchase_order_revision_list = store
        .list_purchase_order_revisions(&purchase_order_uid, &version_id, service_id, offset, limit)
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
            PurchaseOrderStoreError::NotFoundError(_) => ErrorResponse::new(
                404,
                &format!(
                    "Purchase order {} version {} not found",
                    purchase_order_uid, version_id
                ),
            ),
        })?;

    let data = purchase_order_revision_list
        .data
        .into_iter()
        .map(PurchaseOrderRevisionSlice::from)
        .collect();

    let paging = Paging::new(url, purchase_order_revision_list.paging, service_id);

    Ok(PurchaseOrderRevisionListSlice { data, paging })
}

/// Makes a request to return a purchase order version revision
///
/// # Arguments
///
///  * `store` - The store to fetch the order from
///  * `purchase_order_uid` - The UID of the purchase order
///  * `version_id` - ID for the version the revision belongs to
///  * `revision_id` - ID for the revision
///  * `service_id` - optional - The service ID when running on Splinter
pub fn get_purchase_order_revision<'a>(
    store: Box<dyn PurchaseOrderStore + 'a>,
    purchase_order_uid: String,
    version_id: String,
    revision_id: i64,
    service_id: Option<&str>,
) -> Result<PurchaseOrderRevisionSlice, ErrorResponse> {
    let purchase_order_revision = store
        .get_purchase_order_revision(&purchase_order_uid, &version_id, &revision_id, service_id)
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
            PurchaseOrderStoreError::NotFoundError(_) => ErrorResponse::new(
                404,
                &format!(
                    "Purchase order {} version {} revision {} not found",
                    purchase_order_uid, version_id, revision_id
                ),
            ),
        })?;

    Ok(PurchaseOrderRevisionSlice::from(
        purchase_order_revision.ok_or_else(|| {
            ErrorResponse::new(
                404,
                &format!("Purchase order {} not found", purchase_order_uid),
            )
        })?,
    ))
}

pub fn get_latest_revision_id<'a>(
    store: Box<dyn PurchaseOrderStore + 'a>,
    purchase_order_uid: String,
    version_id: String,
    service_id: Option<&str>,
) -> Result<Option<i64>, ErrorResponse> {
    let revision = store
        .get_latest_revision_id(&purchase_order_uid, &version_id, service_id)
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
            PurchaseOrderStoreError::NotFoundError(_) => ErrorResponse::new(
                404,
                &format!(
                    "Purchase order {} version {} latest revision not found",
                    purchase_order_uid, version_id
                ),
            ),
        })?;

    Ok(revision)
}
