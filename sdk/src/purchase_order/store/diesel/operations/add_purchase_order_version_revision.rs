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
    models::{
        NewPurchaseOrderVersionModel, NewPurchaseOrderVersionRevisionModel,
        PurchaseOrderVersionModel, PurchaseOrderVersionRevisionModel,
    },
    schema::{purchase_order_version, purchase_order_version_revision},
    PurchaseOrderStoreError,
};

use crate::commits::MAX_COMMIT_NUM;
use crate::error::InternalError;

use diesel::{
    dsl::{insert_into, update},
    prelude::*,
    result::Error as dsl_error,
};

#[cfg(feature = "postgres")]
pub(in crate) mod pg {
    use super::*;

    pub fn add_purchase_order_version_revision(
        conn: &diesel::pg::PgConnection,
        revision: &NewPurchaseOrderVersionRevisionModel,
        po_id: &str,
    ) -> Result<(), PurchaseOrderStoreError> {
        conn.transaction::<_, PurchaseOrderStoreError, _>(|| {
            let mut query = purchase_order_version_revision::table.into_boxed().filter(
                purchase_order_version_revision::version_id
                    .eq(&revision.version_id)
                    .and(purchase_order_version_revision::revision_id.eq(&revision.revision_id))
                    .and(purchase_order_version_revision::created_at.eq(&revision.created_at))
                    .and(purchase_order_version_revision::end_commit_num.eq(MAX_COMMIT_NUM)),
            );

            if let Some(service_id) = &revision.service_id {
                query = query.filter(purchase_order_version_revision::service_id.eq(service_id));
            } else {
                query = query.filter(purchase_order_version_revision::service_id.is_null());
            }

            let duplicate = query
                .first::<PurchaseOrderVersionRevisionModel>(conn)
                .optional()
                .map_err(|err| {
                    PurchaseOrderStoreError::InternalError(InternalError::from_source(Box::new(
                        err,
                    )))
                })?;

            if duplicate.is_none() {
                insert_into(purchase_order_version_revision::table)
                    .values(revision)
                    .execute(conn)
                    .map(|_| ())
                    .map_err(PurchaseOrderStoreError::from)?;

                let mut version_query = purchase_order_version::table.into_boxed().filter(
                    purchase_order_version::purchase_order_uid
                        .eq(po_id)
                        .and(purchase_order_version::version_id.eq(&revision.version_id))
                        .and(purchase_order_version::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

                if let Some(service_id) = &revision.service_id {
                    version_query =
                        version_query.filter(purchase_order_version::service_id.eq(service_id));
                } else {
                    version_query =
                        version_query.filter(purchase_order_version::service_id.is_null());
                }

                let version = version_query
                    .first::<PurchaseOrderVersionModel>(conn)
                    .map(Some)
                    .map_err(|err| {
                        if err == dsl_error::NotFound {
                            PurchaseOrderStoreError::NotFoundError(
                                "Cannot fetch PO version to update current revision".to_string(),
                            )
                        } else {
                            PurchaseOrderStoreError::InternalError(InternalError::from_source(
                                Box::new(err),
                            ))
                        }
                    })?;

                if let Some(version) = version {
                    if let Some(service_id) = &revision.service_id {
                        update(purchase_order_version::table)
                            .filter(
                                purchase_order_version::purchase_order_uid
                                    .eq(po_id)
                                    .and(
                                        purchase_order_version::version_id.eq(&revision.version_id),
                                    )
                                    .and(purchase_order_version::service_id.eq(service_id))
                                    .and(purchase_order_version::end_commit_num.eq(MAX_COMMIT_NUM)),
                            )
                            .set(
                                purchase_order_version::end_commit_num
                                    .eq(&revision.start_commit_num),
                            )
                            .execute(conn)
                            .map(|_| ())
                            .map_err(PurchaseOrderStoreError::from)?;
                    } else {
                        update(purchase_order_version::table)
                            .filter(
                                purchase_order_version::purchase_order_uid
                                    .eq(po_id)
                                    .and(
                                        purchase_order_version::version_id.eq(&revision.version_id),
                                    )
                                    .and(purchase_order_version::end_commit_num.eq(MAX_COMMIT_NUM)),
                            )
                            .set(
                                purchase_order_version::end_commit_num
                                    .eq(&revision.start_commit_num),
                            )
                            .execute(conn)
                            .map(|_| ())
                            .map_err(PurchaseOrderStoreError::from)?;
                    }

                    let updated_version = NewPurchaseOrderVersionModel::from((
                        version,
                        &revision.revision_id,
                        &revision.start_commit_num,
                    ));

                    insert_into(purchase_order_version::table)
                        .values(&updated_version)
                        .execute(conn)
                        .map(|_| ())
                        .map_err(PurchaseOrderStoreError::from)?;
                }
            }

            Ok(())
        })
    }
}

#[cfg(feature = "sqlite")]
pub(in crate) mod sqlite {
    use super::*;

    pub fn add_purchase_order_version_revision(
        conn: &diesel::sqlite::SqliteConnection,
        revision: &NewPurchaseOrderVersionRevisionModel,
        po_id: &str,
    ) -> Result<(), PurchaseOrderStoreError> {
        conn.transaction::<_, PurchaseOrderStoreError, _>(|| {
            let mut query = purchase_order_version_revision::table.into_boxed().filter(
                purchase_order_version_revision::version_id
                    .eq(&revision.version_id)
                    .and(purchase_order_version_revision::revision_id.eq(&revision.revision_id))
                    .and(purchase_order_version_revision::created_at.eq(&revision.created_at))
                    .and(purchase_order_version_revision::end_commit_num.eq(MAX_COMMIT_NUM)),
            );

            if let Some(service_id) = &revision.service_id {
                query = query.filter(purchase_order_version_revision::service_id.eq(service_id));
            } else {
                query = query.filter(purchase_order_version_revision::service_id.is_null());
            }

            let duplicate = query
                .first::<PurchaseOrderVersionRevisionModel>(conn)
                .optional()
                .map_err(|err| {
                    PurchaseOrderStoreError::InternalError(InternalError::from_source(Box::new(
                        err,
                    )))
                })?;

            if duplicate.is_none() {
                insert_into(purchase_order_version_revision::table)
                    .values(revision)
                    .execute(conn)
                    .map(|_| ())
                    .map_err(PurchaseOrderStoreError::from)?;

                let mut version_query = purchase_order_version::table.into_boxed().filter(
                    purchase_order_version::purchase_order_uid
                        .eq(po_id)
                        .and(purchase_order_version::version_id.eq(&revision.version_id))
                        .and(purchase_order_version::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

                if let Some(service_id) = &revision.service_id {
                    version_query =
                        version_query.filter(purchase_order_version::service_id.eq(service_id));
                } else {
                    version_query =
                        version_query.filter(purchase_order_version::service_id.is_null());
                }

                let version = version_query
                    .first::<PurchaseOrderVersionModel>(conn)
                    .map(Some)
                    .map_err(|err| {
                        if err == dsl_error::NotFound {
                            PurchaseOrderStoreError::NotFoundError(
                                "Cannot fetch PO version to update current revision".to_string(),
                            )
                        } else {
                            PurchaseOrderStoreError::InternalError(InternalError::from_source(
                                Box::new(err),
                            ))
                        }
                    })?;

                if let Some(version) = version {
                    if let Some(service_id) = &revision.service_id {
                        update(purchase_order_version::table)
                            .filter(
                                purchase_order_version::purchase_order_uid
                                    .eq(po_id)
                                    .and(
                                        purchase_order_version::version_id.eq(&revision.version_id),
                                    )
                                    .and(purchase_order_version::service_id.eq(service_id))
                                    .and(purchase_order_version::end_commit_num.eq(MAX_COMMIT_NUM)),
                            )
                            .set(
                                purchase_order_version::end_commit_num
                                    .eq(&revision.start_commit_num),
                            )
                            .execute(conn)
                            .map(|_| ())
                            .map_err(PurchaseOrderStoreError::from)?;
                    } else {
                        update(purchase_order_version::table)
                            .filter(
                                purchase_order_version::purchase_order_uid
                                    .eq(po_id)
                                    .and(
                                        purchase_order_version::version_id.eq(&revision.version_id),
                                    )
                                    .and(purchase_order_version::end_commit_num.eq(MAX_COMMIT_NUM)),
                            )
                            .set(
                                purchase_order_version::end_commit_num
                                    .eq(&revision.start_commit_num),
                            )
                            .execute(conn)
                            .map(|_| ())
                            .map_err(PurchaseOrderStoreError::from)?;
                    }

                    let updated_version = NewPurchaseOrderVersionModel::from((
                        version,
                        &revision.revision_id,
                        &revision.start_commit_num,
                    ));

                    insert_into(purchase_order_version::table)
                        .values(&updated_version)
                        .execute(conn)
                        .map(|_| ())
                        .map_err(PurchaseOrderStoreError::from)?;
                }
            }

            Ok(())
        })
    }
}
