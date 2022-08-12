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

use crate::purchase_order::store::diesel::{
    models::{NewPurchaseOrderVersionModel, PurchaseOrderVersionModel},
    schema::purchase_order_version,
    PurchaseOrderStoreError,
};

use crate::commits::MAX_COMMIT_NUM;
use crate::error::InternalError;

use diesel::{
    dsl::{insert_into, update},
    prelude::*,
};

#[cfg(feature = "postgres")]
pub(crate) mod pg {
    use super::*;

    pub fn add_purchase_order_version(
        conn: &diesel::pg::PgConnection,
        version: &NewPurchaseOrderVersionModel,
    ) -> Result<(), PurchaseOrderStoreError> {
        conn.transaction::<_, PurchaseOrderStoreError, _>(|| {
            let mut query = purchase_order_version::table.into_boxed().filter(
                purchase_order_version::purchase_order_uid
                    .eq(&version.purchase_order_uid)
                    .and(purchase_order_version::version_id.eq(&version.version_id))
                    .and(purchase_order_version::end_commit_num.eq(MAX_COMMIT_NUM)),
            );

            if let Some(service_id) = &version.service_id {
                query = query.filter(purchase_order_version::service_id.eq(service_id));
            } else {
                query = query.filter(purchase_order_version::service_id.is_null());
            }

            let duplicate = query
                .first::<PurchaseOrderVersionModel>(conn)
                .optional()
                .map_err(|err| {
                    PurchaseOrderStoreError::InternalError(InternalError::from_source(Box::new(
                        err,
                    )))
                })?;

            if duplicate.is_some() {
                if let Some(service_id) = &version.service_id {
                    update(purchase_order_version::table)
                        .filter(
                            purchase_order_version::purchase_order_uid
                                .eq(&version.purchase_order_uid)
                                .and(purchase_order_version::version_id.eq(&version.version_id))
                                .and(purchase_order_version::service_id.eq(service_id))
                                .and(purchase_order_version::end_commit_num.eq(MAX_COMMIT_NUM)),
                        )
                        .set(purchase_order_version::end_commit_num.eq(&version.start_commit_num))
                        .execute(conn)
                        .map(|_| ())
                        .map_err(PurchaseOrderStoreError::from)?;
                } else {
                    update(purchase_order_version::table)
                        .filter(
                            purchase_order_version::purchase_order_uid
                                .eq(&version.purchase_order_uid)
                                .and(purchase_order_version::version_id.eq(&version.version_id))
                                .and(purchase_order_version::end_commit_num.eq(MAX_COMMIT_NUM)),
                        )
                        .set(purchase_order_version::end_commit_num.eq(&version.start_commit_num))
                        .execute(conn)
                        .map(|_| ())
                        .map_err(PurchaseOrderStoreError::from)?;
                }
            }

            insert_into(purchase_order_version::table)
                .values(version)
                .execute(conn)
                .map(|_| ())
                .map_err(PurchaseOrderStoreError::from)?;

            Ok(())
        })
    }
}

#[cfg(feature = "sqlite")]
pub(crate) mod sqlite {
    use super::*;

    pub fn add_purchase_order_version(
        conn: &diesel::sqlite::SqliteConnection,
        version: &NewPurchaseOrderVersionModel,
    ) -> Result<(), PurchaseOrderStoreError> {
        conn.transaction::<_, PurchaseOrderStoreError, _>(|| {
            let mut query = purchase_order_version::table.into_boxed().filter(
                purchase_order_version::purchase_order_uid
                    .eq(&version.purchase_order_uid)
                    .and(purchase_order_version::version_id.eq(&version.version_id))
                    .and(purchase_order_version::end_commit_num.eq(MAX_COMMIT_NUM)),
            );

            if let Some(service_id) = &version.service_id {
                query = query.filter(purchase_order_version::service_id.eq(service_id));
            } else {
                query = query.filter(purchase_order_version::service_id.is_null());
            }

            let duplicate = query
                .first::<PurchaseOrderVersionModel>(conn)
                .optional()
                .map_err(|err| {
                    PurchaseOrderStoreError::InternalError(InternalError::from_source(Box::new(
                        err,
                    )))
                })?;

            if duplicate.is_some() {
                if let Some(service_id) = &version.service_id {
                    update(purchase_order_version::table)
                        .filter(
                            purchase_order_version::purchase_order_uid
                                .eq(&version.purchase_order_uid)
                                .and(purchase_order_version::version_id.eq(&version.version_id))
                                .and(purchase_order_version::service_id.eq(service_id))
                                .and(purchase_order_version::end_commit_num.eq(MAX_COMMIT_NUM)),
                        )
                        .set(purchase_order_version::end_commit_num.eq(&version.start_commit_num))
                        .execute(conn)
                        .map(|_| ())
                        .map_err(PurchaseOrderStoreError::from)?;
                } else {
                    update(purchase_order_version::table)
                        .filter(
                            purchase_order_version::purchase_order_uid
                                .eq(&version.purchase_order_uid)
                                .and(purchase_order_version::version_id.eq(&version.version_id))
                                .and(purchase_order_version::end_commit_num.eq(MAX_COMMIT_NUM)),
                        )
                        .set(purchase_order_version::end_commit_num.eq(&version.start_commit_num))
                        .execute(conn)
                        .map(|_| ())
                        .map_err(PurchaseOrderStoreError::from)?;
                }
            }

            insert_into(purchase_order_version::table)
                .values(version)
                .execute(conn)
                .map(|_| ())
                .map_err(PurchaseOrderStoreError::from)?;

            Ok(())
        })
    }
}
