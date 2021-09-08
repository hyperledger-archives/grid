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

use super::{
    add_purchase_order_version, add_purchase_order_version_revision, PurchaseOrderStoreOperations,
};
use crate::purchase_order::store::diesel::{
    models::{
        NewPurchaseOrderModel, NewPurchaseOrderVersionModel, NewPurchaseOrderVersionRevisionModel,
        PurchaseOrderModel,
    },
    schema::purchase_order,
    PurchaseOrderStoreError,
};

use crate::commits::MAX_COMMIT_NUM;
use crate::error::InternalError;

use diesel::{
    dsl::{insert_into, update},
    prelude::*,
};

pub(in crate::purchase_order::store::diesel) trait PurchaseOrderStoreAddPurchaseOrderOperation {
    fn add_purchase_order(
        &self,
        order: NewPurchaseOrderModel,
        versions: Vec<NewPurchaseOrderVersionModel>,
        revisions: Vec<NewPurchaseOrderVersionRevisionModel>,
    ) -> Result<(), PurchaseOrderStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> PurchaseOrderStoreAddPurchaseOrderOperation
    for PurchaseOrderStoreOperations<'a, diesel::pg::PgConnection>
{
    fn add_purchase_order(
        &self,
        order: NewPurchaseOrderModel,
        versions: Vec<NewPurchaseOrderVersionModel>,
        revisions: Vec<NewPurchaseOrderVersionRevisionModel>,
    ) -> Result<(), PurchaseOrderStoreError> {
        self.conn.transaction::<_, PurchaseOrderStoreError, _>(|| {
            for revision in revisions {
                add_purchase_order_version_revision::pg::add_purchase_order_version_revision(
                    self.conn,
                    &revision,
                    &order.purchase_order_uid,
                )?;
            }

            for version in versions {
                add_purchase_order_version::pg::add_purchase_order_version(self.conn, &version)?;
            }

            let mut query = purchase_order::table.into_boxed().filter(
                purchase_order::purchase_order_uid
                    .eq(&order.purchase_order_uid)
                    .and(purchase_order::end_commit_num.eq(MAX_COMMIT_NUM)),
            );

            if let Some(service_id) = &order.service_id {
                query = query.filter(purchase_order::service_id.eq(service_id));
            } else {
                query = query.filter(purchase_order::service_id.is_null());
            }

            let duplicate = query
                .first::<PurchaseOrderModel>(self.conn)
                .optional()
                .map_err(|err| {
                    PurchaseOrderStoreError::InternalError(InternalError::from_source(Box::new(
                        err,
                    )))
                })?;

            if duplicate.is_some() {
                if let Some(service_id) = &order.service_id {
                    update(purchase_order::table)
                        .filter(
                            purchase_order::purchase_order_uid
                                .eq(&order.purchase_order_uid)
                                .and(purchase_order::service_id.eq(service_id))
                                .and(purchase_order::end_commit_num.eq(MAX_COMMIT_NUM)),
                        )
                        .set(purchase_order::end_commit_num.eq(&order.start_commit_num))
                        .execute(self.conn)
                        .map(|_| ())
                        .map_err(PurchaseOrderStoreError::from)?;
                } else {
                    update(purchase_order::table)
                        .filter(
                            purchase_order::purchase_order_uid
                                .eq(&order.purchase_order_uid)
                                .and(purchase_order::end_commit_num.eq(MAX_COMMIT_NUM)),
                        )
                        .set(purchase_order::end_commit_num.eq(&order.start_commit_num))
                        .execute(self.conn)
                        .map(|_| ())
                        .map_err(PurchaseOrderStoreError::from)?;
                }
            }

            insert_into(purchase_order::table)
                .values(&order)
                .execute(self.conn)
                .map(|_| ())
                .map_err(PurchaseOrderStoreError::from)?;

            Ok(())
        })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> PurchaseOrderStoreAddPurchaseOrderOperation
    for PurchaseOrderStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn add_purchase_order(
        &self,
        order: NewPurchaseOrderModel,
        versions: Vec<NewPurchaseOrderVersionModel>,
        revisions: Vec<NewPurchaseOrderVersionRevisionModel>,
    ) -> Result<(), PurchaseOrderStoreError> {
        self.conn.transaction::<_, PurchaseOrderStoreError, _>(|| {
            for revision in revisions {
                add_purchase_order_version_revision::sqlite::add_purchase_order_version_revision(
                    self.conn,
                    &revision,
                    &order.purchase_order_uid,
                )?;
            }

            for version in versions {
                add_purchase_order_version::sqlite::add_purchase_order_version(
                    self.conn, &version,
                )?;
            }

            let mut query = purchase_order::table.into_boxed().filter(
                purchase_order::purchase_order_uid
                    .eq(&order.purchase_order_uid)
                    .and(purchase_order::end_commit_num.eq(MAX_COMMIT_NUM)),
            );

            if let Some(service_id) = &order.service_id {
                query = query.filter(purchase_order::service_id.eq(service_id));
            } else {
                query = query.filter(purchase_order::service_id.is_null());
            }

            let duplicate = query
                .first::<PurchaseOrderModel>(self.conn)
                .optional()
                .map_err(|err| {
                    PurchaseOrderStoreError::InternalError(InternalError::from_source(Box::new(
                        err,
                    )))
                })?;

            if duplicate.is_some() {
                if let Some(service_id) = &order.service_id {
                    update(purchase_order::table)
                        .filter(
                            purchase_order::purchase_order_uid
                                .eq(&order.purchase_order_uid)
                                .and(purchase_order::service_id.eq(service_id))
                                .and(purchase_order::end_commit_num.eq(MAX_COMMIT_NUM)),
                        )
                        .set(purchase_order::end_commit_num.eq(&order.start_commit_num))
                        .execute(self.conn)
                        .map(|_| ())
                        .map_err(PurchaseOrderStoreError::from)?;
                } else {
                    update(purchase_order::table)
                        .filter(
                            purchase_order::purchase_order_uid
                                .eq(&order.purchase_order_uid)
                                .and(purchase_order::end_commit_num.eq(MAX_COMMIT_NUM)),
                        )
                        .set(purchase_order::end_commit_num.eq(&order.start_commit_num))
                        .execute(self.conn)
                        .map(|_| ())
                        .map_err(PurchaseOrderStoreError::from)?;
                }
            }

            insert_into(purchase_order::table)
                .values(&order)
                .execute(self.conn)
                .map(|_| ())
                .map_err(PurchaseOrderStoreError::from)?;

            Ok(())
        })
    }
}
