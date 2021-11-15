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
use crate::paging::Paging;
use crate::purchase_order::store::diesel::{
    models::{PurchaseOrderModel, PurchaseOrderVersionModel, PurchaseOrderVersionRevisionModel},
    schema::{purchase_order, purchase_order_version, purchase_order_version_revision},
    ListVersionFilters, PurchaseOrderVersion, PurchaseOrderVersionList,
};

use crate::purchase_order::store::PurchaseOrderStoreError;
use diesel::prelude::*;

pub(in crate::purchase_order::store::diesel) trait PurchaseOrderStoreListPurchaseOrderVersionsOperation
{
    fn list_purchase_order_versions(
        &self,
        purchase_order_id: &str,
        filters: ListVersionFilters,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<PurchaseOrderVersionList, PurchaseOrderStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> PurchaseOrderStoreListPurchaseOrderVersionsOperation
    for PurchaseOrderStoreOperations<'a, diesel::pg::PgConnection>
{
    fn list_purchase_order_versions(
        &self,
        purchase_order_id: &str,
        filters: ListVersionFilters,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<PurchaseOrderVersionList, PurchaseOrderStoreError> {
        self.conn.transaction::<_, PurchaseOrderStoreError, _>(|| {
            let ListVersionFilters {
                is_accepted,
                is_draft,
            } = filters;

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
                    purchase_order_version::purchase_order_uid
                        .eq(&purchase_order_uid)
                        .and(purchase_order_version::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

            if let Some(is_accepted) = is_accepted {
                let mut po_query = purchase_order::table
                    .into_boxed()
                    .select(purchase_order::all_columns)
                    .filter(
                        purchase_order::purchase_order_uid
                            .eq(&purchase_order_uid)
                            .and(purchase_order::end_commit_num.eq(MAX_COMMIT_NUM)),
                    );

                if let Some(service_id) = service_id {
                    po_query = po_query.filter(purchase_order::service_id.eq(service_id));
                } else {
                    po_query = po_query.filter(purchase_order::service_id.is_null());
                }

                let po_model = po_query
                    .first::<PurchaseOrderModel>(self.conn)
                    .map(Some)
                    .map_err(|err| {
                        PurchaseOrderStoreError::InternalError(InternalError::from_source(
                            Box::new(err),
                        ))
                    })?;

                if let Some(po_model) = po_model {
                    if let Some(accepted_version_id) = po_model.accepted_version_id {
                        if is_accepted {
                            query = query
                                .filter(purchase_order_version::version_id.eq(accepted_version_id))
                        } else {
                            query = query
                                .filter(purchase_order_version::version_id.ne(accepted_version_id))
                        }
                    }
                } else {
                    return Err(PurchaseOrderStoreError::NotFoundError(format!(
                        "could not find purchase order with id: {}",
                        &purchase_order_uid
                    )));
                }
            }

            if let Some(is_draft) = is_draft {
                query = query.filter(purchase_order_version::is_draft.eq(is_draft))
            }

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

            let mut count_query = purchase_order_version::table
                .into_boxed()
                .select(purchase_order_version::all_columns);

            if let Some(service_id) = service_id {
                count_query = count_query.filter(purchase_order_version::service_id.eq(service_id));
            } else {
                count_query = count_query.filter(purchase_order_version::service_id.is_null());
            }

            let total = count_query.count().get_result(self.conn)?;

            let mut versions = Vec::new();

            for version in version_models {
                let mut query = purchase_order_version_revision::table
                    .into_boxed()
                    .select(purchase_order_version_revision::all_columns)
                    .filter(
                        purchase_order_version_revision::version_id
                            .eq(&version.version_id)
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

                versions.push(PurchaseOrderVersion::from((&version, &revision_models)));
            }

            Ok(PurchaseOrderVersionList::new(
                versions,
                Paging::new(offset, limit, total),
            ))
        })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> PurchaseOrderStoreListPurchaseOrderVersionsOperation
    for PurchaseOrderStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn list_purchase_order_versions(
        &self,
        purchase_order_id: &str,
        filters: ListVersionFilters,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<PurchaseOrderVersionList, PurchaseOrderStoreError> {
        self.conn.transaction::<_, PurchaseOrderStoreError, _>(|| {
            let ListVersionFilters {
                is_accepted,
                is_draft,
            } = filters;

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
                    purchase_order_version::purchase_order_uid
                        .eq(&purchase_order_uid)
                        .and(purchase_order_version::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

            if let Some(is_accepted) = is_accepted {
                let mut po_query = purchase_order::table
                    .into_boxed()
                    .select(purchase_order::all_columns)
                    .filter(
                        purchase_order::purchase_order_uid
                            .eq(&purchase_order_uid)
                            .and(purchase_order::end_commit_num.eq(MAX_COMMIT_NUM)),
                    );

                if let Some(service_id) = service_id {
                    po_query = po_query.filter(purchase_order::service_id.eq(service_id));
                } else {
                    po_query = po_query.filter(purchase_order::service_id.is_null());
                }

                let po_model = po_query
                    .first::<PurchaseOrderModel>(self.conn)
                    .map(Some)
                    .map_err(|err| {
                        PurchaseOrderStoreError::InternalError(InternalError::from_source(
                            Box::new(err),
                        ))
                    })?;

                if let Some(po_model) = po_model {
                    if let Some(accepted_version_id) = po_model.accepted_version_id {
                        if is_accepted {
                            query = query
                                .filter(purchase_order_version::version_id.eq(accepted_version_id))
                        } else {
                            query = query
                                .filter(purchase_order_version::version_id.ne(accepted_version_id))
                        }
                    }
                } else {
                    return Err(PurchaseOrderStoreError::NotFoundError(format!(
                        "could not find purchase order with id: {}",
                        &purchase_order_uid
                    )));
                }
            }

            if let Some(is_draft) = is_draft {
                query = query.filter(purchase_order_version::is_draft.eq(is_draft))
            }

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

            let mut count_query = purchase_order_version::table
                .into_boxed()
                .select(purchase_order_version::all_columns);

            if let Some(service_id) = service_id {
                count_query = count_query.filter(purchase_order_version::service_id.eq(service_id));
            } else {
                count_query = count_query.filter(purchase_order_version::service_id.is_null());
            }

            let total = count_query.count().get_result(self.conn)?;

            let mut versions = Vec::new();

            for version in version_models {
                let mut query = purchase_order_version_revision::table
                    .into_boxed()
                    .select(purchase_order_version_revision::all_columns)
                    .filter(
                        purchase_order_version_revision::version_id
                            .eq(&version.version_id)
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

                versions.push(PurchaseOrderVersion::from((&version, &revision_models)));
            }

            Ok(PurchaseOrderVersionList::new(
                versions,
                Paging::new(offset, limit, total),
            ))
        })
    }
}
