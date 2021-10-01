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
use crate::purchase_order::store::diesel::{
    models::PurchaseOrderVersionRevisionModel, schema::purchase_order_version_revision,
    PurchaseOrderVersionRevision,
};

use crate::purchase_order::store::PurchaseOrderStoreError;
use diesel::prelude::*;

pub(in crate::purchase_order::store::diesel) trait PurchaseOrderStoreGetPurchaseOrderRevisionOperation
{
    fn get_purchase_order_revision(
        &self,
        po_uid: &str,
        version_id: &str,
        revision_id: &i64,
        service_id: Option<&str>,
    ) -> Result<Option<PurchaseOrderVersionRevision>, PurchaseOrderStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> PurchaseOrderStoreGetPurchaseOrderRevisionOperation
    for PurchaseOrderStoreOperations<'a, diesel::pg::PgConnection>
{
    fn get_purchase_order_revision(
        &self,
        po_uid: &str,
        version_id: &str,
        revision_id: &i64,
        service_id: Option<&str>,
    ) -> Result<Option<PurchaseOrderVersionRevision>, PurchaseOrderStoreError> {
        self.conn.transaction::<_, PurchaseOrderStoreError, _>(|| {
            let mut query = purchase_order_version_revision::table
                .into_boxed()
                .select(purchase_order_version_revision::all_columns)
                .filter(
                    purchase_order_version_revision::purchase_order_uid
                        .eq(&po_uid)
                        .and(purchase_order_version_revision::version_id.eq(&version_id))
                        .and(purchase_order_version_revision::revision_id.eq(&revision_id))
                        .and(purchase_order_version_revision::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

            if let Some(service_id) = service_id {
                query = query.filter(purchase_order_version_revision::service_id.eq(service_id));
            } else {
                query = query.filter(purchase_order_version_revision::service_id.is_null());
            }

            let revision = query
                .first::<PurchaseOrderVersionRevisionModel>(self.conn)
                .optional()
                .map_err(|err| {
                    PurchaseOrderStoreError::InternalError(InternalError::from_source(Box::new(
                        err,
                    )))
                })?;

            Ok(revision.map(PurchaseOrderVersionRevision::from))
        })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> PurchaseOrderStoreGetPurchaseOrderRevisionOperation
    for PurchaseOrderStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn get_purchase_order_revision(
        &self,
        po_uid: &str,
        version_id: &str,
        revision_id: &i64,
        service_id: Option<&str>,
    ) -> Result<Option<PurchaseOrderVersionRevision>, PurchaseOrderStoreError> {
        self.conn.transaction::<_, PurchaseOrderStoreError, _>(|| {
            let mut query = purchase_order_version_revision::table
                .into_boxed()
                .select(purchase_order_version_revision::all_columns)
                .filter(
                    purchase_order_version_revision::purchase_order_uid
                        .eq(&po_uid)
                        .and(purchase_order_version_revision::version_id.eq(&version_id))
                        .and(purchase_order_version_revision::revision_id.eq(&revision_id))
                        .and(purchase_order_version_revision::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

            if let Some(service_id) = service_id {
                query = query.filter(purchase_order_version_revision::service_id.eq(service_id));
            } else {
                query = query.filter(purchase_order_version_revision::service_id.is_null());
            }

            let revision = query
                .first::<PurchaseOrderVersionRevisionModel>(self.conn)
                .optional()
                .map_err(|err| {
                    PurchaseOrderStoreError::InternalError(InternalError::from_source(Box::new(
                        err,
                    )))
                })?;

            Ok(revision.map(PurchaseOrderVersionRevision::from))
        })
    }
}
