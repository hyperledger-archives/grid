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

use super::PurchaseOrderStoreOperations;
use crate::commits::MAX_COMMIT_NUM;
use crate::error::InternalError;
use crate::paging::Paging;
use crate::purchase_order::store::diesel::{
    models::PurchaseOrderAlternateIdModel, schema::purchase_order_alternate_id,
    PurchaseOrderAlternateId, PurchaseOrderAlternateIdList,
};

use crate::purchase_order::store::PurchaseOrderStoreError;

use diesel::prelude::*;
use std::convert::TryInto;

pub(in crate::purchase_order::store::diesel) trait PurchaseOrderStoreListAlternateIdsForPurchaseOrderOperation
{
    fn list_alternate_ids_for_purchase_order(
        &self,
        purchase_order_uid: &str,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<PurchaseOrderAlternateIdList, PurchaseOrderStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> PurchaseOrderStoreListAlternateIdsForPurchaseOrderOperation
    for PurchaseOrderStoreOperations<'a, diesel::pg::PgConnection>
{
    fn list_alternate_ids_for_purchase_order(
        &self,
        purchase_order_uid: &str,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<PurchaseOrderAlternateIdList, PurchaseOrderStoreError> {
        self.conn.transaction::<_, PurchaseOrderStoreError, _>(|| {
            let mut query = purchase_order_alternate_id::table
                .into_boxed()
                .select(purchase_order_alternate_id::all_columns)
                .filter(
                    purchase_order_alternate_id::purchase_order_uid
                        .eq(&purchase_order_uid)
                        .and(purchase_order_alternate_id::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

            if let Some(service_id) = service_id {
                query = query.filter(purchase_order_alternate_id::service_id.eq(service_id));
            } else {
                query = query.filter(purchase_order_alternate_id::service_id.is_null());
            }

            let alt_id_models = query
                .load::<PurchaseOrderAlternateIdModel>(self.conn)
                .map_err(|err| {
                    PurchaseOrderStoreError::InternalError(InternalError::from_source(Box::new(
                        err,
                    )))
                })?;

            let total = alt_id_models.len().try_into().map_err(|err| {
                PurchaseOrderStoreError::InternalError(InternalError::from_source(Box::new(err)))
            })?;

            let ids = alt_id_models
                .iter()
                .map(PurchaseOrderAlternateId::from)
                .collect();

            Ok(PurchaseOrderAlternateIdList::new(
                ids,
                Paging::new(offset, limit, total),
            ))
        })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> PurchaseOrderStoreListAlternateIdsForPurchaseOrderOperation
    for PurchaseOrderStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn list_alternate_ids_for_purchase_order(
        &self,
        purchase_order_uid: &str,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<PurchaseOrderAlternateIdList, PurchaseOrderStoreError> {
        self.conn.transaction::<_, PurchaseOrderStoreError, _>(|| {
            let mut query = purchase_order_alternate_id::table
                .into_boxed()
                .select(purchase_order_alternate_id::all_columns)
                .filter(
                    purchase_order_alternate_id::purchase_order_uid
                        .eq(&purchase_order_uid)
                        .and(purchase_order_alternate_id::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

            if let Some(service_id) = service_id {
                query = query.filter(purchase_order_alternate_id::service_id.eq(service_id));
            } else {
                query = query.filter(purchase_order_alternate_id::service_id.is_null());
            }

            let alt_id_models = query
                .load::<PurchaseOrderAlternateIdModel>(self.conn)
                .map_err(|err| {
                    PurchaseOrderStoreError::InternalError(InternalError::from_source(Box::new(
                        err,
                    )))
                })?;

            let total = alt_id_models.len().try_into().map_err(|err| {
                PurchaseOrderStoreError::InternalError(InternalError::from_source(Box::new(err)))
            })?;

            let ids = alt_id_models
                .iter()
                .map(PurchaseOrderAlternateId::from)
                .collect();

            Ok(PurchaseOrderAlternateIdList::new(
                ids,
                Paging::new(offset, limit, total),
            ))
        })
    }
}
