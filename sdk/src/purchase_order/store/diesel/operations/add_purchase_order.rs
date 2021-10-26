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
    add_alternate_id, add_purchase_order_version, add_purchase_order_version_revision,
    remove_alternate_id, PurchaseOrderStoreOperations,
};
use crate::purchase_order::store::diesel::{
    models::{
        NewPurchaseOrderAlternateIdModel, NewPurchaseOrderModel, NewPurchaseOrderVersionModel,
        NewPurchaseOrderVersionRevisionModel, PurchaseOrderAlternateIdModel, PurchaseOrderModel,
    },
    schema::{purchase_order, purchase_order_alternate_id},
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
        alternate_ids: Vec<NewPurchaseOrderAlternateIdModel>,
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
        alternate_ids: Vec<NewPurchaseOrderAlternateIdModel>,
    ) -> Result<(), PurchaseOrderStoreError> {
        self.conn.transaction::<_, PurchaseOrderStoreError, _>(|| {
            let mut alt_id_query = purchase_order_alternate_id::table
                .into_boxed()
                .select(purchase_order_alternate_id::all_columns)
                .filter(
                    purchase_order_alternate_id::purchase_order_uid
                        .eq(&order.purchase_order_uid)
                        .and(purchase_order_alternate_id::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

            if let Some(service_id) = &order.service_id {
                alt_id_query =
                    alt_id_query.filter(purchase_order_alternate_id::service_id.eq(service_id));
            } else {
                alt_id_query =
                    alt_id_query.filter(purchase_order_alternate_id::service_id.is_null());
            }

            let existing_alt_ids = alt_id_query
                .load::<PurchaseOrderAlternateIdModel>(self.conn)
                .map_err(|err| {
                    PurchaseOrderStoreError::InternalError(InternalError::from_source(Box::new(
                        err,
                    )))
                })?;

            for e in existing_alt_ids {
                if !alternate_ids.iter().any(|id| {
                    e.alternate_id_type == id.alternate_id_type && e.alternate_id == id.alternate_id
                }) {
                    remove_alternate_id::pg::remove_alternate_id(
                        self.conn,
                        &e,
                        &order.end_commit_num,
                    )?;
                }
            }

            for id in alternate_ids {
                add_alternate_id::pg::add_alternate_id(self.conn, &id)?;
            }

            for version in versions {
                add_purchase_order_version::pg::add_purchase_order_version(self.conn, &version)?;
            }

            for revision in revisions {
                add_purchase_order_version_revision::pg::add_purchase_order_version_revision(
                    self.conn,
                    &revision,
                    &order.purchase_order_uid,
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

#[cfg(feature = "sqlite")]
impl<'a> PurchaseOrderStoreAddPurchaseOrderOperation
    for PurchaseOrderStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn add_purchase_order(
        &self,
        order: NewPurchaseOrderModel,
        versions: Vec<NewPurchaseOrderVersionModel>,
        revisions: Vec<NewPurchaseOrderVersionRevisionModel>,
        alternate_ids: Vec<NewPurchaseOrderAlternateIdModel>,
    ) -> Result<(), PurchaseOrderStoreError> {
        self.conn.transaction::<_, PurchaseOrderStoreError, _>(|| {
            let mut alt_id_query = purchase_order_alternate_id::table
                .into_boxed()
                .select(purchase_order_alternate_id::all_columns)
                .filter(
                    purchase_order_alternate_id::purchase_order_uid
                        .eq(&order.purchase_order_uid)
                        .and(purchase_order_alternate_id::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

            if let Some(service_id) = &order.service_id {
                alt_id_query =
                    alt_id_query.filter(purchase_order_alternate_id::service_id.eq(service_id));
            } else {
                alt_id_query =
                    alt_id_query.filter(purchase_order_alternate_id::service_id.is_null());
            }

            let existing_alt_ids = alt_id_query
                .load::<PurchaseOrderAlternateIdModel>(self.conn)
                .map_err(|err| {
                    PurchaseOrderStoreError::InternalError(InternalError::from_source(Box::new(
                        err,
                    )))
                })?;

            for e in existing_alt_ids {
                if !alternate_ids.iter().any(|id| {
                    e.alternate_id_type == id.alternate_id_type && e.alternate_id == id.alternate_id
                }) {
                    remove_alternate_id::sqlite::remove_alternate_id(
                        self.conn,
                        &e,
                        &order.end_commit_num,
                    )?;
                }
            }

            for id in alternate_ids {
                add_alternate_id::sqlite::add_alternate_id(self.conn, &id)?;
            }

            for version in versions {
                add_purchase_order_version::sqlite::add_purchase_order_version(
                    self.conn, &version,
                )?;
            }

            for revision in revisions {
                add_purchase_order_version_revision::sqlite::add_purchase_order_version_revision(
                    self.conn,
                    &revision,
                    &order.purchase_order_uid,
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
