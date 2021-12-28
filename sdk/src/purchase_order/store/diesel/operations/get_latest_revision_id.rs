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

use super::{get_uid_from_alternate_id, PurchaseOrderStoreOperations};
use crate::commits::MAX_COMMIT_NUM;
use crate::error::InternalError;

use crate::purchase_order::store::diesel::schema::purchase_order_version_revision;

use crate::purchase_order::store::PurchaseOrderStoreError;
use diesel::dsl::max;
use diesel::prelude::*;

pub(in crate::purchase_order::store::diesel) trait PurchaseOrderStoreGetLatestRevisionIdOperation {
    fn get_latest_revision_id(
        &self,
        purchase_order_id: &str,
        version_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<i64>, PurchaseOrderStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> PurchaseOrderStoreGetLatestRevisionIdOperation
    for PurchaseOrderStoreOperations<'a, diesel::pg::PgConnection>
{
    fn get_latest_revision_id(
        &self,
        purchase_order_id: &str,
        version_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<i64>, PurchaseOrderStoreError> {
        self.conn.transaction::<_, PurchaseOrderStoreError, _>(|| {
            let mut purchase_order_uid = purchase_order_id.to_string();
            if purchase_order_id.contains(':') {
                purchase_order_uid = get_uid_from_alternate_id::pg::get_uid_from_alternate_id(
                    self.conn,
                    purchase_order_id,
                    service_id,
                )?;
            }

            let mut query = purchase_order_version_revision::table
                .into_boxed()
                .select(max(purchase_order_version_revision::revision_id))
                .filter(
                    purchase_order_version_revision::purchase_order_uid
                        .eq(&purchase_order_uid)
                        .and(purchase_order_version_revision::version_id.eq(&version_id))
                        .and(purchase_order_version_revision::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

            if let Some(service_id) = service_id {
                query = query.filter(purchase_order_version_revision::service_id.eq(service_id));
            } else {
                query = query.filter(purchase_order_version_revision::service_id.is_null());
            }

            let num = query.first::<Option<i64>>(self.conn).map_err(|err| {
                PurchaseOrderStoreError::InternalError(InternalError::from_source(Box::new(err)))
            })?;

            Ok(num)
        })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> PurchaseOrderStoreGetLatestRevisionIdOperation
    for PurchaseOrderStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn get_latest_revision_id(
        &self,
        purchase_order_id: &str,
        version_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<i64>, PurchaseOrderStoreError> {
        self.conn.transaction::<_, PurchaseOrderStoreError, _>(|| {
            let mut purchase_order_uid = purchase_order_id.to_string();
            if purchase_order_id.contains(':') {
                purchase_order_uid = get_uid_from_alternate_id::sqlite::get_uid_from_alternate_id(
                    self.conn,
                    purchase_order_id,
                    service_id,
                )?;
            }

            let mut query = purchase_order_version_revision::table
                .into_boxed()
                .select(max(purchase_order_version_revision::revision_id))
                .filter(
                    purchase_order_version_revision::purchase_order_uid
                        .eq(&purchase_order_uid)
                        .and(purchase_order_version_revision::version_id.eq(&version_id))
                        .and(purchase_order_version_revision::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

            if let Some(service_id) = service_id {
                query = query.filter(purchase_order_version_revision::service_id.eq(service_id));
            } else {
                query = query.filter(purchase_order_version_revision::service_id.is_null());
            }

            let num = query.first::<Option<i64>>(self.conn).map_err(|err| {
                PurchaseOrderStoreError::InternalError(InternalError::from_source(Box::new(err)))
            })?;

            Ok(num)
        })
    }
}
