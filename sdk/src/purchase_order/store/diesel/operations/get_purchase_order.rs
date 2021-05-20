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
    models::{PurchaseOrderModel, PurchaseOrderVersionModel, PurchaseOrderVersionRevisionModel},
    schema::{purchase_order, purchase_order_version, purchase_order_version_revision},
    PurchaseOrder, PurchaseOrderVersion,
};

use crate::purchase_order::store::PurchaseOrderStoreError;
use diesel::{prelude::*, result::Error::NotFound};

pub(in crate::purchase_order::store::diesel) trait PurchaseOrderStoreGetPurchaseOrderOperation {
    fn get_purchase_order(
        &self,
        uuid: &str,
        service_id: Option<&str>,
    ) -> Result<Option<PurchaseOrder>, PurchaseOrderStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> PurchaseOrderStoreGetPurchaseOrderOperation
    for PurchaseOrderStoreOperations<'a, diesel::pg::PgConnection>
{
    fn get_purchase_order(
        &self,
        uuid: &str,
        service_id: Option<&str>,
    ) -> Result<Option<PurchaseOrder>, PurchaseOrderStoreError> {
        self.conn.transaction::<_, PurchaseOrderStoreError, _>(|| {
            let mut query = purchase_order::table
                .into_boxed()
                .select(purchase_order::all_columns)
                .filter(
                    purchase_order::uuid
                        .eq(&uuid)
                        .and(purchase_order::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

            if let Some(service_id) = service_id {
                query = query.filter(purchase_order::service_id.eq(service_id));
            } else {
                query = query.filter(purchase_order::service_id.is_null());
            }

            let order = query
                .first::<PurchaseOrderModel>(self.conn)
                .map(Some)
                .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
                .map_err(|err| {
                    PurchaseOrderStoreError::InternalError(InternalError::from_source(Box::new(
                        err,
                    )))
                })?;

            let mut query = purchase_order_version::table
                .into_boxed()
                .select(purchase_order_version::all_columns)
                .filter(
                    purchase_order_version::purchase_order_uuid
                        .eq(&uuid)
                        .and(purchase_order_version::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

            if let Some(service_id) = service_id {
                query = query.filter(purchase_order_version::service_id.eq(service_id));
            } else {
                query = query.filter(purchase_order_version::service_id.is_null());
            }

            let version_models =
                query
                    .load::<PurchaseOrderVersionModel>(self.conn)
                    .map_err(|err| {
                        PurchaseOrderStoreError::InternalError(InternalError::from_source(
                            Box::new(err),
                        ))
                    })?;

            let mut versions = Vec::new();

            for v in version_models {
                let mut query = purchase_order_version_revision::table
                    .into_boxed()
                    .select(purchase_order_version_revision::all_columns)
                    .filter(
                        purchase_order_version_revision::version_id
                            .eq(&v.version_id)
                            .and(purchase_order_version_revision::org_id.eq(&v.org_id))
                            .and(
                                purchase_order_version_revision::end_commit_num.eq(MAX_COMMIT_NUM),
                            ),
                    );

                if let Some(service_id) = service_id {
                    query =
                        query.filter(purchase_order_version_revision::service_id.eq(service_id));
                } else {
                    query = query.filter(purchase_order_version_revision::service_id.is_null());
                }

                let revision_models = query
                    .load::<PurchaseOrderVersionRevisionModel>(self.conn)
                    .map_err(|err| {
                        PurchaseOrderStoreError::InternalError(InternalError::from_source(
                            Box::new(err),
                        ))
                    })?;

                versions.push(PurchaseOrderVersion::from((&v, &revision_models)))
            }

            Ok(order.map(|order| PurchaseOrder::from((order, versions))))
        })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> PurchaseOrderStoreGetPurchaseOrderOperation
    for PurchaseOrderStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn get_purchase_order(
        &self,
        uuid: &str,
        service_id: Option<&str>,
    ) -> Result<Option<PurchaseOrder>, PurchaseOrderStoreError> {
        self.conn.transaction::<_, PurchaseOrderStoreError, _>(|| {
            let mut query = purchase_order::table
                .into_boxed()
                .select(purchase_order::all_columns)
                .filter(
                    purchase_order::uuid
                        .eq(&uuid)
                        .and(purchase_order::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

            if let Some(service_id) = service_id {
                query = query.filter(purchase_order::service_id.eq(service_id));
            } else {
                query = query.filter(purchase_order::service_id.is_null());
            }

            let order = query
                .first::<PurchaseOrderModel>(self.conn)
                .map(Some)
                .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
                .map_err(|err| {
                    PurchaseOrderStoreError::InternalError(InternalError::from_source(Box::new(
                        err,
                    )))
                })?;

            let mut query = purchase_order_version::table
                .into_boxed()
                .select(purchase_order_version::all_columns)
                .filter(
                    purchase_order_version::purchase_order_uuid
                        .eq(&uuid)
                        .and(purchase_order_version::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

            if let Some(service_id) = service_id {
                query = query.filter(purchase_order_version::service_id.eq(service_id));
            } else {
                query = query.filter(purchase_order_version::service_id.is_null());
            }

            let version_models =
                query
                    .load::<PurchaseOrderVersionModel>(self.conn)
                    .map_err(|err| {
                        PurchaseOrderStoreError::InternalError(InternalError::from_source(
                            Box::new(err),
                        ))
                    })?;

            let mut versions = Vec::new();

            for v in version_models {
                let mut query = purchase_order_version_revision::table
                    .into_boxed()
                    .select(purchase_order_version_revision::all_columns)
                    .filter(
                        purchase_order_version_revision::version_id
                            .eq(&v.version_id)
                            .and(purchase_order_version_revision::org_id.eq(&v.org_id))
                            .and(
                                purchase_order_version_revision::end_commit_num.eq(MAX_COMMIT_NUM),
                            ),
                    );

                if let Some(service_id) = service_id {
                    query =
                        query.filter(purchase_order_version_revision::service_id.eq(service_id));
                } else {
                    query = query.filter(purchase_order_version_revision::service_id.is_null());
                }

                let revision_models = query
                    .load::<PurchaseOrderVersionRevisionModel>(self.conn)
                    .map_err(|err| {
                        PurchaseOrderStoreError::InternalError(InternalError::from_source(
                            Box::new(err),
                        ))
                    })?;

                versions.push(PurchaseOrderVersion::from((&v, &revision_models)))
            }

            Ok(order.map(|order| PurchaseOrder::from((order, versions))))
        })
    }
}
