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
use crate::purchase_order::store::diesel::{
    models::{PurchaseOrderVersionModel, PurchaseOrderVersionRevisionModel},
    schema::{purchase_order_version, purchase_order_version_revision},
    PurchaseOrderVersion,
};

use crate::purchase_order::store::PurchaseOrderStoreError;
use diesel::prelude::*;

pub(in crate::purchase_order::store::diesel) trait PurchaseOrderStoreGetPurchaseOrderVersionOperation
{
    fn get_purchase_order_version(
        &self,
        purchase_order_id: &str,
        version_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<PurchaseOrderVersion>, PurchaseOrderStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> PurchaseOrderStoreGetPurchaseOrderVersionOperation
    for PurchaseOrderStoreOperations<'a, diesel::pg::PgConnection>
{
    fn get_purchase_order_version(
        &self,
        purchase_order_id: &str,
        version_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<PurchaseOrderVersion>, PurchaseOrderStoreError> {
        self.conn.transaction::<_, PurchaseOrderStoreError, _>(|| {
            let mut purchase_order_uid = purchase_order_id.to_string();
            if purchase_order_id.contains(':') {
                purchase_order_uid = get_uid_from_alternate_id::pg::get_uid_from_alternate_id(
                    self.conn,
                    purchase_order_id,
                    service_id,
                )?;
            }
            let mut query = purchase_order_version::table
                .into_boxed()
                .select(purchase_order_version::all_columns)
                .filter(
                    purchase_order_version::version_id
                        .eq(&version_id)
                        .and(purchase_order_version::purchase_order_uid.eq(&purchase_order_uid))
                        .and(purchase_order_version::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

            if let Some(service_id) = service_id {
                query = query.filter(purchase_order_version::service_id.eq(service_id));
            } else {
                query = query.filter(purchase_order_version::service_id.is_null());
            }

            let version = query
                .first::<PurchaseOrderVersionModel>(self.conn)
                .optional()
                .map_err(|err| {
                    PurchaseOrderStoreError::InternalError(InternalError::from_source(Box::new(
                        err,
                    )))
                })?;

            match version {
                Some(version) => {
                    let mut query = purchase_order_version_revision::table
                        .into_boxed()
                        .select(purchase_order_version_revision::all_columns)
                        .filter(
                            purchase_order_version_revision::version_id
                                .eq(&version_id)
                                .and(
                                    purchase_order_version_revision::end_commit_num
                                        .eq(MAX_COMMIT_NUM),
                                ),
                        );

                    if let Some(service_id) = service_id {
                        query = query
                            .filter(purchase_order_version_revision::service_id.eq(service_id));
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

                    Ok(Some(PurchaseOrderVersion::from((
                        &version,
                        &revision_models,
                    ))))
                }
                None => Ok(None),
            }
        })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> PurchaseOrderStoreGetPurchaseOrderVersionOperation
    for PurchaseOrderStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn get_purchase_order_version(
        &self,
        purchase_order_id: &str,
        version_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<PurchaseOrderVersion>, PurchaseOrderStoreError> {
        self.conn.transaction::<_, PurchaseOrderStoreError, _>(|| {
            let mut purchase_order_uid = purchase_order_id.to_string();
            if purchase_order_id.contains(':') {
                purchase_order_uid = get_uid_from_alternate_id::sqlite::get_uid_from_alternate_id(
                    self.conn,
                    purchase_order_id,
                    service_id,
                )?;
            }
            let mut query = purchase_order_version::table
                .into_boxed()
                .select(purchase_order_version::all_columns)
                .filter(
                    purchase_order_version::version_id
                        .eq(&version_id)
                        .and(purchase_order_version::purchase_order_uid.eq(&purchase_order_uid))
                        .and(purchase_order_version::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

            if let Some(service_id) = service_id {
                query = query.filter(purchase_order_version::service_id.eq(service_id));
            } else {
                query = query.filter(purchase_order_version::service_id.is_null());
            }

            let version = query
                .first::<PurchaseOrderVersionModel>(self.conn)
                .optional()
                .map_err(|err| {
                    PurchaseOrderStoreError::InternalError(InternalError::from_source(Box::new(
                        err,
                    )))
                })?;

            match version {
                Some(version) => {
                    let mut query = purchase_order_version_revision::table
                        .into_boxed()
                        .select(purchase_order_version_revision::all_columns)
                        .filter(
                            purchase_order_version_revision::version_id
                                .eq(&version_id)
                                .and(
                                    purchase_order_version_revision::end_commit_num
                                        .eq(MAX_COMMIT_NUM),
                                ),
                        );

                    if let Some(service_id) = service_id {
                        query = query
                            .filter(purchase_order_version_revision::service_id.eq(service_id));
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

                    Ok(Some(PurchaseOrderVersion::from((
                        &version,
                        &revision_models,
                    ))))
                }
                None => Ok(None),
            }
        })
    }
}
