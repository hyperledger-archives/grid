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

use crate::error::InternalError;
use crate::{
    commits::MAX_COMMIT_NUM,
    purchase_order::store::diesel::{
        models::PurchaseOrderAlternateIdModel, schema::purchase_order_alternate_id,
        PurchaseOrderStoreError,
    },
};

use diesel::{dsl::update, prelude::*};

#[cfg(feature = "postgres")]
pub(crate) mod pg {
    use super::*;

    pub fn remove_alternate_id(
        conn: &diesel::pg::PgConnection,
        alternate_id: &PurchaseOrderAlternateIdModel,
        end_commit_num: &i64,
    ) -> Result<(), PurchaseOrderStoreError> {
        conn.transaction::<_, PurchaseOrderStoreError, _>(|| {
            let mut query = purchase_order_alternate_id::table
                .into_boxed()
                .select(purchase_order_alternate_id::all_columns)
                .filter(
                    purchase_order_alternate_id::purchase_order_uid
                        .eq(&alternate_id.purchase_order_uid)
                        .and(
                            purchase_order_alternate_id::alternate_id_type
                                .eq(&alternate_id.alternate_id_type),
                        )
                        .and(
                            purchase_order_alternate_id::alternate_id
                                .eq(&alternate_id.alternate_id),
                        )
                        .and(purchase_order_alternate_id::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

            if let Some(service_id) = &alternate_id.service_id {
                query = query.filter(purchase_order_alternate_id::service_id.eq(service_id));
            } else {
                query = query.filter(purchase_order_alternate_id::service_id.is_null());
            }

            let duplicate = query
                .first::<PurchaseOrderAlternateIdModel>(conn)
                .optional()
                .map_err(|err| {
                    PurchaseOrderStoreError::InternalError(InternalError::from_source(Box::new(
                        err,
                    )))
                })?;

            if duplicate.is_some() {
                if let Some(service_id) = &alternate_id.service_id {
                    update(purchase_order_alternate_id::table)
                        .filter(
                            purchase_order_alternate_id::purchase_order_uid
                                .eq(&alternate_id.purchase_order_uid)
                                .and(
                                    purchase_order_alternate_id::alternate_id_type
                                        .eq(&alternate_id.alternate_id_type),
                                )
                                .and(
                                    purchase_order_alternate_id::alternate_id
                                        .eq(&alternate_id.alternate_id),
                                )
                                .and(purchase_order_alternate_id::service_id.eq(&service_id))
                                .and(
                                    purchase_order_alternate_id::end_commit_num.eq(MAX_COMMIT_NUM),
                                ),
                        )
                        .set(purchase_order_alternate_id::end_commit_num.eq(&end_commit_num))
                        .execute(conn)
                        .map(|_| ())
                        .map_err(PurchaseOrderStoreError::from)?;
                } else {
                    update(purchase_order_alternate_id::table)
                        .filter(
                            purchase_order_alternate_id::purchase_order_uid
                                .eq(&alternate_id.purchase_order_uid)
                                .and(
                                    purchase_order_alternate_id::alternate_id_type
                                        .eq(&alternate_id.alternate_id_type),
                                )
                                .and(
                                    purchase_order_alternate_id::alternate_id
                                        .eq(&alternate_id.alternate_id),
                                )
                                .and(
                                    purchase_order_alternate_id::end_commit_num.eq(MAX_COMMIT_NUM),
                                ),
                        )
                        .set(purchase_order_alternate_id::end_commit_num.eq(&end_commit_num))
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
pub(crate) mod sqlite {
    use super::*;

    pub fn remove_alternate_id(
        conn: &diesel::sqlite::SqliteConnection,
        alternate_id: &PurchaseOrderAlternateIdModel,
        end_commit_num: &i64,
    ) -> Result<(), PurchaseOrderStoreError> {
        conn.transaction::<_, PurchaseOrderStoreError, _>(|| {
            let mut query = purchase_order_alternate_id::table
                .into_boxed()
                .select(purchase_order_alternate_id::all_columns)
                .filter(
                    purchase_order_alternate_id::purchase_order_uid
                        .eq(&alternate_id.purchase_order_uid)
                        .and(
                            purchase_order_alternate_id::alternate_id_type
                                .eq(&alternate_id.alternate_id_type),
                        )
                        .and(
                            purchase_order_alternate_id::alternate_id
                                .eq(&alternate_id.alternate_id),
                        )
                        .and(purchase_order_alternate_id::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

            if let Some(service_id) = &alternate_id.service_id {
                query = query.filter(purchase_order_alternate_id::service_id.eq(service_id));
            } else {
                query = query.filter(purchase_order_alternate_id::service_id.is_null());
            }

            let duplicate = query
                .first::<PurchaseOrderAlternateIdModel>(conn)
                .optional()
                .map_err(|err| {
                    PurchaseOrderStoreError::InternalError(InternalError::from_source(Box::new(
                        err,
                    )))
                })?;

            if duplicate.is_some() {
                if let Some(service_id) = &alternate_id.service_id {
                    update(purchase_order_alternate_id::table)
                        .filter(
                            purchase_order_alternate_id::purchase_order_uid
                                .eq(&alternate_id.purchase_order_uid)
                                .and(
                                    purchase_order_alternate_id::alternate_id_type
                                        .eq(&alternate_id.alternate_id_type),
                                )
                                .and(
                                    purchase_order_alternate_id::alternate_id
                                        .eq(&alternate_id.alternate_id),
                                )
                                .and(purchase_order_alternate_id::service_id.eq(&service_id))
                                .and(
                                    purchase_order_alternate_id::end_commit_num.eq(MAX_COMMIT_NUM),
                                ),
                        )
                        .set(purchase_order_alternate_id::end_commit_num.eq(&end_commit_num))
                        .execute(conn)
                        .map(|_| ())
                        .map_err(PurchaseOrderStoreError::from)?;
                } else {
                    update(purchase_order_alternate_id::table)
                        .filter(
                            purchase_order_alternate_id::purchase_order_uid
                                .eq(&alternate_id.purchase_order_uid)
                                .and(
                                    purchase_order_alternate_id::alternate_id_type
                                        .eq(&alternate_id.alternate_id_type),
                                )
                                .and(
                                    purchase_order_alternate_id::alternate_id
                                        .eq(&alternate_id.alternate_id),
                                )
                                .and(
                                    purchase_order_alternate_id::end_commit_num.eq(MAX_COMMIT_NUM),
                                ),
                        )
                        .set(purchase_order_alternate_id::end_commit_num.eq(&end_commit_num))
                        .execute(conn)
                        .map(|_| ())
                        .map_err(PurchaseOrderStoreError::from)?;
                }
            }

            Ok(())
        })
    }
}
