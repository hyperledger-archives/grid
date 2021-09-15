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
use crate::purchase_order::store::diesel::{
    models::{
        NewPurchaseOrderModel, NewPurchaseOrderVersionModel, NewPurchaseOrderVersionRevisionModel,
        PurchaseOrderModel, PurchaseOrderVersionModel, PurchaseOrderVersionRevisionModel,
    },
    schema::{purchase_order, purchase_order_version, purchase_order_version_revision},
    PurchaseOrderStoreError,
};

use crate::commits::MAX_COMMIT_NUM;
use crate::error::InternalError;

use diesel::{
    dsl::{insert_into, update},
    prelude::*,
    result::Error as dsl_error,
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
                .map(Some)
                .or_else(|err| {
                    if err == dsl_error::NotFound {
                        Ok(None)
                    } else {
                        Err(err)
                    }
                })
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

            for version in versions {
                let mut query = purchase_order_version::table.into_boxed().filter(
                    purchase_order_version::purchase_order_uid
                        .eq(&order.purchase_order_uid)
                        .and(purchase_order_version::version_id.eq(&version.version_id))
                        .and(purchase_order_version::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

                if let Some(service_id) = &order.service_id {
                    query = query.filter(purchase_order_version::service_id.eq(service_id));
                } else {
                    query = query.filter(purchase_order_version::service_id.is_null());
                }

                let duplicate = query
                    .first::<PurchaseOrderVersionModel>(self.conn)
                    .map(Some)
                    .or_else(|err| {
                        if err == dsl_error::NotFound {
                            Ok(None)
                        } else {
                            Err(err)
                        }
                    })
                    .map_err(|err| {
                        PurchaseOrderStoreError::InternalError(InternalError::from_source(
                            Box::new(err),
                        ))
                    })?;

                if duplicate.is_some() {
                    if let Some(service_id) = &order.service_id {
                        update(purchase_order_version::table)
                            .filter(
                                purchase_order_version::purchase_order_uid
                                    .eq(&order.purchase_order_uid)
                                    .and(purchase_order_version::version_id.eq(&version.version_id))
                                    .and(purchase_order_version::service_id.eq(service_id))
                                    .and(purchase_order_version::end_commit_num.eq(MAX_COMMIT_NUM)),
                            )
                            .set(
                                purchase_order_version::end_commit_num
                                    .eq(&version.start_commit_num),
                            )
                            .execute(self.conn)
                            .map(|_| ())
                            .map_err(PurchaseOrderStoreError::from)?;
                    } else {
                        update(purchase_order_version::table)
                            .filter(
                                purchase_order_version::purchase_order_uid
                                    .eq(&order.purchase_order_uid)
                                    .and(purchase_order_version::version_id.eq(&version.version_id))
                                    .and(purchase_order_version::end_commit_num.eq(MAX_COMMIT_NUM)),
                            )
                            .set(
                                purchase_order_version::end_commit_num
                                    .eq(&version.start_commit_num),
                            )
                            .execute(self.conn)
                            .map(|_| ())
                            .map_err(PurchaseOrderStoreError::from)?;
                    }
                }

                insert_into(purchase_order_version::table)
                    .values(&version)
                    .execute(self.conn)
                    .map(|_| ())
                    .map_err(PurchaseOrderStoreError::from)?;
            }

            for revision in revisions {
                let mut query = purchase_order_version_revision::table.into_boxed().filter(
                    purchase_order_version_revision::version_id
                        .eq(&revision.version_id)
                        .and(purchase_order_version_revision::revision_id.eq(&revision.revision_id))
                        .and(purchase_order_version_revision::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

                if let Some(service_id) = &order.service_id {
                    query =
                        query.filter(purchase_order_version_revision::service_id.eq(service_id));
                } else {
                    query = query.filter(purchase_order_version_revision::service_id.is_null());
                }

                let duplicate = query
                    .first::<PurchaseOrderVersionRevisionModel>(self.conn)
                    .map(Some)
                    .or_else(|err| {
                        if err == dsl_error::NotFound {
                            Ok(None)
                        } else {
                            Err(err)
                        }
                    })
                    .map_err(|err| {
                        PurchaseOrderStoreError::InternalError(InternalError::from_source(
                            Box::new(err),
                        ))
                    })?;

                if duplicate.is_none() {
                    insert_into(purchase_order_version_revision::table)
                        .values(&revision)
                        .execute(self.conn)
                        .map(|_| ())
                        .map_err(PurchaseOrderStoreError::from)?;
                }
            }

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
                .map(Some)
                .or_else(|err| {
                    if err == dsl_error::NotFound {
                        Ok(None)
                    } else {
                        Err(err)
                    }
                })
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

            for version in versions {
                let mut query = purchase_order_version::table.into_boxed().filter(
                    purchase_order_version::purchase_order_uid
                        .eq(&order.purchase_order_uid)
                        .and(purchase_order_version::version_id.eq(&version.version_id))
                        .and(purchase_order_version::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

                if let Some(service_id) = &order.service_id {
                    query = query.filter(purchase_order_version::service_id.eq(service_id));
                } else {
                    query = query.filter(purchase_order_version::service_id.is_null());
                }

                let duplicate = query
                    .first::<PurchaseOrderVersionModel>(self.conn)
                    .map(Some)
                    .or_else(|err| {
                        if err == dsl_error::NotFound {
                            Ok(None)
                        } else {
                            Err(err)
                        }
                    })
                    .map_err(|err| {
                        PurchaseOrderStoreError::InternalError(InternalError::from_source(
                            Box::new(err),
                        ))
                    })?;

                if duplicate.is_some() {
                    if let Some(service_id) = &order.service_id {
                        update(purchase_order_version::table)
                            .filter(
                                purchase_order_version::purchase_order_uid
                                    .eq(&order.purchase_order_uid)
                                    .and(purchase_order_version::version_id.eq(&version.version_id))
                                    .and(purchase_order_version::service_id.eq(service_id))
                                    .and(purchase_order_version::end_commit_num.eq(MAX_COMMIT_NUM)),
                            )
                            .set(
                                purchase_order_version::end_commit_num
                                    .eq(&version.start_commit_num),
                            )
                            .execute(self.conn)
                            .map(|_| ())
                            .map_err(PurchaseOrderStoreError::from)?;
                    } else {
                        update(purchase_order_version::table)
                            .filter(
                                purchase_order_version::purchase_order_uid
                                    .eq(&order.purchase_order_uid)
                                    .and(purchase_order_version::version_id.eq(&version.version_id))
                                    .and(purchase_order_version::end_commit_num.eq(MAX_COMMIT_NUM)),
                            )
                            .set(
                                purchase_order_version::end_commit_num
                                    .eq(&version.start_commit_num),
                            )
                            .execute(self.conn)
                            .map(|_| ())
                            .map_err(PurchaseOrderStoreError::from)?;
                    }
                }

                insert_into(purchase_order_version::table)
                    .values(&version)
                    .execute(self.conn)
                    .map(|_| ())
                    .map_err(PurchaseOrderStoreError::from)?;
            }

            for revision in revisions {
                let mut query = purchase_order_version_revision::table.into_boxed().filter(
                    purchase_order_version_revision::version_id
                        .eq(&revision.version_id)
                        .and(purchase_order_version_revision::revision_id.eq(&revision.revision_id))
                        .and(purchase_order_version_revision::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

                if let Some(service_id) = &order.service_id {
                    query =
                        query.filter(purchase_order_version_revision::service_id.eq(service_id));
                } else {
                    query = query.filter(purchase_order_version_revision::service_id.is_null());
                }

                let duplicate = query
                    .first::<PurchaseOrderVersionRevisionModel>(self.conn)
                    .map(Some)
                    .or_else(|err| {
                        if err == dsl_error::NotFound {
                            Ok(None)
                        } else {
                            Err(err)
                        }
                    })
                    .map_err(|err| {
                        PurchaseOrderStoreError::InternalError(InternalError::from_source(
                            Box::new(err),
                        ))
                    })?;

                if duplicate.is_none() {
                    insert_into(purchase_order_version_revision::table)
                        .values(&revision)
                        .execute(self.conn)
                        .map(|_| ())
                        .map_err(PurchaseOrderStoreError::from)?;
                }
            }

            Ok(())
        })
    }
}
